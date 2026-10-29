//! The scene over time (E17 M2 T4a; `docs/epics/E17-design.md` §7): a record's commands become
//! each stack's animation at the original's timing. [`timeline`] plays the events of each
//! command on the original's clock (the frame deadline on its grid, each group's frame time, the
//! strike's iterations, the shot's frames and projectile, the waits outside the deadline) and
//! keeps every pose a stack takes with the time it is drawn and the time it shows.
//! [`Timeline::at`] is what shows at any time, [`Timeline::lines`] the capture harness's
//! timeline format (design §7.6), which the recorded sequences check.
//!
//! T4b-1 (design §7.9) added the projectile's flights, the idle steps and the outlined stack, and
//! checked every frame of the recorded sequences against the original's screen (the walk's
//! offsets confirmed; the shooter's strike loop and the flight's points corrected).
//!
//! T4b-2a added the turns, the draw layers and the viewer's own fidgets, T5b the sounds' cues.
//!
//! *Provisional* (the M2 review list): the next command's default start, a command without a
//! time counting as the computer's (the idle step and the outline), the flight's cycles rounding.
//! Confirmed by the recorded `fly` (E17 M3 T1b-2): a flier's cycles over every row, a breath
//! told by its second stack's place. Not yet (E17 M3 T1): the spells' sprites, deaths other than
//! by a strike, created stacks (a strike by a stack the timeline does not know plays its
//! reactions alone).

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt::Write as _;

use h3_protocol::wire::{Event, Observation, Outcome, ServerMessage, Side, Stack, Target};
use h3_render::Bugs;
use h3_render::recolour::Effect;

use crate::battle_log::{Logged, Spread};
use crate::hero::{Action, Happening, Heroes};
use crate::layout::BattleLayout;
use crate::ray::{self, Stroke};
use crate::siege::{FIELD_HEXES, Walls};
use crate::tables::Groups;

/// Every frame outside a walk's group and the shot's: 100 ms times the factor (§7.1).
const TICK: u32 = 100;
/// The wait before a retaliation or a second strike, outside the deadline (§7.4).
const PAUSE: u32 = 150;
/// A projectile's step, outside the deadline (§7.4).
const STEP: u32 = 33;
/// The wait before a ray, outside the deadline (0x43F52F), and a ray's tick (`DrawRay`'s
/// argument, 0x43F58B; its own clock, not the frame deadline: §7.12).
const RAY_WAIT: u32 = 115;
const RAY_TICK: u32 = 10;
/// The idle step's period, whatever the combat speed (0x4960D0, §7.1).
const IDLE_STEP: u32 = 100;
/// The draw layers (+0x8, §4.6, §7.9): standing (0x43D750), walking up and down (0x43F0B0), a
/// melee's striker (0x441CC1) and a struck stack (the harness's reading of memory), the corpses'
/// pass.
const STANDING: u8 = 4;
const WALK_UP: u8 = 3;
const WALK_DOWN: u8 = 7;
const STRIKER: u8 = 6;
const STRUCK: u8 = 5;
const CORPSE: u8 = 1;
/// Not a pass: a flier between its hexes (E17 M2 T5a, design §7.10). The flight takes it off its
/// hexes (0x468730, from 0x4B4BFE), so no row pass draws it, and draws it by itself at each
/// point over the whole battlefield (0x4B4D60); it stands on the landing hex again before group
/// 21 (0x4B4E24). The scene draws this layer after every row: the recorded `fly`'s cycles across
/// rows match the original's frames (E17 M3 T1b-2).
pub const FLYING: u8 = 8;
/// Not a pass: a stack gone from the battlefield without a corpse (E17 M3 T9d, recorded in
/// `sacrifice`): `stack_death` 0x444120 leaves none for a sacrificed or summoned stack (its
/// +0x84 flags 0x10000000 or 0x400000, 0x4442C5) and marks it for the dissolve
/// ([`Player::remove_gone`], E17 M5 T5a); a corpse the Pit Lord's Demons rise from. The scene
/// draws no stack in this layer.
pub const GONE: u8 = u8::MAX;
/// A projectile's step length in pixels: trunc((d + 20) / 40) steps (§7.4).
const STEP_PIXELS: u64 = 40;
/// Half a hex: a two-hex target is aimed at from its middle (0x43F323).
const HALF_HEX: i32 = 22;
/// Place 3's offset from the standing frame's front (0x43E922: its left margin + width − 0xC4).
const BESIDE: i32 = 196;
/// tan(25°) × 10^15: a shot is up above 25°, down below −25° (0x469880).
const TAN_25: i64 = 466_307_658_154_999;
/// A magic animation at a hex of kind 1 rises this far above half its height over the hex's
/// anchor (`play_magic_animation_at_hex` 0x496A10; E17 M3 T7c-1).
const AT_HEX_RISE: i32 = 37;
/// The cell picture's bottom below a hex's top (the hex record's +0xC, design §4.1): a magic
/// animation at a hex of kinds 0 and 4 stands on it (E17 M3 T7c-1).
const CELL_BOTTOM: i32 = 52;
/// The scale of [`TAN_25`] and of [`tangent`].
const TAN_SCALE: i64 = 1_000_000_000_000_000;

/// The combat speed (the setting `Combat Speed`, 0 to 2; 0x6987EC) and its factor F (§7.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Speed(u8);

impl Speed {
    /// The setting `setting` (0, 1 or 2; a higher one is 2).
    pub fn new(setting: u8) -> Self {
        Self(setting.min(2))
    }

    /// The setting.
    pub const fn setting(self) -> u8 {
        self.0
    }

    /// F as the f32 the executable multiplies with (0x63CF7C: 1.0, 0.63, 0.40): its mantissa
    /// over a power of two.
    const fn factor(self) -> (u64, u32) {
        match self.0 {
            0 => (1, 0),
            1 => (10_569_646, 24),
            _ => (13_421_773, 25),
        }
    }

    /// trunc(`ms` × F): the f32 operands multiplied exactly, as the x87 does at 53 bits (100 ms
    /// gives 100, 62, 40).
    pub fn scale(self, ms: u32) -> u32 {
        self.ratio(ms, 1)
    }

    /// trunc(`ms` × F / `n`): a walk's frame (§7.3); `n` of 0 counts as 1.
    pub fn ratio(self, ms: u32, n: u32) -> u32 {
        let (mantissa, shift) = self.factor();
        let v = u64::from(ms) * mantissa / (u64::from(n.max(1)) << shift);
        u32::try_from(v).unwrap_or(u32::MAX)
    }
}

/// A creature's animation numbers: its times from the asset tables and its DEF's groups.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Look {
    /// The frames of each DEF group.
    pub frames: BTreeMap<u32, u32>,
    /// `cranim.txt`'s times: fidget interval, walk, attack, flight distance
    /// ([`crate::tables::Creature::times`]).
    pub times: [u32; 4],
    /// The attack's climax frame (0: none).
    pub climax: u32,
    /// Where a projectile starts, up, straight and down (x, y; facing right).
    pub missile_offsets: [[i16; 2]; 3],
    /// The top edge of frame 0 of the standing group in its canvas: a shot aims at the target's
    /// anchor y minus (267 − it) / 2 (0x43F338; the 267 is the canvas's anchor,
    /// [`BattleLayout::stack_canvas`]).
    pub top: i32,
    /// The right edge of frame 0 of the standing group in its canvas (its left margin and width,
    /// the DEF frame's +0x38 and +0x30): a magic animation of place 3 is drawn beside it (E17 M3
    /// T7b-1).
    pub front: i32,
    /// A shooter's projectile (none: no projectile is drawn).
    pub missile: Option<Missile>,
    /// A ray shooter's colours by the distance from the ray's edge (R, G, B; none: it shoots
    /// a projectile; §7.12).
    pub ray: Option<Vec<[u8; 3]>>,
    /// The explosion its projectile ends in (the Magogs, the Liches; E17 M3 T7c-3).
    pub explosion: Option<Explosion>,
}

/// The explosion a shooter's projectile ends in (`stack_shot` 0x43FA15, 0x43FE07; E17 M3
/// T7c-3): a spell's magic animation and sound, a frame a redraw of `time` ms (times the speed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Explosion {
    /// The spell (its handle): its sound.
    pub spell: u16,
    /// Its magic animation.
    pub animation: u8,
    /// Each frame's time.
    pub time: u32,
}

/// A shooter's projectile DEF and how its frame follows the shot's angle (`shot_angle_group`
/// 0x469880, `shot_animation` 0x43F5D6; design §7.9).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Missile {
    /// The DEF.
    pub def: String,
    /// Its canvas's width and height (the DEF's): each step's canvas is centred on its point.
    pub size: [i32; 2],
    /// The frames of its group 0 (a rock's cycle through them, E17 M4 T2b).
    pub frames: u32,
    /// The creature's angle in degrees for each frame of group 0, from high to low: frame f is
    /// drawn when the first f midpoints between consecutive angles are at least the shot's angle
    /// and the next is not (0x469957).
    pub angles: Vec<i16>,
}

/// A projectile in flight (`shot_animation` 0x43F5D6 to 0x43F826, design §7.9): `steps` steps
/// of one frame, the first shown with the shooter's last frame before it, one every `step` ms,
/// each over the screen as it stands (nothing else is drawn meanwhile); erased after the last.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flight {
    /// The DEF.
    pub def: String,
    /// Its frame in group 0 (at the first step).
    pub frame: u32,
    /// The frames of group 0 it cycles through, a frame a step (a spell's projectile, E17 M3
    /// T7b-1); 0: it keeps its frame.
    pub cycle: u32,
    /// Drawn mirrored: the target lies left of the start.
    pub mirrored: bool,
    /// When the first step shows.
    pub from: u32,
    /// Each step's time, trunc(33 F).
    pub step: u32,
    /// The steps, trunc((d + 20) / 40).
    pub steps: u32,
    /// The first step's canvas: the start point less half the DEF's size.
    pub start: [i32; 2],
    /// The move per step: the distance to the target over the steps, truncated.
    pub delta: [i32; 2],
    /// Each step's canvas, where the steps follow no line (a wall shot's rock, E17 M4 T2b);
    /// empty: `start` and `delta`.
    pub path: Vec<[i32; 2]>,
}

/// A ray (`DrawRay` 0x5A5C20, [`crate::ray`], design §7.12): the ticks' strokes, tick k shown
/// k × `step` after `from` (all at `from` without a step: Chain Lightning's, E17 M3 T7c-3); erased
/// at `until` (a shooter's as its last tick shows: `DrawRay` redraws the battle then without
/// waiting for the frame deadline, so the last tick shows for no time).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Beam {
    /// When the ray starts (a shooter's after the wait of trunc(115 F)).
    pub from: u32,
    /// Each tick's time, trunc(10 F) (0: none).
    pub step: u32,
    /// When it is erased.
    pub until: u32,
    /// Each tick's strokes.
    pub ticks: Vec<Vec<Stroke>>,
    /// The colours by the distance from the edge (R, G, B).
    pub colours: Vec<[u8; 3]>,
    /// When the mouse's cursor is hidden and when it shows again (E17 M3 T6c-2): a shooter's
    /// `draw_ray` hides it as it starts (0x5A5C46) and shows it after the shot group's rest
    /// (0x5A61D4); Chain Lightning's handler from its start to its end (0x5A6380, 0x5A669C).
    pub cursor: [u32; 2],
}

/// A ray as it shows at a time: the strokes drawn so far, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ray {
    /// The strokes.
    pub strokes: Vec<Stroke>,
    /// The colours by the distance from the edge (R, G, B).
    pub colours: Vec<[u8; 3]>,
}

/// A projectile as it shows at a time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Projectile {
    /// The DEF.
    pub def: String,
    /// Its frame in group 0.
    pub frame: u32,
    /// Its canvas's top-left.
    pub at: [i32; 2],
    /// Drawn mirrored.
    pub mirrored: bool,
}

impl Flight {
    /// The projectile it shows at `t`, if it flies then (E17 M5 T1a: [`Timeline::projectile`]'s
    /// and the playhead's).
    pub fn at(&self, t: u32) -> Option<Projectile> {
        let k = t.checked_sub(self.from)? / self.step.max(1);
        (k < self.steps).then(|| {
            let frame = match self.cycle {
                0 => self.frame,
                n => (self.frame + k) % n,
            };
            let at = usize::try_from(k)
                .ok()
                .and_then(|k| self.path.get(k))
                .copied()
                .unwrap_or_else(|| {
                    let k = i32::try_from(k).unwrap_or(0);
                    [
                        self.start[0] + k * self.delta[0],
                        self.start[1] + k * self.delta[1],
                    ]
                });
            Projectile {
                def: self.def.clone(),
                frame,
                at,
                mirrored: self.mirrored,
            }
        })
    }

    /// When it has flown: the end of its last step ([`Flight::at`] shows none from then on).
    fn until(&self) -> u64 {
        u64::from(self.from) + u64::from(self.steps) * u64::from(self.step.max(1))
    }
}

impl Beam {
    /// The ray it shows at `t`, if it is drawn then: the strokes of the ticks shown by then (E17
    /// M5 T1a: [`Timeline::ray`]'s and the playhead's).
    pub fn at(&self, t: u32) -> Option<Ray> {
        let k = match self.step {
            0 => self.ticks.len(),
            step => usize::try_from(t.checked_sub(self.from)? / step).unwrap_or(usize::MAX),
        };
        (t >= self.from && t < self.until && k >= 1).then(|| Ray {
            strokes: self.ticks.iter().take(k).flatten().copied().collect(),
            colours: self.colours.clone(),
        })
    }
}

impl Look {
    /// The frames of `group` (0 if the DEF lacks it).
    fn count(&self, group: u8) -> u32 {
        self.frames.get(&u32::from(group)).copied().unwrap_or(0)
    }

    /// A shot group's climax in its `n` frames: the creature's, else (none, or past them) its
    /// last.
    fn climax_in(&self, n: u32) -> u32 {
        if self.climax == 0 || self.climax > n {
            n
        } else {
            self.climax
        }
    }
}

/// Where the battle is drawn: the layout, the field's width and the DEF groups, and whether the
/// original's drawing bugs are reproduced.
#[derive(Debug, Clone, Copy)]
pub struct Stage<'a> {
    /// The battle screen's layout.
    pub layout: &'a BattleLayout,
    /// Hexes per row.
    pub width: u8,
    /// The DEF groups.
    pub groups: Groups,
    /// The magic animations the record shows, by number (E17 M3 T3b).
    pub magic: &'a BTreeMap<u8, MagicLook>,
    /// What the record's casts draw (E17 M3 T7b-1).
    pub casts: &'a Casts,
    /// Whether the timeline reproduces the original's drawing bugs (E17 M3 T7d, CLT-Q8;
    /// `h3_render::Bugs`): fixed for what the client shows, the original's for the comparisons
    /// with its captures (`h3-view sequences`). The design notes name each one (§7.16): a
    /// creature caster's group by the angle, a caster its spell missed standing at once, the
    /// mouse's stack unmarked as a creature's cast starts (CLT-Q8); a mass animation's every frame
    /// shown, a mass reaction's hit ending on the standing group's frame 0, Chain Lightning's
    /// opening bolt on the screen, each Fire Wall object animated at its own hex, Animate Dead's
    /// own animation, the Liches' cloud translucent, Armageddon's line with the caster's bonuses,
    /// Sacrifice's line naming its victim by its number before its death (E17 M3 T9d, CLT-Q9 to
    /// CLT-Q14, CLT-Q16, CLT-Q17).
    pub bugs: Bugs,
    /// A siege's turrets and explosions (E17 M4 T2b; none: no siege).
    pub siege: Option<&'a SiegeStage>,
    /// The samples' lengths when the battle plays its sounds (E17 M5 T5b2): the waits for them
    /// ([`crate::sound::Lengths`]); none: no sound, nothing waited for.
    pub lengths: Option<&'a crate::sound::Lengths>,
}

/// What a battle's casts draw (E17 M3 T7b-1, design §7.16): each spell the record casts by its
/// handle, each side's hero as it casts, and the heroes' groups for it; and the creatures whose
/// Fear a frightened stack's line names (E17 M3 T7b-2b-2b).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Casts {
    /// The spells, by handle.
    pub spells: BTreeMap<u16, SpellLook>,
    /// Each side's hero (none: no hero, or none that casts).
    pub casters: [Option<Caster>; 2],
    /// The heroes' casting group (4) and standing group (0).
    pub groups: [u8; 2],
    /// The creatures that frighten, by handle (the ruleset's `fear`; the original's creature 132,
    /// the Azure Dragon, `fear_check` 0x464D40).
    pub frighteners: BTreeSet<u16>,
    /// The creatures with a Fire Shield of their own, by handle (the ruleset's innate spell; the
    /// Efreet Sultans, `fire_shield_factor` 0x443130; E17 M3 T7c-3).
    pub shielded: BTreeSet<u16>,
    /// The draws Chain Lightning's rays turn by, in order (E17 M3 T7c-3): the original's, which a
    /// capture keeps for the frame check; past them the viewer draws its own ([`crate::ray::chain`]).
    pub rays: Vec<i32>,
    /// The bodies of the creatures the record's battle creates, by handle (E17 M5 T5a: the
    /// ruleset's; a clone takes its original's).
    pub bodies: BTreeMap<u16, Body>,
}

/// A spell as its cast draws it (E17 M3 T7b-1): its magic animation (the spell record's +8; the
/// stage's `magic` holds its numbers) and its projectile's DEFs, if its handler flies one; its
/// number and its sign (E17 M3 T7b-2a: the handler, the count box's tint, a walk's time).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellLook {
    /// The executable's spell number (its handler, [`HASTE`], [`SLOW`], …).
    pub original: u16,
    /// The spell record's +0: 1 positive, −1 negative, 0 neither ([`crate::scene::spell_hue`]).
    pub sign: i8,
    /// Its effect at each side's hero's mastery (the attacker's first): Disrupting Ray's lowered
    /// defense (E17 M3 T7b-2a).
    pub value: [u16; 2],
    /// Its magic animation's number.
    pub animation: Option<u8>,
    /// Its projectile's DEFs from the level one to the one pointing down, none for most spells.
    pub projectile: Vec<Shell>,
    /// The battlefield objects its handler places, one per size (E17 M3 T7c-2).
    pub objects: Vec<ObjectLook>,
    /// Its handler plays a sound with each object it places ([`Sound::Object`]: Quicksand's,
    /// Land Mine's).
    pub object_sound: bool,
}

/// A battlefield object a spell places (E17 M3 T7c-2, the asset tables' `objects`): as
/// [`Object`] draws it, the hexes a placement of it takes, and the magic animations at a hex that
/// place it and that it vanishes with.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ObjectLook {
    /// Its DEF.
    pub def: String,
    /// The frames of its DEF's group 0.
    pub frames: u32,
    /// The hexes it covers.
    pub hexes: u8,
    /// Its height in hex rows.
    pub rows: u8,
    /// Its layer is 1.
    pub before_stacks: bool,
    /// The animation that places it.
    pub placed: u8,
    /// The animation it vanishes with.
    pub vanish: u8,
}

/// One DEF of a spell's projectile (`spell_projectile` 0x467DB0): its name, its canvas's width
/// and height, the frames of its group 0 (they cycle) and its angle in degrees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shell {
    /// The DEF.
    pub def: String,
    /// Its canvas's width and height.
    pub size: [i32; 2],
    /// The frames of its group 0.
    pub frames: u32,
    /// Its angle (0 level, negative down).
    pub angle: i16,
}

/// A side's hero as it casts (`cast_spell` 0x5A01CF, 0x5A20B8; design §7.16): its cast point on
/// the screen, where projectiles start (`hero_battle_defs` +4, +8: (x − 43, y − 19) on the left,
/// (693 + W − x, y − 19) on the right, W its DEF's width), the casting group's frames played before
/// the spell (+0xC) and the group's frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Caster {
    /// The cast point.
    pub point: [i32; 2],
    /// The frames before the spell.
    pub before: u32,
    /// The casting group's frames.
    pub frames: u32,
}

/// A frame of a side's hero that a command draws (E17 M3 T7b-1: its cast), drawn at `drawn`,
/// shown from `shown`; its idle motions and reactions are [`crate::hero::Heroes`]'s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeroKey {
    /// The side (0 the attacker's).
    pub side: u8,
    /// When it is drawn.
    pub drawn: u32,
    /// When it shows.
    pub shown: u32,
    /// The hero DEF's group.
    pub group: u32,
    /// The frame in it.
    pub frame: u32,
}

/// A magic animation's numbers (`PlayMagicAnimation` 0x496840, design §7.15; E17 M3 T3b): its
/// DEF, the frames of its group 0, its width and height (the DEF's), where it is drawn over a
/// stack ([`crate::tables::Magic::place`]) and whether translucent.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MagicLook {
    /// The DEF.
    pub def: String,
    /// The frames of its group 0.
    pub frames: u32,
    /// The DEF's width and height.
    pub size: [i32; 2],
    /// Where it is drawn over a stack: 0 standing on the hex, 1 centred on the creature, 2 above
    /// it, 3 beside its front (E17 M3 T7b-1).
    pub place: u8,
    /// Drawn by the translucent drawer.
    pub translucent: bool,
}

/// The magic animation of the First Aid Tent's heal (`first_aid_tent_action` 0x478D14: number
/// 0x4F over the patient, 100 ms a frame, without the patient's hit).
pub const HEAL: u8 = 79;

/// The executable's numbers of the spells whose handlers the viewer tells apart (E17 M3 T7b-2a,
/// design §7.16): Dispel plays its animation before its effect (0x5A15E0), Cure after its effect
/// (0x5A1808), Disrupting Ray flies its projectile, then its animation (0x5A1A84); Haste and Slow
/// change the walk time (0x444CE5, 0x444D3A).
pub const DISPEL: u16 = 35;
/// Cure ([`DISPEL`]).
pub const CURE: u16 = 37;
/// Disrupting Ray ([`DISPEL`]).
pub const DISRUPTING_RAY: u16 = 47;
/// Haste ([`DISPEL`]): the walk time W trunc(W0 × 0.65) while it lasts.
pub const HASTE: u16 = 53;
/// Slow ([`DISPEL`]): the walk time W trunc(W0 × 1.5) while it lasts.
pub const SLOW: u16 = 54;

/// The hue a stack's count box takes from a time on (E17 M3 T7b-2a, design §4.3): none, the
/// layout's (no spell active), else [`crate::scene::spell_hue`] of its spells' signs; it shows
/// with the first frame drawn after the change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TintKey {
    /// The stack's id.
    pub stack: u8,
    /// When it shows.
    pub shown: u32,
    /// The hue's f32 bits (none: the layout's tint).
    pub hue: Option<u32>,
}

/// A stack's palette effect by its spells from a time on (E17 M5 T5b1, design §7.26): a glow's
/// step while Bloodlust's or Stone Gaze's handler redraws it, then the petrified look while Stone
/// Gaze holds it (none: its own colours, or a clone's, which the scene takes from the stack).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LookKey {
    /// The stack's id.
    pub stack: u8,
    /// When it shows.
    pub shown: u32,
    /// The effect.
    pub look: Option<Effect>,
}

/// Hypnotize (the executable's spell number): the other side controls the stack while it holds
/// (`stack_activation` 0x465330 sets the acting side +0x132C0 to the controller; E17 M5 T5b2).
pub const HYPNOTIZE: u16 = 60;
/// Bloodlust (the executable's spell number): its handler glows the stacks it reached
/// (`cast_spell` 0x5A122B, 0x5A1359; E17 M5 T5b1).
pub const BLOODLUST: u16 = 43;
/// Stone Gaze (the executable's spell number): its handler glows the stack, which then looks
/// petrified while the spell lasts (0x5A1121, `draw_battle_stack` 0x43E3CB; E17 M5 T5b1).
pub const STONE_GAZE: u16 = 70;
/// A glow's steps: Bloodlust's strength 0.0 to 0.9 then 1.0 to 0.1 (0x5A12A3 to 0x5A1311),
/// Stone Gaze's 1.0 to 0.1 (0x5A1199 to 0x5A11CF), in tenths.
const BLOODLUST_STEPS: [u8; 20] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
const STONE_STEPS: [u8; 10] = [10, 9, 8, 7, 6, 5, 4, 3, 2, 1];

/// The magic animation of a spell resisted by the stack it was aimed at (`cast_spell` 0x5A2043:
/// number 0x4E over it, 100 ms a frame, without its hit; E17 M3 T7b-1).
pub const RESISTED: u8 = 78;

/// The magic animations of the rolled effects over one stack (design §7.16, player 1; E17 M3
/// T7b-2b-2b), 100 ms a frame each: good morale 0x14 without the stack's hit (`good_morale_roll`
/// 0x464A84), bad morale 0x1E and Fear 0xF with it (`bad_morale_check` 0x464D10, `fear_check`
/// 0x464F33), good luck 0x12 without it (`melee_strike` 0x4418B3, `stack_shot` 0x43F9D1).
pub const GOOD_MORALE: u8 = 20;
/// Bad morale's ([`GOOD_MORALE`]).
pub const BAD_MORALE: u8 = 30;
/// Fear's ([`GOOD_MORALE`]).
pub const FEAR: u8 = 15;
/// Good luck's ([`GOOD_MORALE`]).
pub const GOOD_LUCK: u8 = 18;

/// The magic animations of the striker's effects and the spells after a strike (design §7.16;
/// E17 M3 T7b-2b-3), over one stack (player 1) unless named: the bolt from the sky 1 (`C11spA0`,
/// one frame at [`BOLT_TIME`]: Lightning Bolt's 0x5A0B15 and the Thunderbird's 0x441315), then
/// [`THUNDERBOLT`] over the marked stack (player 2); Life Drain 0x4A over the drinkers (0x440E83),
/// Death Stare 0x50 (0x44113F) and the Rust Dragon's acid 0x51 (0x44156A) over the stack struck,
/// each 100 ms a frame without its hit.
pub const BOLT: u8 = 1;
/// The bolt from the sky's frame time, ms ([`BOLT`]).
pub const BOLT_TIME: u32 = 10;
/// The Thunderbird's bolt over the marked stack (`strike_animation` 0x44135B: 0x31, Lightning
/// Bolt's own; [`BOLT`]).
pub const THUNDERBOLT: u8 = 49;
/// Life Drain's ([`BOLT`]).
pub const LIFE_DRAIN: u8 = 74;
/// Death Stare's ([`BOLT`]).
pub const DEATH_STARE: u8 = 80;
/// The acid's ([`BOLT`]).
pub const ACID: u8 = 81;

/// The executable's numbers of the spells of the strikes' effects (E17 M3 T7b-2b-3, design
/// §7.16): Lightning Bolt and the Titans' (the bolt from the sky, 0x5A0B0C), Poison (its handler
/// plays its animation with the stack's hit, 0x5A15AF), Death Stare (its sound for the striker's
/// effect, 0x44111A), Acid Breath (its handler lowers the defense by 3 and writes the executable's
/// own line, 0x5A1E70; its sound and name for the acid, 0x44150E).
pub const LIGHTNING_BOLT: u16 = 17;
/// The Titans' Lightning Bolt ([`LIGHTNING_BOLT`]).
pub const TITAN_BOLT: u16 = 57;
/// Poison ([`LIGHTNING_BOLT`]).
pub const POISON: u16 = 71;
/// Death Stare ([`LIGHTNING_BOLT`]).
pub const DEATH_STARE_SPELL: u16 = 79;
/// Acid Breath ([`LIGHTNING_BOLT`]).
pub const ACID_BREATH: u16 = 80;
/// The spells a melee strike leaves pending on the stack it struck (`strike_spell_choice`
/// 0x440500 sets +0xEC: Curse, Blind, Stone Gaze, Poison, Bind, Disease, Paralyze, Age, Acid
/// Breath), cast as a creature's at the end of the strike's animation, before the dead die
/// (`strike_animation` 0x469102); the striker's own effects come after the strike's line
/// (`striker_effects` 0x440BC0).
const PENDING: [u16; 9] = [42, 62, 70, 71, 72, 73, 74, 75, 80];
/// Acid Breath's lowering of the defense (0x5A1E76).
const ACID_DEFENSE: i32 = 3;

/// The executable's numbers of the spells aimed at no stack whose handlers the viewer tells apart
/// (E17 M3 T7c-1, design §7.16): the area spells play their animation at the hex aimed at, then
/// their damage's line and the reactions (`area_spell_handler` 0x5A4970); Death Ripple writes its
/// cast's and its damage's lines, then plays its animation over every stack struck with their
/// reactions (0x5A0C81); Destroy Undead its cast's line, the animation, then its damage's line
/// (0x5A0E7E).
pub const FROST_RING: u16 = 20;
/// Fireball ([`FROST_RING`]).
pub const FIREBALL: u16 = 21;
/// Inferno ([`FROST_RING`]).
pub const INFERNO: u16 = 22;
/// Meteor Shower ([`FROST_RING`]).
pub const METEOR_SHOWER: u16 = 23;
/// Death Ripple ([`FROST_RING`]).
pub const DEATH_RIPPLE: u16 = 24;
/// Destroy Undead ([`FROST_RING`]).
pub const DESTROY_UNDEAD: u16 = 25;
/// Armageddon (E17 M3 T7c-2, design §7.16): its handler's own loop tiles its animation over the
/// whole battlefield with every stack's reaction (`mass_damage_handler` 0x5A4BC0), then writes its
/// damage's line.
pub const ARMAGEDDON: u16 = 26;
/// The spells whose handlers place battlefield objects (E17 M3 T7c-2, design §7.16): each object
/// its animation at a hex, kept until the object shows (Quicksand 0x5A0354 and Land Mine 0x5A0536
/// at each object's hex with their own sound, Fire Wall 0x5A0821 at the hex aimed at for each, a
/// redraw shown after each; Force Field 0x5A0759 once, shown with the next redraw), then the
/// cast's line (0x5A0817, 0x5A04FF).
pub const QUICKSAND: u16 = 10;
/// Land Mine ([`QUICKSAND`]).
pub const LAND_MINE: u16 = 11;
/// The magic animation over the stack a Land Mine hurts (`land_mine_trigger` 0x59FC78,
/// `strike_animation(57, 1)`; E17 M4 T2c-1).
pub const MINE_BLAST: u8 = 57;
/// Force Field ([`QUICKSAND`]).
pub const FORCE_FIELD: u16 = 12;
/// Earthquake (E17 M4 T2c-2, design §10.2): its handler shakes the screen and plays the
/// explosions over the walls it hits (`earthquake` 0x5A7C80; [`Timeline::shake`]).
pub const EARTHQUAKE: u16 = 14;
/// Fire Wall ([`QUICKSAND`]).
pub const FIRE_WALL: u16 = 13;

