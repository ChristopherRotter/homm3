//! The grid and the shadows (E17 M6 T4c; `docs/epics/E17-design.md` §12.16): the options Show
//! Combat Grid (0x6987BC), Show Combat Mouse Hex (the cursor shadow, 0x6987C0) and Combat Shade
//! Level (the movement shadow, 0x6987C4), the hexes each draws into the battle's background and
//! the cursor shadow's list as `battle_shade_hexes` (0x493EA0) keeps it.
//!
//! The background (+0x53B0) is composed from the field, then (1) the movement shadow (the tactics
//! phase's shade, [`crate::tactics::shade`], and with the option every human turn's, [`reach`]),
//! (2) the grid, (3) the cursor shadow's hexes, halved again over both: a hex under both shadows
//! comes out a quarter, the grid's lines in it a half ([`crate::scene::Overlays`]). The cursor
//! shadow saves each hex's block before it shades it and puts the blocks back before its next
//! hexes ([`Cells`]): a composition between the two leaves the old blocks to come back over the
//! new background (oddity 2, CLT-Q52: fixed, the original's for the comparisons only).

use alloc::collections::BTreeSet;
use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::Side;
use h3_render::Bugs;

use crate::mouse::aim::{Aim, Area};
use crate::mouse::{COLUMNS, Field, ROWS};
use crate::options::Settings;
use crate::scene::{Overlays, Stale};

/// Force Field's original number: its aim is the wall's (`aim_wall_handler` 0x5A3250).
pub const FORCE_FIELD: u16 = 12;
/// Fire Wall's original number: the wall's aim shading its hexes (0x5A334D).
pub const FIRE_WALL: u16 = 13;
/// Sacrifice's original number: its aim sets no shades (§7.18).
const SACRIFICE: u16 = 40;
/// Teleport's original number: its aim sets no shades (§7.18).
const TELEPORT: u16 = 63;
/// The save picture's slots (855 × 52 at the battle's start, 19 blocks: Inferno's fit).
const SLOTS: usize = 19;
/// The key of no hex (0x6772D8's −1).
const NONE: i32 = -1;

/// The three options as the battle reads them (`settings_names`: Show Combat Grid, Show Combat
/// Mouse Hex, Combat Shade Level).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Options {
    /// The grid (0x6987BC).
    pub grid: bool,
    /// The cursor shadow (0x6987C0).
    pub cursor: bool,
    /// The movement shadow (0x6987C4).
    pub movement: bool,
}

/// One of the keys F6 to F8 (`battle_buttons` 0x47593F to 0x475BDE): the option it toggles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Toggle {
    /// F6: the grid.
    Grid,
    /// F7: the cursor shadow.
    Cursor,
    /// F8: the movement shadow.
    Movement,
}

impl Options {
    /// The options of `settings`.
    #[must_use]
    pub const fn of(settings: &Settings) -> Self {
        Self {
            grid: settings.grid,
            cursor: settings.cursor_shade,
            movement: settings.movement_shade,
        }
    }

    /// Written into `settings`.
    pub const fn set_in(self, settings: &mut Settings) {
        settings.grid = self.grid;
        settings.cursor_shade = self.cursor;
        settings.movement_shade = self.movement;
    }

    /// With `toggle`'s option toggled.
    #[must_use]
    pub const fn toggled(self, toggle: Toggle) -> Self {
        let mut out = self;
        match toggle {
            Toggle::Grid => out.grid = !out.grid,
            Toggle::Cursor => out.cursor = !out.cursor,
            Toggle::Movement => out.movement = !out.movement,
        }
        out
    }

    /// The options an aim of the spell numbered `original` sets as it starts (`enter_player_cast`
    /// 0x59EFB8, 0x59F05F: `battle_set_shades(grid, 1, 0, 1)`): with both shadows on, the aims
    /// of `aim_handler` (every targeted and area spell) and the wall's turn the movement shadow
    /// off, saved so, until the aim ends; Teleport's and Sacrifice's set none (§7.18). None when
    /// it sets none.
    #[must_use]
    pub const fn aiming(self, original: u16) -> Option<Self> {
        if self.cursor && self.movement && !matches!(original, TELEPORT | SACRIFICE) {
            Some(Self {
                movement: false,
                ..self
            })
        } else {
            None
        }
    }
}