/// A battlefield object a spell places, as the scene draws it (E17 M3 T7c-2, design §4.4,
/// §7.16): drawn as a usual obstacle in its layer's pass at its anchor, group 0, the frame the
/// idle steps' count modulo `frames` (`battle_redraw` 0x494680).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Object {
    /// Its DEF.
    pub def: String,
    /// The frames of its group 0.
    pub frames: u32,
    /// Its height in hex rows: its canvas rises 42 pixels a row above its anchor hex's box.
    pub rows: u8,
    /// Drawn in the pass before every stack (its layer 1), else in its anchor row's.
    pub before_stacks: bool,
    /// The hexes it blocks (Force Field's: a count box does not take them, 0x43E605).
    pub blocked: Vec<u8>,
    /// The side that alone sees it while it is not revealed (a Quicksand pit's or a Land Mine's
    /// placed out of the other side's sight, BAT-MAG-031; E17 M5 T2b): it shows only when that
    /// side is the screen's ([`Timeline::screen`]); none, both see it.
    pub owner: Option<Side>,
}

/// Whether a screen that shows `screen`'s hidden objects shows `object` (none: every object; E17
/// M5 T2b, [`Timeline::screen`]).
fn sees(screen: Option<Side>, object: &Object) -> bool {
    screen.is_none() || object.owner.is_none_or(|owner| Some(owner) == screen)
}

/// The other side.
const fn opponent(side: Side) -> Side {
    match side {
        Side::Attacker => Side::Defender,
        Side::Defender => Side::Attacker,
    }
}

/// A battlefield object placed at `anchor` from `shown` on, or (`object` none) gone from then
/// (E17 M3 T7c-2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectKey {
    /// When it shows.
    pub shown: u32,
    /// Its anchor hex.
    pub anchor: u8,
    /// The object, none once removed.
    pub object: Option<Object>,
}

/// A frame of the magic animation over the whole battlefield (Armageddon's, design §7.16, player
/// 6; E17 M3 T7c-2), or (`frame` none) the battlefield shown without it: tiled from the
/// battlefield's top-left over its rectangle, plain, from `shown`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldKey {
    /// When it shows.
    pub shown: u32,
    /// The frame of group 0, none once gone.
    pub frame: Option<u32>,
    /// The DEF.
    pub def: String,
}

/// The magic animation over the whole battlefield as it shows at a time (E17 M3 T7c-2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// The DEF.
    pub def: String,
    /// The frame of group 0.
    pub frame: u32,
}

/// The count a stack's box shows from `shown` on (E17 M3 T5b, design §4.3): a strike's kills
/// show at its end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CountKey {
    /// The stack's id.
    pub stack: u8,
    /// When it shows.
    pub shown: u32,
    /// The count.
    pub count: u32,
}

/// A frame of a magic animation drawn over stacks, or (`frame` none) its erasure: drawn at
/// `drawn`, shown from `shown` (E17 M3 T3b).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MagicKey {
    /// Its number.
    pub animation: u8,
    /// When it is drawn.
    pub drawn: u32,
    /// When it shows.
    pub shown: u32,
    /// The frame of group 0, none once erased.
    pub frame: Option<u32>,
    /// The DEF.
    pub def: String,
    /// Drawn by the translucent drawer.
    pub translucent: bool,
    /// The stacks it is drawn over, each with the canvas's top-left corner and whether mirrored.
    pub over: Vec<(u8, [i32; 2], bool)>,
}

/// A magic animation as it shows at a time: its frame over each of its stacks, drawn after the
/// stack's sprite (`draw_battle_stack`'s tail 0x43E795; E17 M3 T3b).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Magic {
    /// The DEF.
    pub def: String,
    /// The frame of group 0.
    pub frame: u32,
    /// Drawn by the translucent drawer.
    pub translucent: bool,
    /// The stacks, each with the canvas's top-left corner and whether mirrored.
    pub over: Vec<(u8, [i32; 2], bool)>,
}

/// A frame of a magic animation at a hex (`play_magic_animation_at_hex` 0x496A10, design §7.16,
/// player 5; E17 M3 T7c-1), or (`frame` none) its erasure: shown from `shown`, drawn over everything
/// at its canvas's top-left `at`. Memory does not show it (the harness's timelines leave it out).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpotKey {
    /// Its number.
    pub animation: u8,
    /// When it shows.
    pub shown: u32,
    /// The frame of group 0, none once erased.
    pub frame: Option<u32>,
    /// The DEF.
    pub def: String,
    /// Drawn by the translucent drawer.
    pub translucent: bool,
    /// The canvas's top-left.
    pub at: [i32; 2],
}

/// A magic animation at a hex as it shows at a time (E17 M3 T7c-1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spot {
    /// The DEF.
    pub def: String,
    /// The frame of group 0.
    pub frame: u32,
    /// Drawn by the translucent drawer.
    pub translucent: bool,
    /// The canvas's top-left.
    pub at: [i32; 2],
}

impl MagicKey {
    /// The magic animation it shows (none: erased).
    fn shows(&self) -> Option<Magic> {
        self.frame.map(|frame| Magic {
            def: self.def.clone(),
            frame,
            translucent: self.translucent,
            over: self.over.clone(),
        })
    }
}

impl SpotKey {
    /// The magic animation at a hex it shows (none: erased).
    fn shows(&self) -> Option<Spot> {
        self.frame.map(|frame| Spot {
            def: self.def.clone(),
            frame,
            translucent: self.translucent,
            at: self.at,
        })
    }
}

impl FieldKey {
    /// The magic animation over the battlefield it shows (none: erased).
    fn shows(&self) -> Option<Field> {
        self.frame.map(|frame| Field {
            def: self.def.clone(),
            frame,
        })
    }
}

impl Stage<'_> {
    /// The anchor of `hex` (§4.1).
    fn anchor(&self, hex: u8) -> [i32; 2] {
        let width = self.width.max(1);
        self.layout.anchor(hex / width, hex % width)
    }

    /// The row of `hex`.
    fn row(&self, hex: u8) -> u8 {
        hex / self.width.max(1)
    }

    /// The neighbour of `hex` in direction `d` (0 up and right, then clockwise: right, down and
    /// right, down and left, left, up and left; the executable's table at +0x13468): the hex of
    /// the rows around whose anchor lies a column's width, or half of it and a row's height,
    /// that way. None off the field's columns or above it.
    fn neighbor(&self, hex: u8, d: u8) -> Option<u8> {
        let width = self.width.max(1);
        let (row, column) = (hex / width, hex % width);
        let [x, y] = self.anchor(hex);
        let [w, h] = self.layout.hex_step;
        let want = match d {
            0 => [x + w / 2, y - h],
            1 => [x + w, y],
            2 => [x + w / 2, y + h],
            3 => [x - w / 2, y + h],
            4 => [x - w, y],
            _ => [x - w / 2, y - h],
        };
        (row.saturating_sub(1)..=row.saturating_add(1))
            .flat_map(|r| {
                (column.saturating_sub(1)..=column.saturating_add(1)).map(move |c| (r, c))
            })
            .filter(|&(_, c)| c < width)
            .find(|&(r, c)| self.layout.anchor(r, c) == want)
            .and_then(|(r, c)| u8::try_from(u16::from(r) * u16::from(width) + u16::from(c)).ok())
    }
}

/// What a stack shows: a frame of a DEF group on a hex, moved by an offset in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pose {
    /// The DEF group.
    pub group: u32,
    /// The frame in it.
    pub frame: u32,
    /// The stack's position as the executable keeps it (+0x38): during a walk the hex it is
    /// leaving, during a flight its start until it lands.
    pub hex: u8,
    /// Where it is drawn from its hex's anchor (+0x104, +0x100).
    pub offset: [i32; 2],
    /// Drawn mirrored: it faces left (+0x44 = 0; E17 M2 T4b-2a, turns).
    pub mirrored: bool,
    /// Its draw layer (+0x8), the redraw's pass that draws it in its row (§4.6): 4 standing, 3 or
    /// 7 walking up or down, 6 a melee's striker, 5 a struck stack, 1 among the corpses; or
    /// [`FLYING`], over every row.
    pub layer: u8,
    /// Its count box hidden (+0x30, H3API's `skipCountDraw`; E17 M3 T5a, design §4.3): set
    /// through a walk, from before its first turn to its end (`stack_walk` 0x442145, 0x4423D9),
    /// and through a flight, from its cycles to the standing frame after it lands
    /// (`stack_flight_animation` 0x4B4BD8, 0x4B4E73).
    pub boxless: bool,
}

impl Pose {
    /// Whether it holds its hex against another stack's count box (0x43E605 to 0x43E627): it
    /// stands on the hex records, which a death leaves at the strike's end (`stack_death`
    /// 0x444400, from `strike_animation` 0x4692C4; from then among the corpses), and its box is
    /// not hidden.
    #[must_use]
    pub const fn holds_hex(&self) -> bool {
        !self.boxless && self.layer != CORPSE && self.layer != GONE
    }
}

/// One pose of one stack: drawn at `drawn` (the harness reads it from then, design §7.1), shown
/// from `shown`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    /// The stack's id.
    pub stack: u8,
    /// When the frame is drawn.
    pub drawn: u32,
    /// When it shows.
    pub shown: u32,
    /// The pose.
    pub pose: Pose,
}

/// A stack the timeline animates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Member {
    /// Its id.
    pub stack: u8,
    /// Its side.
    pub side: Side,
    /// Its place among its side's stacks (the harness's record index).
    pub record: u8,
    /// Its pose at the start.
    pub start: Pose,
}

/// A command's events and when it was given (ms after the record's first snapshot; `None`:
/// the default, [`timeline`]).
#[derive(Debug, Clone, Copy)]
pub struct Command<'a> {
    /// When the command was given.
    pub at: Option<u32>,
    /// What it did, as the viewer saw it.
    pub events: &'a [Event],
}

/// The commands of a record's messages: each `Events` message one command, without a time (the
/// first given at the snapshot's moment, each later one when the previous one ended, [`timeline`]).
pub fn commands(messages: &[ServerMessage]) -> Vec<Command<'_>> {
    messages
        .iter()
        .filter_map(|m| match m {
            ServerMessage::Events { events, .. } => Some(Command { at: None, events }),
            _ => None,
        })
        .collect()
}

/// A creature's sounds, in the order of a stack's eight (+0x170 to +0x18C, loaded by
/// `stack_load_graphics` 0x43D9F0; E17 M2 T5b, design §8.2). Which file each is, and whether a
/// creature has it, the asset tables say.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Sound {
    /// Moving: started with a move's first cycle, looping until the move ends.
    Move,
    /// A melee strike.
    Attack,
    /// Struck.
    Wince,
    /// A shot.
    Shot,
    /// Dying.
    Death,
    /// Struck while defending.
    Defend,
    /// Before a move (group 20).
    Start,
    /// After a move (group 21).
    End,
    /// The First Aid Tent's heal, cued on its patient: the battle's sound, not the creature's
    /// (the asset tables' `heal_sound`, category 2: `play_sound_file` 0x59A470 from
    /// `first_aid_tent_action` 0x478C46; E17 M3 T3b).
    Heal,
    /// A spell resisted by the stack it was aimed at, cued on that stack: the battle's sound (the
    /// asset tables' `resist_sound`, `play_sound_file` from `cast_spell` 0x5A1F98; E17 M3
    /// T7b-1).
    Resisted,
    /// A spell's own sound (its handle), cued on its target: the spell record's (the asset
    /// tables' spell `sound`, started after the resistance roll, 0x5A0331; E17 M3 T7b-1).
    Spell(u16),
    /// Good morale, cued on its stack: the battle's sound (the asset tables' `good_morale_sound`,
    /// category 2 as the heal's: `play_sound_file` from `good_morale_roll` 0x464A71; E17 M3
    /// T7b-2b-2b).
    GoodMorale,
    /// Bad morale, cued on its stack (`bad_morale_sound`, `play_sound_file` from
    /// `bad_morale_check` 0x464C89).
    BadMorale,
    /// Fear, cued on the frightened stack (`fear_sound`, `play_sound_file` from `fear_check`
    /// 0x464E98).
    Fear,
    /// Good luck, cued on the striker or shooter (`luck_sound`, category 3 as a spell's:
    /// `play_sound_async` from `melee_strike` 0x441860 and `stack_shot` 0x43F97E).
    Luck,
    /// Life Drain, cued on the drinkers (`drain_sound`, category 2: `play_sound_file` from
    /// `striker_effects` 0x440E6D; E17 M3 T7b-2b-3).
    Drain,
    /// The Thunderbird's bolt, cued on the stack struck (`thunderbolt_sound`, `play_sound_file`
    /// from 0x4412FF).
    Thunderbolt,
    /// The sound a spell's handler plays with each object it places (its handle: the asset
    /// tables' spell `object_sound`, `play_sample` 0x5A040A, Land Mine's 0x5A061A; E17 M3
    /// T7c-2), cued on no stack (stack 0).
    Object(u16),
    /// A spell's own sound (its handle) started as a file, category 2, cued on the stack struck:
    /// Death Stare's (`play_sound_file` 0x441126) and the Rust Dragon's acid (0x44151A).
    SpellFile(u16),
    /// A turret's shot, cued on its stack: its shooter creature's shot sound started as a file,
    /// category 2 (`play_sound_file` from `turret_shot` 0x465C6D; E17 M4 T2b).
    TurretShot,
    /// A wall shot's landing with damage, cued on its shooter: the battle's sound by name
    /// (`WallHit.82m`, `wav_load` 0x446199, `play_sample` 0x4462C8: category 0, volume 127).
    WallHit,
    /// A wall shot's landing without damage (`WallMiss.82m`), as [`Sound::WallHit`].
    WallMiss,
    /// The drawbridge moving, cued on no stack (stack 0): the battle's sound played as a file,
    /// category 2 (the asset tables' `siege_sounds`, `play_sound_file` from `gate_open` 0x46723B
    /// and `gate_close` 0x4673D2; E17 M4 T2c-1).
    Drawbridge,
    /// A Land Mine's blast, cued on the stack it hurts (`landkill.wav`, `play_sound_file` from
    /// `land_mine_trigger` 0x59FC63), as [`Sound::Drawbridge`].
    Mine,
}

/// A sound a stack starts at a time, or (`stop`) the move sound it ends then (E17 M2 T5b; the
/// sites: design §8.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cue {
    /// When: the time of the frame it goes with.
    pub at: u32,
    /// The stack.
    pub stack: u8,
    /// Its sound.
    pub sound: Sound,
    /// Stops the sound instead of starting it.
    pub stop: bool,
}

/// The poses of a record's stacks over time.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Timeline {
    /// The stacks living at the start, by id.
    pub members: Vec<Member>,
    /// Every pose taken, in the order drawn.
    pub keys: Vec<Key>,
    /// The active stack from a time on (its turn started then).
    pub turns: Vec<(u32, u8)>,
    /// Each turn's acting side, in the order of `turns` (E17 M5 T5b2, R3-1): the active stack's
    /// controller as `stack_activation` 0x465330 sets +0x132C0, the other side while Hypnotize
    /// holds it, a created stack's side.
    pub controllers: Vec<Side>,
    /// The projectiles' flights, in order.
    pub flights: Vec<Flight>,
    /// The rays, in order.
    pub beams: Vec<Beam>,
    /// The magic animations' frames and erasures, in order (E17 M3 T3b).
    pub magics: Vec<MagicKey>,
    /// The magic animations' frames and erasures at a hex, in order (E17 M3 T7c-1).
    pub spots: Vec<SpotKey>,
    /// The battlefield objects placed and gone, in order (E17 M3 T7c-2): the snapshot's at 0.
    pub objects: Vec<ObjectKey>,
    /// The frames and erasures of the magic animation over the whole battlefield, in order (E17
    /// M3 T7c-2).
    pub fields: Vec<FieldKey>,
    /// The heroes' frames the commands draw, in order (E17 M3 T7b-1: their casts).
    pub hero_keys: Vec<HeroKey>,
    /// The counts the stacks' boxes show from a time on, in order (E17 M3 T5b); before its
    /// first, a stack's box shows the snapshot's count.
    pub counts: Vec<CountKey>,
    /// The hues the stacks' boxes take from a time on by their spells, in order (E17 M3 T7b-2a).
    pub tints: Vec<TintKey>,
    /// The stacks' palette effects by their spells from a time on, in order (E17 M5 T5b1).
    pub looks: Vec<LookKey>,
    /// The stack outlined as active from a time on (none: no outline), in order (§7.9).
    pub outlines: Vec<(u32, Option<u8>)>,
    /// When each idle step ran, up to the last command's start (none runs while a command
    /// plays); [`Timeline::idle`] goes on from its end.
    pub steps: Vec<u32>,
    /// The idle steps after the last command.
    pub idle: Idle,
    /// The viewer's own fidgets as they stand after the last command (none: no fidgets, as the
    /// reference checks take the original's).
    pub fidgets: Option<Fidgets>,
    /// The creatures' sounds, in time order.
    pub cues: Vec<Cue>,
    /// Each command's end and what it did that the heroes answer, in order (E17 M3 T3a).
    pub actions: Vec<Action>,
    /// The heroes beside the battlefield (none: none drawn; [`Timeline::with_heroes`]).
    pub heroes: Option<Heroes>,
    /// What the battle writes into its log and does to it, with its time, in order (E17 M3 T6b,
    /// design §9.8; `crate::battle_log::texts`).
    pub log: Vec<(u32, Logged)>,
    /// When the battle is decided and how (E17 M3 T8, design §8.2): the end of the command whose
    /// events end it, when `battle_end` plays the result's tune ([`crate::sound::end_tune`]).
    pub ended: Option<(u32, Outcome)>,
    /// A siege's turret records' poses, in the order drawn (E17 M4 T2b): each built turret's
    /// standing at 0.
    pub turrets: Vec<TurretKey>,
    /// A siege's walls' states, in the order drawn (E17 M4 T2b): the snapshot's at 0.
    pub walls: Vec<WallKey>,
    /// A siege's wall shots' explosions, in order (E17 M4 T2b).
    pub blasts: Vec<BlastKey>,
    /// The spans in which the screen shows the drawbridge's rectangle alone (E17 M4 T2c-1; the
    /// original's, CLT-Q23).
    pub bridges: Vec<BridgeKey>,
    /// Earthquake's shakes and the mouse hidden through them (E17 M4 T2c-2).
    pub quakes: Vec<QuakeKey>,
    /// Whether each side's player is a human here (the snapshot's `human`; E17 M5 T2b): the
    /// screen shows the side to act's hidden objects ([`Timeline::screen`]).
    pub humans: [bool; 2],
    /// The dissolves, in order (E17 M5 T5a).
    pub dissolves: Vec<DissolveKey>,
    /// The stacks the battle created, in order (E17 M5 T5a): a reused id's later key replaces its
    /// earlier one.
    pub created: Vec<CreatedKey>,
    /// The message boxes the battle opens, with their times, in order (E17 M5 T5a).
    pub notices: Vec<(u32, Notice)>,
    /// When the last frame shows.
    pub end: u32,
}

/// The viewer's own fidgets (§7.2, E17 M2 T4b-2a). The executable draws them from its rules'
/// generator, which the engine does not share (ADR-0003): the viewer draws them from a generator
/// of its own (the C library's `rand()` sequence, *provisional*), so they cannot match the
/// original's frame for frame. At the battle's start and after every action each stack's fidget
/// clock is drawn again: its fidget starts at the first idle step after now + 2r, r =
/// random(50, t), t its fidget interval; then one frame of group 1 per idle step, each advance
/// stalled when a draw of 0 to 100 gives less than 8; after its last frame it stands, the next
/// fidget 0.75 t to 1.25 t later (0x4960D0). A stack the action drew stops fidgeting; not read:
/// the blind, petrified and paralysed stacks the executable skips.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fidgets {
    rng: u32,
    group: u8,
    stacks: Vec<Fidget>,
}

/// One stack's fidget: its interval and frames, its clock (+0xFC), the frame it shows (none:
/// standing) and its standing pose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Fidget {
    stack: u8,
    interval: u32,
    frames: u32,
    clock: i64,
    frame: Option<u32>,
    base: Pose,
}

impl Fidgets {
    /// The C library's `rand()`: 15 bits of a linear congruential sequence.
    fn rand(&mut self) -> u32 {
        self.rng = self.rng.wrapping_mul(214_013).wrapping_add(2_531_011);
        (self.rng >> 16) & 0x7FFF
    }

    /// A draw from `low` to `high` (`low` ≤ `high`).
    fn between(&mut self, low: u32, high: u32) -> u32 {
        low + self.rand() % (high - low + 1)
    }

    /// After an action at `now` (and at the start): the stacks still living (`base`: their
    /// standing poses, by id) keep their fidgets, those `drawn` stop; each clock drawn again.
    fn reset(&mut self, now: u32, base: &BTreeMap<u8, Pose>, drawn: &[u8]) {
        self.stacks.retain(|f| base.contains_key(&f.stack));
        for i in 0..self.stacks.len() {
            let f = self.stacks[i];
            let mut next = Fidget {
                base: base.get(&f.stack).copied().unwrap_or(f.base),
                frame: f.frame.filter(|_| !drawn.contains(&f.stack)),
                ..f
            };
            if f.interval > 51 {
                let r = self.between(50, f.interval);
                next.clock = i64::from(now) + 2 * i64::from(r) - i64::from(f.interval);
            }
            self.stacks[i] = next;
        }
    }

    /// The idle step at `now`: the poses that change.
    fn step(&mut self, now: u32) -> Vec<(u8, Pose)> {
        let mut out = Vec::new();
        for i in 0..self.stacks.len() {
            let mut f = self.stacks[i];
            match f.frame {
                Some(k) => {
                    if self.between(0, 100) < 8 {
                        continue;
                    }
                    if k + 1 < f.frames {
                        f.frame = Some(k + 1);
                    } else {
                        f.frame = None;
                        let r = i64::from(self.between(0, f.interval));
                        f.clock = (4 * i64::from(now) + 2 * r - i64::from(f.interval)) / 4;
                    }
                }
                None if i64::from(now) - f.clock > i64::from(f.interval) => f.frame = Some(0),
                None => continue,
            }
            let pose = f.frame.map_or(f.base, |frame| Pose {
                group: u32::from(self.group),
                frame,
                ..f.base
            });
            out.push((f.stack, pose));
            self.stacks[i] = f;
        }
        out
    }
}

/// The battle loop's idle steps (0x4960D0, §7.1, §7.9): while the loop ticks (from `from`), a
/// step runs whenever the time reached `next`, which then moves on to the old one plus
/// max(the time since it, 100 ms): on the grid while idle, twice at once after an animation
/// held them up by 100 ms or more.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Idle {
    /// The loop ticks from then.
    pub from: u32,
    /// The next step's time (0x6989B8).
    pub next: u32,
}

impl Idle {
    /// Runs the steps up to `until` (included), each time given to `step`.
    pub(crate) fn run(&mut self, until: u32, mut step: impl FnMut(u32)) {
        loop {
            let now = self.next.max(self.from);
            if now > until {
                return;
            }
            step(now);
            self.next += (now - self.next).max(IDLE_STEP);
        }
    }

    /// One step at `now` if it is due (the computer acts on its turn's first tick); whether
    /// one ran.
    fn once(&mut self, now: u32, steps: &mut Vec<u32>) -> bool {
        let due = self.next <= now;
        if due {
            steps.push(now);
            self.next += (now - self.next).max(IDLE_STEP);
        }
        due
    }
}

impl Timeline {
    /// The index of the first cue at or after `t`: a player that seeks to `t` goes on from there.
    #[must_use]
    pub fn cue_from(&self, t: u32) -> usize {
        self.cues.partition_point(|c| c.at < t)
    }

    /// Each stack's pose shown at `t`.
    pub fn at(&self, t: u32) -> BTreeMap<u8, Pose> {
        let mut poses: BTreeMap<u8, Pose> =
            self.members.iter().map(|m| (m.stack, m.start)).collect();
        for key in self.keys.iter().filter(|k| k.shown <= t) {
            poses.insert(key.stack, key.pose);
        }
        // After the last command the idle steps go on, and the fidgets with them.
        if let Some(fidgets) = &self.fidgets {
            let (mut idle, mut fidgets) = (self.idle, fidgets.clone());
            idle.run(t, |now| poses.extend(fidgets.step(now)));
        }
        poses
    }

    /// Whether the mouse's cursor is hidden at `t`: while a ray's drawer hides it (E17 M3 T6c-2,
    /// [`Beam::cursor`]).
    #[must_use]
    pub fn cursor_hidden(&self, t: u32) -> bool {
        self.beams
            .iter()
            .any(|b| (b.cursor[0]..b.cursor[1]).contains(&t))
            || self.quakes.iter().any(|q| (q.from..q.until).contains(&t))
    }

    /// The projectile showing at `t`, if one flies then.
    pub fn projectile(&self, t: u32) -> Option<Projectile> {
        self.flights.iter().find_map(|f| f.at(t))
    }

    /// The ray showing at `t`, if one is drawn then: the strokes of the ticks shown by then.
    pub fn ray(&self, t: u32) -> Option<Ray> {
        self.beams.iter().find_map(|b| b.at(t))
    }

    /// The magic animation showing at `t`, if one is drawn then (E17 M3 T3b).
    pub fn magic(&self, t: u32) -> Option<Magic> {
        self.magics.iter().rev().find(|k| k.shown <= t)?.shows()
    }

    /// Each turret record's pose shown at `t` (E17 M4 T2b; none: a record no turret sets up).
    pub fn turrets(&self, t: u32) -> [Option<TurretPose>; 3] {
        [0, 1, 2].map(|record| {
            self.turrets
                .iter()
                .rev()
                .find(|k| k.shown <= t && k.record == record)
                .map(|k| k.pose)
        })
    }

    /// A siege's walls' states shown at `t` (E17 M4 T2b; none: no siege with a fort).
    pub fn walls(&self, t: u32) -> Option<Walls> {
        self.walls
            .iter()
            .rev()
            .find(|k| k.shown <= t)
            .map(|k| k.walls)
    }

    /// The dissolve blending at `t` and its step (E17 M5 T5a), if any.
    #[must_use]
    pub fn dissolving(&self, t: u32) -> Option<(&DissolveKey, u8)> {
        self.dissolves
            .iter()
            .find_map(|d| d.step_at(t).map(|step| (d, step)))
    }

    /// The created stacks the screen draws at `t`, by id (E17 M5 T5a): each id's last created key
    /// shown by then.
    #[must_use]
    pub fn created_at(&self, t: u32) -> BTreeMap<u8, Stack> {
        self.created
            .iter()
            .filter(|k| k.shown <= t)
            .map(|k| (k.member.stack, k.stack.clone()))
            .collect()
    }

    /// Stack `stack`'s record and start (a created stack's first key's, E17 M5 T5a).
    #[must_use]
    pub fn member(&self, stack: u8) -> Option<&Member> {
        self.all_members().find(|m| m.stack == stack)
    }

    /// The stack in `side`'s record `record` (a created stack's too, E17 M5 T5a).
    #[must_use]
    pub fn member_at(&self, side: Side, record: u8) -> Option<&Member> {
        self.all_members()
            .find(|m| m.side == side && m.record == record)
    }

    /// The stacks living at the start, then the created ones in the order created.
    fn all_members(&self) -> impl Iterator<Item = &Member> {
        self.members
            .iter()
            .chain(self.created.iter().map(|k| &k.member))
    }

    /// While the screen shows the drawbridge's rectangle alone at `t` (E17 M4 T2c-1, the
    /// original's: CLT-Q23), the time whose screen it shows elsewhere: the one before the
    /// drawbridge's first redraw showed.
    pub fn bridge(&self, t: u32) -> Option<u32> {
        self.bridges
            .iter()
            .find(|b| b.from <= t && t < b.until)
            .map(|b| b.from.saturating_sub(1))
    }

    /// A wall shot's explosion showing at `t` (E17 M4 T2b): its last frame shown by then, and
    /// unless each frame is drawn on a restored background, the frames before it.
    pub fn blast(&self, t: u32) -> Option<Blast> {
        let last = self.blasts.iter().rev().find(|k| k.shown <= t)?;
        let frame = last.frame?;
        let frames = if last.restored {
            vec![(frame, last.walls)]
        } else {
            self.blasts
                .iter()
                .filter(|k| k.blast == last.blast && k.shown <= t)
                .filter_map(|k| k.frame.map(|f| (f, k.walls)))
                .collect()
        };
        Some(Blast {
            def: last.def.clone(),
            landings: last.landings.clone(),
            frames,
        })
    }

    /// How many of Earthquake's shake's steps have shown at `t`, if it shakes then (E17 M4
    /// T2c-2): the screen saved at its start shows shifted by each offset in turn
    /// ([`shake_offsets`]).
    pub fn shake(&self, t: u32) -> Option<u32> {
        self.quakes.iter().find_map(|q| q.steps(t))
    }

    /// The battlefield objects showing at `t`, by anchor (E17 M3 T7c-2).
    pub fn objects(&self, t: u32) -> BTreeMap<u8, Object> {
        let mut shown = BTreeMap::new();
        for k in self.objects.iter().filter(|k| k.shown <= t) {
            match &k.object {
                Some(o) => shown.insert(k.anchor, o.clone()),
                None => shown.remove(&k.anchor),
            };
        }
        let screen = self.screen(t);
        shown.retain(|_, o| sees(screen, o));
        shown
    }

    /// The side whose hidden objects the screen shows at `t` (E17 M5 T2b; `battle_redraw`
    /// 0x49460A to 0x49463A, design §7.16): the side to act when a human here plays it, else the
    /// other one; none when no human plays here (a record holds only what its viewer sees: every
    /// object shows). The side to act is the acting side +0x132C0 its turn's start set (E17 M5
    /// T5b2: [`Timeline::controllers`], a hypnotized stack's controller), before the first turn
    /// the opening's active stack's side.
    pub fn screen(&self, t: u32) -> Option<Side> {
        if !self.humans.contains(&true) {
            return None;
        }
        let i = self.turns.partition_point(|(at, _)| *at <= t);
        let side = match i.checked_sub(1) {
            Some(i) => self.controllers.get(i).copied()?,
            None => {
                let stack = self.outlines.first().and_then(|(_, s)| *s);
                self.member(stack?)?.side
            }
        };
        let human = self.humans[usize::from(side == Side::Defender)];
        Some(if human { side } else { opponent(side) })
    }

    /// The magic animation over the whole battlefield showing at `t`, if one is drawn then (E17
    /// M3 T7c-2).
    pub fn field(&self, t: u32) -> Option<Field> {
        self.fields.iter().rev().find(|k| k.shown <= t)?.shows()
    }

    /// The magic animation at a hex showing at `t`, if one is drawn then (E17 M3 T7c-1).
    pub fn spot(&self, t: u32) -> Option<Spot> {
        self.spots.iter().rev().find(|k| k.shown <= t)?.shows()
    }

    /// The counts the stacks' boxes show at `t`, by id, of the stacks whose count changed by then
    /// (E17 M3 T5b).
    pub fn counts(&self, t: u32) -> BTreeMap<u8, u32> {
        self.counts
            .iter()
            .filter(|k| k.shown <= t)
            .map(|k| (k.stack, k.count))
            .collect()
    }

    /// The hues of the boxes of the stacks with active spells at `t`, by id (E17 M3 T7b-2a).
    pub fn hues(&self, t: u32) -> BTreeMap<u8, u32> {
        let mut hues = BTreeMap::new();
        for k in self.tints.iter().filter(|k| k.shown <= t) {
            match k.hue {
                Some(hue) => hues.insert(k.stack, hue),
                None => hues.remove(&k.stack),
            };
        }
        hues
    }

    /// The palette effects by their spells of the stacks at `t`, by id (E17 M5 T5b1).
    pub fn looks(&self, t: u32) -> BTreeMap<u8, Effect> {
        let mut looks = BTreeMap::new();
        for k in self.looks.iter().filter(|k| k.shown <= t) {
            match k.look {
                Some(look) => looks.insert(k.stack, look),
                None => looks.remove(&k.stack),
            };
        }
        looks
    }

    /// The stack outlined as active at `t` (§7.9): the snapshot's active stack until the first
    /// command's first frame shows; none while a command plays; a turn's stack from its start
    /// when a player acts next, none when the computer does (*provisional*).
    pub fn outlined(&self, t: u32) -> Option<u8> {
        self.outlines
            .iter()
            .rev()
            .find(|(at, _)| *at <= t)
            .and_then(|(_, stack)| *stack)
    }

    /// Whether nothing shown changes after the last frame: no stack outlined then, none left
    /// that fidgets (a window need not look further; E17 M2 T5c, the reviewer).
    pub fn still(&self) -> bool {
        self.outlined(self.end).is_none()
            && self.fidgets.as_ref().is_none_or(|f| f.stacks.is_empty())
            && self.heroes.as_ref().is_none_or(|h| !h.any())
    }

    /// The timeline with its battle's heroes drawn (E17 M3 T3a).
    #[must_use]
    pub fn with_heroes(self, heroes: Heroes) -> Self {
        Self {
            heroes: Some(heroes),
            ..self
        }
    }

    /// The idle steps taken by `t` (the outline's pulse, §4.2): the opening's at 0, then as
    /// [`Idle`] says.
    pub fn pulse(&self, t: u32) -> u32 {
        let mut count = self.steps.iter().filter(|s| **s <= t).count();
        let mut idle = self.idle;
        idle.run(t, |_| count += 1);
        u32::try_from(count).unwrap_or(u32::MAX)
    }

    /// Every time up to `until` at which what shows may change (E17 M2 T4b-2a, `h3-view
    /// play`): 0, each pose's, each projectile step's and its erasure's, each ray tick's and its
    /// erasure's, each magic animation's frame and erasure (over stacks, at a hex: E17 M3 T7c-2,
    /// which adds them, and over the field), each hero's frame, each object placed or gone,
    /// each count and hue shown (T5b), each outline's and each idle step's (the outline's colour),
    /// and the last frame's; in order, each once.
    pub fn changes(&self, until: u32) -> Vec<u32> {
        let mut times = vec![0, self.end];
        times.extend(self.keys.iter().map(|k| k.shown));
        for f in &self.flights {
            times.extend((0..=f.steps).map(|k| f.from + k * f.step));
        }
        for b in &self.beams {
            let ticks = u32::try_from(b.ticks.len()).unwrap_or(0);
            times.extend((1..=ticks).map(|k| b.from + k * b.step));
        }
        times.extend(self.magics.iter().map(|k| k.shown));
        times.extend(self.spots.iter().map(|k| k.shown));
        times.extend(self.objects.iter().map(|k| k.shown));
        times.extend(self.fields.iter().map(|k| k.shown));
        times.extend(self.hero_keys.iter().map(|k| k.shown));
        times.extend(self.counts.iter().map(|k| k.shown));
        times.extend(self.tints.iter().map(|k| k.shown));
        times.extend(self.looks.iter().map(|k| k.shown));
        times.extend(self.outlines.iter().map(|(t, _)| *t));
        times.extend(self.turrets.iter().map(|k| k.shown));
        times.extend(self.walls.iter().map(|k| k.shown));
        times.extend(self.blasts.iter().map(|k| k.shown));
        for q in &self.quakes {
            times.extend((0..SHAKES * 15).map(|k| q.from + k * q.step));
            times.push(q.until);
        }
        // Each dissolve's steps and its end (E17 M5 T5a), each created stack's showing.
        for d in &self.dissolves {
            times.extend((0..=DISSOLVE_STEPS).map(|k| d.start + k * d.step));
        }
        times.extend(self.created.iter().map(|k| k.shown));
        times.extend(&self.steps);
        let mut idle = self.idle;
        idle.run(until, |t| times.push(t));
        times.retain(|t| *t <= until);
        times.sort_unstable();
        times.dedup();
        times
    }

    /// The active stack at `t` (its turn's start), if a turn started by then.
    pub fn active(&self, t: u32) -> Option<u8> {
        self.turns
            .iter()
            .rev()
            .find(|(at, _)| *at <= t)
            .map(|(_, stack)| *stack)
    }

    /// The timeline in the capture harness's format (design §7.6): one line per change of a
    /// stack's group, frame or hex, `<ms> <side>.<record> g<group> f<frame> h<hex>`, every stack
    /// at 0; a change at a time is the last pose drawn then. After a time's stacks, a change of
    /// the magic animation drawn (E17 M3 T3b): `<ms> magic <number> f<frame> <side>.<record>…`
    /// over its stacks, `<ms> magic -` once erased. Then `<ms> end` when the battle is decided
    /// (E17 M3 T8).
    pub fn lines(&self) -> String {
        self.lines_with(None)
    }

    /// [`Timeline::lines`] with the heroes' changes by `until` (the harness's `heroes`, E17 M3
    /// T7b-1): after a time's stacks, each change of a hero's group or frame as drawn, `<ms> hero
    /// <side> g<group> f<frame>`, both heroes at 0 (none without [`Timeline::heroes`]).
    pub fn lines_with_heroes(&self, until: u32) -> String {
        self.lines_with(Some(until))
    }

    fn lines_with(&self, heroes: Option<u32>) -> String {
        let mut now: BTreeMap<u8, Pose> = self.members.iter().map(|m| (m.stack, m.start)).collect();
        let mut printed: BTreeMap<u8, (u32, u32, u8)> = BTreeMap::new();
        let mut out = String::new();
        let mut times: Vec<u32> = self.keys.iter().map(|k| k.drawn).collect();
        times.extend(self.created.iter().map(|k| k.drawn));
        times.extend(self.magics.iter().map(|k| k.drawn));
        times.extend(self.turrets.iter().map(|k| k.drawn));
        times.extend(self.walls.iter().map(|k| k.drawn));
        times.extend(self.ended.map(|(t, _)| t));
        times.push(0);
        let shown_heroes = self.heroes.as_ref().zip(heroes);
        if let Some((_, until)) = shown_heroes {
            times.extend(self.hero_keys.iter().map(|k| k.drawn));
            times.extend(self.steps.iter().copied());
            let mut idle = self.idle;
            idle.run(until, |t| times.push(t));
            times.retain(|t| *t <= until);
        }
        times.sort_unstable();
        times.dedup();
        let mut heroes_printed: [Option<(u32, u32)>; 2] = [None; 2];
        let mut turrets_now: BTreeMap<u8, TurretPose> = BTreeMap::new();
        let mut turrets_printed: BTreeMap<u8, TurretPose> = BTreeMap::new();
        let mut walls_printed: Option<Walls> = None;
        for t in times {
            for key in self.keys.iter().filter(|k| k.drawn == t) {
                now.insert(key.stack, key.pose);
            }
            // The stacks living at the start, then the records the battle set up by then (E17 M5
            // T5a: the harness lists a record from its set-up; a reused id keeps its record).
            let mut set_up: Vec<&Member> = self.members.iter().collect();
            for m in self
                .created
                .iter()
                .filter(|k| k.drawn <= t)
                .map(|k| &k.member)
            {
                if set_up.iter().all(|n| n.stack != m.stack) {
                    set_up.push(m);
                }
            }
            // By side and record, as the harness reads the records.
            set_up.sort_by_key(|m| (m.side == Side::Defender, m.record));
            for m in set_up {
                let p = now.get(&m.stack).copied().unwrap_or(m.start);
                let seen = (p.group, p.frame, p.hex);
                if printed.insert(m.stack, seen) != Some(seen) {
                    let side = u8::from(m.side == Side::Defender);
                    let _ = writeln!(
                        out,
                        "{t} {side}.{} g{} f{} h{}",
                        m.record, p.group, p.frame, p.hex
                    );
                }
            }
            if let Some((h, _)) = shown_heroes {
                for (side, pose) in h.drawn_at(self, t).iter().enumerate() {
                    let Some(p) = pose else {
                        continue;
                    };
                    let seen = Some((p.group, p.frame));
                    if heroes_printed[side] != seen {
                        heroes_printed[side] = seen;
                        let _ = writeln!(out, "{t} hero {side} g{} f{}", p.group, p.frame);
                    }
                }
            }
            // A siege's turret records and walls (E17 M4 T2b), as the harness reads them.
            for key in self.turrets.iter().filter(|k| k.drawn == t) {
                turrets_now.insert(key.record, key.pose);
            }
            for (&record, &pose) in &turrets_now {
                if turrets_printed.insert(record, pose) != Some(pose) {
                    out.push_str(&siege::turret_line(t, record, pose));
                }
            }
            if let Some(key) = self.walls.iter().rev().find(|k| k.drawn == t)
                && walls_printed != Some(key.walls)
            {
                walls_printed = Some(key.walls);
                out.push_str(&siege::walls_line(t, &key.walls));
            }
            // Each key drawn differs from the one before it (a frame on, or the erasure).
            if let Some(key) = self.magics.iter().rev().find(|k| k.drawn == t) {
                let _ = match key.frame {
                    Some(frame) => {
                        let over = key
                            .over
                            .iter()
                            .filter_map(|(stack, ..)| self.member(*stack));
                        let records: Vec<String> = over
                            .map(|m| format!("{}.{}", u8::from(m.side == Side::Defender), m.record))
                            .collect();
                        let records = records.join(" ");
                        writeln!(out, "{t} magic {} f{frame} {records}", key.animation)
                    }
                    None => writeln!(out, "{t} magic -"),
                };
            }
            // The harness's `end`: the battle decided (E17 M3 T8).
            if self.ended.is_some_and(|(at, _)| at == t) {
                let _ = writeln!(out, "{t} end");
            }
        }
        out
    }
}

/// The original's clock: the frame deadline 0x698998 on its grid (§7.1).
#[derive(Debug, Clone, Copy)]
struct Clock {
    now: u32,
    deadline: u32,
    grid: u32,
}

impl Clock {
    /// A command at `at`: the clock moves there; the battle's tick (0x474040) has kept the
    /// deadline on its grid, so it is the grid's next point after it.
    fn start(&mut self, at: u32) {
        self.now = self.now.max(at);
        if self.deadline <= self.now {
            let behind = self.now - self.deadline;
            self.deadline += (behind / self.grid + 1) * self.grid;
        }
    }

    /// A frame drawn now waits for the deadline (`wait_until` 0x4F82F0), which then moves on by
    /// max(`time`, the lateness); returns when it shows.
    fn show(&mut self, time: u32) -> u32 {
        let shown = self.now.max(self.deadline);
        let late = shown - self.deadline;
        self.deadline += time.max(late);
        self.now = shown;
        shown
    }

    /// A wait outside the deadline.
    fn wait(&mut self, ms: u32) {
        self.now += ms;
    }
}

/// A stack as the timeline follows it.
#[derive(Debug, Clone)]
struct Live {
    side: Side,
    /// The engine's position (a two-hex stack's rear hex by its side, BAT-GEO-002).
    hex: u8,
    count: u32,
    flies: bool,
    two_hex: bool,
    defending: bool,
    /// A war machine (E17 M3 T3b).
    war_machine: bool,
    /// It faces right (+0x44 = 1); the setup faces the attacker right (0x43D846).
    right: bool,
    /// Its count box hidden: a walk or a flight under way ([`Pose::boxless`]).
    boxless: bool,
    look: Look,
    /// Its active spells, by handle (E17 M3 T7b-2a).
    spells: BTreeSet<u16>,
    /// Its defense skill (the snapshot's, lowered by Disrupting Ray; E17 M3 T7b-2a): a defend's
    /// gain and Disrupting Ray's line.
    defense: i32,
    /// Its creature frightens ([`Casts::frighteners`]; E17 M3 T7b-2b-2b).
    frightens: bool,
    /// Its creature has a Fire Shield of its own ([`Casts::shielded`]; E17 M3 T7c-3).
    shielded: bool,
    /// A siege's turret: its record ([`siege::turret_record`]; E17 M4 T2b).
    turret: Option<u8>,
}

impl Live {
    /// It faces the way its side does at the setup.
    fn at_home(&self) -> bool {
        self.right == (self.side == Side::Attacker)
    }

    /// The executable's position (+0x38) for the engine's `hex`: a two-hex stack turned away
    /// from its side's facing stands on its other hex, + 1 for the attacker, − 1 for the
    /// defender (`stack_turn_around` 0x446720).
    fn position(&self, hex: u8) -> u8 {
        match (self.two_hex && !self.at_home(), self.side) {
            (false, _) => hex,
            (true, Side::Attacker) => hex.saturating_add(1),
            (true, Side::Defender) => hex.saturating_sub(1),
        }
    }

    /// The hexes it covers on the executable's field: its position, and a two-hex stack's other
    /// hex ahead of it (0x4466A0).
    fn covers(&self, hex: u8) -> [u8; 2] {
        let at = self.position(hex);
        match (self.two_hex, self.right) {
            (false, _) => [at, at],
            (true, true) => [at, at.saturating_add(1)],
            (true, false) => [at, at.saturating_sub(1)],
        }
    }
}

/// Why a record has no timeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnimError {
    /// A living stack's creature (its handle) has no [`Look`].
    Look(u16),
}

impl core::fmt::Display for AnimError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Look(handle) => write!(f, "no animation numbers for creature handle {handle}"),
        }
    }
}