/// Whether the grid draws its cell at `hex` (0x493C4C): hexes 0 to 186 but columns 0 and 16.
#[must_use]
pub const fn gridded(hex: u8) -> bool {
    let column = hex % COLUMNS;
    hex < COLUMNS * ROWS && column != 0 && column != COLUMNS - 1
}

/// The cursor shadow's key and hexes for the screen hex `hex` (`shade_mouse_hex` 0x494390: a
/// hex's number, or another of `screen_to_hex`'s answers): the hex alone when it is a hex of the
/// field outside the side columns, else none with the key −1.
#[must_use]
pub fn mouse_hexes(hex: i32) -> (i32, Vec<u8>) {
    match u8::try_from(hex).ok().filter(|&h| gridded(h)) {
        Some(h) => (hex, vec![h]),
        None => (NONE, Vec::new()),
    }
}

/// `hex_distance` 0x469670 from the screen hex `from` (a hex's number, or another of
/// `screen_to_hex`'s answers in the executable's arithmetic: −1 is row 0's column −1) to `to`.
fn distance_from(from: i32, to: u8) -> i32 {
    let columns = i32::from(COLUMNS);
    let (y1, x1) = (from / columns, from % columns);
    let (y2, x2) = (i32::from(to / COLUMNS), i32::from(to % COLUMNS));
    let p = (y1 + 1) / 2 - (y2 + 1) / 2 - x1 + x2;
    let q = y2 / 2 - y1 / 2 - x1 + x2;
    if (p < 0) == (q < 0) {
        p.abs().max(q.abs())
    } else {
        p.abs() + q.abs()
    }
}

/// Whether `hex` lies in `area` aimed at the screen hex `centre` (`area_hexes` 0x5A4170,
/// `berserk_area_hexes` 0x5A4430: within the radius, in columns 1 to 15 and rows 0 to 10, the
/// centre only with `area.centre`); off the field by the executable's arithmetic (a side column's
/// hex: the part in columns 1 to 15; −1: row 0's column −1, hexes 1 and 18 at radius 2).
#[must_use]
pub fn in_area(centre: i32, area: Area, hex: u8) -> bool {
    let d = distance_from(centre, hex);
    gridded(hex) && d <= i32::from(area.radius) && (d != 0 || area.centre)
}

/// The hexes an area aim at the screen hex `hex` shades with the cursor shadow on (`aim_area_marks`
/// 0x59F8A0 after its marks): its area's ([`in_area`]: Berserk's radius by the mastery), but a
/// hex whose living stack its marks leave out (the spell may not affect it): the empty hexes and
/// those of the stacks marked; none without an area. Off the field (−1: the panel, the margins)
/// the original's profile shades its arithmetic's hexes (oddity 5: hexes 1 and 18 at radius 2);
/// the client's none (CLT-Q55, fixed, the user's answer of 2026-11-04).
#[must_use]
pub fn area_shade(aim: &Aim, field: &Field<'_>, hex: i32) -> Vec<u8> {
    let Some(area) = aim.area else {
        return Vec::new();
    };
    let on_field = (0..i32::from(COLUMNS * ROWS)).contains(&hex);
    if field.bugs == Bugs::Fixed && !on_field {
        return Vec::new();
    }
    let marks = aim.marks(field, hex);
    (0..COLUMNS * ROWS)
        .filter(|&h| in_area(hex, area, h))
        .filter(|&h| field.at(h).is_none_or(|s| marks.contains(&s.id)))
        .collect()
}

/// Whether the spell numbered `original` is aimed by the wall's handler (`aim_wall_handler`
/// 0x5A3250: Force Field and Fire Wall).
#[must_use]
pub const fn wall(original: u16) -> bool {
    matches!(original, FORCE_FIELD | FIRE_WALL)
}