/// The poses of the battle `snapshot` shows when `commands` follow, at combat speed `speed`;
/// `looks` holds each creature's numbers by its handle.
///
/// The first command is given at 0, the snapshot's moment (the harness clicks at the frozen
/// opening); a later one without a time as soon as the previous one's last frame showed
/// (*provisional*: in the melee sequence the computer acts as the flight's second standing frame
/// shows, 0x4B4A15; after Wait or Defend the next stack acts at once, §7.4).
///
/// # Errors
/// A living stack whose creature has no [`Look`].
pub fn timeline(
    snapshot: &Observation,
    commands: &[Command<'_>],
    looks: &BTreeMap<u16, Look>,
    stage: Stage<'_>,
    speed: Speed,
) -> Result<Timeline, AnimError> {
    played(snapshot, commands, looks, stage, speed, None)
}

/// As [`timeline`], the stacks fidgeting while the battle idles, from the viewer's own generator
/// seeded with `seed` ([`Fidgets`]; E17 M2 T4b-2a).
///
/// # Errors
/// As [`timeline`].
pub fn fidgeting(
    snapshot: &Observation,
    commands: &[Command<'_>],
    looks: &BTreeMap<u16, Look>,
    stage: Stage<'_>,
    speed: Speed,
    seed: u32,
) -> Result<Timeline, AnimError> {
    played(snapshot, commands, looks, stage, speed, Some(seed))
}

/// [`timeline`] or [`fidgeting`]: the commands grown one by one, the next one's being the
/// computer's when it has no time ([`Growing::push`]).
fn played(
    snapshot: &Observation,
    commands: &[Command<'_>],
    looks: &BTreeMap<u16, Look>,
    stage: Stage<'_>,
    speed: Speed,
    seed: Option<u32>,
) -> Result<Timeline, AnimError> {
    let mut growing = Growing::new(snapshot, looks, stage, speed, seed)?;
    for (i, command) in commands.iter().enumerate() {
        let computer = commands.get(i + 1).is_some_and(|c| c.at.is_none());
        growing.push(command, computer);
    }
    Ok(growing.finish())
}

/// A timeline grown a command at a time (E17 M5 T1b): the commands of a battle being fought
/// played as they arrive, the timeline growing with each (its lists only appended to, nothing
/// before the last command's end changed), and finished when the battle is or the record ends.
/// [`timeline`] and [`fidgeting`] grow theirs at once.
///
/// What a command's last frames leave to the battle's next redraw (the layer changes, the counts,
/// the hues, an erasure: [`Timeline::changes`]) shows with the next command's first idle step,
/// or at the end when it finishes (*provisional*); until then [`Growing::timeline`] does not show
/// it, so a frame past its end waits for the next command or the finish. The drawbridge's
/// redraws that began (CLT-Q23) show the rectangle alone until that redraw: their span is open
/// until then ([`BridgeKey::until`] at its greatest), the one entry a later command changes.
#[derive(Debug)]
pub struct Growing<'a> {
    player: Player<'a>,
    timeline: Timeline,
    /// The commands played.
    commands: usize,
    /// The stack whose turn the computer plays next (outlined at the loop's tick as it acts).
    waiting: Option<u8>,
    /// The timeline's last drawbridge span is open (its end at the next redraw).
    open_bridge: bool,
}

impl<'a> Growing<'a> {
    /// The battle `snapshot` shows, before its first command, at combat speed `speed`; `looks`
    /// holds each creature's numbers by its handle; the stacks fidget while it idles, from the
    /// viewer's own generator seeded with `seed` (none: no fidgets, [`Fidgets`]).
    ///
    /// # Errors
    /// A living stack whose creature has no [`Look`].
    pub fn new(
        snapshot: &Observation,
        looks: &BTreeMap<u16, Look>,
        stage: Stage<'a>,
        speed: Speed,
        seed: Option<u32>,
    ) -> Result<Self, AnimError> {
        let standing = u32::from(stage.groups.standing);
        let mut members = Vec::new();
        // The stacks whose DEF has a fidget (group 1).
        let mut fidgeters = Vec::new();
        let mut stacks = BTreeMap::new();
        let mut records = [0u8; 2];
        // The harness's records follow the original's setup: each side's army, then its war
        // machines in `battle_war_machine_stacks`' order (E17 M3 T3b). A siege's turrets are the
        // defender's last (E17 M4 T2b): off the field, their shooters drawn by the town's records
        // (`crate::siege`), never fidgeting.
        let mut living: Vec<&Stack> = snapshot.stacks.iter().filter(|s| s.alive).collect();
        living.sort_by_key(|s| {
            (
                s.side == Side::Defender,
                s.war_machine,
                setup_order(s.slot),
                s.id,
            )
        });
        for s in living {
            let look = looks.get(&s.creature).ok_or(AnimError::Look(s.creature))?;
            let side = usize::from(s.side == Side::Defender);
            let start = Pose {
                mirrored: s.side == Side::Defender,
                ..still(standing, s.hex)
            };
            members.push(Member {
                stack: s.id,
                side: s.side,
                record: records[side],
                start,
            });
            records[side] += 1;
            let frames = look.count(stage.groups.mouse_over);
            let turret = turret_record(s.hex).filter(|_| s.hex >= FIELD_HEXES);
            if frames > 0 && turret.is_none() {
                fidgeters.push(Fidget {
                    stack: s.id,
                    interval: look.times[0],
                    frames,
                    clock: 0,
                    frame: None,
                    base: start,
                });
            }
            stacks.insert(
                s.id,
                Live {
                    side: s.side,
                    hex: s.hex,
                    count: s.count,
                    flies: s.flies,
                    two_hex: s.two_hex,
                    defending: s.defending,
                    war_machine: s.war_machine,
                    right: s.side == Side::Attacker,
                    boxless: false,
                    look: look.clone(),
                    spells: s.effects.iter().map(|e| e.spell).collect(),
                    defense: s.defense,
                    frightens: stage.casts.frighteners.contains(&s.creature),
                    shielded: stage.casts.shielded.contains(&s.creature),
                    turret,
                },
            );
        }
        let grid = speed.scale(TICK).max(1);
        let mut player = Player {
            stage,
            speed,
            clock: Clock {
                now: 0,
                deadline: 0,
                grid,
            },
            stacks,
            keys: Vec::new(),
            pending: Vec::new(),
            turns: Vec::new(),
            controllers: Vec::new(),
            flights: Vec::new(),
            beams: Vec::new(),
            magics: Vec::new(),
            spots: Vec::new(),
            hero_keys: Vec::new(),
            counts: Vec::new(),
            tints: Vec::new(),
            pending_tints: Vec::new(),
            look_keys: Vec::new(),
            pending_looks: Vec::new(),
            outlined: Vec::new(),
            machine: None,
            idle: Idle::default(),
            steps: Vec::new(),
            cues: Vec::new(),
            happenings: Vec::new(),
            log: Vec::new(),
            outcome: None,
            tactics: snapshot.tactics.is_some(),
            acting: snapshot.active,
            strike_casts: Vec::new(),
            vanishing: BTreeSet::new(),
            erasing: Vec::new(),
            pending_counts: Vec::new(),
            objects: Vec::new(),
            pending_objects: Vec::new(),
            placed: BTreeMap::new(),
            hidden: BTreeSet::new(),
            secret: BTreeMap::new(),
            kept: None,
            fields: Vec::new(),
            field_kept: None,
            corpses: BTreeMap::new(),
            lying: BTreeMap::new(),
            draws: chain::Draws::new(&stage.casts.rays),
            siege: None,
            turret_keys: Vec::new(),
            wall_keys: Vec::new(),
            blasts: Vec::new(),
            blast_count: 0,
            bridges: Vec::new(),
            bridge_from: None,
            quakes: Vec::new(),
            dissolves: Vec::new(),
            created: Vec::new(),
            summoned: BTreeSet::new(),
            vanished: Vec::new(),
            records: members
                .iter()
                .map(|m| (m.stack, (m.side, m.record)))
                .collect(),
            next_record: records,
            looks: looks.clone(),
            notices: Vec::new(),
        };
        // A siege with a fort (E17 M4 T2b): the walls' states and hit points, each turret standing.
        if let Some(s) = snapshot.siege.as_ref().filter(|s| s.fort > 0) {
            let mut hit_points = [0; 8];
            for (hp, &v) in hit_points.iter_mut().zip(&s.parts) {
                *hp = v;
            }
            let walls = Walls::of(s);
            player.siege = Some(siege::Siege { walls, hit_points });
            player.wall_keys.push(WallKey {
                drawn: 0,
                shown: 0,
                walls,
            });
            let standing = TurretPose {
                group: u32::from(stage.groups.standing),
                frame: 0,
                right: false,
            };
            for record in player.stacks.values().filter_map(|l| l.turret) {
                player.turret_keys.push(TurretKey {
                    record,
                    drawn: 0,
                    shown: 0,
                    pose: standing,
                });
            }
        }
        let mut fidgets = seed.map(|rng| Fidgets {
            rng,
            group: stage.groups.mouse_over,
            stacks: fidgeters,
        });
        if let Some(f) = fidgets.as_mut() {
            f.reset(0, &player.bases(), &[]);
        }
        // The snapshot's spells tint their boxes from the start (E17 M3 T7b-2a).
        let casts = player.stage.casts;
        player.pending_tints = player
            .stacks
            .iter()
            .filter(|(_, l)| !l.spells.is_empty())
            .map(|(&id, l)| (id, hue(l, casts)))
            .collect();
        // The snapshot's petrified stacks look so from the start (E17 M5 T5b1).
        player.pending_looks = player
            .stacks
            .iter()
            .filter_map(|(&id, l)| rest_look(l, casts).map(|look| (id, Some(look))))
            .collect();
        // The snapshot's battlefield objects show from the start (E17 M3 T7c-2); one the other
        // side does not see, only to its own (E17 M5 T2b).
        for o in &snapshot.spell_obstacles {
            player.place(o.spell, &o.hexes, (!o.revealed).then_some(o.side));
        }
        player.flush(0, 0);
        let humans = [0, 1].map(|i| snapshot.sides.get(i).is_some_and(|s| s.human));
        let timeline = Timeline {
            members,
            outlines: vec![(0, snapshot.active)],
            fidgets,
            humans,
            ..Timeline::default()
        };
        let mut growing = Self {
            player,
            timeline,
            commands: 0,
            waiting: snapshot.active,
            open_bridge: false,
        };
        growing.hand_over();
        Ok(growing)
    }

    /// [`Growing::push`] with each of the command's events seen by one side only or by both:
    /// `seen` gives, for each event in order, the side that alone sees it (none: both, as for an
    /// event missing from it). The events are every side's (E17 M5 T2b: a battle in hot seat or
    /// against the computer, whose screen shows the side to act's hidden objects,
    /// [`Timeline::screen`]): an object placed by an event one side alone sees is that side's
    /// until it is revealed.
    pub fn push_seen(&mut self, command: &Command<'_>, computer: bool, seen: &[Option<Side>]) {
        self.player.secret = command
            .events
            .iter()
            .zip(seen)
            .filter_map(|(e, &side)| match e {
                Event::ObstaclePlaced { hexes, .. } => Some((*hexes.iter().max()?, side?)),
                _ => None,
            })
            .collect();
        self.push(command, computer);
        self.player.secret.clear();
    }

    /// The timeline as the battle waits at its end for the next command (E17 M5 T2b: a human's
    /// turn): what the last command left to the battle's next redraw shows with the first idle
    /// step after the end, the idle steps and the fidgets going on. The next command given at
    /// `t` grows a timeline with the same moments before `t` ([`Growing::push`]).
    #[must_use]
    pub fn waiting(&self) -> Timeline {
        let mut tail = Self {
            player: self.player.clone(),
            timeline: self.timeline.clone(),
            commands: self.commands,
            waiting: self.waiting,
            open_bridge: self.open_bridge,
        };
        let idle = tail.player.idle;
        let first = idle.next.max(idle.from);
        tail.player.flush(first, first);
        tail.hand_over();
        tail.timeline
    }

    /// The turn the last command started is a human's after all (E17 M5 T4c1: Auto Combat
    /// stopped as it started, the command pushed as followed by the computer's): its stack
    /// outlined from its start, as [`Growing::push`] outlines a human's turn.
    pub fn outline_waiting(&mut self) {
        if let Some(&(at, stack)) = self.timeline.turns.last() {
            self.timeline.outlines.push((at, Some(stack)));
        }
    }

    /// Plays `command` after those before it (a command without a time as soon as the previous
    /// one's last frame showed; *provisional*, [`timeline`]); `computer`: the next command is the
    /// computer's (a turn this one starts is then not outlined: the computer's stack is outlined
    /// at the loop's tick as it acts, §7.9).
    pub fn push(&mut self, command: &Command<'_>, computer: bool) {
        let Self {
            player,
            timeline,
            commands,
            waiting,
            ..
        } = self;
        let Timeline {
            outlines,
            fidgets,
            actions,
            ended,
            ..
        } = timeline;
        let first = *commands == 0;
        let default = if first { 0 } else { player.clock.now };
        player.clock.start(command.at.unwrap_or(default));
        // A command without a time after the first is the computer's: the loop did not tick
        // since the last one ended, but once when it acts, which outlines its stack until its
        // first frame shows (*provisional*, §7.9).
        let ran = player.steps.len();
        let now = player.clock.now;
        if !first && command.at.is_none() {
            if player.idle.once(now, &mut player.steps) {
                outlines.push((now, *waiting));
            }
        } else {
            let Player { idle, steps, .. } = player;
            idle.run(now, |t| steps.push(t));
        }
        // The last command's layer changes show with the battle's next redraw: the first idle
        // step since, if one ran, before any fidget (*provisional*, as at the end; E17 M2 T5c).
        if let Some(&first) = player.steps.get(ran) {
            player.flush(first, first);
        }
        if let Some(f) = fidgets.as_mut() {
            let steps = player.steps[ran..].to_vec();
            for now in steps {
                player
                    .keys
                    .extend(f.step(now).into_iter().map(|(stack, pose)| Key {
                        stack,
                        drawn: now,
                        shown: now,
                        pose,
                    }));
            }
        }
        let (before, turns) = (player.keys.len(), player.turns.len());
        let (heroes_before, magics_before) = (player.hero_keys.len(), player.magics.len());
        // The action starts (`action_dispatcher` 0x478D80's head, E17 M3 T6b).
        let started = player.clock.now;
        player.log.push((started, Logged::Action));
        // A hero's cast (E17 M3 T7b-1, the recorded `cast`): its action clears the outline at
        // once (no stack active while it plays); its stack's turn goes on after it, outlined
        // again as the hero stands.
        let cast = command
            .events
            .iter()
            .take_while(|e| !matches!(e, Event::TurnStarted { .. }))
            .any(|e| matches!(e, Event::SpellCast { .. }));
        if cast {
            outlines.push((started, None));
        }
        player.command(command.events);
        // The outline goes with the command's first frame shown (a stack's, a hero's as it
        // casts, E17 M3 T7b-1, or a magic animation's, T7b-2b-2b: good luck before a shot); a
        // turn started brings its stack's, unless the computer acts next.
        let first = [
            player.keys.get(before).map(|k| k.shown),
            player.hero_keys.get(heroes_before).map(|k| k.shown),
            player.magics.get(magics_before).map(|k| k.shown),
        ];
        // A creature's cast keeps it outlined until its cast starts (E17 M3 T7c-1: `cast_spell`
        // clears it, 0x5A004C).
        let creature_cast = command
            .events
            .iter()
            .take_while(|e| !matches!(e, Event::TurnStarted { .. }))
            .any(|e| matches!(e, Event::StackCast { .. }));
        if let Some(first) = first.into_iter().flatten().min().filter(|_| !creature_cast) {
            outlines.push((first, None));
        }
        outlines.append(&mut player.outlined);
        if let Some(&(at, stack)) = player.turns.get(turns..).and_then(<[_]>::last) {
            outlines.push((at, (!computer).then_some(stack)));
            *waiting = Some(stack);
        } else if cast {
            outlines.push((player.clock.now, *waiting));
        }
        player.idle.from = player.clock.now;
        // The battle decided: `battle_end` follows the action's last frame (design §8.2).
        let now = player.clock.now;
        *ended = ended.or_else(|| player.outcome.map(|o| (now, o)));
        let (stands, unmarks) = calms(command.events, player.stage.bugs);
        actions.push(Action {
            start: started,
            end: player.clock.now,
            happenings: core::mem::take(&mut player.happenings),
            stands,
            unmarks,
        });
        if let Some(f) = fidgets.as_mut() {
            let drawn: Vec<u8> = player.keys[before..].iter().map(|k| k.stack).collect();
            f.reset(player.clock.now, &player.bases(), &drawn);
        }
        *commands += 1;
        self.hand_over();
    }

    /// The timeline of the commands played so far ([`Growing`]: what their last frames leave to
    /// the next redraw not shown yet).
    #[must_use]
    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    /// How many commands were played.
    #[must_use]
    pub fn commands(&self) -> usize {
        self.commands
    }

    /// With its battle's heroes drawn (E17 M3 T3a, [`Timeline::with_heroes`]).
    #[must_use]
    pub fn with_heroes(mut self, heroes: Heroes) -> Self {
        self.timeline.heroes = Some(heroes);
        self
    }

    /// The timeline finished: the last layer changes show with the battle's next redraw, at its
    /// end (*provisional*).
    #[must_use]
    pub fn finish(mut self) -> Timeline {
        let now = self.player.clock.now;
        self.player.flush(now, now);
        self.hand_over();
        self.timeline
    }

    /// Moves what the player drew into the timeline: only the command under way reads its own
    /// keys back; the idle steps' state and the end with them.
    fn hand_over(&mut self) {
        let (p, t) = (&mut self.player, &mut self.timeline);
        // The open drawbridge span gives way to its end, or stays open.
        if core::mem::take(&mut self.open_bridge) {
            t.bridges.pop();
        }
        t.keys.append(&mut p.keys);
        t.turns.append(&mut p.turns);
        t.controllers.append(&mut p.controllers);
        t.flights.append(&mut p.flights);
        t.beams.append(&mut p.beams);
        t.magics.append(&mut p.magics);
        t.spots.append(&mut p.spots);
        t.objects.append(&mut p.objects);
        t.fields.append(&mut p.fields);
        t.hero_keys.append(&mut p.hero_keys);
        t.counts.append(&mut p.counts);
        t.tints.append(&mut p.tints);
        t.looks.append(&mut p.look_keys);
        t.steps.append(&mut p.steps);
        t.cues.append(&mut p.cues);
        t.log.append(&mut p.log);
        t.turrets.append(&mut p.turret_keys);
        t.walls.append(&mut p.wall_keys);
        t.blasts.append(&mut p.blasts);
        t.bridges.append(&mut p.bridges);
        if let Some(from) = p.bridge_from {
            t.bridges.push(BridgeKey {
                from,
                until: u32::MAX,
            });
            self.open_bridge = true;
        }
        t.quakes.append(&mut p.quakes);
        t.dissolves.append(&mut p.dissolves);
        t.created.append(&mut p.created);
        t.notices.append(&mut p.notices);
        t.idle = p.idle;
        t.end = p.clock.now;
    }
}

/// The hue of `live`'s box by its active spells' signs in `casts` (none: no spell active, the
/// layout's tint; `draw_battle_stack` 0x43E6AA, design §4.3; E17 M3 T7b-2a).
fn hue(live: &Live, casts: &Casts) -> Option<u32> {
    (!live.spells.is_empty()).then(|| {
        crate::scene::spell_hue(
            live.spells
                .iter()
                .map(|h| casts.spells.get(h).map_or(0, |l| l.sign)),
        )
    })
}

/// A stack's palette effect by its spells outside a glow (E17 M5 T5b1): petrified while Stone
/// Gaze lasts (its duration +0x2B0 above 0, `draw_battle_stack` 0x43E3CB).
fn rest_look(live: &Live, casts: &Casts) -> Option<Effect> {
    live.spells
        .iter()
        .any(|h| {
            casts
                .spells
                .get(h)
                .is_some_and(|l| l.original == STONE_GAZE)
        })
        .then_some(Effect::Petrified)
}

/// What a command's action does at its start to the stacks in group 1 and to the mouse's stack
/// (E17 M3 T6c-2), by what it does before the next turn: a walk, a strike or a shot, a cast or the
/// Catapult's shot stands the fidgeting stacks (`battle_stand_fidgets` 0x479DE0) and unmarks the
/// mouse's (`battle_clear_highlight` 0x477E10); a creature's cast only stands them, keeping the
/// mouse's stack marked through its frames until `cast_spell` starts (0x5A0055, a drawing bug:
/// CLT-Q8), unless `bugs` are fixed (E17 M3 T7d: unmarked at its start, as a hero's); a defend, a
/// wait or the First Aid Tent's heal does neither.
fn calms(events: &[Event], bugs: Bugs) -> (bool, bool) {
    let own = events
        .iter()
        .take_while(|e| !matches!(e, Event::TurnStarted { .. } | Event::RoundStarted { .. }));
    let (mut stands, mut unmarks) = (false, false);
    for e in own {
        match e {
            Event::Moved { .. }
            | Event::Struck { .. }
            | Event::SpellCast { .. }
            | Event::WallShot { .. } => (stands, unmarks) = (true, true),
            Event::StackCast { .. } => {
                stands = true;
                unmarks |= bugs == Bugs::Fixed;
            }
            _ => {}
        }
    }
    (stands, unmarks)
}

/// Where a war machine of army slot `slot` comes in the original's setup (`battle_war_machine_
/// stacks` 0x463600: the Catapult, the Ballista, the First Aid Tent, the Ammo Cart, then the
/// turrets; the engine's slots are 16 + its kind, BAT-WAR-001: the Ballista 16, the Ammo Cart 17,
/// the tent 18, the Catapult 19, the turrets 20 to 22); any other slot after them, by itself.
fn setup_order(slot: u8) -> (usize, u8) {
    const ORDER: [u8; 4] = [19, 16, 18, 17];
    let at = ORDER.iter().position(|&s| s == slot);
    (at.unwrap_or(ORDER.len()), slot)
}

/// A still pose: frame 0 of `group` on `hex`, facing right, standing's layer.
const fn still(group: u32, hex: u8) -> Pose {
    Pose {
        group,
        frame: 0,
        hex,
        offset: [0, 0],
        mirrored: false,
        layer: STANDING,
        boxless: false,
    }
}

/// Whether the `TurnStarted` of `stack` followed by the events `after` is a turn its pick freezes
/// (bad morale, Fear: `next_stack_to_act` 0x465283; E17 M3 T7b-2b-2b): the stack never becomes
/// the active one. A first turn's regeneration and the effects its start ends come before the
/// roll (the engine's `pick`; E17 M3 T9b, the reviewer).
#[must_use]
pub fn freezes(after: &[Event], stack: u8) -> bool {
    after
        .iter()
        .find(|e| {
            !matches!(
                e,
                Event::Regenerated { .. } | Event::EffectApplied { .. } | Event::EffectEnded { .. }
            )
        })
        .is_some_and(|e| {
            matches!(*e, Event::BadMorale { stack: s } | Event::Frightened { stack: s } if s == stack)
        })
}

/// Whether `event` is what a hit ends before the strike's next hit (the engine's `strike` and
/// `effects::damaged`): an effect the damage ends, a clone gone with its original.
fn damage_ended(event: &Event) -> bool {
    matches!(event, Event::EffectEnded { .. } | Event::Vanished { .. })
}

/// A frame of `group` on `hex`, facing right, standing's layer.
#[cfg(test)]
fn pose(group: u8, frame: u32, hex: u8) -> Pose {
    Pose {
        frame,
        ..still(u32::from(group), hex)
    }
}

/// `part` of `whole` pixels, `of` the way (`of` > 0).
fn share(whole: i32, part: u32, of: u32) -> i32 {
    let v = i64::from(whole) * i64::from(part) / i64::from(of.max(1));
    i32::try_from(v).unwrap_or(0)
}

/// A struck stack and how it reacts: its reaction's group and frames, whether it died, its count
/// after the strike's kills if any (its box shows it from the strike's end), and its standing
/// pose (its position and facing).
#[derive(Debug, Clone, Copy)]
struct Reaction {
    stack: u8,
    group: u8,
    frames: u32,
    /// The frames its part of the loop counts (`strike_animation` 0x468BD7 to 0x468C84): the
    /// death group's for a death, else the hit group's, a defending stack's too, whose group 4
    /// it draws (E17 M3 T7b-2b-3, `thunder`'s Vampire Lords).
    counted: u32,
    died: bool,
    count: Option<u32>,
    sound: Sound,
    base: Pose,
}

/// A shot's aim (0x469880): its group, its projectile's steps, start and target, and the slope
/// its angle comes from.
#[derive(Debug, Clone, Copy)]
struct Aim {
    /// 0 up, 1 straight, 2 down.
    direction: usize,
    steps: u32,
    /// The start point (the chosen group's missile offset) and the target point.
    from: [i32; 2],
    to: [i32; 2],
    /// |dx| and dy from the straight group's start point to the target: the angle is
    /// atan(−dy / |dx|), and ±90° the other way round for dx = 0.
    slope: [i64; 2],
}

impl Default for Aim {
    /// No target: the straight group, no projectile.
    fn default() -> Self {
        Self {
            direction: 1,
            steps: 0,
            from: [0, 0],
            to: [0, 0],
            slope: [0, 0],
        }
    }
}

impl Aim {
    /// The projectile's frame (0x469957): the count of leading midpoints between consecutive
    /// angles that are at least the shot's angle. The original compares the angle as an f32
    /// with the midpoint; comparing tangents is the same but within an f32's rounding of a
    /// midpoint (*provisional*).
    fn frame(&self, missile: &Missile) -> u32 {
        let [dx, dy] = self.slope;
        let at_least = |pair: &[i16]| {
            let midpoint = tangent(i32::from(pair[0]) + i32::from(pair[1]));
            match (midpoint, dx) {
                (i64::MAX, _) => true,
                // Straight below the angle is +90°, above −90° (the sign inverted, 0x4698EA).
                (_, 0) => dy <= 0,
                _ => {
                    i128::from(midpoint) * i128::from(dx) >= -i128::from(dy) * i128::from(TAN_SCALE)
                }
            }
        };
        let leading = missile
            .angles
            .windows(2)
            .take_while(|p| at_least(p))
            .count();
        u32::try_from(leading).unwrap_or(0)
    }
}

/// A magic animation drawn over the marked stacks through a strike's loop (E17 M3 T7b-1): its
/// number, its numbers and the stacks with its canvas's corner over each.
#[derive(Debug, Clone)]
struct Riding {
    number: u8,
    look: MagicLook,
    over: Vec<(u8, [i32; 2], bool)>,
}

impl Riding {
    /// Its frame `frame` (none: erased), drawn at `drawn`, shown from `shown`.
    fn key(&self, drawn: u32, shown: u32, frame: Option<u32>) -> MagicKey {
        MagicKey {
            animation: self.number,
            drawn,
            shown,
            frame,
            def: self.look.def.clone(),
            translucent: self.look.translucent,
            over: self.over.clone(),
        }
    }
}

/// Plays the commands.
#[derive(Debug, Clone)]
struct Player<'a> {
    stage: Stage<'a>,
    speed: Speed,
    clock: Clock,
    stacks: BTreeMap<u8, Live>,
    keys: Vec<Key>,
    /// Poses that change without a frame of their own (a layer set or restored): they show with
    /// the next frame drawn, as the executable draws them at its next redraw.
    pending: Vec<(u8, Pose)>,
    turns: Vec<(u32, u8)>,
    controllers: Vec<Side>,
    flights: Vec<Flight>,
    beams: Vec<Beam>,
    /// The magic animations' frames (E17 M3 T3b).
    magics: Vec<MagicKey>,
    /// The magic animations' frames at a hex (E17 M3 T7c-1).
    spots: Vec<SpotKey>,
    /// The heroes' frames (E17 M3 T7b-1).
    hero_keys: Vec<HeroKey>,
    /// The counts the boxes show from a time on (E17 M3 T5b).
    counts: Vec<CountKey>,
    /// The hues the boxes take from a time on (E17 M3 T7b-2a).
    tints: Vec<TintKey>,
    /// The hues changed since the last frame drawn: they show with the next.
    pending_tints: Vec<(u8, Option<u32>)>,
    /// The palette effects by spells from a time on (E17 M5 T5b1).
    look_keys: Vec<LookKey>,
    /// The palette effects changed since the last frame drawn: they show with the next.
    pending_looks: Vec<(u8, Option<Effect>)>,
    /// The outline's changes inside the command so far: a war machine's turn that plays out
    /// in it (E17 M3 T3b).
    outlined: Vec<(u32, Option<u8>)>,
    /// The war machine outlined at its turn's idle step until its action's first frame shows.
    machine: Option<u8>,
    /// The battle loop's idle steps: their clock and when each ran (§7.1).
    idle: Idle,
    steps: Vec<u32>,
    cues: Vec<Cue>,
    /// What the command so far did that the heroes answer (E17 M3 T3a).
    happenings: Vec<Happening>,
    /// What the battle wrote into its log so far, with its time (E17 M3 T6b).
    log: Vec<(u32, Logged)>,
    /// How the battle ended, once a command's events say so (E17 M3 T8).
    outcome: Option<Outcome>,
    /// The tactics phase runs (no wait's line, no round's).
    tactics: bool,
    /// The stack the battle outlines between actions (manager +0x132C8; E17 M3 T7b-2b-2b): the
    /// last turn's, which a move's, strike's or shot's handler clears while it plays and sets
    /// again at its end (0x445996, 0x4459ED; a Defend leaves it); none from a round's start
    /// (0x4761EE). Shown by the redraws of good morale and of a freeze before the next turn.
    acting: Option<u8>,
    /// The spells the melee strike playing leaves pending, cast at its animation's end
    /// ([`PENDING`]; E17 M3 T7b-2b-3).
    strike_casts: Vec<StrikeCast>,
    /// The stacks whose death at the next strike's end leaves no corpse ([`GONE`]: Sacrifice's
    /// victim; E17 M3 T9d).
    vanishing: BTreeSet<u8>,
    /// The magic animations whose marks were cleared without a redraw (`mass_spell_animation`
    /// 0x5A6B50; E17 M3 T7c-1): erased with the next frame drawn, each with the time memory lost
    /// it.
    erasing: Vec<MagicKey>,
    /// The counts the boxes show with the next frame drawn (E17 M3 T7c-1: `cast_spell`'s tail
    /// resets every stack's +0x50, 0x5A2089).
    pending_counts: Vec<(u8, u32)>,
    /// The battlefield objects placed and gone (E17 M3 T7c-2).
    objects: Vec<ObjectKey>,
    /// The objects placed or removed since the last frame drawn: they show with the next.
    pending_objects: Vec<(u8, Option<Object>)>,
    /// The objects on the battlefield in memory by anchor, each with the animation it vanishes
    /// with.
    placed: BTreeMap<u8, u8>,
    /// The anchors of those one side alone sees (E17 M5 T2b).
    hidden: BTreeSet<u8>,
    /// The anchors of the objects the command under way places out of the other side's sight,
    /// with the side that sees them (E17 M5 T2b, [`Growing::push_seen`]).
    secret: BTreeMap<u8, Side>,
    /// The last frame of a magic animation at a hex that its caller keeps (`keep`, E17 M3
    /// T7c-2): on the screen until the next frame drawn erases it.
    kept: Option<SpotKey>,
    /// The frames of the magic animation over the whole battlefield (E17 M3 T7c-2).
    fields: Vec<FieldKey>,
    /// Its last frame, on the screen until the next frame drawn.
    field_kept: Option<FieldKey>,
    /// The dead stacks as they fell, by id, which a raise brings back (E17 M3 T7c-3).
    corpses: BTreeMap<u8, Live>,
    /// Their poses as they lie (E17 M5 T5a: a corpse a Pit Lord's Demons rise from leaves in it).
    lying: BTreeMap<u8, Pose>,
    /// The draws of Chain Lightning's rays (E17 M3 T7c-3).
    draws: chain::Draws,
    /// A siege's walls in memory (E17 M4 T2b; none: no siege with a fort).
    siege: Option<siege::Siege>,
    /// The turret records' poses (E17 M4 T2b).
    turret_keys: Vec<TurretKey>,
    /// The walls' states over time (E17 M4 T2b).
    wall_keys: Vec<WallKey>,
    /// The wall shots' explosions (E17 M4 T2b).
    blasts: Vec<BlastKey>,
    /// The explosions so far.
    blast_count: usize,
    /// The drawbridge's rectangle-only spans (E17 M4 T2c-1).
    bridges: Vec<BridgeKey>,
    /// The start of the span the drawbridge's last redraws opened: the next frame shown ends it.
    bridge_from: Option<u32>,
    /// Earthquake's shakes (E17 M4 T2c-2).
    quakes: Vec<QuakeKey>,
    /// The dissolves (E17 M5 T5a).
    dissolves: Vec<DissolveKey>,
    /// The stacks the battle created (E17 M5 T5a).
    created: Vec<CreatedKey>,
    /// The living created stacks, which die without a corpse (E17 M5 T5a: the summoned stacks and
    /// clones, +0x84 0x400000; not a Pit Lord's Demons).
    summoned: BTreeSet<u8>,
    /// The clones gone with their originals in the command under way: they dissolve with the
    /// next deaths (E17 M5 T5a).
    vanished: Vec<u8>,
    /// Each stack's side and record, by id (E17 M5 T5a: a created stack's too).
    records: BTreeMap<u8, (Side, u8)>,
    /// Each side's next record.
    next_record: [u8; 2],
    /// Each creature's numbers by handle (E17 M5 T5a: a created stack's).
    looks: BTreeMap<u16, Look>,
    /// The message boxes the battle opens (E17 M5 T5a).
    notices: Vec<(u32, Notice)>,
}

/// A spell a melee strike leaves pending on the stack it struck ([`PENDING`]): the stack, the
/// spell's handle and the events of its effect.
#[derive(Debug, Clone)]
struct StrikeCast {
    target: u8,
    spell: u16,
    effects: Vec<Event>,
}

/// A command's melee (`move_and_attack` 0x478900, `melee_attack` 0x441CB0): its striker, its
/// target, how each faced before it (the striker before its move) and whether the target has
/// turned back, its strikes over.
#[derive(Debug, Clone, Copy)]
struct Melee {
    attacker: u8,
    target: u8,
    attacker_right: bool,
    target_right: bool,
    over: bool,
}

impl Player<'_> {
    /// One frame: `poses` drawn now, shown at the deadline; the deadline then moves on by `time`.
    /// The pending poses of the other stacks show with it.
    fn frame(&mut self, poses: &[(u8, Pose)], time: u32) {
        let drawn = self.clock.now;
        let shown = self.clock.show(time);
        if self.machine.take().is_some() {
            self.outlined.push((shown, None));
        }
        self.pending
            .retain(|(s, _)| poses.iter().all(|(p, _)| p != s));
        self.flush(drawn, shown);
        self.keys.extend(poses.iter().map(|&(stack, pose)| Key {
            stack,
            drawn,
            shown,
            pose,
        }));
    }

    /// A frame drawn and shown now, the frame deadline left as it is (`battle_redraw` without
    /// its sixth argument): the pending poses show.
    fn redraw(&mut self) {
        let now = self.clock.now;
        self.flush(now, now);
    }

    /// `stack`'s `sound` starts (or, `stop`, ends) with the next frame drawn; returns the cue,
    /// which a wait for its sample names ([`Player::wait_for`]).
    fn cue(&mut self, stack: u8, sound: Sound, stop: bool) -> Cue {
        let cue = Cue {
            at: self.clock.now,
            stack,
            sound,
            stop,
        };
        self.cues.push(cue);
        cue
    }

    /// The battle waits while the sample `cue` started plays (E17 M5 T5b2, design §8.3;
    /// `wait_for_sample` 0x59A4C0, `sample_wait` 0x599EC0): from now, until the sample's end or
    /// [`crate::sound::SAMPLE_WAIT`] ms, whichever comes first, handling messages but drawing
    /// nothing (the screen stands still, the idle steps too). Nothing without the samples'
    /// lengths (no sound), for a cue without a sample or one already over.
    fn wait_for(&mut self, cue: Cue) {
        let Some(ms) = self.stage.lengths.and_then(|l| l.of(cue.stack, cue.sound)) else {
            return;
        };
        let now = self.clock.now;
        let end = cue
            .at
            .saturating_add(ms)
            .min(now.saturating_add(crate::sound::SAMPLE_WAIT));
        if end > now {
            self.clock.wait(end - now);
        }
    }

    /// The battle waits while `stack`'s `sound` last started in the command under way plays
    /// ([`Player::wait_for`]): `sample_wait` 0x599EC0 on the stack's own sound's handle (a shot's
    /// end, 0x43FDF9; a creature's cast's, 0x4486E2).
    fn wait_last(&mut self, stack: u8, sound: Sound) {
        let last = self
            .cues
            .iter()
            .rev()
            .find(|c| c.stack == stack && c.sound == sound && !c.stop)
            .copied();
        if let Some(cue) = last {
            self.wait_for(cue);
        }
    }

    /// The pose `r` lies in after its death: its group's last frame among the corpses.
    fn fallen(r: &Reaction) -> Pose {
        let base = Pose {
            layer: CORPSE,
            ..r.base
        };
        match r.frames.checked_sub(1) {
            Some(last) => Pose {
                group: u32::from(r.group),
                frame: last,
                ..base
            },
            None => base,
        }
    }

    /// The pending poses, drawn at `drawn` and shown at `shown`, and the pending hues and erasures
    /// with them.
    fn flush(&mut self, drawn: u32, shown: u32) {
        // The first frame after the drawbridge's redraws shows beyond its rectangle again.
        if let Some(from) = self.bridge_from.take() {
            self.bridges.push(BridgeKey { from, until: shown });
        }
        self.show_tints(shown);
        for erased in core::mem::take(&mut self.erasing) {
            self.magics.push(MagicKey { shown, ..erased });
        }
        if let Some(kept) = self.kept.take() {
            self.spots.push(SpotKey {
                shown,
                frame: None,
                ..kept
            });
        }
        if let Some(kept) = self.field_kept.take() {
            self.fields.push(FieldKey {
                shown,
                frame: None,
                ..kept
            });
        }
        for (anchor, object) in core::mem::take(&mut self.pending_objects) {
            self.objects.push(ObjectKey {
                shown,
                anchor,
                object,
            });
        }
        for (stack, count) in core::mem::take(&mut self.pending_counts) {
            self.counts.push(CountKey {
                stack,
                shown,
                count,
            });
        }
        let pending = core::mem::take(&mut self.pending);
        self.keys
            .extend(pending.into_iter().map(|(stack, pose)| Key {
                stack,
                drawn,
                shown,
                pose,
            }));
    }

    /// Frame `frame` of `group` of `live` on the engine's `hex`: at its position, facing its way,
    /// in standing's layer.
    fn posed(live: &Live, group: u8, frame: u32, hex: u8) -> Pose {
        Pose {
            group: u32::from(group),
            frame,
            hex: live.position(hex),
            offset: [0, 0],
            mirrored: !live.right,
            layer: STANDING,
            boxless: live.boxless,
        }
    }

    /// `live` standing on the engine's `hex`.
    fn standing(&self, live: &Live, hex: u8) -> Pose {
        Self::posed(live, self.stage.groups.standing, 0, hex)
    }

    /// Every living stack's standing pose, by id.
    fn bases(&self) -> BTreeMap<u8, Pose> {
        self.stacks
            .iter()
            .map(|(&id, live)| (id, self.standing(live, live.hex)))
            .collect()
    }

    /// A living stack as the timeline follows it now.
    fn live(&self, stack: u8) -> Option<Live> {
        self.stacks.get(&stack).cloned()
    }

    /// A dead stack leaves the living for the corpses (a raise may bring it back), lying in
    /// `lying`.
    fn bury(&mut self, stack: u8, lying: Pose) {
        self.lying.insert(stack, lying);
        // A death ends every spell with a duration, unreported (the engine's `remove_dead`, the
        // original's 0x444120): Disrupting Ray stays, so a stack raised or reborn comes back
        // without the others' hue and walk (E17 M3 T9b, the reviewer).
        let casts = self.stage.casts;
        let live = self.stacks.remove(&stack).map(|mut l| {
            let before = l.spells.len();
            // Its corpse no longer petrified (E17 M5 T5b1).
            if rest_look(&l, casts).is_some() {
                self.pending_looks.push((stack, None));
            }
            l.spells.retain(|h| {
                casts
                    .spells
                    .get(h)
                    .is_some_and(|s| s.original == DISRUPTING_RAY)
            });
            (l.spells.len() != before, l)
        });
        self.pending_tints.extend(
            live.iter()
                .filter(|(ended, _)| *ended)
                .map(|(_, l)| (stack, hue(l, casts))),
        );
        self.corpses.extend(live.map(|(_, l)| (stack, l)));
    }

    /// One command's events, in order.
    fn command(&mut self, events: &[Event]) {
        let mut i = 0;
        // A melee strike of this command played: the next one waits (§7.4).
        let mut struck = false;
        let mut melee: Option<Melee> = None;
        // The striker's facing before the move that brought it to its target.
        let mut before_move: Option<(u8, bool)> = None;
        // A shooter and its facing before its shots.
        let mut shooter: Option<(u8, bool)> = None;
        // The stack the last melee strike struck and its number before it: Life Drain's line
        // names it (E17 M3 T7b-2b-3).
        let mut last_hit: Option<(u8, u32)> = None;
        // The events up to this index belong to a round's start whose expired objects went
        // (E17 M3 T7c-2).
        let mut expired = 0;
        // The stacks the last melee strike struck: their Fire Shields strike back (E17 M3 T7c-3).
        let mut shields: Vec<u8> = Vec::new();
        // The clones gone with their originals in the action (before a round's start): they
        // dissolve with its deaths (E17 M5 T5a).
        self.vanished = events
            .iter()
            .take_while(|e| !matches!(e, Event::RoundStarted { .. }))
            .filter_map(|e| match *e {
                Event::Vanished { stack, .. } => Some(stack),
                _ => None,
            })
            .collect();
        while let Some(event) = events.get(i) {
            i += 1;
            match *event {
                Event::Moved { stack, from, to } => {
                    let mut path = vec![from, to];
                    // A walk comes hex by hex (CLT-Q1): the run of this stack's moves, with what
                    // its hexes bring in a siege (E17 M4 T2c-1): the gate opened at a step, a
                    // mine's blast or the moat's damage after one.
                    let mut entered = siege::Entered::default();
                    while let Some(e) = events.get(i) {
                        match *e {
                            Event::Moved {
                                stack: s,
                                from: f,
                                to: t,
                            } if s == stack && path.last() == Some(&f) => path.push(t),
                            Event::GateOpened {} if entered.gate.is_none() => {
                                entered.gate = Some(path.len() - 2);
                            }
                            Event::MoatDamage { stack: s, .. } if s == stack => {
                                entered.after.push((path.len() - 1, vec![e.clone()]));
                            }
                            Event::ObstacleRevealed { spell, .. }
                                if self.mine_under(stack, spell, events.get(i + 1)) =>
                            {
                                // The blast's events, to the mine's removal.
                                let end = events[i..]
                                    .iter()
                                    .position(|e| matches!(e, Event::ObstacleRemoved { .. }))
                                    .map_or(i + 1, |k| i + k);
                                entered
                                    .after
                                    .push((path.len() - 1, events[i..=end].to_vec()));
                                i = end;
                            }
                            _ => break,
                        }
                        i += 1;
                    }
                    // A melee's target turns back before its striker returns (the Harpies,
                    // 0x4789F8).
                    self.melee_over(melee.as_mut());
                    // A move that brings its stack to strike does not turn back (0x478971: the
                    // attack moves with the flag clear); a plain move does (0x479106).
                    let strikes = events[i..].iter().any(|e| {
                        matches!(*e, Event::Struck { attacker, flags, .. }
                            if attacker == stack && !flags.ranged && !flags.retaliation)
                    });
                    if strikes && melee.is_none() {
                        before_move = self.live(stack).map(|l| (stack, l.right));
                    }
                    self.moved(stack, &path, !strikes, &entered);
                }
                Event::Struck {
                    attacker,
                    target,
                    ref damage,
                    kills,
                    flags,
                } if !flags.secondary => {
                    let mut hit = vec![(target, kills)];
                    let mut hits = vec![(target, damage.0, kills)];
                    // What a hit ends comes before the next hit (the engine's `strike`: Blind,
                    // Paralyze or Stone the damage ends, a clone gone with its original): it takes
                    // effect after the strike (E17 M3 T9b, the reviewer).
                    let mut ended = Vec::new();
                    let mut j = i;
                    while let Some(e) = events.get(j) {
                        match *e {
                            Event::Struck {
                                attacker: a,
                                target: t,
                                damage: ref d,
                                kills: k,
                                flags: f,
                            } if a == attacker && f.secondary => {
                                hit.push((t, k));
                                hits.push((t, d.0, k));
                                ended.extend(events[i..j].iter().cloned());
                                j += 1;
                                i = j;
                            }
                            _ if damage_ended(e) => j += 1,
                            _ => break,
                        }
                    }
                    // The line (E17 M3 T6b): the striker by its number before the strike.
                    let count = self.live(attacker).map_or(0, |l| l.count);
                    // A siege's turret shoots on its own record (E17 M4 T2b): its line names
                    // the arrow tower.
                    let turret = self.live(attacker).and_then(|l| l.turret);
                    if let Some(record) = turret.filter(|_| flags.ranged) {
                        let line = Logged::Turret {
                            hit: (target, damage.0, kills),
                        };
                        self.turret_shot((attacker, record), &hit, line);
                    } else if flags.ranged {
                        if shooter.is_none() {
                            shooter = self.live(attacker).map(|l| (attacker, l.right));
                            self.face_shot(attacker, target);
                        }
                        // Good luck as the shot starts (E17 M3 T7b-2b-2b).
                        if flags.lucky {
                            self.luck(attacker);
                        }
                        // A shooter whose projectile explodes shoots at an area, whatever it
                        // struck (the Magogs, the Liches: 0x43FA0C, 0x43FCFF; E17 M3 T7c-3).
                        let area = hits.len() > 1
                            || self
                                .live(attacker)
                                .is_some_and(|l| l.look.explosion.is_some());
                        let start = self.shot(attacker, &hit);
                        // One target's line after its reactions (0x43FDB4), an area shot's of
                        // every hit as they start (0x43FCE0, 0x4400DC).
                        let (spread, at) = if area {
                            (Spread::Around, start)
                        } else {
                            (Spread::Target, self.clock.now)
                        };
                        let line = Logged::Strike {
                            attacker,
                            count,
                            spread,
                            hits,
                        };
                        self.log.push((at, line));
                        // A one-target shot waits for its shot sound (`stack_shot` 0x43FDF9;
                        // E17 M5 T5b2); an area shot does not (0x4400EB).
                        if !area {
                            self.wait_last(attacker, Sound::Shot);
                        }
                    } else {
                        if struck {
                            self.clock.wait(self.speed.scale(PAUSE));
                        }
                        let layer = if struck { STRUCK } else { STRIKER };
                        struck = true;
                        if melee.is_none() && !flags.retaliation {
                            melee = self.melee(attacker, target, before_move);
                        }
                        // Good luck as the strike starts (E17 M3 T7b-2b-2b).
                        if flags.lucky {
                            self.luck(attacker);
                        }
                        // A breath's line is its target's, a multi-headed striker's every hit's
                        // (0x441B9B, 0x4404ED).
                        let around = hit.len() > 1
                            && !self.live(attacker).is_some_and(|l| self.breath(&l, &hit));
                        let spread = if around {
                            Spread::Around
                        } else {
                            Spread::Target
                        };
                        // The spells it leaves pending, cast at its animation's end (E17 M3
                        // T7b-2b-3).
                        let (casts, next) = self.pending_casts(events, i, attacker, &mut ended);
                        self.strike_casts = casts;
                        i = next;
                        last_hit = self.live(target).map(|l| (target, l.count));
                        shields = hit.iter().map(|h| h.0).collect();
                        self.strike(attacker, &hit, layer);
                        // After the strike's animation (`melee_strike` 0x441C5D).
                        let line = Logged::Strike {
                            attacker,
                            count,
                            spread,
                            hits,
                        };
                        self.log.push((self.clock.now, line));
                    }
                    self.effects(&ended);
                }
                // A hit no strike leads: its reaction alone, its kills taken (E17 M3 T9b, the
                // reviewer: they were counted with no reaction, the dead never buried).
                Event::Struck { target, kills, .. } => {
                    self.react_alone(&[(target, kills)], [STANDING, STRUCK]);
                }
                // A spell after a strike that is the striker's own effect (E17 M3 T7b-2b-3): the
                // Thunderbird's bolt with its damage; another's effects at once.
                Event::StrikeSpell { target, spell, .. } => {
                    let from = i;
                    while events.get(i).is_some_and(|e| {
                        matches!(
                            e,
                            Event::EffectApplied { .. }
                                | Event::EffectEnded { .. }
                                | Event::SpellDamage { .. }
                        )
                    }) {
                        i += 1;
                    }
                    self.strike_spell(target, spell, &events[from..i]);
                }
                Event::StrikeDamage {
                    target,
                    ref damage,
                    kills,
                    ..
                } => self.acid(target, (damage.0, kills)),
                Event::LifeDrained {
                    stack,
                    ref hit_points,
                    resurrected,
                } => {
                    if let Some(hit) = last_hit {
                        self.drain(stack, (hit_points.0, resurrected), hit);
                    }
                }
                Event::DeathStared {
                    attacker,
                    target,
                    kills,
                } => self.death_stare(attacker, target, kills),
                // Fire Shield strikes back at the striker (E17 M3 T7c-3).
                Event::Reflected {
                    stack,
                    ref damage,
                    kills,
                } => self.shield_return(stack, (damage.0, kills), &shields),
                // A Pit Lord's Demons (E17 M5 T5a): the stack they make follows.
                Event::CreatureRaise { stack, target }
                    if let Some(&Event::RoseFromCorpse {
                        stack: new,
                        corpse,
                        creature,
                        count,
                        hex,
                    }) = events.get(i)
                        && corpse == target =>
                {
                    i += 1;
                    self.demons(stack, corpse, (new, creature, count, hex));
                }
                // An Archangel's raise (E17 M3 T7c-3): the stack it raises follows.
                Event::CreatureRaise { stack, target } => {
                    let raised = match events.get(i) {
                        Some(&Event::SpellRaised {
                            target: t,
                            creatures,
                            ..
                        }) if t == target => {
                            i += 1;
                            creatures
                        }
                        _ => 0,
                    };
                    self.creature_raise(stack, target, raised);
                }
                // A Phoenix reborn (E17 M3 T7c-3).
                Event::Reborn { stack, creatures } => self.reborn(stack, creatures),
                Event::Defended { stack } => {
                    self.defends(stack, true);
                    // A war machine writes no line (0x479758).
                    if let Some(live) = self.live(stack).filter(|l| !l.war_machine) {
                        let line = Logged::Defend {
                            stack,
                            count: live.count,
                            defense: live.defense,
                        };
                        self.log.push((self.clock.now, line));
                    }
                }
                Event::Waited { stack } => {
                    // None in the tactics phase (0x479918).
                    if let Some(live) = self.live(stack).filter(|_| !self.tactics) {
                        let line = Logged::Wait {
                            stack,
                            count: live.count,
                        };
                        self.log.push((self.clock.now, line));
                    }
                }
                Event::Healed {
                    healer,
                    target,
                    hit_points,
                } => self.heal(healer, target, hit_points),
                Event::SpellCast {
                    side,
                    spell,
                    target,
                    ..
                } => {
                    // Its effects follow it (E17 M3 T7b-1; the objects it places, T7c-2; Magic
                    // Mirror's redirect before them, T9b).
                    let from = i;
                    while events.get(i).is_some_and(|e| {
                        matches!(
                            e,
                            Event::SpellRedirected { .. }
                                | Event::EffectApplied { .. }
                                | Event::EffectEnded { .. }
                                | Event::SpellResisted { .. }
                                | Event::SpellDamage { .. }
                                | Event::SpellHealed { .. }
                                | Event::ObstaclePlaced { .. }
                                | Event::SpellRaised { .. }
                                | Event::Sacrificed { .. }
                                | Event::WallQuaked { .. }
                                | Event::WallDestroyed { .. }
                                | Event::StackCreated { .. }
                                | Event::NotCreated { .. }
                                | Event::TurnedSummoned { .. }
                                | Event::Teleported { .. }
                        )
                    }) {
                        i += 1;
                    }
                    self.cast(side, spell, target, &events[from..i]);
                }
                // A creature's cast as its action (E17 M3 T7c-1): its effects follow it.
                Event::StackCast {
                    stack,
                    spell,
                    target,
                } => {
                    let from = i;
                    while events.get(i).is_some_and(|e| {
                        matches!(
                            e,
                            Event::EffectApplied { .. }
                                | Event::EffectEnded { .. }
                                | Event::SpellResisted { .. }
                                | Event::SpellDamage { .. }
                                | Event::SpellHealed { .. }
                        )
                    }) {
                        i += 1;
                    }
                    self.creature_cast(stack, spell, target, &events[from..i]);
                }
                // An effect's start or end outside a hero's cast: a round's start ends the spells
                // that ran out (0x447120), a creature's ability or cast adds one (E17 M3 T7b-2a).
                Event::EffectApplied { .. } | Event::EffectEnded { .. } => self.effect(event),
                // Good morale once the action is over (E17 M3 T7b-2b-2b): its shooter or striker
                // turned back.
                Event::GoodMorale { stack } => {
                    self.attack_over(&mut melee, &mut shooter);
                    self.good_morale(stack);
                }
                Event::BadMorale { stack } => self.frozen(stack, false),
                Event::Frightened { stack } => self.frozen(stack, true),
                Event::TacticsEnded {} => self.tactics = false,
                Event::BattleEnded { result } => self.outcome = Some(result),
                // A wall shot (E17 M4 T2b): its part's destruction is its damage's.
                Event::WallShot {
                    stack,
                    part,
                    damage,
                    ..
                } => self.wall_shot(stack, usize::from(part), damage),
                // The gate (E17 M4 T2c-1): closed after a walk, a flight, a teleporting move or
                // an action (`gate_close` 0x4672E0); opened outside a move (a summoned
                // elemental placed on it, 0x5A71F1; *provisional*: not recorded).
                Event::GateClosed {} => self.drawbridge(false),
                Event::GateOpened {} => self.drawbridge(true),
                // The moat's damage on a stack that acts where it stands in it (the re-entry,
                // `reenter_own_hexes` 0x441F70), as on entering it without the walk.
                Event::MoatDamage {
                    stack,
                    ref damage,
                    kills,
                    ..
                } => self.moat(stack, (damage.0, kills)),
                Event::RoundStarted { .. } => {
                    self.acting = None;
                    self.happenings.push(Happening::Round);
                    // The objects whose last round passed go first, each with its vanish
                    // animation, then the round's line (`round_start` 0x476152 to 0x4761E9; E17
                    // M3 T7c-2).
                    let start = &events[i..];
                    let ends = start
                        .iter()
                        .position(|e| {
                            matches!(e, Event::TurnStarted { .. } | Event::RoundStarted { .. })
                        })
                        .unwrap_or(start.len());
                    // The clones whose rounds ran out first, in one dissolve (0x47614D; E17 M5
                    // T5a), after any gone with an original no death of the action played.
                    self.remove_gone(&[]);
                    self.vanish_at_round(&start[..ends]);
                    for e in &start[..ends] {
                        if let Event::ObstacleRemoved { hexes, .. } = e {
                            self.vanish(hexes);
                        }
                    }
                    expired = i + ends;
                    if !self.tactics {
                        self.log.push((self.clock.now, Logged::Round));
                    }
                }
                // An object gone outside a round's start (a spell's removal; E17 M3 T7c-2,
                // *provisional*: not recorded).
                Event::ObstacleRemoved { ref hexes, .. } if i > expired => self.vanish(hexes),
                // An object placed outside a hero's cast (a town's mines at the battle's start)
                // or one the viewer did not see revealed (E17 M3 T7c-2): drawn from the next
                // frame on; one the viewer saw hidden from the other side shows to both from then
                // on (E17 M5 T2b).
                Event::ObstaclePlaced {
                    spell, ref hexes, ..
                }
                | Event::ObstacleRevealed {
                    spell, ref hexes, ..
                } => {
                    let anchor = hexes.iter().max();
                    let known = anchor.is_some_and(|a| self.placed.contains_key(a));
                    let revealed = matches!(event, Event::ObstacleRevealed { .. });
                    if !known {
                        let owner = self.owner(hexes);
                        self.place(spell, hexes, owner.filter(|_| !revealed));
                    } else if revealed && anchor.is_some_and(|a| self.hidden.contains(a)) {
                        self.place(spell, hexes, None);
                    }
                }
                Event::TurnStarted { stack } => {
                    // The attack ends before the next turn starts (E17 M2 T5c, the reviewer); a
                    // turn the command plays on (a berserk stack's, a war machine's) is an attack
                    // of its own (E17 M3 T9b, the reviewer).
                    self.attack_over(&mut melee, &mut shooter);
                    struck = false;
                    before_move = None;
                    last_hit = None;
                    shields.clear();
                    // A war machine's turn that drew nothing keeps its outline no longer (E17 M3
                    // T9b, the reviewer: the next command's first frame cleared it).
                    self.machine = None;
                    // The turn's start redraws the battle (its outline): the hues a round's start
                    // changed show (E17 M3 T7b-2a, `resist` at 2 900: Slow's end).
                    self.show_tints(self.clock.now);
                    self.defends(stack, false);
                    // A stack its pick freezes (bad morale, Fear) never becomes the active one:
                    // the pick goes on to the next (`next_stack_to_act` 0x465283; E17 M3
                    // T7b-2b-2b).
                    let frozen = freezes(&events[i..], stack);
                    if !frozen {
                        self.turns.push((self.clock.now, stack));
                        let controller = self.controller(stack);
                        self.controllers.push(controller);
                        self.log.push((self.clock.now, Logged::Turn(stack)));
                        self.acting = Some(stack);
                    }
                    // A war machine's turn the command plays out (another turn follows in it):
                    // the battle loop ticks once as it acts, as for the computer's, which
                    // outlines it until its action's first frame shows (E17 M3 T3b, `machines`).
                    let later = events[i..]
                        .iter()
                        .any(|e| matches!(e, Event::TurnStarted { .. }));
                    let machine = self.stacks.get(&stack).is_some_and(|l| l.war_machine);
                    let now = self.clock.now;
                    if later && machine && self.idle.once(now, &mut self.steps) {
                        self.outlined.push((now, Some(stack)));
                        self.machine = Some(stack);
                    }
                }
                _ => {}
            }
        }
        self.attack_over(&mut melee, &mut shooter);
        // A clone gone with its original where no death played (E17 M5 T5a).
        self.remove_gone(&[]);
    }

    /// The First Aid Tent heals `target` (`first_aid_tent_action` 0x478B90, design §7.15; E17
    /// M3 T3b): the tent does not move and stays the active stack, outlined from its turn's idle
    /// step (or, commanded, the heal's first frame) to the heal's end; the heal's sound starts, then magic animation
    /// [`HEAL`] over the patient, a frame per redraw of trunc(100 F) on the deadline
    /// (`PlayMagicAnimation` 0x496840 without the hit), and a last redraw that erases it. The
    /// battle then waits for the sound to end (`wait_for_sample` 0x59A4C0; E17 M5 T5b2), which
    /// it has at speed 0 (`Regener`, 1 881 ms, against the animation's 2 100), not at the faster
    /// speeds; the tent's outline goes with the next redraw, after the wait. The log's line comes
    /// between the sound and the animation (0x478D0A, E17 M3 T6b).
    fn heal(&mut self, healer: u8, target: u8, hit_points: u32) {
        let cue = self.cue(target, Sound::Heal, false);
        let count = |stack: u8| self.live(stack).map_or(0, |l| l.count);
        let line = Logged::Heal {
            healer,
            target,
            counts: [count(healer), count(target)],
            hit_points,
        };
        self.log.push((self.clock.now, line));
        let Some(look) = self.stage.magic.get(&HEAL).cloned() else {
            return;
        };
        let over: Vec<(u8, [i32; 2], bool)> = self
            .live(target)
            .map(|live| (target, self.magic_at(&live, &look), !live.right))
            .into_iter()
            .collect();
        // The tent stays the active stack: outlined through the heal.
        let ticked = self.machine.take() == Some(healer);
        let time = self.speed.scale(TICK);
        for frame in (0..look.frames).map(Some).chain([None]) {
            let drawn = self.clock.now;
            let shown = self.clock.show(time);
            self.flush(drawn, shown);
            if frame == Some(0) && !ticked {
                self.outlined.push((shown, Some(healer)));
            }
            self.magics.push(MagicKey {
                animation: HEAL,
                drawn,
                shown,
                frame,
                def: look.def.clone(),
                translucent: look.translucent,
                over: over.clone(),
            });
        }
        self.wait_for(cue);
        self.outlined.push((self.clock.now, None));
    }

    /// The canvas's top-left corner of magic animation `look` over `live` (`draw_battle_stack`
    /// 0x43E7FD to 0x43E98C): centred on its position's anchor (on a two-hex stack's middle, 22
    /// pixels toward its front), standing on the anchor (place 0), centred on the creature, whose
    /// height is 267 less its standing frame's top (+0x16C, 0x43DD62; place 1), or above it
    /// (place 2); the halves truncated toward zero. Place 3 (0x43E906, E17 M3 T7b-1): beside the
    /// standing frame's front, its left edge the anchor + (the front − 196) facing right, its
    /// right edge the anchor − that facing left, no two-hex shift; its height as place 1. The
    /// higher ones are not over a stack: they stand on the anchor (*provisional*).
    fn magic_at(&self, live: &Live, look: &MagicLook) -> [i32; 2] {
        let [x, y] = self.stage.anchor(live.position(live.hex));
        let middle = if !live.two_hex {
            0
        } else if live.right {
            HALF_HEX
        } else {
            -HALF_HEX
        };
        let [w, h] = look.size;
        let height = self.stage.layout.stack_canvas[1] - live.look.top;
        let top = match look.place {
            1 | 3 => y - h / 2 - height / 2,
            2 => y - h - height,
            _ => y - h,
        };
        if look.place == 3 {
            let beside = live.look.front - BESIDE;
            let left = if live.right {
                x + beside
            } else {
                x - beside - w
            };
            return [left, top];
        }
        [x + middle - w / 2, top]
    }

    /// A magic animation over one stack (`play_magic_animation` 0x496840, design §7.15, §7.16):
    /// with `hit` (E17 M3 T7b-2b-2b: bad morale, Fear) the stack first plays its hit group from
    /// frame 0, a redraw at trunc(100 F) on the deadline each, the animation's frame i beside it
    /// (none past its last: a hit longer than the animation, not met, *provisional*), then stands,
    /// shown at once when the hit had at least as many frames (0x49697D); then the animation's
    /// frames left at `time` each on the deadline, and a last redraw that erases it. Returns when
    /// its first frame shows (none: the stage has no such animation).
    fn over_one(
        &mut self,
        number: u8,
        (target, live): (u8, &Live),
        time: u32,
        hit: bool,
    ) -> Option<u32> {
        let look = self.stage.magic.get(&number).cloned()?;
        let over = vec![(target, self.magic_at(live, &look), !live.right)];
        let mut first = None;
        let mut left = 0;
        if hit {
            let group = self.stage.groups.hit;
            let frames = live.look.count(group);
            let tick = self.speed.scale(TICK);
            for f in 0..frames {
                let pose = Self::posed(live, group, f, live.hex);
                self.pending.push((target, pose));
                let frame = (f < look.frames).then_some(f);
                let shown = self.magic_frame((number, &look), &over, frame, tick);
                first = first.or(Some(shown));
            }
            self.pending.push((target, self.standing(live, live.hex)));
            if frames >= look.frames {
                self.redraw();
            }
            left = frames;
        }
        for frame in (left..look.frames).map(Some).chain([None]) {
            let shown = self.magic_frame((number, &look), &over, frame, time);
            first = first.or(Some(shown));
        }
        first
    }

    /// One redraw of a magic animation over its stacks (`battle_redraw` 0x494440 waiting `time`
    /// on the deadline): `frame` (none: erased) drawn now and shown at the deadline; returns
    /// when it shows.
    fn magic_frame(
        &mut self,
        (number, look): (u8, &MagicLook),
        over: &[(u8, [i32; 2], bool)],
        frame: Option<u32>,
        time: u32,
    ) -> u32 {
        let drawn = self.clock.now;
        let shown = self.clock.show(time);
        self.flush(drawn, shown);
        self.magics.push(MagicKey {
            animation: number,
            drawn,
            shown,
            frame,
            def: look.def.clone(),
            translucent: look.translucent,
            over: over.to_vec(),
        });
        shown
    }

    /// Good luck on `stack`'s strike or shot (`melee_strike` 0x441837, `stack_shot` 0x43F955;
    /// E17 M3 T7b-2b-2b, design §7.16): its sound starts, the log's line, then animation
    /// [`GOOD_LUCK`] over the striker or shooter without its hit, before the strike's or the
    /// shot's own animation (after the shooter's turn and a melee's pause and turns).
    fn luck(&mut self, stack: u8) {
        let Some(live) = self.live(stack) else {
            return;
        };
        self.cue(stack, Sound::Luck, false);
        let line = Logged::Luck {
            stack,
            count: live.count,
        };
        self.log.push((self.clock.now, line));
        self.over_one(GOOD_LUCK, (stack, &live), self.speed.scale(TICK), false);
    }

    /// Good morale after `stack`'s action (`good_morale_roll` 0x4649D3; E17 M3 T7b-2b-2b, design
    /// §7.16): its sound starts, animation [`GOOD_MORALE`] over it without its hit, the log's line
    /// after it (0x464AD5), then the sound waited for (`wait_for_sample` 0x59A4C0, E17 M5 T5b2;
    /// nothing with the sound off) and a redraw shown at once (0x464B23). The stack that acted is outlined again from the animation's
    /// first frame ([`Player::acting`], the recorded `morale`).
    fn good_morale(&mut self, stack: u8) {
        let Some(live) = self.live(stack) else {
            return;
        };
        let cue = self.cue(stack, Sound::GoodMorale, false);
        let first = self.over_one(GOOD_MORALE, (stack, &live), self.speed.scale(TICK), false);
        if let Some(first) = first {
            self.outlined.push((first, self.acting));
        }
        let line = Logged::Morale {
            stack,
            count: live.count,
            good: true,
        };
        self.log.push((self.clock.now, line));
        self.wait_for(cue);
        self.redraw();
    }

    /// `stack` frozen as its pick asks (`next_stack_to_act` 0x465283, 0x465293; E17 M3
    /// T7b-2b-2b, design §7.16): bad morale's or (`fear`) Fear's sound starts, the log's line,
    /// then animation [`BAD_MORALE`] or [`FEAR`] over it with its hit (`bad_morale_check`
    /// 0x464D10, `fear_check` 0x464F33), the sound waited for as good morale's. Fear's line names
    /// the frightening creature by the number of those on the other side (0x464EE0): the first
    /// such stack's, their numbers summed (*provisional*: one creature frightens, the original's
    /// 132; the stack's side, not a hypnotized stack's controller).
    fn frozen(&mut self, stack: u8, fear: bool) {
        let Some(live) = self.live(stack) else {
            return;
        };
        let count = live.count;
        let (sound, number, line) = if fear {
            let mut frighteners = self
                .stacks
                .iter()
                .filter(|(_, l)| l.frightens && l.side != live.side && l.count > 0);
            let by = frighteners.next().map(|(&id, l)| (id, l.count));
            let (by, first) = by.unwrap_or((stack, 0));
            let number = first + frighteners.map(|(_, l)| l.count).sum::<u32>();
            let line = Logged::Fear {
                stack,
                count,
                by,
                number,
            };
            (Sound::Fear, FEAR, line)
        } else {
            let line = Logged::Morale {
                stack,
                count,
                good: false,
            };
            (Sound::BadMorale, BAD_MORALE, line)
        };
        let cue = self.cue(stack, sound, false);
        self.log.push((self.clock.now, line));
        // The stack that acted last stays outlined ([`Player::acting`], the recorded `bad`).
        if let Some(first) = self.over_one(number, (stack, &live), self.speed.scale(TICK), true) {
            self.outlined.push((first, self.acting));
        }
        // 0x464D1D, 0x464F3D (E17 M5 T5b2).
        self.wait_for(cue);
    }

    /// The spells a melee strike of `attacker` leaves pending ([`PENDING`]), from `events[i]` on:
    /// each its target, its handle and the effects that follow it; and where the events go on.
    fn pending_casts(
        &self,
        events: &[Event],
        mut i: usize,
        attacker: u8,
        ended: &mut Vec<Event>,
    ) -> (Vec<StrikeCast>, usize) {
        let mut casts = Vec::new();
        loop {
            // What the last hit ended comes first ([`damage_ended`]).
            let mut j = i;
            while events.get(j).is_some_and(damage_ended) {
                j += 1;
            }
            let Some(&Event::StrikeSpell {
                striker,
                target,
                spell,
            }) = events.get(j)
            else {
                break;
            };
            let pending = self
                .stage
                .casts
                .spells
                .get(&spell)
                .is_some_and(|l| PENDING.contains(&l.original));
            if striker != attacker || !pending {
                break;
            }
            ended.extend(events[i..j].iter().cloned());
            let from = j + 1;
            let mut end = from;
            while events.get(end).is_some_and(|e| {
                matches!(e, Event::EffectApplied { .. } | Event::EffectEnded { .. })
            }) {
                end += 1;
            }
            casts.push(StrikeCast {
                target,
                spell,
                effects: events[from..end].to_vec(),
            });
            i = end;
        }
        (casts, i)
    }

    /// A spell a melee strike left pending, cast as a creature's at its animation's end
    /// (`strike_animation` 0x469102, `cast_spell` 0x59FE30 with caster kind 1; design §7.16; E17
    /// M3 T7b-2b-3): its own sound (category 3, 0x5A0331), then its handler: Poison's effect, its
    /// line (`battle_log_cast` 0x5A8EFC) and its animation over the stack with the stack's hit
    /// (0x5A15AF); Acid Breath its animation, the defense lowered by 3 (to 0 at least) and the
    /// executable's own line (0x5A1E70 to 0x5A1F53), no spell set; the others' effects at once
    /// (not built: their handlers and lines). A cast clears the outline as it starts (0x5A004C,
    /// 0x477B60) and outlines the stack whose turn it is at its end (`cast_spell`'s tail
    /// 0x5A216B, 0x477B16), each in a redraw shown at once (the recorded `acid`: the
    /// Rust Dragon outlined from Acid Breath's end, not from the Wyvern Monarchs' Poison's start
    /// to its end); the tail draws nothing else for a creature.
    fn strike_cast(
        &mut self,
        StrikeCast {
            target,
            spell,
            effects,
        }: StrikeCast,
    ) {
        let look = self
            .stage
            .casts
            .spells
            .get(&spell)
            .cloned()
            .unwrap_or_default();
        // Stone Gaze takes hold only where the engine applied it (E17 M5 T5b1).
        let stone = look.original == STONE_GAZE
            && effects
                .iter()
                .any(|e| matches!(*e, Event::EffectApplied { target: t, .. } if t == target));
        let Some(live) = self
            .live(target)
            .filter(|_| stone || matches!(look.original, POISON | ACID_BREATH))
        else {
            self.effects(&effects);
            return;
        };
        let time = self.speed.scale(TICK);
        // The outline cleared as the cast starts, in a redraw shown at once, if a cast set it.
        if self.outlined.last().is_some_and(|(_, s)| s.is_some()) {
            self.outlined.push((self.clock.now, None));
        }
        self.cue(target, Sound::Spell(spell), false);
        if stone {
            // Stone Gaze (0x5A10F1): its effect, its line (GENRLTXT 559 or 560 by the stack's
            // number, `battle_log_cast` 0x5A8E25), its glow; then the stack looks petrified.
            self.effects(&effects);
            let line = Logged::Petrified {
                stack: target,
                count: live.count,
            };
            self.log.push((self.clock.now, line));
            self.glow(&[target], &STONE_STEPS, Effect::StoneGlow);
            self.outlined.push((self.clock.now, self.acting));
            return;
        }
        let poison = look.original == POISON;
        // Acid Breath's defense lowered (none for Poison).
        let lowered = (!poison).then(|| live.defense.clamp(0, ACID_DEFENSE));
        if let Some(lowered) = lowered {
            self.stacks
                .entry(target)
                .and_modify(|l| l.defense -= lowered);
        } else {
            self.effects(&effects);
            let line = Logged::Poisoned {
                stack: target,
                count: live.count,
            };
            self.log.push((self.clock.now, line));
        }
        if let Some(number) = look.animation {
            self.over_one(number, (target, &live), time, poison);
        }
        if let Some(lowered) = lowered {
            let line = Logged::AcidBreath {
                stack: target,
                count: live.count,
                lowered,
            };
            self.log.push((self.clock.now, line));
        }
        // The outline: the turn's stack's from the cast's end.
        self.outlined.push((self.clock.now, self.acting));
    }

    /// A spell after a strike that is the striker's own effect (`striker_effects` 0x440BC0; E17
    /// M3 T7b-2b-3): the Thunderbird's bolt (0x441185 to 0x441374) with its damage: the log's
    /// line, its sound, the bolt from the sky over the stack, the damage's line (named by the
    /// spell), then [`THUNDERBOLT`] over the marked stack with its reaction; the sound waited for
    /// as good morale's (0x441374, E17 M5 T5b2). Any other its effects at once (the flies'
    /// dispels and Weakness: not built), as a pending one the strike did not take.
    fn strike_spell(&mut self, target: u8, spell: u16, effects: &[Event]) {
        let damage = effects.iter().find_map(|e| match *e {
            Event::SpellDamage {
                target: t,
                ref damage,
                kills,
                ..
            } if t == target => Some((damage.0, kills)),
            _ => None,
        });
        let (Some((damage, kills)), Some(live)) = (damage, self.live(target)) else {
            self.effects(effects);
            return;
        };
        let line = Logged::Thunderbolt {
            stack: target,
            count: live.count,
        };
        self.log.push((self.clock.now, line));
        let cue = self.cue(target, Sound::Thunderbolt, false);
        self.over_one(BOLT, (target, &live), self.speed.scale(BOLT_TIME), false);
        let line = Logged::SpellDamage {
            spell,
            hit: (target, damage, kills),
        };
        self.log.push((self.clock.now, line));
        self.marked((target, &live), kills, Some(THUNDERBOLT));
        self.wait_for(cue);
        self.effects(effects);
    }

    /// The handle of the record's spell numbered `original` in the executable, if a cast needs it.
    fn spell_handle(&self, original: u16) -> Option<u16> {
        self.stage
            .casts
            .spells
            .iter()
            .find(|(_, l)| l.original == original)
            .map(|(&h, _)| h)
    }

    /// The Rust Dragon's acid on the stack it struck (`striker_effects` 0x44149F to 0x441586;
    /// E17 M3 T7b-2b-3): Acid Breath's sound (category 2), the damage and its line (named by Acid
    /// Breath), animation [`ACID`] over the stack without its hit, the sound waited for
    /// (0x441577, E17 M5 T5b2), then the stack's reaction (`strike_animation` without an
    /// animation).
    fn acid(&mut self, target: u8, (damage, kills): (u64, u32)) {
        let Some(live) = self.live(target) else {
            return;
        };
        let mut cue = None;
        if let Some(spell) = self.spell_handle(ACID_BREATH) {
            cue = Some(self.cue(target, Sound::SpellFile(spell), false));
            let line = Logged::SpellDamage {
                spell,
                hit: (target, damage, kills),
            };
            self.log.push((self.clock.now, line));
        }
        self.over_one(ACID, (target, &live), self.speed.scale(TICK), false);
        if let Some(cue) = cue {
            self.wait_for(cue);
        }
        self.react_alone(&[(target, kills)], [STRIKER, STRUCK]);
    }

    /// Life Drain after a strike of `stack` (`striker_effects` 0x440C01 to 0x440E8D; E17 M3
    /// T7b-2b-3): the creatures it raised join the stack, the log's line (the drinkers by their
    /// number before them, the stack `hit` struck by its number before the strike), its sound,
    /// then animation [`LIFE_DRAIN`] over the drinkers without their hit, the box showing their
    /// new number from its first frame; the sound waited for (0x440E8D, E17 M5 T5b2).
    fn drain(
        &mut self,
        stack: u8,
        (hit_points, raised): (u64, u32),
        (target, target_count): (u8, u32),
    ) {
        let Some(live) = self.stacks.get_mut(&stack) else {
            return;
        };
        let count = live.count;
        live.count = live.count.saturating_add(raised);
        let live = live.clone();
        let line = Logged::Drain {
            stack,
            count,
            hit_points,
            raised,
            target,
            target_count,
        };
        self.log.push((self.clock.now, line));
        let cue = self.cue(stack, Sound::Drain, false);
        let first = self.over_one(LIFE_DRAIN, (stack, &live), self.speed.scale(TICK), false);
        if let Some(shown) = first.filter(|_| raised > 0) {
            self.counts.push(CountKey {
                stack,
                shown,
                count: live.count,
            });
        }
        self.wait_for(cue);
    }

    /// Death Stare after a strike of `starer` on `target` (`striker_effects` 0x440EB6 to
    /// 0x441178; E17 M3 T7b-2b-3): the log's line, Death Stare's sound (category 2), animation
    /// [`DEATH_STARE`] over the stack without its hit, then the kills and the stack's reaction
    /// (`strike_animation` without an animation); the sound waited for (0x441178, E17 M5 T5b2).
    fn death_stare(&mut self, starer: u8, target: u8, kills: u32) {
        let (Some(gorgons), Some(live)) = (self.live(starer), self.live(target)) else {
            return;
        };
        let line = Logged::DeathStare {
            stack: starer,
            count: gorgons.count,
            target,
            kills,
        };
        self.log.push((self.clock.now, line));
        let cue = self
            .spell_handle(DEATH_STARE_SPELL)
            .map(|spell| self.cue(target, Sound::SpellFile(spell), false));
        self.over_one(DEATH_STARE, (target, &live), self.speed.scale(TICK), false);
        self.react_alone(&[(target, kills)], [STRIKER, STRUCK]);
        if let Some(cue) = cue {
            self.wait_for(cue);
        }
    }

    /// A frame of the hero of `side` (0 the attacker's): drawn now and shown at the deadline,
    /// which then moves on by `time` (a redraw with its frame wait); with `at_once`, shown now,
    /// the deadline as it is (E17 M3 T7b-1).
    fn hero_frame(&mut self, side: u8, (group, frame): (u8, u32), time: u32, at_once: bool) {
        let drawn = self.clock.now;
        let shown = if at_once {
            drawn
        } else {
            self.clock.show(time)
        };
        self.flush(drawn, shown);
        self.hero_keys.push(HeroKey {
            side,
            drawn,
            shown,
            group: u32::from(group),
            frame,
        });
    }

    /// A hero's cast of `spell` at `target` with its `effects` (`cast_spell` 0x59FE30, design
    /// §7.16; E17 M3 T7b-1, T7b-2a): the hero's casting frames before the spell, a redraw at
    /// trunc(100 F) each; then, if the stack it was aimed at resisted, the resistance's sound, the
    /// log's line and animation [`RESISTED`] over it (0x5A1F89), else the spell's own sound
    /// (0x5A0331) and its handler: a timed spell on one stack its effect and line, then its
    /// animation over it (0x5A107F; player 1); Magic Arrow and Ice Bolt their projectile, then
    /// their animation over the struck stack with its reaction and their damage's line
    /// (0x5A098A; player 2); Dispel its animation, then its effect and line (0x5A15E0); Cure its
    /// effect, then its animation and line (0x5A17F5); Disrupting Ray its projectile, its
    /// animation, then its effect and its own line (0x5A1A84). The common tail (0x5A2058): the
    /// rest of the casting group at trunc(100 F) each, then the hero stands in a redraw shown at
    /// once. Other spells play the hero's frames alone and take effect at the cast's end (T7c).
    fn cast(&mut self, side: Side, spell: u16, target: Target, effects: &[Event]) {
        let s = u8::from(side == Side::Defender);
        let casts = self.stage.casts;
        let caster = casts.casters[usize::from(s)];
        let [casting, standing] = casts.groups;
        let time = self.speed.scale(TICK);
        if let Some(c) = caster {
            for f in 0..c.before {
                self.hero_frame(s, (casting, f), time, false);
            }
        }
        // Magic Mirror sends it on to a stack of the caster's side (the engine's `mirror`: its
        // `SpellRedirected` comes before the effects; E17 M3 T9b, the reviewer: it was drawn at
        // the mirroring stack, its kills lost). The mirror's own animation is not built.
        let target = effects
            .iter()
            .find_map(|e| match *e {
                Event::SpellRedirected { to, .. } => Some(Target::Stack { stack: to }),
                _ => None,
            })
            .unwrap_or(target);
        let aimed = match target {
            Target::Stack { stack } => self.live(stack).map(|l| (stack, l)),
            _ => None,
        };
        let look = casts.spells.get(&spell).cloned().unwrap_or_default();
        let resisted = |t: u8| {
            effects
                .iter()
                .any(|e| matches!(*e, Event::SpellResisted { target } if target == t))
        };
        // The raises (E17 M3 T7c-3: `raise_stack`, player 4) and Chain Lightning (its handler's
        // own rays) aim at a corpse or at several stacks; Earthquake at the walls (E17 M4 T2c-2).
        let mut done = self.raising(s, (spell, &look), effects)
            || (look.original == CHAIN_LIGHTNING && self.chain(spell, &look, effects))
            || (look.original == EARTHQUAKE && self.quaking(spell, effects))
            || self.creating(s, spell, effects)
            || (look.original == TELEPORT && self.teleporting(spell, effects));
        match aimed.filter(|_| !done) {
            Some((t, live)) if resisted(t) => {
                let cue = self.cue(t, Sound::Resisted, false);
                let line = Logged::Resisted {
                    stack: t,
                    count: live.count,
                };
                self.log.push((self.clock.now, line));
                self.over_one(RESISTED, (t, &live), time, false);
                // 0x5A2053 (E17 M5 T5b2).
                self.wait_for(cue);
            }
            Some((t, live)) => {
                self.cue(t, Sound::Spell(spell), false);
                let line = Logged::Cast {
                    side: s,
                    spell,
                    target: t,
                    count: live.count,
                };
                let over = |player: &mut Self| {
                    if let Some(number) = look.animation {
                        player.over_one(number, (t, &live), time, false);
                    }
                };
                done = true;
                match look.original {
                    DISPEL => {
                        over(self);
                        self.effects(effects);
                        self.log.push((self.clock.now, line));
                    }
                    CURE => {
                        self.effects(effects);
                        over(self);
                        self.log.push((self.clock.now, line));
                    }
                    DISRUPTING_RAY => {
                        if let Some(c) = caster.filter(|_| !look.projectile.is_empty()) {
                            self.spell_projectile(c.point, &live, &look.projectile);
                        }
                        // 0x5A1A8C to 0x5A1ACB: its defense lowered by the value, to 0 at least.
                        let lowered = live.defense.min(i32::from(look.value[usize::from(s)]));
                        self.stacks
                            .entry(t)
                            .and_modify(|l| l.defense -= lowered.max(0));
                        over(self);
                        self.effects(effects);
                        let line = Logged::Disrupted {
                            stack: t,
                            lowered: lowered.max(0),
                        };
                        self.log.push((self.clock.now, line));
                    }
                    _ => {
                        let damage = effects.iter().find_map(|e| match *e {
                            Event::SpellDamage {
                                target,
                                ref damage,
                                kills,
                                ..
                            } if target == t => Some((damage.0, kills)),
                            _ => None,
                        });
                        let applied = effects.iter().any(
                            |e| matches!(*e, Event::EffectApplied { target, .. } if target == t),
                        );
                        match (damage, caster) {
                            // Lightning Bolt (0x5A0B0C; E17 M3 T7b-2b-3): the bolt from the sky,
                            // then its animation over the marked stack, the line after it.
                            (Some((damage, kills)), _)
                                if matches!(look.original, LIGHTNING_BOLT | TITAN_BOLT) =>
                            {
                                let bolt = self.speed.scale(BOLT_TIME);
                                self.over_one(BOLT, (t, &live), bolt, false);
                                self.marked((t, &live), kills, look.animation);
                                let line = Logged::SpellDamage {
                                    spell,
                                    hit: (t, damage, kills),
                                };
                                self.log.push((self.clock.now, line));
                            }
                            (Some((damage, kills)), Some(c)) if !look.projectile.is_empty() => {
                                self.spell_projectile(c.point, &live, &look.projectile);
                                self.marked((t, &live), kills, look.animation);
                                let line = Logged::SpellDamage {
                                    spell,
                                    hit: (t, damage, kills),
                                };
                                self.log.push((self.clock.now, line));
                            }
                            (Some(_), _) => self.spell_hits((spell, look.animation), effects),
                            _ if applied => {
                                self.effects(effects);
                                self.log.push((self.clock.now, line));
                                // Bloodlust glows instead of an animation (0x5A122B; E17 M5
                                // T5b1).
                                if look.original == BLOODLUST {
                                    self.glow(&[t], &BLOODLUST_STEPS, Effect::Bloodlust);
                                } else {
                                    over(self);
                                }
                            }
                            _ => done = false,
                        }
                    }
                }
            }
            None if !done => done = self.untargeted(s, (spell, &look), target, effects),
            None => {}
        }
        if !done {
            self.effects(effects);
        }
        if let Some(c) = caster {
            for f in c.before..c.frames {
                self.hero_frame(s, (casting, f), time, false);
            }
            self.hero_frame(s, (standing, 0), time, true);
        }
    }

    /// A hero's spell aimed at no stack (E17 M3 T7c-1, design §7.16; the cast's sound has started
    /// at the handler, 0x5A0331): Death Ripple its cast's line, its damage's line, then animation
    /// 8 over every stack struck with their reactions (player 3, 0x5A0C81), and a redraw shown at
    /// once (0x5A0E6D); Destroy Undead the same with its damage's line after the animation
    /// (0x5A0FC6, *provisional*: not recorded); a timed spell on several stacks (Expert, or a mass
    /// spell) its effects, its cast's line, then its animation over them without reactions
    /// (0x5A10C2 to 0x5A10E7); an area spell at a hex its animation there, its damage's line and
    /// the reactions ([`Player::area`]). Every box shows its count again at the cast's tail
    /// (0x5A2089), with the next frame drawn. Returns whether it drew the spell (its effects
    /// taken); the others take effect at the cast's end (T7c-2, T7c-3).
    fn untargeted(
        &mut self,
        side: u8,
        (spell, look): (u16, &SpellLook),
        target: Target,
        effects: &[Event],
    ) -> bool {
        // Each stack struck: its damage before and after its own modifiers, its kills.
        let hits: Vec<(u8, u64, u64, u32)> = effects
            .iter()
            .filter_map(|e| match *e {
                Event::SpellDamage {
                    target,
                    ref caster,
                    ref damage,
                    kills,
                    ..
                } => Some((target, caster.0, damage.0, kills)),
                _ => None,
            })
            .collect();
        let applied: Vec<u8> = effects
            .iter()
            .filter_map(|e| match *e {
                Event::EffectApplied { target, .. } => Some(target),
                _ => None,
            })
            .collect();
        let owner = hits.first().map_or(applied.first().copied(), |h| Some(h.0));
        let cast_line = Logged::CastAll { side, spell };
        match look.original {
            DEATH_RIPPLE | DESTROY_UNDEAD => {
                self.cue(owner.unwrap_or(0), Sound::Spell(spell), false);
                self.log.push((self.clock.now, cast_line));
                // One stack's damage when it struck one, else the caster's (0x5A0DC6); no line
                // without a stack struck (the record gives no value then, *provisional*).
                let damage = match hits.as_slice() {
                    [(_, _, damage, _)] => Some(*damage),
                    _ => hits.first().map(|h| h.1),
                };
                let undead = look.original == DESTROY_UNDEAD;
                let line = damage.map(|damage| Logged::MassDamage {
                    spell,
                    damage,
                    undead,
                });
                let struck: Vec<(u8, u32)> = hits.iter().map(|h| (h.0, h.3)).collect();
                if let Some(line) = line.clone().filter(|_| !undead) {
                    self.log.push((self.clock.now, line));
                }
                if !struck.is_empty() {
                    self.mass(look.animation, &struck, true);
                }
                if let Some(line) = line.filter(|_| undead) {
                    self.log.push((self.clock.now, line));
                }
                self.redraw();
                self.recount(&struck);
                true
            }
            FROST_RING..=METEOR_SHOWER => {
                let Target::Hex { hex } = target else {
                    return false;
                };
                self.cue(owner.unwrap_or(0), Sound::Spell(spell), false);
                self.area((spell, look), hex, &hits, None);
                true
            }
            QUICKSAND..=FIRE_WALL => {
                self.cue(0, Sound::Spell(spell), false);
                self.place_objects((spell, look), target, effects);
                self.log.push((self.clock.now, cast_line));
                true
            }
            ARMAGEDDON => {
                self.cue(owner.unwrap_or(0), Sound::Spell(spell), false);
                let struck: Vec<(u8, u32)> = hits.iter().map(|h| (h.0, h.3)).collect();
                self.field(look.animation, &struck);
                // Its damage's line once it struck a stack (0x5A51DF to 0x5A5244): the spell's
                // value without the caster's bonuses (0x5A78E0 given no hero, CLT-Q16); with the
                // bugs fixed (E17 M3 T9d) with them, as every stack's damage has them.
                let fixed = self.stage.bugs == Bugs::Fixed;
                let value = effects.iter().find_map(|e| match *e {
                    Event::SpellDamage {
                        ref value,
                        ref caster,
                        ..
                    } => Some(if fixed { caster.0 } else { value.0 }),
                    _ => None,
                });
                if let Some(damage) = value {
                    self.log
                        .push((self.clock.now, Logged::Armageddon { damage }));
                }
                self.recount(&struck);
                true
            }
            _ if !applied.is_empty() => {
                self.cue(owner.unwrap_or(0), Sound::Spell(spell), false);
                self.effects(effects);
                self.log.push((self.clock.now, cast_line));
                if look.original == BLOODLUST {
                    // Every stack it reached glows together (0x5A1359 to 0x5A1577; E17 M5 T5b1).
                    self.glow(&applied, &BLOODLUST_STEPS, Effect::Bloodlust);
                } else {
                    let marked: Vec<(u8, u32)> = applied.iter().map(|&t| (t, 0)).collect();
                    self.mass(look.animation, &marked, false);
                }
                true
            }
            _ => false,
        }
    }

    /// The boxes of the stacks `hit` show their counts with the next frame drawn (`cast_spell`'s
    /// tail resets every stack's +0x50, 0x5A2089; E17 M3 T7c-1).
    fn recount(&mut self, hit: &[(u8, u32)]) {
        for &(stack, _) in hit {
            let count = self.stacks.get(&stack).map_or(0, |l| l.count);
            self.pending_counts.push((stack, count));
        }
    }

    /// An area spell aimed at `hex` (`area_spell_handler` 0x5A4970, design §7.16; E17 M3 T7c-1):
    /// its animation at the hex (player 5, trunc(100 F) a frame, 0x5A49B8), then, if it struck any
    /// stack, its damage's line and the reactions ([`Player::spell_reactions`]), `caster` the
    /// creature that cast it (none: a hero), which stands at the cast's tail.
    fn area(
        &mut self,
        (spell, look): (u16, &SpellLook),
        hex: u8,
        hits: &[(u8, u64, u64, u32)],
        caster: Option<u8>,
    ) {
        if let Some(number) = look.animation {
            self.at_hex(number, hex, self.speed.scale(TICK), false);
        }
        let line = match hits {
            [] => None,
            [(target, _, damage, kills)] => Some((*damage, *kills, Some(*target))),
            [(_, caster_damage, ..), ..] => {
                let kills = hits.iter().fold(0_u32, |k, h| k.saturating_add(h.3));
                Some((*caster_damage, kills, None))
            }
        };
        if let Some((damage, kills, target)) = line {
            let line = Logged::AreaDamage {
                spell,
                damage,
                kills,
                target,
            };
            self.log.push((self.clock.now, line));
            let struck: Vec<(u8, u32)> = hits.iter().map(|h| (h.0, h.3)).collect();
            self.spell_reactions(&struck, caster);
        } else if let Some(c) = caster {
            self.stand_caster(c);
        }
    }

    /// Magic animation `number` at `hex` (`play_magic_animation_at_hex` 0x496A10, design §7.16,
    /// player 5; E17 M3 T7c-1): each frame a redraw restoring the background and waiting `time`
    /// on the deadline, then the frame drawn over everything and the battlefield shown; then,
    /// unless its caller keeps it, a last redraw at `time` erases it (E17 M3 T7c-2: a placement
    /// keeps its last frame until the next frame drawn, which shows the object instead). Placed by
    /// its kind's low nibble at the hex: 1 centred 37 above its anchor, 4 from its left on the
    /// cell's lower edge, else centred on that edge (the cell picture's bottom, the hex's top +
    /// 52).
    fn at_hex(&mut self, number: u8, hex: u8, time: u32, keep: bool) {
        let Some(look) = self.stage.magic.get(&number).cloned() else {
            return;
        };
        let width = self.stage.width.max(1);
        let [left, top] = self.stage.layout.hex_box(hex / width, hex % width);
        let [x, y] = self.stage.anchor(hex);
        let [w, h] = look.size;
        let at = match look.place & 0xF {
            1 => [x - w / 2, y - h / 2 - AT_HEX_RISE],
            4 => [left, top + CELL_BOTTOM - h],
            _ => [x - w / 2, top + CELL_BOTTOM - h],
        };
        let erase = (!keep).then_some(None);
        for frame in (0..look.frames).map(Some).chain(erase) {
            let drawn = self.clock.now;
            let shown = self.clock.show(time);
            self.flush(drawn, shown);
            self.spots.push(SpotKey {
                animation: number,
                shown,
                frame,
                def: look.def.clone(),
                translucent: look.translucent,
                at,
            });
        }
        if keep {
            self.kept = Some(SpotKey {
                animation: number,
                shown: self.clock.now,
                frame: None,
                def: look.def,
                translucent: look.translucent,
                at,
            });
        }
    }

    /// The look of the object `spell` places on `hexes` (E17 M3 T7c-2): the one of that many
    /// hexes (Force Field's two or three by its mastery), else its first.
    fn object_look(&self, spell: u16, hexes: &[u8]) -> Option<ObjectLook> {
        let objects = &self.stage.casts.spells.get(&spell)?.objects;
        objects
            .iter()
            .find(|o| usize::from(o.hexes) == hexes.len())
            .or_else(|| objects.first())
            .cloned()
    }

    /// The battlefield object `spell` placed on `hexes` (E17 M3 T7c-2; `place_battle_object`
    /// 0x4669B0): in memory now, at its anchor (the lowest hex, its bottom row's), drawn from the
    /// next frame on.
    fn place(&mut self, spell: u16, hexes: &[u8], owner: Option<Side>) {
        let (Some(look), Some(&anchor)) = (self.object_look(spell, hexes), hexes.iter().max())
        else {
            return;
        };
        if owner.is_some() {
            self.hidden.insert(anchor);
        } else {
            self.hidden.remove(&anchor);
        }
        let forced = self
            .stage
            .casts
            .spells
            .get(&spell)
            .is_some_and(|s| s.original == FORCE_FIELD);
        let blocked = if forced { hexes.to_vec() } else { Vec::new() };
        self.placed.insert(anchor, look.vanish);
        let object = Object {
            def: look.def,
            frames: look.frames,
            rows: look.rows,
            before_stacks: look.before_stacks,
            blocked,
            owner,
        };
        self.pending_objects.push((anchor, Some(object)));
    }

    /// The side that alone sees the object the command under way places on `hexes` (E17 M5
    /// T2b, [`Growing::push_seen`]); none, both.
    fn owner(&self, hexes: &[u8]) -> Option<Side> {
        self.secret.get(hexes.iter().max()?).copied()
    }

    /// The battlefield object on `hexes` goes (E17 M3 T7c-2; `remove_battle_object` 0x466B30):
    /// out of memory at once, then its vanish animation plays at its anchor (`round_start`
    /// 0x4761A9: player 5, not kept), its first frame drawn without the object.
    fn vanish(&mut self, hexes: &[u8]) {
        let Some(&anchor) = hexes.iter().max() else {
            return;
        };
        let vanish = self.placed.remove(&anchor);
        self.hidden.remove(&anchor);
        self.pending_objects.push((anchor, None));
        if let Some(number) = vanish {
            self.at_hex(number, anchor, self.speed.scale(TICK), false);
        }
    }

    /// A hero's spell that places battlefield objects (E17 M3 T7c-2, design §7.16; its sound
    /// started at the handler, 0x5A0331): each object in turn its own sound (Quicksand's, Land
    /// Mine's), its placing animation at a hex, kept, then the object placed and (Force Field
    /// excepted, 0x5A0806) a redraw shown at once; Fire Wall plays every object's at the hex aimed
    /// at (0x5A0884), Force Field at that hex, the others at the object's own. Then the cast's
    /// line (0x5A0817, 0x5A04FF). With the bugs fixed (E17 M3 T9d) each Fire Wall object's at its
    /// own hex (CLT-Q12).
    fn place_objects(
        &mut self,
        (spell, look): (u16, &SpellLook),
        target: Target,
        effects: &[Event],
    ) {
        let aimed = match target {
            Target::Hex { hex } => Some(hex),
            _ => None,
        };
        let tick = self.speed.scale(TICK);
        for e in effects {
            let Event::ObstaclePlaced { ref hexes, .. } = *e else {
                continue;
            };
            let Some(&anchor) = hexes.iter().max() else {
                continue;
            };
            let cue = look
                .object_sound
                .then(|| self.cue(0, Sound::Object(spell), false));
            let at = match look.original {
                FIRE_WALL if self.stage.bugs == Bugs::Fixed => anchor,
                FIRE_WALL | FORCE_FIELD => aimed.unwrap_or(anchor),
                _ => anchor,
            };
            if let Some(o) = self.object_look(spell, hexes) {
                self.at_hex(o.placed, at, tick, true);
            }
            self.place(spell, hexes, self.owner(hexes));
            if look.original != FORCE_FIELD {
                self.redraw();
            }
            // Quicksand's and Land Mine's loops wait for each object's sample (`sample_wait`
            // 0x5A04DD, 0x5A06FF; E17 M5 T5b2); Fire Wall's and Force Field's do not.
            if let Some(cue) = cue.filter(|_| matches!(look.original, QUICKSAND | LAND_MINE)) {
                self.wait_for(cue);
            }
        }
    }

    /// The reactions of the stacks `hit` by a spell without a striker or an animation over them
    /// (`strike_animation` 0x468990 with −1, design §7.16; E17 M3 T7c-1): with no striker to wait
    /// for each starts at once, its group (death, a defending stack's defend, else hit) a frame a
    /// redraw at trunc(100 F) and its sound with its first; the loop runs as many iterations as
    /// the longest hit or death group (a defending stack's hit group counts, 0x468C4F), a hit
    /// standing after its last frame. Then every stack not standing plays on (0x4691A9 to
    /// 0x469268): a frame a redraw at trunc(100 F), standing after its group's last (a death stays
    /// on its last), until none changes: the reactions left over, and `caster` (a creature, its
    /// cast group's frames, kept on its last) stands, shown only with the end's redraw if it did
    /// not react (CLT-Q8: at once with the bugs fixed). Then the dead lie as corpses, the boxes show
    /// their counts and the battle is redrawn at once (0x46938A); a creature caster is outlined
    /// at the cast's tail ([`Player::stand_caster`]).
    fn spell_reactions(&mut self, hit: &[(u8, u32)], caster: Option<u8>) {
        let reactions = self.reactions(hit);
        let looped = reactions.iter().map(|r| r.counted).max().unwrap_or(0);
        let tick = self.speed.scale(TICK);
        let pose = |r: &Reaction, i: u32| {
            let frame = if i < r.frames {
                Some(i)
            } else if r.died {
                r.frames.checked_sub(1)
            } else {
                None
            };
            let pose = frame.map_or(r.base, |f| Pose {
                group: u32::from(r.group),
                frame: f,
                ..r.base
            });
            Pose {
                layer: STRUCK,
                ..pose
            }
        };
        for i in 0..looped {
            if i == 0 {
                for r in &reactions {
                    self.cue(r.stack, r.sound, false);
                }
            }
            let poses: Vec<(u8, Pose)> = reactions.iter().map(|r| (r.stack, pose(r, i))).collect();
            self.frame(&poses, tick);
        }
        // Every stack not standing plays on: each reaction still in its group (from iteration
        // `looped`), the caster on its cast group's last frame unless it reacted (standing next);
        // each with its standing pose.
        let mut playing: Vec<(u8, u32, u32, Pose)> = reactions
            .iter()
            .filter(|r| !r.died && r.frames >= looped)
            .map(|r| {
                let standing = Pose {
                    layer: STRUCK,
                    ..r.base
                };
                (r.stack, looped, r.frames, standing)
            })
            .collect();
        let casting = caster.filter(|c| reactions.iter().all(|r| r.stack != *c));
        let standing = casting.and_then(|c| self.live(c).map(|l| (c, self.standing(&l, l.hex))));
        playing.extend(standing.map(|(c, p)| (c, 0, 0, p)));
        // The redraws of this loop are partial (the reacting stacks' rectangle, 0x469263): a
        // caster that did not react stands in memory but shows standing only with the end's
        // redraw (`dragon`'s round 3), a drawing bug (CLT-Q8) the fixed bugs do not keep: there
        // it shows standing at once (E17 M3 T7d).
        let stale = casting.filter(|_| self.stage.bugs == Bugs::Original);
        let mut unseen = None;
        loop {
            let mut poses = Vec::new();
            let mut changed = false;
            for (stack, i, frames, standing) in &mut playing {
                if *i < *frames {
                    let r = reactions.iter().find(|r| r.stack == *stack);
                    poses.push((*stack, r.map_or(*standing, |r| pose(r, *i))));
                    changed = true;
                } else if *i == *frames {
                    if stale == Some(*stack) {
                        unseen = Some((*stack, self.clock.now, *standing));
                    } else {
                        poses.push((*stack, *standing));
                    }
                    changed = true;
                }
                *i += 1;
            }
            if !changed {
                break;
            }
            self.frame(&poses, tick);
        }
        // The end (0x4692A4 to 0x46938A): those that leave no corpse dissolve away (E17 M5 T5a),
        // the dead lie as corpses, every stack's layer and box as at a strike's end, a redraw at
        // once.
        let dying: Vec<(u8, Pose)> = reactions
            .iter()
            .filter(|r| r.died)
            .map(|r| {
                let corpse = Pose {
                    layer: CORPSE,
                    ..pose(r, r.frames)
                };
                (r.stack, corpse)
            })
            .collect();
        let gone = self.remove_gone(&dying);
        let shown = self.clock.now;
        self.counts.extend(reactions.iter().filter_map(|r| {
            r.count.map(|count| CountKey {
                stack: r.stack,
                shown,
                count,
            })
        }));
        for r in reactions.into_iter().filter(|r| !gone.contains(&r.stack)) {
            if r.died {
                let corpse = Pose {
                    layer: CORPSE,
                    ..pose(&r, r.frames)
                };
                self.pending.push((r.stack, corpse));
                self.bury(r.stack, corpse);
            } else {
                self.pending.push((r.stack, r.base));
            }
        }
        self.redraw();
        let shown = self.clock.now;
        self.keys.extend(unseen.map(|(stack, drawn, pose)| Key {
            stack,
            drawn,
            shown,
            pose,
        }));
        if let Some(c) = caster {
            self.stand_caster(c);
        }
    }

    /// A creature caster stands at its cast's tail (`outline_turn_stack` 0x477AC0 from
    /// `cast_spell` 0x5A216B; E17 M3 T7c-1): the stack whose turn it is, standing, outlined again,
    /// in a redraw shown at once.
    fn stand_caster(&mut self, stack: u8) {
        let standing = self.live(stack).map(|l| (stack, self.standing(&l, l.hex)));
        self.pending.extend(standing);
        let now = self.clock.now;
        self.outlined.push((now, Some(stack)));
        self.acting = Some(stack);
        self.redraw();
    }

    /// A creature's cast as its action (`creature_cast_action` 0x448260, design §7.16; E17 M3
    /// T7c-1): turned toward the hex aimed at, its shot sound starts and its group by the angle
    /// from its centre (before it turns) to the hex's anchor, y down (above 25° 17, above −25° 18,
    /// else 19; straight up 17, down 19; the strike's 11 to 13 when its DEF lacks the group; with
    /// the bugs fixed, y up between the rows, [`Player::creature_casting`]),
    /// plays at trunc(trunc(A / n) F) a frame, its last kept through the cast (0x44851C). The cast
    /// (caster kind 1: no hero frames, no resistance roll) clears the outline in a redraw at once
    /// (0x5A004C), its spell's sound starts and an area spell's handler plays ([`Player::area`]),
    /// the caster standing outlined at its tail; then its sound ends and it turns back if it
    /// turned (0x4486E2, 0x44872F). Other spells take effect at once (T7c-3).
    fn creature_cast(&mut self, stack: u8, spell: u16, target: Target, effects: &[Event]) {
        let look = self
            .stage
            .casts
            .spells
            .get(&spell)
            .cloned()
            .unwrap_or_default();
        let (Target::Hex { hex }, true, FROST_RING..=METEOR_SHOWER) =
            (target, self.stacks.contains_key(&stack), look.original)
        else {
            self.spell_hits((spell, look.animation), effects);
            self.effects(effects);
            return;
        };
        let turned = self.creature_casting(stack, hex);
        // `cast_spell`'s start: the outline and the mouse's stack cleared, a redraw at once each
        // (the mouse's stack unmarked already at the action's start with the bugs fixed, as after
        // a hero's: [`calms`]).
        self.outlined.push((self.clock.now, None));
        self.log.push((self.clock.now, Logged::Unmark));
        self.redraw();
        self.cue(stack, Sound::Spell(spell), false);
        let hits: Vec<(u8, u64, u64, u32)> = effects
            .iter()
            .filter_map(|e| match *e {
                Event::SpellDamage {
                    target,
                    ref caster,
                    ref damage,
                    kills,
                    ..
                } => Some((target, caster.0, damage.0, kills)),
                _ => None,
            })
            .collect();
        self.area((spell, &look), hex, &hits, Some(stack));
        self.creature_cast_over(stack, turned);
    }

    /// A creature caster's frames toward the hex `hex` (`creature_cast_action` 0x448380 to
    /// 0x44851C, [`Player::creature_cast`]): turned toward it, its shot sound started, its group by
    /// the angle played, its last frame kept. Returns whether it turned.
    ///
    /// The original's angle is a drawing bug (CLT-Q8): its Δy is not negated as a shot's is and
    /// runs from the caster's mid-height to the hex's anchor, so a caster rears up toward a target
    /// beside or below it and looks down toward one above. With the bugs fixed (E17 M3 T7d) the
    /// angle is VCMI's choice (`RangedAttackAnimation::setAnimationGroup`: hex to hex, y up) at the
    /// shot's 25°: from the caster's centre to the same point over the hex, so the rise is the
    /// anchors' (*provisional*): above 25° up, below −25° down, else straight (the same row too).
    fn creature_casting(&mut self, stack: u8, hex: u8) -> bool {
        let Some(live) = self.live(stack) else {
            return false;
        };
        let [cx, cy] = self.centre(&live);
        let [tx, ty] = self.stage.anchor(hex);
        let turned = (tx > cx && !live.right) || (tx < cx && live.right);
        let live = if turned { self.turn(stack, live) } else { live };
        let dx = i64::from(tx - cx).abs();
        let direction = match self.stage.bugs {
            // atan(dy / |dx|) in degrees, the target's y less the caster's (0x4483B3 to 0x448442).
            Bugs::Original => {
                let dy = i64::from(ty - cy);
                if dx == 0 {
                    if dy <= 0 { 0 } else { 2 }
                } else if dy * TAN_SCALE > dx * TAN_25 {
                    0
                } else if dy * TAN_SCALE > -dx * TAN_25 {
                    1
                } else {
                    2
                }
            }
            Bugs::Fixed => {
                let rise = i64::from(self.stage.anchor(live.hex)[1] - ty);
                if rise * TAN_SCALE > dx * TAN_25 {
                    0
                } else if rise * TAN_SCALE < -dx * TAN_25 {
                    2
                } else {
                    1
                }
            }
        };
        let g = self.stage.groups;
        let group = Some(g.breath[direction])
            .filter(|&b| live.look.count(b) > 0)
            .unwrap_or(g.strike[direction]);
        let frames = live.look.count(group);
        let time = self.speed.scale(live.look.times[2] / frames.max(1));
        self.cue(stack, Sound::Shot, false);
        for f in 0..frames {
            self.frame(&[(stack, Self::posed(&live, group, f, live.hex))], time);
        }
        turned
    }

    /// A creature's cast is over (0x4486E2, 0x44872F): the battle waits for its shot sound
    /// (`sample_wait`, E17 M5 T5b2), which then ends, and it turns back if it turned.
    fn creature_cast_over(&mut self, stack: u8, turned: bool) {
        self.wait_last(stack, Sound::Shot);
        self.cue(stack, Sound::Shot, true);
        if let Some(live) = self.live(stack).filter(|_| turned) {
            self.turn(stack, live);
        }
    }

    /// One step of a reaction in the mass players' loops ([`Player::mass`], [`Player::field`]):
    /// its group's next frame; after a hit's last frame the standing group's frame 0, from which
    /// the original's loops run on a frame a step to the standing group's last and stay there
    /// (CLT-Q10). `fixed` (E17 M3 T9d): the stack stays on the standing group's frame 0, as every
    /// other reaction ends. A death stays on its last frame.
    fn react(live: &Live, (group, frame): (&mut u8, &mut u32), g: Groups, fixed: bool) {
        if fixed && *group == g.standing {
            return;
        }
        if *frame + 1 < live.look.count(*group) {
            *frame += 1;
        } else if *group == g.hit {
            *group = g.standing;
            *frame = 0;
        }
    }

    /// Magic animation `number` over the stacks `hit` (`mass_spell_animation` 0x5A67C0, design
    /// §7.16, player 3; E17 M3 T7c-1), with their `reactions` to their kills: each takes its death
    /// group (no creature left) or its hit group at frame 0 and its sound as the animation starts;
    /// L = max(n, the longest such group) redraws at trunc(100 F), shown, the reactions advancing
    /// first (frame 0 never shows; a hit runs on into the standing group, a frame a redraw, to its
    /// last frame, where the stack stays until its next animation; a death stays on its last), the
    /// animation's frame i set only while i + 1 < n (its last never shows, the one before it twice:
    /// recorded at T7a-2b). Then the marks are cleared without a redraw (the animation goes with
    /// the next frame drawn), the dead die and, if any did, a redraw is shown at once (0x5A6C47).
    /// The count boxes keep the number before the kills (the stack's +0x50, which only
    /// `strike_animation`'s end resets, design §4.3: *provisional*). With the bugs fixed (E17 M3
    /// T9d) every frame of the animation shows once (CLT-Q9) and a hit ends on the standing group's
    /// frame 0, as every other reaction does (CLT-Q10).
    fn mass(&mut self, number: Option<u8>, hit: &[(u8, u32)], reactions: bool) {
        let look = number.and_then(|n| Some((n, self.stage.magic.get(&n).cloned()?)));
        let g = self.stage.groups;
        let fixed = self.stage.bugs == Bugs::Fixed;
        let mut over = Vec::new();
        // Each reacting stack: its id, its state then, its group and frame, whether it died.
        let mut reacting: Vec<(u8, Live, u8, u32, bool)> = Vec::new();
        let mut loops = look.as_ref().map_or(0, |(_, l)| l.frames);
        for &(stack, kills) in hit {
            let Some(live) = self.stacks.get_mut(&stack) else {
                continue;
            };
            live.count = live.count.saturating_sub(kills);
            let live = live.clone();
            if let Some((_, l)) = &look {
                over.push((stack, self.magic_at(&live, l), !live.right));
            }
            if reactions {
                let died = live.count == 0;
                let (group, sound) = if died {
                    (g.death, Sound::Death)
                } else {
                    (g.hit, Sound::Wince)
                };
                loops = loops.max(live.look.count(group));
                self.cue(stack, sound, false);
                if died {
                    self.happenings.push(Happening::Died(live.side));
                }
                reacting.push((stack, live, group, 0, died));
            }
        }
        let tick = self.speed.scale(TICK);
        let mut drawn_frame = None;
        for i in 0..loops {
            let mut poses = Vec::with_capacity(reacting.len());
            for (stack, live, group, frame, _) in &mut reacting {
                Self::react(live, (group, frame), g, fixed);
                poses.push((*stack, Self::posed(live, *group, *frame, live.hex)));
            }
            let frames = look.as_ref().map_or(0, |(_, l)| l.frames);
            let shows = if fixed { i < frames } else { i + 1 < frames };
            let frame = shows.then_some(i).or(drawn_frame);
            let drawn = self.clock.now;
            self.frame(&poses, tick);
            if let Some((n, l)) = look.as_ref().filter(|_| frame != drawn_frame) {
                self.magics.push(MagicKey {
                    animation: *n,
                    drawn,
                    shown: self.clock.now,
                    frame,
                    def: l.def.clone(),
                    translucent: l.translucent,
                    over: over.clone(),
                });
            }
            drawn_frame = frame;
        }
        // The marks cleared (0x5A6B50): erased with the next frame drawn.
        if let Some((n, l)) = &look {
            self.erasing.push(MagicKey {
                animation: *n,
                drawn: self.clock.now,
                shown: self.clock.now,
                frame: None,
                def: l.def.clone(),
                translucent: l.translucent,
                over,
            });
        }
        // The dead die (`stack_death` 0x444120): those that leave no corpse dissolve away
        // (0x5A6C59; E17 M5 T5a), the others' corpses with the redraw.
        let dying: Vec<(u8, Pose)> = reacting
            .iter()
            .filter(|r| r.4)
            .map(|(stack, live, group, frame, _)| {
                let corpse = Pose {
                    layer: CORPSE,
                    ..Self::posed(live, *group, *frame, live.hex)
                };
                (*stack, corpse)
            })
            .collect();
        let gone = self.remove_gone(&dying);
        let mut died = false;
        for (stack, live, group, frame, dead) in reacting {
            if dead && !gone.contains(&stack) {
                let corpse = Pose {
                    layer: CORPSE,
                    ..Self::posed(&live, group, frame, live.hex)
                };
                self.pending.push((stack, corpse));
                self.bury(stack, corpse);
                died = true;
            }
        }
        if died {
            self.redraw();
        }
    }

    /// Magic animation `number` over the whole battlefield with the reactions of the stacks
    /// `hit` (Armageddon's own loop in `mass_damage_handler`, 0x5A4D47 to 0x5A50C6, design
    /// §7.16, player 6; E17 M3 T7c-2): each stack struck takes its death group (none left) or
    /// its hit group at frame 0 with its sound; L = max(n, the longest such group) iterations,
    /// each the reactions advanced first (a hit runs on into the standing group, a frame an
    /// iteration, to its last), a redraw at trunc(100 F) and, while the animation has a frame
    /// i, that frame tiled over the battlefield, shown (all n, unlike `mass_spell_animation`);
    /// the last stays on the screen until the next frame drawn. Then the dead die and, if one
    /// did, a redraw shows at once (0x5A51C2). With the bugs fixed (E17 M3 T9d) a hit ends on the
    /// standing group's frame 0, as in [`Player::mass`] (the same slip, CLT-Q10).
    fn field(&mut self, number: Option<u8>, hit: &[(u8, u32)]) {
        let look = number.and_then(|n| self.stage.magic.get(&n).cloned());
        let g = self.stage.groups;
        let fixed = self.stage.bugs == Bugs::Fixed;
        let mut reacting: Vec<(u8, Live, u8, u32, bool)> = Vec::new();
        let mut loops = look.as_ref().map_or(0, |l| l.frames);
        for &(stack, kills) in hit {
            let Some(live) = self.stacks.get_mut(&stack) else {
                continue;
            };
            live.count = live.count.saturating_sub(kills);
            let live = live.clone();
            let died = live.count == 0;
            let (group, sound) = if died {
                (g.death, Sound::Death)
            } else {
                (g.hit, Sound::Wince)
            };
            loops = loops.max(live.look.count(group));
            self.cue(stack, sound, false);
            if died {
                self.happenings.push(Happening::Died(live.side));
            }
            reacting.push((stack, live, group, 0, died));
        }
        let tick = self.speed.scale(TICK);
        let mut drawn_frame = None;
        for i in 0..loops {
            let mut poses = Vec::with_capacity(reacting.len());
            for (stack, live, group, frame, _) in &mut reacting {
                Self::react(live, (group, frame), g, fixed);
                poses.push((*stack, Self::posed(live, *group, *frame, live.hex)));
            }
            self.frame(&poses, tick);
            let Some(l) = look.as_ref() else {
                continue;
            };
            let frame = (i < l.frames).then_some(i);
            if frame != drawn_frame {
                self.fields.push(FieldKey {
                    shown: self.clock.now,
                    frame,
                    def: l.def.clone(),
                });
            }
            drawn_frame = frame;
        }
        if let (Some(l), Some(_)) = (look, drawn_frame) {
            self.field_kept = Some(FieldKey {
                shown: self.clock.now,
                frame: None,
                def: l.def,
            });
        }
        // Those that leave no corpse dissolve away (0x5A51D3; E17 M5 T5a).
        let dying: Vec<(u8, Pose)> = reacting
            .iter()
            .filter(|r| r.4)
            .map(|(stack, live, group, frame, _)| {
                let corpse = Pose {
                    layer: CORPSE,
                    ..Self::posed(live, *group, *frame, live.hex)
                };
                (*stack, corpse)
            })
            .collect();
        let gone = self.remove_gone(&dying);
        let mut died = false;
        for (stack, live, group, frame, dead) in reacting {
            if dead && !gone.contains(&stack) {
                let corpse = Pose {
                    layer: CORPSE,
                    ..Self::posed(&live, group, frame, live.hex)
                };
                self.pending.push((stack, corpse));
                self.bury(stack, corpse);
                died = true;
            }
        }
        if died {
            self.redraw();
        }
    }

    /// The pending hues (E17 M3 T7b-2a) and palette effects (E17 M5 T5b1) show from `shown`.
    fn show_tints(&mut self, shown: u32) {
        let tints = core::mem::take(&mut self.pending_tints);
        self.tints.extend(
            tints
                .into_iter()
                .map(|(stack, hue)| TintKey { stack, shown, hue }),
        );
        let looks = core::mem::take(&mut self.pending_looks);
        self.look_keys
            .extend(
                looks
                    .into_iter()
                    .map(|(stack, look)| LookKey { stack, shown, look }),
            );
    }

    /// A glow over `stacks` (`cast_spell` 0x5A122B, 0x5A1121, 0x5A1359; E17 M5 T5b1, design
    /// §7.26): the flag set, then a redraw at trunc(100 F) for each of `steps` (its effect at
    /// that strength), shown; the flag cleared and a redraw shown at once, each stack back to
    /// its look by its spells.
    fn glow(&mut self, stacks: &[u8], steps: &[u8], effect: fn(u8) -> Effect) {
        let time = self.speed.scale(TICK);
        // The flag goes before any look the spell gave (0x43E280's order).
        self.pending_looks.retain(|(s, _)| !stacks.contains(s));
        for &step in steps {
            self.pending_looks
                .extend(stacks.iter().map(|&s| (s, Some(effect(step)))));
            self.frame(&[], time);
        }
        let casts = self.stage.casts;
        let rest: Vec<(u8, Option<Effect>)> = stacks
            .iter()
            .map(|&s| (s, self.stacks.get(&s).and_then(|l| rest_look(l, casts))))
            .collect();
        self.pending_looks.extend(rest);
        self.redraw();
    }

    /// The spells `events` start and end on stacks: each stack's active spells, and its box's
    /// hue with the next frame drawn (E17 M3 T7b-2a, design §4.3).
    fn effects(&mut self, events: &[Event]) {
        for event in events {
            self.effect(event);
        }
    }

    /// The side that controls `stack` now (E17 M5 T5b2): its own, the other while Hypnotize holds
    /// it (`stack_activation` 0x465330, installation.md §7.21's opening spells); a stack the
    /// timeline does not follow the attacker.
    fn controller(&self, stack: u8) -> Side {
        let Some(live) = self.stacks.get(&stack) else {
            return Side::Attacker;
        };
        let spells = &self.stage.casts.spells;
        let hypnotized = live
            .spells
            .iter()
            .any(|h| spells.get(h).is_some_and(|l| l.original == HYPNOTIZE));
        if hypnotized {
            opponent(live.side)
        } else {
            live.side
        }
    }

    /// A spell started or ended on a stack ([`Self::effects`]); any other event does nothing.
    fn effect(&mut self, event: &Event) {
        let (stack, spell, on) = match *event {
            Event::EffectApplied { target, spell, .. } => (target, spell, true),
            Event::EffectEnded { target, spell } => (target, spell, false),
            _ => return,
        };
        let casts = self.stage.casts;
        // Acid Breath only lowers the defense: no spell the box's hue counts (0x5A1E70; E17 M3
        // T7b-2b-3, [`Player::strike_cast`]).
        if casts
            .spells
            .get(&spell)
            .is_some_and(|l| l.original == ACID_BREATH)
        {
            return;
        }
        let Some(live) = self.stacks.get_mut(&stack) else {
            return;
        };
        let was = rest_look(live, casts);
        let changed = if on {
            live.spells.insert(spell)
        } else {
            live.spells.remove(&spell)
        };
        if changed {
            self.pending_tints.push((stack, hue(live, casts)));
            let look = rest_look(live, casts);
            if look != was {
                self.pending_looks.push((stack, look));
            }
        }
    }

    /// A stack's walk time W, ms a hex (+0x158; design §7.3): its creature's, trunc(W × 0.65)
    /// under Haste (0x444CE5), trunc(W × 1.5) under Slow (0x444D3A), as the x87 rounds the
    /// products of the integer and the doubles (exactly these fractions: the products of W and
    /// the doubles fall at least 0.05 from the next integer or on it).
    fn walk_time(&self, live: &Live) -> u32 {
        let w = live.look.times[1];
        let has = |number: u16| {
            live.spells.iter().any(|h| {
                self.stage
                    .casts
                    .spells
                    .get(h)
                    .is_some_and(|l| l.original == number)
            })
        };
        if has(HASTE) {
            w * 65 / 100
        } else if has(SLOW) {
            w * 3 / 2
        } else {
            w
        }
    }

    /// A spell's projectile from `from` to the centre of `target` (`spell_projectile` 0x467DB0,
    /// design §7.16): n = trunc((trunc(d) + 15) / 31) positions (one at least) trunc(33 F) apart
    /// outside the deadline, each moved by the distance over n (truncated toward zero), the first
    /// at the start, shown at once; the DEF whose angle is nearest atan(−Δy / |Δx|) (the first
    /// midpoint between consecutive angles below it ends the count; Δx = 0: the last for a
    /// target below, else the first), mirrored for a target to the left, its group 0's frames
    /// cycling; erased as the next frame shows, n × trunc(33 F) after the start.
    fn spell_projectile(&mut self, from: [i32; 2], target: &Live, shells: &[Shell]) {
        let to = self.centre(target);
        let [dx, dy] = [to[0] - from[0], to[1] - from[1]];
        let d = isqrt(i64::from(dx).pow(2) + i64::from(dy).pow(2));
        let n = u32::try_from((d + 15) / 31).unwrap_or(0);
        let shell = &shells[shell_index(shells, [dx, dy])];
        let steps = i32::try_from(n.max(1)).unwrap_or(1);
        let step = self.speed.scale(STEP);
        let [w, h] = shell.size;
        self.flights.push(Flight {
            def: shell.def.clone(),
            frame: 0,
            cycle: shell.frames,
            mirrored: dx < 0,
            from: self.clock.now,
            step,
            steps: n,
            start: [from[0] - w / 2, from[1] - h / 2],
            delta: [dx / steps, dy / steps],
            path: Vec::new(),
        });
        self.clock.wait(n * step);
    }

    /// The centre a projectile aims at (`stack_screen_x` 0x446660, `stack_screen_y` 0x446630):
    /// its position's anchor, a two-hex stack's middle (22 pixels toward its front), and up by
    /// half its height (267 less its standing frame's top, truncated toward zero).
    fn centre(&self, live: &Live) -> [i32; 2] {
        let [x, y] = self.stage.anchor(live.position(live.hex));
        let x = match (live.two_hex, live.right) {
            (false, _) => x,
            (true, true) => x + HALF_HEX,
            (true, false) => x - HALF_HEX,
        };
        [
            x,
            y - (self.stage.layout.stack_canvas[1] - live.look.top) / 2,
        ]
    }

    /// A spell's animation `number` over the marked stack `target` with its reaction to `kills`
    /// (`strike_animation` 0x468990 without a striker; design §7.16, player 2): iteration i draws
    /// the animation's frame i while it has one, then keeps its last; the reaction from frame 0 at
    /// once, as after a projectile (L = R + 1, §7.4), the loop at least as long as the animation;
    /// at its end the mark is cleared, the dead die and the battle is redrawn at once.
    fn marked(&mut self, (target, live): (u8, &Live), kills: u32, number: Option<u8>) {
        let riding = number.and_then(|number| {
            let look = self.stage.magic.get(&number).cloned()?;
            let over = vec![(target, self.magic_at(live, &look), !live.right)];
            Some(Riding { number, look, over })
        });
        let reactions = self.reactions(&[(target, kills)]);
        self.iterate(
            None,
            (0, 0, 0, 2),
            (&reactions, false),
            [STANDING, STRUCK],
            riding.as_ref(),
        );
    }

    /// The hits of a damage spell the timeline draws no other way (a hero's Implosion, a Faerie
    /// Dragon's Ice Bolt at a stack; E17 M3 T9b, the reviewer: their kills were dropped, the
    /// dead left standing): each its spell's animation over the stack with its reaction, as
    /// [`Self::marked`], its line after (*provisional*: their handlers not read).
    fn spell_hits(&mut self, (spell, animation): (u16, Option<u8>), effects: &[Event]) {
        let hits: Vec<(u8, Live, u64, u32)> = effects
            .iter()
            .filter_map(|e| match *e {
                Event::SpellDamage {
                    target,
                    ref damage,
                    kills,
                    ..
                } => self.live(target).map(|l| (target, l, damage.0, kills)),
                _ => None,
            })
            .collect();
        for (t, live, damage, kills) in hits {
            self.marked((t, &live), kills, animation);
            let line = Logged::SpellDamage {
                spell,
                hit: (t, damage, kills),
            };
            self.log.push((self.clock.now, line));
        }
    }

    /// An attack is over: a melee's target turns back, then its striker at the attack's end
    /// (0x478A57), a shooter after its shots (0x4402F2), each only alive and turned.
    fn attack_over(&mut self, melee: &mut Option<Melee>, shooter: &mut Option<(u8, bool)>) {
        self.melee_over(melee.as_mut());
        let back = [
            melee.take().map(|m| (m.attacker, m.attacker_right)),
            shooter.take(),
        ];
        for (stack, right) in back.into_iter().flatten() {
            if let Some(live) = self.live(stack).filter(|l| l.right != right) {
                self.turn(stack, live);
            }
        }
    }

    /// A melee starts (`strike_action` 0x445C6A, `melee_attack` 0x441CC1, 0x441D79): the
    /// striker turns to face its target if it is not on its front side, takes the striker's
    /// layer, and the target turns to face it likewise.
    fn melee(&mut self, attacker: u8, target: u8, before: Option<(u8, bool)>) -> Option<Melee> {
        let (Some(mut striker), Some(struck)) = (self.live(attacker), self.live(target)) else {
            return None;
        };
        let attacker_right = before
            .filter(|(s, _)| *s == attacker)
            .map_or(striker.right, |(_, r)| r);
        let target_right = struck.right;
        if !self.faces(&striker, &struck) {
            striker = self.turn(attacker, striker);
        }
        let pose = Pose {
            layer: STRIKER,
            ..self.standing(&striker, striker.hex)
        };
        self.pending.push((attacker, pose));
        if !self.faces(&struck, &striker) {
            self.turn(target, struck);
        }
        Some(Melee {
            attacker,
            target,
            attacker_right,
            target_right,
            over: false,
        })
    }

    /// A melee's strikes are over: its target turns back if it lives and turned (0x441F36).
    fn melee_over(&mut self, melee: Option<&mut Melee>) {
        let Some(m) = melee.filter(|m| !m.over) else {
            return;
        };
        m.over = true;
        if let Some(live) = self.live(m.target).filter(|l| l.right != m.target_right) {
            self.turn(m.target, live);
        }
    }

    /// Whether `stack` has `other` on its front side, so a strike between them needs no turn
    /// (0x445BCE, 0x4458B0 through 0x523DF0): one of the three hexes ahead of a one-hex stack,
    /// any neighbour of a two-hex stack's front hex.
    fn faces(&self, s: &Live, o: &Live) -> bool {
        let theirs = o.covers(o.hex);
        let front = s.covers(s.hex)[1];
        let ahead: &[u8] = match (s.two_hex, s.right) {
            (true, _) => &[0, 1, 2, 3, 4, 5],
            (false, true) => &[0, 1, 2],
            (false, false) => &[3, 4, 5],
        };
        ahead
            .iter()
            .filter_map(|d| self.stage.neighbor(front, *d))
            .any(|h| theirs.contains(&h))
    }

    /// A shooter turns to face its target (`shot_action` 0x440160): the target's position right
    /// of the shooter's other hex (by their anchors' x) while it faces left, or left of it while
    /// it faces right.
    fn face_shot(&mut self, shooter: u8, target: u8) {
        let (Some(s), Some(t)) = (self.live(shooter), self.live(target)) else {
            return;
        };
        let [own, _] = self.stage.anchor(s.covers(s.hex)[1]);
        let [theirs, _] = self.stage.anchor(t.position(t.hex));
        if (theirs > own && !s.right) || (theirs < own && s.right) {
            self.turn(shooter, s);
        }
    }

    /// `stack` turns around, animated (`stack_turn_around` 0x446720 with its flag): facing right
    /// group 7, the flip, group 10; facing left group 9, the flip, group 8; each all its frames
    /// at trunc(100 F), then one standing; a two-hex stack's position moves onto its other hex.
    fn turn(&mut self, stack: u8, mut live: Live) -> Live {
        let g = self.stage.groups;
        let [before, after] = if live.right {
            g.turn_left
        } else {
            g.turn_right
        };
        let tick = self.speed.scale(TICK);
        for f in 0..live.look.count(before) {
            self.frame(&[(stack, Self::posed(&live, before, f, live.hex))], tick);
        }
        live.right = !live.right;
        for f in 0..live.look.count(after) {
            self.frame(&[(stack, Self::posed(&live, after, f, live.hex))], tick);
        }
        let standing = self.standing(&live, live.hex);
        self.frame(&[(stack, standing)], tick);
        self.stacks.insert(stack, live.clone());
        live
    }

    /// Sets whether `stack` defends.
    fn defends(&mut self, stack: u8, defending: bool) {
        self.stacks
            .entry(stack)
            .and_modify(|s| s.defending = defending);
    }

    /// A move along `path` (its start first): a walk hex by hex, or a flight (§7.3); a plain move
    /// turns back at its end to the facing it started with (`turn_back`). In a siege (E17 M4
    /// T2c-1, design §10.2) what its hexes bring: the closed gate opening before a walk's step
    /// (`stack_walk` 0x442186: mid-walk the walk stops first and goes on with a first step) or
    /// before a flight (0x4B4A95, *provisional*: not recorded), a mine's blast after a step and
    /// the moat's damage after the last (`enter_hex` 0x46A570 from 0x44237E).
    fn moved(&mut self, stack: u8, path: &[u8], turn_back: bool, entered: &siege::Entered) {
        let Some(mut live) = self.live(stack) else {
            return;
        };
        let home = live.right;
        let g = self.stage.groups;
        let tick = self.speed.scale(TICK);
        let n = live.look.count(g.moving);
        let step = self.speed.ratio(self.walk_time(&live), n);
        let start = path.first().copied().unwrap_or(live.hex);
        let end = path.last().copied().unwrap_or(live.hex);
        if live.flies {
            // A flyer that lands on the gate's hexes stands in a whole redraw shown at once and
            // the gate opens before its flight (0x4B4A95).
            if entered.gate.is_some() {
                self.pending.push((stack, self.standing(&live, start)));
                self.redraw();
                self.drawbridge(true);
            }
            // A flight (0x4B4A40) turns first when it goes the other way by columns.
            let width = self.stage.width.max(1);
            let (from, to) = (start % width, end % width);
            if (to < from && live.right) || (to > from && !live.right) {
                live = self.turn(stack, live);
            }
            // Its sounds (0x4B4BD4): the start's with group 20, the move's (looping) after it, the
            // end's with group 21, when the move's stops (0x4B4E3B).
            self.cue(stack, Sound::Start, false);
            for f in 0..live.look.count(g.move_start) {
                self.frame(&[(stack, Self::posed(&live, g.move_start, f, start))], tick);
            }
            self.cue(stack, Sound::Move, false);
            // Its count box hidden from the cycles on (0x4B4BD8).
            live.boxless = true;
            // k cycles of the moving group along the straight line, the point moved on before
            // each frame by (Δ / k) / n, every value an f32 (0x4B4C7B to 0x4B4CEC), drawn at its
            // truncation (§7.9).
            let [ax, ay] = self.stage.anchor(start);
            let [bx, by] = self.stage.anchor(end);
            let (dx, dy) = (bx - ax, by - ay);
            let d = isqrt(i64::from(dx).pow(2) + i64::from(dy).pow(2));
            let distance = u64::from(live.look.times[3]);
            // k = (d + D / 2) / D; one without a flight distance.
            let cycles = (d + distance / 2).checked_div(distance).unwrap_or(1);
            let k = i128::from(cycles).max(1);
            let total = u32::try_from(cycles).unwrap_or(0) * n;
            let per_frame = |delta: i32| {
                let per_cycle = f32_round(i128::from(delta), k);
                f32_round(i128::from(per_cycle), i128::from(n.max(1)) << F32_FRAC)
            };
            let step_by = [per_frame(dx), per_frame(dy)];
            let mut at = [i64::from(ax) << F32_FRAC, i64::from(ay) << F32_FRAC];
            for j in 0..total {
                for (a, s) in at.iter_mut().zip(step_by) {
                    *a = f32_round(i128::from(*a) + i128::from(s), 1 << F32_FRAC);
                }
                let mut p = Self::posed(&live, g.moving, j % n, start);
                p.offset = [f32_trunc(at[0]) - ax, f32_trunc(at[1]) - ay];
                p.layer = FLYING;
                self.frame(&[(stack, p)], step);
            }
            // The flight ends with group 21 and a frame standing (0x4B4E59, 0x4B4E66); then
            // `stack_fly` turns a plain move back (0x4B49F5) and draws one more frame standing
            // (0x4B4A15).
            self.cue(stack, Sound::End, false);
            self.cue(stack, Sound::Move, true);
            for f in 0..live.look.count(g.move_end) {
                self.frame(&[(stack, Self::posed(&live, g.move_end, f, end))], tick);
            }
            live.hex = end;
            self.frame(&[(stack, self.standing(&live, end))], tick);
            // Shown again after that frame (0x4B4E73), with no redraw of its own: the next frame
            // draws it.
            live.boxless = false;
            self.stacks.insert(stack, live.clone());
            // Where it lands, the moat's re-entry or a mine (0x4B4A1E; *provisional*: not
            // recorded).
            for events in entered.at(path.len() - 1) {
                self.entered((stack, &live), events, false);
                let Some(l) = self.live(stack) else {
                    self.redraw();
                    return;
                };
                live = l;
            }
            if turn_back && live.right != home {
                live = self.turn(stack, live);
            }
            let standing = self.standing(&live, end);
            self.frame(&[(stack, standing)], tick);
            return;
        }
        // A walk (`walk_step` 0x43F0B0): before each step the stack faces it, the first step's
        // turn animated, a later one at once (0x43F0E5); its layer by the step's direction
        // (0x43F15E: up 3, down 7); one cycle per hex, the stack drawn along the step. Its count
        // box is hidden from before the first turn (0x442145).
        live.boxless = true;
        // A step is a first one at the walk's start and after the gate opened (walk_step's flag,
        // 0x442247): its turn animated, group 20 and the start's sound before it.
        let mut first = true;
        let last = path.len().saturating_sub(2);
        for (i, pair) in path.windows(2).enumerate() {
            if entered.gate == Some(i) {
                if first {
                    // Its box hidden already (0x442145): the gate's redraws draw it so.
                    self.pending.push((stack, self.standing(&live, pair[0])));
                } else {
                    self.halt(stack, &mut live, pair[0]);
                }
                self.drawbridge(true);
                first = true;
            }
            let [ax, ay] = self.stage.anchor(pair[0]);
            let [bx, by] = self.stage.anchor(pair[1]);
            if (bx > ax) != live.right {
                if first {
                    live = self.turn(stack, live);
                } else {
                    live.right = !live.right;
                }
            }
            let layer = match by.cmp(&ay) {
                Ordering::Less => WALK_UP,
                Ordering::Greater => WALK_DOWN,
                Ordering::Equal => STANDING,
            };
            if first {
                // The first step's sounds (0x43F19B): the start's with group 20, the move's
                // (looping) with the first cycle.
                self.cue(stack, Sound::Start, false);
                for f in 0..live.look.count(g.move_start) {
                    let p = Pose {
                        layer,
                        ..Self::posed(&live, g.move_start, f, pair[0])
                    };
                    self.frame(&[(stack, p)], tick);
                }
                self.cue(stack, Sound::Move, false);
            }
            first = false;
            for f in 0..n {
                let mut p = Self::posed(&live, g.moving, f, pair[0]);
                p.offset = [share(bx - ax, f, n), share(by - ay, f, n)];
                p.layer = layer;
                self.frame(&[(stack, p)], step);
            }
            // What the hex entered brings on the way (a mine's blast; `enter_hex` with the walk
            // going on): the stack stands on it meanwhile (0x4687C0).
            for events in entered.at(i + 1).filter(|_| i < last) {
                live.hex = pair[1];
                self.stacks.insert(stack, live.clone());
                self.entered((stack, &live), events, true);
                // A blast that kills ends the walk there: the engine reports no step after it.
                live = self.live(stack).unwrap_or(live);
            }
        }
        // The last step (0x43F267): the end's sound with group 21, the move's stopped.
        self.cue(stack, Sound::End, false);
        self.cue(stack, Sound::Move, true);
        for f in 0..live.look.count(g.move_end) {
            self.frame(&[(stack, Self::posed(&live, g.move_end, f, end))], tick);
        }
        let standing = self.standing(&live, end);
        self.frame(&[(stack, standing)], tick);
        live.hex = end;
        self.stacks.insert(stack, live.clone());
        // What the last hex brings (the moat's damage, a mine's blast), before the turn back.
        for events in entered.at(path.len() - 1) {
            self.entered((stack, &live), events, false);
            let Some(l) = self.live(stack) else {
                // Dead: no turn back, the box's redraw (0x4423D1).
                self.redraw();
                return;
            };
            live = l;
        }
        // `stack_walk` 0x4423A2: a plain move turns back at its end.
        if turn_back && live.right != home {
            live = self.turn(stack, live);
        }
        // Then its box shows again with a redraw of the whole battle, shown at once (0x4423D9,
        // `battle_redraw` 0x4423E7).
        live.boxless = false;
        self.stacks.insert(stack, live.clone());
        self.pending.push((stack, self.standing(&live, end)));
        self.redraw();
    }

    /// The struck stacks' reactions; their kills taken.
    fn reactions(&mut self, hit: &[(u8, u32)]) -> Vec<Reaction> {
        let g = self.stage.groups;
        let mut out = Vec::new();
        for &(stack, kills) in hit {
            let Some(live) = self.stacks.get_mut(&stack) else {
                continue;
            };
            let living = live.count > 0;
            live.count = live.count.saturating_sub(kills);
            let died = live.count == 0;
            let (group, sound) = if died {
                (g.death, Sound::Death)
            } else if live.defending {
                (g.defend, Sound::Defend)
            } else {
                (g.hit, Sound::Wince)
            };
            let frames = live.look.count(group);
            let counted = if died { frames } else { live.look.count(g.hit) };
            let live = live.clone();
            if died && living {
                self.happenings.push(Happening::Died(live.side));
            }
            out.push(Reaction {
                stack,
                group,
                frames,
                counted,
                died,
                count: (kills > 0).then_some(live.count),
                sound,
                base: self.standing(&live, live.hex),
            });
        }
        out
    }

    /// A melee strike of `attacker` on the stacks `hit` (§7.4), the striker in `layer`.
    fn strike(&mut self, attacker: u8, hit: &[(u8, u32)], layer: u8) {
        let Some(live) = self.live(attacker) else {
            self.react_alone(hit, [layer, STRUCK]);
            return;
        };
        let breath = self.breath(&live, hit);
        let reactions = self.reactions(hit);
        let own = self.stage.row(live.position(live.hex));
        let target_row = reactions
            .first()
            .map_or(own, |r| self.stage.row(r.base.hex));
        let direction = match target_row.cmp(&own) {
            Ordering::Less => 0,
            Ordering::Equal => 1,
            Ordering::Greater => 2,
        };
        // A breath on two stacks takes groups 17 to 19 if the DEF has them, else the strike's
        // (`melee_strike` 0x441ACA to 0x441B66).
        let g = self.stage.groups;
        let group = Some(g.breath[direction])
            .filter(|&b| breath && live.look.count(b) > 0)
            .unwrap_or(g.strike[direction]);
        let frames = live.look.count(group);
        self.iterate(
            Some((attacker, &live)),
            (group, 0, frames, frames),
            (&reactions, true),
            [layer, STRUCK],
            None,
        );
    }

    /// Whether `live`'s strike on `hit` is a breath: a second stack struck stands beyond its
    /// neighbours. `melee_strike` strikes the stack behind its target only for a creature with
    /// flag 0x8 (0x44171B; its second stack [ebp − 8], 0x441A35); a multi-headed striker's
    /// stacks all stand beside it (the record does not name the ability; the recorded `fly`'s
    /// breath confirms it, E17 M3 T1b-2).
    fn breath(&self, live: &Live, hit: &[(u8, u32)]) -> bool {
        let near: Vec<u8> = live
            .covers(live.hex)
            .iter()
            .flat_map(|&h| (0..6).filter_map(move |d| self.stage.neighbor(h, d)))
            .collect();
        hit.iter()
            .skip(1)
            .filter_map(|(t, _)| self.stacks.get(t))
            .any(|t| t.covers(t.hex).iter().all(|h| !near.contains(h)))
    }

    /// The stacks `hit` by a striker or shooter the timeline does not know (a stack created in
    /// the battle, E17 M3 T1): their reactions alone, as after a projectile lands, their kills
    /// taken (E17 M2 T5c, the reviewer).
    fn react_alone(&mut self, hit: &[(u8, u32)], layers: [u8; 2]) {
        let reactions = self.reactions(hit);
        self.iterate(None, (0, 0, 0, 2), (&reactions, false), layers, None);
    }

    /// A shot of `shooter` at the stacks `hit` (`shot_animation` 0x43F2C0, §7.4); returns when
    /// its reactions' loop (`strike_animation`) starts.
    fn shot(&mut self, shooter: u8, hit: &[(u8, u32)]) -> u32 {
        let Some(live) = self.live(shooter) else {
            let start = self.clock.now;
            self.react_alone(hit, [STANDING, STRUCK]);
            return start;
        };
        let target = hit.first().and_then(|(t, _)| self.live(*t));
        let aimed = target.is_some();
        let aim = target.map_or_else(Aim::default, |t| self.aim(&live, &t));
        let group = self.stage.groups.shoot[aim.direction];
        let n = live.look.count(group);
        let climax = live.look.climax_in(n);
        let time = self.speed.scale(live.look.times[2] / climax.max(1));
        // The shot's sound with its first frame (0x43F424).
        self.cue(shooter, Sound::Shot, false);
        for f in 0..climax {
            self.frame(&[(shooter, Self::posed(&live, group, f, live.hex))], time);
        }
        if let Some(colours) = live.look.ray.clone().filter(|_| aimed) {
            self.beam((shooter, &live), &aim, colours, (group, climax, n));
            let reactions = self.reactions(hit);
            let start = self.clock.now;
            // After the ray the shooter's group is over: it stands as the loop starts
            // (*provisional*, the recorded `shots`).
            self.iterate(
                Some((shooter, &live)),
                (group, n, 0, 2),
                (&reactions, false),
                [STANDING, STRUCK],
                None,
            );
            return start;
        }
        self.fly(&live.look, &aim);
        // The Magogs' and the Liches' explosion over their target (E17 M3 T7c-3).
        if let (Some(e), Some(&(t, _))) = (live.look.explosion, hit.first()) {
            self.explosion(e, t);
        }
        let reactions = self.reactions(hit);
        // A shot marks no striker (`melee_strike` does, 0x441A87; `stack_shot` does not), so
        // `strike_animation`'s loop counts the reactions alone: the longest starts as the
        // projectile lands, whatever is left of the shot group (E17 M2 T5b, design §7.4). The
        // shooter, still in its shot group (0x468A55), shows its frames from the climax on beside
        // it, one per iteration, its frame advancing before each draw, then stands, the loop
        // going on until it has (the Cyclops, sequence `shots`). The reactions' part counts two
        // striker frames, so it has one iteration more than the longest reaction's frames
        // (L = R + 1: the Marksmen's recorded sequence, §7.9; the reading's L = R,
        // *provisional*); its first iterations show at once, the deadline being behind. The
        // shooter stays in its layer, the struck take 5 as in a melee (the original's memory in
        // `behind` and `shots`, E17 M3 T5b-2a: a hit Pegasus's wing over its neighbour).
        let start = self.clock.now;
        self.iterate(
            Some((shooter, &live)),
            (group, climax, n - climax, 2),
            (&reactions, false),
            [STANDING, STRUCK],
            None,
        );
        start
    }

    /// A ray shooter's shot after its climax frames (0x43F51D, `DrawRay` 0x5A5C20, design
    /// §7.12): the wait of trunc(115 F); the ray's ticks on its own clock, each trunc(10 F);
    /// the battle redrawn without the ray at once, the frame deadline left as it is; then, if the
    /// creature has a climax frame (+0x150), the rest of its shot group from the frame shown on,
    /// each trunc(A / n) on the deadline, the last drawn but not shown (0x5A6136 to 0x5A61C9).
    fn beam(
        &mut self,
        (shooter, live): (u8, &Live),
        aim: &Aim,
        colours: Vec<[u8; 3]>,
        (group, climax, n): (u8, u32, u32),
    ) {
        self.clock.wait(self.speed.scale(RAY_WAIT));
        let ticks = ray::trace(aim.from, aim.to);
        let step = self.speed.scale(RAY_TICK);
        let count = u32::try_from(ticks.len()).unwrap_or(0);
        let beam = self.beams.len();
        let from = self.clock.now;
        self.beams.push(Beam {
            from,
            step,
            until: from + count * step,
            ticks,
            colours,
            cursor: [from, u32::MAX],
        });
        self.clock.wait(count * step);
        self.redraw();
        if live.look.climax != 0 {
            let time = self.speed.scale(live.look.times[2] / n.max(1));
            for f in climax.saturating_sub(1)..n {
                let pose = Self::posed(live, group, f, live.hex);
                if f + 1 < n {
                    self.frame(&[(shooter, pose)], time);
                } else {
                    // Drawn, never shown: the timeline has it as drawn (the original's memory).
                    let drawn = self.clock.now;
                    self.clock.show(time);
                    self.keys.push(Key {
                        stack: shooter,
                        drawn,
                        shown: u32::MAX,
                        pose,
                    });
                }
            }
        }
        self.beams[beam].cursor[1] = self.clock.now;
    }

    /// A shot's projectile along `aim`, if `look` has one (0x43F5D6; a turret's arrow,
    /// `turret_arrow_flight` 0x468220, the same): the canvas centred on the start, moved by the
    /// distance over the steps (each truncated toward zero) every trunc(33 F) ms on its own
    /// clock, mirrored when the target lies left of the start; the wait for its steps.
    fn fly(&mut self, look: &Look, aim: &Aim) {
        let step = self.speed.scale(STEP);
        if let Some(missile) = look.missile.as_ref().filter(|_| aim.steps > 0) {
            let [w, h] = missile.size;
            let steps = i32::try_from(aim.steps).unwrap_or(i32::MAX);
            let [dx, dy] = [aim.to[0] - aim.from[0], aim.to[1] - aim.from[1]];
            self.flights.push(Flight {
                def: missile.def.clone(),
                frame: aim.frame(missile),
                cycle: 0,
                mirrored: aim.to[0] < aim.from[0],
                from: self.clock.now,
                step,
                steps: aim.steps,
                start: [aim.from[0] - w / 2, aim.from[1] - h / 2],
                delta: [dx / steps, dy / steps],
                path: Vec::new(),
            });
        }
        self.clock.wait(aim.steps * step);
    }

    /// The shot's group (0 up, 1 straight, 2 down), its start and target points and its
    /// projectile's steps (0x469880, 0x43F4A7).
    fn aim(&self, shooter: &Live, target: &Live) -> Aim {
        let anchor = self.stage.anchor(shooter.position(shooter.hex));
        self.aim_from(anchor, shooter.right, &shooter.look, target)
    }

    /// [`Player::aim`] from the point `[sx, sy]` facing right or not with `look`'s offsets (a
    /// stack's anchor; a siege's turret's record point, E17 M4 T2b).
    fn aim_from(&self, from: [i32; 2], right: bool, look: &Look, target: &Live) -> Aim {
        let start = |i: usize| launch(from, right, look, i);
        // A two-hex target is aimed at between its hexes, by its facing (0x43F323).
        let [tx, ty] = self.centre(target);
        let [x, y] = start(1);
        let (dx, dy) = (i64::from(tx - x).abs(), i64::from(ty - y));
        // atan(−dy / |dx|) in degrees; straight up or down the sign is inverted (0x469880).
        let direction = if dx == 0 {
            if dy > 0 { 0 } else { 2 }
        } else if -dy * TAN_SCALE > dx * TAN_25 {
            0
        } else if -dy * TAN_SCALE > -dx * TAN_25 {
            1
        } else {
            2
        };
        let [x, y] = start(direction);
        let d = isqrt(i64::from(tx - x).pow(2) + i64::from(ty - y).pow(2));
        let steps = (d + STEP_PIXELS / 2) / STEP_PIXELS;
        Aim {
            direction,
            steps: u32::try_from(steps).unwrap_or(0),
            from: [x, y],
            to: [tx, ty],
            slope: [dx, dy],
        }
    }

    /// The strike's iterations (`strike_animation` 0x468990): the striker (its id and how the
    /// timeline follows it; none: one it does not know) plays `frames` of `group` from frame
    /// `from` and counts `counted` for the loop's length, each reaction ends with the loop; the
    /// striker and the struck stacks drawn in `layers`. Then everyone stands in standing's
    /// layer, a dead stack on its death's last frame among the corpses, shown with the next
    /// frame (*provisional*: the executable moves it to the corpses' list when its death ends).
    fn iterate(
        &mut self,
        striker: Option<(u8, &Live)>,
        (group, from, frames, counted): (u8, u32, u32, u32),
        (reactions, melee): (&[Reaction], bool),
        [striking, struck]: [u8; 2],
        riding: Option<&Riding>,
    ) {
        // L = A + R − 1 (with a magic animation, at least its frames: not yet); at least A + 1,
        // so a striker without a reaction to wait for still stands at the end (the original
        // always has one: a target's reaction has six frames or more). The reactions end with
        // those; a shooter plays the rest of its group out beyond them and stands after its last
        // frame (the Cyclops' 10 frames after a 6-frame hit, sequence `shots`, E17 M3 T1b-1;
        // *provisional*: not read).
        // A melee strike marks its striker and counts each reaction's hit group, a defending
        // stack's too (0x468C4F; E17 M3 T7b-2b-3, `thunder`); a shot or a spell marks none, and
        // the recordings show a defending stack's whole group 4 after them (`machines`, `arrow`):
        // there the group each draws counts (*provisional*: the reading counts the hit group
        // there too).
        let longest = reactions
            .iter()
            .map(|r| if melee { r.counted } else { r.frames })
            .max()
            .unwrap_or(0);
        let reacting = (counted + longest).saturating_sub(1).max(counted + 1);
        // After a shot or a spell, the iteration a hit's reaction spends standing again is not
        // a dying stack's: when the longest reactions all die the loop ends with their last
        // frame (the reading's L = R; E17 M3 T7c-3, `raise`: the Marksmen shoot 100 ms after the
        // Pikeman's death shows its last frame).
        let dying = !melee
            && longest > 0
            && reactions
                .iter()
                .filter(|r| r.frames == longest)
                .all(|r| r.died);
        // A magic animation over the marked stacks lengthens the loop to its frames (E17 M3
        // T7b-1, design §7.16).
        let loops = (reacting - u32::from(dying))
            .max(frames + 1)
            .max(riding.map_or(0, |r| r.look.frames));
        let tick = self.speed.scale(TICK);
        let striker = striker.map(|(id, live)| (id, live, self.standing(live, live.hex)));
        for i in 0..loops {
            let mut poses = Vec::with_capacity(reactions.len() + 1);
            if let Some((id, live, standing)) = striker {
                // Each stack's sound as its group starts (0x468F00 to 0x468F8D): the striker's
                // attack (a shooter goes on with its group: none), each reaction's by its group.
                if i == 0 && from == 0 {
                    self.cue(id, Sound::Attack, false);
                }
                let pose = if i < frames {
                    Self::posed(live, group, from + i, live.hex)
                } else {
                    standing
                };
                poses.push((
                    id,
                    Pose {
                        layer: striking,
                        ..pose
                    },
                ));
            }
            for r in reactions {
                if i == (reacting - 1).saturating_sub(r.frames) {
                    self.cue(r.stack, r.sound, false);
                }
            }
            for r in reactions {
                // It reacts once the iterations left are no more than its frames.
                let k = i64::from(i) - (i64::from(reacting) - 1 - i64::from(r.frames));
                // Without a striker a reaction at its last frame holds it while the iterations
                // left are more than its frames (0x468EBA, 0x468FA1; E17 M4 T2c-1, `mine`: the
                // Titans' 7 frames under the mine's 17).
                let held = striker.is_none()
                    && k >= i64::from(r.frames)
                    && i64::from(i) < i64::from(loops) - 1 - i64::from(r.frames);
                let frame = if (0..i64::from(r.frames)).contains(&k) {
                    u32::try_from(k).ok()
                } else if (r.died && k >= 0) || held {
                    r.frames.checked_sub(1)
                } else {
                    None
                };
                let pose = frame.map_or(r.base, |f| Pose {
                    group: u32::from(r.group),
                    frame: f,
                    ..r.base
                });
                poses.push((
                    r.stack,
                    Pose {
                        layer: struck,
                        ..pose
                    },
                ));
            }
            let drawn = self.clock.now;
            self.frame(&poses, tick);
            if let Some(r) = riding {
                // Its frame i while it has one, then its last (0x46905A).
                let frame = i.min(r.look.frames.saturating_sub(1));
                self.magics.push(r.key(drawn, self.clock.now, Some(frame)));
            }
        }
        // The spells the strike left pending, cast before the dead die (0x469102; E17 M3
        // T7b-2b-3).
        for cast in core::mem::take(&mut self.strike_casts) {
            self.strike_cast(cast);
        }
        // The strike's end (`strike_animation` 0x4692A4 to 0x46938A, design §4.3; E17 M3 T5b):
        // the dead leave their hexes for the corpses, every stack's layer is 4 again and its
        // count box's number +0x4C (+0x50, the count before the kills, back to −1), and the
        // whole battle is redrawn and shown at once, the deadline as it is. A stack dying
        // without a corpse dissolves first (0x469316; E17 M3 T9d): the eight steps of
        // [`DISSOLVE_STEP`] outside the deadline.
        if let Some((id, _, standing)) = striker {
            self.pending.push((id, standing));
        }
        let dying: Vec<(u8, Pose)> = reactions
            .iter()
            .filter(|r| r.died)
            .map(|r| (r.stack, Self::fallen(r)))
            .collect();
        let gone = self.remove_gone(&dying);
        let shown = self.clock.now;
        self.counts.extend(reactions.iter().filter_map(|r| {
            r.count.map(|count| CountKey {
                stack: r.stack,
                shown,
                count,
            })
        }));
        for r in reactions.iter().filter(|r| !gone.contains(&r.stack)) {
            let pose = if r.died { Self::fallen(r) } else { r.base };
            self.pending.push((r.stack, pose));
            if r.died {
                self.bury(r.stack, pose);
            }
        }
        // Every stack's mark cleared (0x46905A): the magic animation is gone with the redraw.
        if let Some(r) = riding {
            self.magics.push(r.key(shown, shown, None));
        }
        self.redraw();
    }
}

/// Where a projectile starts from `[sx, sy]` (a stack's anchor, a turret's point) with the
/// creature's missile offset `i` (0 up, 1 straight, 2 down), mirrored facing left (0x43F4A7).
fn launch([sx, sy]: [i32; 2], right: bool, look: &Look, i: usize) -> [i32; 2] {
    let [x, y] = look.missile_offsets[i];
    let x = i32::from(x);
    [if right { sx + x } else { sx - x }, sy + i32::from(y)]
}

/// The projectile DEF of `shells` a spell flies along `[dx, dy]` (screen y down;
/// `spell_projectile` 0x467E5D to 0x467F6F): Δx = 0 takes the last for a target below, else the
/// first; otherwise the count of leading midpoints between consecutive angles that are at least
/// atan(−Δy / |Δx|) (the original compares the angle as an f32 with each midpoint as an f32;
/// comparing tangents is the same but within an f32's rounding of a midpoint, *provisional*), the
/// last when none falls below it.
fn shell_index(shells: &[Shell], [dx, dy]: [i32; 2]) -> usize {
    let last = shells.len().saturating_sub(1);
    if dx == 0 {
        return if dy > 0 { last } else { 0 };
    }
    let (dx, dy) = (i128::from(dx.unsigned_abs()), i128::from(dy));
    let at_least = |pair: &[Shell]| {
        let midpoint = tangent(i32::from(pair[0].angle) + i32::from(pair[1].angle));
        // tan(midpoint) ≥ −dy / dx, dx > 0.
        i128::from(midpoint) * dx >= -dy * i128::from(TAN_SCALE)
    };
    shells
        .windows(2)
        .take_while(|p| at_least(p))
        .count()
        .min(last)
}