/// The hexes the wall's aim at `hex` shades with the cursor shadow on (0x5A334D): Fire Wall's own
/// for the side to act `side` at `mastery` (the hex, its upper right neighbour for the attacker,
/// 16 lower on an even row and 17 on an odd one, or its upper left for the defender, 17 and 18,
/// and from Advanced 34 lower; those off the field left out); none for Force Field (CLT-Q54 (2),
/// copied).
#[must_use]
pub fn wall_shade(original: u16, hex: u8, (side, mastery): (Side, u8)) -> Vec<u8> {
    if original != FIRE_WALL {
        return Vec::new();
    }
    let odd = (hex / COLUMNS) % 2 == 1;
    let next = match (side, odd) {
        (Side::Attacker, false) => 16,
        (Side::Attacker, true) | (Side::Defender, false) => 17,
        (Side::Defender, true) => 18,
    };
    let mut list = vec![hex];
    list.extend(hex.checked_sub(next));
    if mastery >= 2 {
        list.extend(hex.checked_sub(34));
    }
    list
}

/// The movement shadow's hexes on a turn of `field` outside the tactics phase (§12.16's layer
/// (1): §7.23's overlay 0x4937D0 over the reach map `ai_pathfinder_map` at the stack's speed):
/// none without a living acting stack or for one no human plays here (+0x54A6); else the stack's
/// own hexes, every position it may move to with its front hex there by its facing now (+0x4A,
/// +0x4B), and the hexes of every stack it may strike in melee (0x4938F9). *Provisional:* the
/// engine's legal moves and strikes stand for the original's reach map, the strikes' targets for
/// the stacks of the other side its search marks.
#[must_use]
pub fn reach(field: &Field<'_>) -> BTreeSet<u8> {
    let mut shaded = BTreeSet::new();
    let observation = field.observation;
    let Some(mover) = field.acting().filter(|s| s.alive) else {
        return shaded;
    };
    if !observation.sides[usize::from(mover.controller == Side::Defender)].human {
        return shaded;
    }
    let [own, front] = field.hexes(mover);
    shaded.extend([own, front]);
    for &to in field.legal.moves() {
        // The front hex as far from the position as it is now (none for a one-hex stack).
        let there = (i16::from(to) + i16::from(front) - i16::from(own)).clamp(0, 255);
        shaded.extend([to, u8::try_from(there).unwrap_or(to)]);
    }
    for &(target, _) in field.legal.attacks() {
        let stacks = observation.stacks.iter();
        for stack in stacks.filter(|s| s.alive && s.id == target) {
            shaded.extend(field.hexes(stack));
        }
    }
    shaded
}

/// The movement shade's hexes on the turn of `field` with the movement shadow `on`: the tactics
/// phase's whatever the option ([`crate::tactics::shade`]), else with it [`reach`].
#[must_use]
pub fn movement(field: &Field<'_>, on: bool) -> BTreeSet<u8> {
    if field.observation.tactics.is_some() {
        crate::tactics::shade(field.observation)
    } else if on {
        reach(field)
    } else {
        BTreeSet::new()
    }
}

/// What a composition of the background drew besides the field: the grid and the movement
/// shadow (the options as it was composed).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Composed {
    grid: bool,
    movement: bool,
}

/// What the background draws from a time on (E17 M6 T4c): the movement shade on a turn outside
/// the tactics phase ([`reach`] with the movement shadow on, the turn waiting) and the grid's and
/// the cursor shadow's ([`Overlays`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Drawn {
    /// The movement shade.
    pub shade: BTreeSet<u8>,
    /// The grid and the cursor shadow.
    pub overlays: Overlays,
}

/// The cursor shadow's list (`battle_shade_hexes` 0x493EA0) and the background it is shaded
/// into, with the options in effect: each new hex's block saved before any is shaded, the blocks
/// put back before the next hexes; a composition of the background between the two leaves the
/// list (oddity 2), whose blocks then put the earlier composition's back over the new one until
/// the next ([`Overlays::stale`]): the original's profile; the client's forgets the list as the
/// background is composed again, so the next move shades the mouse's hex on the new one
/// (CLT-Q52, fixed, the user's answer of 2026-11-04).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cells {
    options: Options,
    /// The background's last composition.
    composed: Composed,
    /// The last key (0x6772D8; −1 in the data).
    last: i32,
    /// The list (0x6969B0): each hex with the composition its block was saved from.
    list: Vec<(u8, Composed)>,
    /// The list's hexes are shaded in the background (none since a composition).
    shaded: bool,
    /// The blocks put back from an earlier composition, by hex, until the next.
    stale: Vec<(u8, Composed)>,
    bugs: Bugs,
}