/// The fractional bits of the fixed-point integers that hold f32 values exactly here: every f32
/// the flights compute (|x| < 2^23, at least 2^-16 unless 0) is a multiple of 2^-40.
const F32_FRAC: u32 = 40;

/// `num` / `den` (`den` > 0) rounded to the nearest f32, ties to even, as a fixed-point integer
/// (the value × 2^[`F32_FRAC`]): what the x87 stores with `fstp dword` after an operation on
/// f32 operands (the 53-bit intermediate rounds the same, §7.9). A value too small for the
/// fixed point is cut to its grid.
fn f32_round(num: i128, den: i128) -> i64 {
    /// Guard bits below the fixed point's grid.
    const GUARD: u32 = 8;
    let negative = (num < 0) != (den < 0);
    let (num, den) = (num.unsigned_abs(), den.unsigned_abs().max(1));
    let scaled = num << (F32_FRAC + GUARD);
    let (q, sticky) = (scaled / den, !scaled.is_multiple_of(den));
    let bits = 128 - q.leading_zeros();
    let shift = bits.saturating_sub(24).max(GUARD);
    let (kept, rest) = (q >> shift, q & ((1 << shift) - 1));
    let half = 1 << (shift - 1);
    let up = rest > half || (rest == half && (sticky || kept & 1 == 1));
    let magnitude = (kept + u128::from(up)) << (shift - GUARD);
    let magnitude = i64::try_from(magnitude).unwrap_or(i64::MAX);
    if negative { -magnitude } else { magnitude }
}

/// A fixed-point f32's truncation toward zero (`crt_ftol`).
fn f32_trunc(value: i64) -> i32 {
    i32::try_from(value / (1 << F32_FRAC)).unwrap_or(0)
}

/// tan(`half_degrees` / 2 degrees) × [`TAN_SCALE`], truncated: `i64::MAX` from 90° on,
/// `i64::MIN` from −90° down. Sine and cosine by their series in fixed point (no platform
/// transcendental, ADR-0003 §3), exact to about 10^-14.
fn tangent(half_degrees: i32) -> i64 {
    /// 1 and π, times 10^18.
    const ONE: i128 = 1_000_000_000_000_000_000;
    const PI: i128 = 3_141_592_653_589_793_238;
    match half_degrees {
        180.. => i64::MAX,
        ..=-180 => i64::MIN,
        h => {
            let x = i128::from(h) * PI / 360;
            let (mut sin, mut cos, mut term) = (0, 0, ONE);
            for n in 0..40 {
                match n % 4 {
                    0 => cos += term,
                    1 => sin += term,
                    2 => cos -= term,
                    _ => sin -= term,
                }
                term = term * x / ONE / (n + 1);
            }
            i64::try_from(sin * i128::from(TAN_SCALE) / cos).unwrap_or(0)
        }
    }
}