impl Cells {
    /// The battle's start with `options`, in profile `bugs`: no hex shaded, the background
    /// composed with the options.
    #[must_use]
    pub const fn new(options: Options, bugs: Bugs) -> Self {
        Self {
            options,
            composed: Composed {
                grid: options.grid,
                movement: options.movement,
            },
            last: NONE,
            list: Vec::new(),
            shaded: false,
            stale: Vec::new(),
            bugs,
        }
    }

    /// The options in effect.
    #[must_use]
    pub const fn options(&self) -> Options {
        self.options
    }

    /// `battle_shade_hexes(key, hexes, force)`: nothing with the cursor shadow off or for the last
    /// key unless forced; else the list's blocks put back, each of `hexes` (at most the save
    /// picture's 19) saved from the background, then shaded; the key the last.
    pub fn shade(&mut self, key: i32, hexes: &[u8], force: bool) {
        if !self.options.cursor || (key == self.last && !force) {
            return;
        }
        for (hex, from) in core::mem::take(&mut self.list) {
            self.stale.retain(|&(h, _)| h != hex);
            if from != self.composed {
                self.stale.push((hex, from));
            }
        }
        self.list = (hexes.iter().take(SLOTS))
            .map(|&hex| (hex, self.at(hex)))
            .collect();
        self.shaded = true;
        self.last = key;
    }

    /// `shade_mouse_hex(hex, force)` (0x494390) for the screen hex `hex` ([`mouse_hexes`]).
    pub fn mouse(&mut self, hex: i32, force: bool) {
        let (key, hexes) = mouse_hexes(hex);
        self.shade(key, &hexes, force);
    }

    /// An action's start (`action_dispatcher` 0x478E45: `shade_mouse_hex(−1, 1)`): the shade
    /// goes; the blocks of an earlier composition with it (*provisional*: the movement overlay's
    /// redraw there, 0x478E51, taken for a composition).
    pub fn action(&mut self) {
        self.mouse(NONE, true);
        self.stale.clear();
    }

    /// The keys F6 to F8 and `battle_set_shades` (0x479FF6): the shade off while the option is
    /// still on (`shade_mouse_hex(−1, 0)`), `options` in effect, the background composed again.
    pub fn set(&mut self, options: Options) {
        self.mouse(NONE, false);
        self.compose(options);
    }

    /// The background composed again with `options` without the shade's call (the combat options
    /// dialog's close, `battle_combat_options` 0x4686B0; the battle's start): the shade gone
    /// from it; the list and the last key kept in the original's profile (oddity 2), forgotten
    /// in the client's (CLT-Q52).
    pub fn compose(&mut self, options: Options) {
        self.options = options;
        self.composed = Composed {
            grid: options.grid,
            movement: options.movement,
        };
        self.shaded = false;
        self.stale.clear();
        if self.bugs == Bugs::Fixed {
            self.list.clear();
            self.last = NONE;
        }
    }

    /// The composition the background shows at `hex`'s block: a block put back's, else the last.
    fn at(&self, hex: u8) -> Composed {
        (self.stale.iter().rev())
            .find(|&&(h, _)| h == hex)
            .map_or(self.composed, |&(_, c)| c)
    }

    /// What the background draws with `reach` the turn's movement shade outside the tactics phase
    /// (none while it does not wait): the movement shade with the movement shadow composed, the
    /// grid, the blocks put back and the hexes shaded.
    #[must_use]
    pub fn drawn(&self, reach: &BTreeSet<u8>) -> Drawn {
        let shade = |movement: bool| {
            if movement {
                reach.clone()
            } else {
                BTreeSet::new()
            }
        };
        let cursor = if self.shaded {
            self.list.iter().map(|&(hex, _)| hex).collect()
        } else {
            Vec::new()
        };
        Drawn {
            shade: shade(self.composed.movement),
            overlays: Overlays {
                grid: self.composed.grid,
                cursor,
                stale: (self.stale.iter())
                    .map(|&(hex, c)| Stale {
                        hex,
                        grid: c.grid,
                        shade: shade(c.movement),
                    })
                    .collect(),
            },
        }
    }
}

#[cfg(test)]
mod tests;