/// ⌊√`n`⌋ (`n` ≥ 0): the executable's `crt_sqrt` truncated, which the steps' and cycles'
/// rounding needs only to the integer.
fn isqrt(n: i64) -> u64 {
    u64::try_from(n).unwrap_or(0).isqrt()
}

/// The harness's timeline `recorded` without the original's own fidgets (E17 M3 T6c-2): each run
/// of group 1 of a stack (`<side>.<record>`) not in `kept`, its lines from the first in group 1
/// to the next not in it (the stand), included. The viewer draws no fidgets of its own in a check
/// (§7.2), but the stacks the mouse entered fidget as the original's do.
#[must_use]
pub fn unfidgeted(recorded: &str, kept: &BTreeSet<String>) -> String {
    let mut fidgeting = BTreeSet::new();
    let mut out = String::new();
    for line in recorded.lines() {
        let mut words = line.split(' ');
        let (_, who, group) = (words.next(), words.next(), words.next());
        let skip = match (who, group) {
            (Some(who), Some(group)) if !kept.contains(who) && !who.starts_with("magic") => {
                if group == "g1" {
                    fidgeting.insert(String::from(who));
                    true
                } else {
                    fidgeting.remove(who)
                }
            }
            _ => false,
        };
        if !skip {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// The harness's `recorded` timeline without the heroes' rolled reactions (E17 M3 T7c-3,
/// `raise`): a hero's defeat or victory group (`rolled`) after a death is the original's own
/// draw (`Heroes`' rolls, 0x49642B), which the check does not share, as a stack's fidget is not
/// (the frame check takes the hero's frame from the original's memory); its lines are left out,
/// with the hero's return to `standing` after them.
#[must_use]
pub fn unrolled(recorded: &str, standing: u8, rolled: [u8; 2]) -> String {
    let rolled = rolled.map(|g| format!("g{g}"));
    let standing = format!("g{standing}");
    let mut rolling = BTreeSet::new();
    let mut out = String::new();
    for line in recorded.lines() {
        let words: Vec<&str> = line.split(' ').collect();
        let skip = match words.as_slice() {
            [_, "hero", side, group, ..] => {
                if rolled.iter().any(|g| g == group) {
                    rolling.insert(String::from(*side));
                    true
                } else {
                    rolling.remove(*side) && *group == standing
                }
            }
            _ => false,
        };
        if !skip {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// A timeline's lines that differ from `expected`'s (the harness's format): each differing
/// line with `-` for the expected and `+` for ours.
pub fn differences(ours: &str, expected: &str) -> Vec<String> {
    let mut out = Vec::new();
    let (a, b): (Vec<&str>, Vec<&str>) = (expected.lines().collect(), ours.lines().collect());
    for i in 0..a.len().max(b.len()) {
        match (a.get(i), b.get(i)) {
            (Some(x), Some(y)) if x == y => {}
            (x, y) => {
                out.extend(x.map(|x| format!("- {x}")));
                out.extend(y.map(|y| format!("+ {y}")));
            }
        }
    }
    out
}

mod chain;
mod created;
mod creature;
mod entered;
mod playhead;
mod raise;
mod siege;

pub use chain::{CHAIN_LIGHTNING, CHAIN_START};
pub use created::{
    BATTLEFIELD, Body, CREATION_STEP, CreatedKey, DISSOLVE_STEPS, Dissolve, DissolveArea,
    DissolveKey, Notice, TELEPORT, TELEPORT_STEP, created_stack,
};
pub use creature::{FIRE_SHIELD, SHIELD_RETURN};
pub use playhead::Playhead;
pub use raise::{ANIMATE_DEAD, RAISE, RESURRECTION, SACRIFICE, SACRIFICED};
pub use siege::{
    Blast, BlastKey, BridgeKey, Burst, DRAWBRIDGE_RECT, Landing, QuakeKey, SHAKE, SHAKES,
    SiegeStage, TARGET_POINTS, TurretKey, TurretPose, WallKey, shake_offsets, turret_record,
};

#[cfg(test)]
pub(crate) mod tests;
