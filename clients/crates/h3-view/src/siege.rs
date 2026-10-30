//! The siege's drawing (E17 M4 T1b, `docs/epics/E17-design.md` §10.1): a town's walls, gate,
//! moat and turrets as the executable's redraw draws them with a fort (`battle_redraw` 0x494440),
//! from a record's [`Siege`] and the town's tables ([`crate::tables::Town`]). The scene
//! ([`crate::scene`]) places what this module lists among its own draws.

use alloc::string::String;
use alloc::vec::Vec;

use h3_protocol::wire::{Siege, Stack};

use crate::anim::TurretPose;
use crate::layout::BattleLayout;
use crate::scene::{Draw, SceneError};
use crate::tables::{SIEGE_PARTS, Tables, Town};

/// The parts of the wall-shot targets (`wall_shot_targets` 0x63BE60): target `t`'s part is
/// entry `t` (the upper tower, the upper wall, the mid-upper wall, the gate's arch, the mid-lower
/// wall, the lower wall, the lower tower, the keep).
pub const TARGET_PARTS: [usize; 8] = [5, 6, 8, 9, 10, 12, 13, 14];
/// Where the mouse aims at each wall-shot target (`wall_shot_targets`' +0; E17 M4 T2c-2): the
/// upper tower's rectangle ("hex" 0xFF), the walls' and the gate's hexes, the lower tower's hex
/// 183, the keep's rectangle (0xFE).
pub const TARGET_AIMS: [u8; 8] = [0xFF, 29, 62, 96, 130, 182, 183, 0xFE];
/// The upper tower's and the lower tower's targets, and the keep's.
pub const TOWER_TARGETS: [usize; 2] = [0, 6];
/// The keep's target.
pub const KEEP_TARGET: usize = 7;
/// Each row's wall hex (`siege_wall_hexes` 0x63BD00): a hex left of it is outside the walls, it
/// and those right of it inside (`shot_wall_test` 0x467510; E17 M4 T2c-2).
pub const WALL_HEXES: [u8; 11] = [12, 29, 45, 62, 78, 96, 112, 130, 147, 165, 182];
/// The gate's hex: on a shot's line it blocks while the gate is closed (0x467840).
pub const GATE_HEX: u8 = 96;
/// The gate's wall-shot target.
const GATE_TARGET: usize = 3;
/// The wall hexes of the targets that open one when destroyed, by target (BAT-SIE-012: the upper
/// wall's 29, the mid-upper wall's 78, the gate's 96, the mid-lower wall's 130, the lower wall's
/// 182).
const TARGET_WALLS: [(usize, u8); 5] = [(1, 29), (2, 78), (3, 96), (4, 130), (5, 182)];
/// The covers, the part each covers and its turret ([`Turret`]): the keep's, the lower tower's,
/// the upper tower's (0x494E0B to 0x494E33).
const COVERS: [(usize, usize, Turret); 3] = [
    (15, 14, Turret::Keep),
    (16, 13, Turret::Lower),
    (17, 5, Turret::Upper),
];
/// The parts drawn whole at their hex, not in strips (0x494C82 to 0x494CBB): the towers, the
/// keep, the gate's arch and the covers.
const WHOLE: [usize; 7] = [5, 9, 13, 14, 15, 16, 17];
/// The drawbridge's part, its rope's, the moat's, its lip's and the back wall's.
const DRAWBRIDGE: usize = 0;
const ROPE: usize = 1;
const MOAT: usize = 2;
const LIP: usize = 3;
const BACK_WALL: usize = 4;
/// A part's picture state while it stands whole (WALLS.TXT's hit points, copied at the siege's
/// start, 0x4662E3): the state the executable never changes but by a hit (1) or a destruction
/// (0, `wall_part_damage` 0x465990).
const WHOLE_STATE: u8 = 2;
/// The gate's states (+0x53A4): destroyed, open, moving, closed.
pub const GATE_DESTROYED: u8 = 0;
/// The gate open (its rope drawn).
pub const GATE_OPEN: u8 = 1;
/// The drawbridge moving, between open and closed (E17 M4 T2c-1: `gate_open` 0x4671C0).
pub const GATE_MOVING: u8 = 2;
/// The gate closed (no drawbridge: the arch over the background).
pub const GATE_CLOSED: u8 = 3;
/// The hex whose parts the redraw draws before the rows (the upper tower's), with the rows'
/// first direction.
pub const BEFORE_ROWS: (u8, i8) = (255, 1);
/// The hexes whose parts it draws after the rows (the lower wall's last strip, the lower
/// tower's), with the rows' last direction.
pub const AFTER_ROWS: [(u8, i8); 2] = [(200, -1), (251, -1)];
/// The battlefield's hexes (11 rows of 17): a stack beyond them is a siege's turret, never drawn
/// as a stack (`draw_hex_stack` 0x494F55).
pub const FIELD_HEXES: u8 = 187;
/// The gate's row: its drawbridge drawn before its passes, its rope after them.
pub const GATE_ROW: u8 = 5;
/// The first row whose columns run right to left (0x49481F).
const LEFTWARD_FROM: u8 = 6;
/// The last pass, which draws no moat over its stacks (0x494FF4).
const LAST_PASS: u8 = 7;
/// A shooter's canvas rows drawn (`draw_turret_shooter` 0x495183).
pub const SHOOTER_ROWS: u32 = 232;
/// The middle strip's width: a hex's (0x494DB0).
const STRIP: i32 = 44;
/// The moat's strip over a stack: its rows of the hex box, from and to (`moat_over_stack`
/// 0x495A4E to 0x495A5D: 36 to 41).
const MOAT_ROWS: [i32; 2] = [36, 42];
/// Each row's moat hex (`moat_hexes` 0x63BCE8): the moat drawn over a stack is by these fixed
/// rows, not the record's moat, which leaves out the bridges while the gate is open (E17 M5
/// T6b2, R1-3: a record that opens with the gate open).
const MOAT_HEXES: [u8; 11] = [11, 28, 44, 61, 77, 95, 111, 129, 146, 164, 181];
/// The Fortress's second moat row (`moat_hexes_fortress` 0x63BCF4).
const FORTRESS_MOAT_HEXES: [u8; 11] = [10, 27, 43, 60, 76, 94, 110, 128, 145, 163, 180];

/// A turret: the keep, the lower tower or the upper tower (the manager's records +0x13D78,
/// +0x13D9C, +0x13DC0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Turret {
    Keep,
    Lower,
    Upper,
}

impl Turret {
    /// Its stack's position (BAT-SIE-016, the original's).
    fn hex(self) -> u8 {
        match self {
            Self::Keep => 254,
            Self::Lower => 251,
            Self::Upper => 255,
        }
    }

    /// Whether a fort level (0 to 3) sets its record up (0x466C50: the keep's with a Citadel,
    /// the towers' with a Castle).
    fn built(self, fort: u8) -> bool {
        match self {
            Self::Keep => fort >= 2,
            Self::Lower | Self::Upper => fort >= 3,
        }
    }

    /// Its shooter's point in the town's tables.
    fn point(self, town: &Town) -> [i16; 2] {
        match self {
            Self::Keep => town.turrets.keep,
            Self::Lower => town.turrets.lower,
            Self::Upper => town.turrets.upper,
        }
    }
}

/// The parts' and the gate's picture states as the executable keeps them (+0x13FA8, +0x53A4;
/// design §10.1): a part's state is the slot of its pictures it shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Walls {
    /// Each part's picture state, by index.
    pub parts: [u8; SIEGE_PARTS],
    /// The gate's: [`GATE_DESTROYED`], [`GATE_OPEN`], [`GATE_MOVING`], [`GATE_CLOSED`].
    pub gate: u8,
}

impl Walls {
    /// The states a record's `siege` shows: each wall-shot target's part whole while it has hit
    /// points, destroyed (0) at none; a cover 1 while its tower or keep stands, else 0; the gate
    /// destroyed at no hit points, else open or closed; the other parts 0, their only picture.
    /// A hit part's state, 1, follows from its history, not its hit points (a Castle's walls
    /// start with 3): a moment's ([`crate::scene::Moment::walls`], *provisional*).
    pub fn of(siege: &Siege) -> Self {
        let mut parts = [0; SIEGE_PARTS];
        let standing = |t: usize| siege.parts.get(t).is_some_and(|&hp| hp > 0);
        for (t, &part) in TARGET_PARTS.iter().enumerate() {
            parts[part] = if standing(t) { WHOLE_STATE } else { 0 };
        }
        for (cover, part, _) in COVERS {
            parts[cover] = u8::from(parts[part] > 0);
        }
        let gate = match (standing(GATE_TARGET), siege.gate_open) {
            (false, _) => GATE_DESTROYED,
            (true, true) => GATE_OPEN,
            (true, false) => GATE_CLOSED,
        };
        Self { parts, gate }
    }

    /// The hexes of `siege`'s walls a count box cannot take (the hex records' blocked flag,
    /// `draw_battle_stack` 0x43E629; E17 M4 T2c-1, `moat`: the Champions' box beside the wall at
    /// 78): the record's wall hexes but those of the targets these states show destroyed.
    pub fn blocking<'a>(&self, siege: &'a Siege) -> impl Iterator<Item = u8> + 'a {
        let open: Vec<u8> = TARGET_WALLS
            .iter()
            .filter(|(t, _)| self.parts[TARGET_PARTS[*t]] == 0)
            .map(|&(_, hex)| hex)
            .collect();
        siege
            .walls
            .iter()
            .copied()
            .filter(move |h| !open.contains(h))
    }
}

/// A turret's shooter as its record holds it: the creature's DEF, its pose, its point and its
/// outline.
#[derive(Debug, Clone)]
struct Shooter {
    def: String,
    group: u32,
    frame: u32,
    mirrored: bool,
    anchor: [i32; 2],
    outline: Option<u8>,
}

/// A siege as the redraw draws it: the town's tables, the states, the moat and the shooters.
#[derive(Debug)]
pub struct SiegeLook<'t> {
    town: &'t Town,
    walls: Walls,
    /// The moat flag (+0x53A8, 0x463BA5): a Citadel's or a Castle's moat, but not a Tower's (its
    /// mines: no moat pictures) nor a Stronghold's in a game under RoE or AB rules (the record's
    /// moat absent).
    moat: bool,
    /// The Fortress flag (+0x53A9): the moat's second row (0x63BCF4) besides its first, found as
    /// the record's moat holding one of its hexes.
    fortress: bool,
    /// The keep's, the lower tower's and the upper tower's shooter, where its record is set up.
    shooters: [Option<Shooter>; 3],
    width: u8,
}

/// What the scene gives the siege's drawing besides the record's siege: the states (the
/// moment's, or the siege's own), the stacks, the active stack and the outline it shows.
#[derive(Debug, Clone, Copy)]
pub struct Context<'a> {
    /// The states drawn.
    pub walls: Walls,
    /// The record's stacks (the turrets' among them).
    pub stacks: &'a [Stack],
    /// The active stack's id and its outline's palette entry.
    pub active: Option<(u8, u8)>,
    /// Each turret record's pose (E17 M4 T2b: a shot's; none: standing, facing left).
    pub turrets: [Option<TurretPose>; 3],
}

impl<'t> SiegeLook<'t> {
    /// The siege `faction` names (a key of `tables`), or none below a Fort (fought on an
    /// ordinary battlefield).
    ///
    /// # Errors
    /// A town or a shooter the tables lack.
    pub fn new(
        siege: &Siege,
        faction: &str,
        tables: &'t Tables,
        context: Context<'_>,
        width: u8,
    ) -> Result<Option<Self>, SceneError> {
        if siege.fort == 0 {
            return Ok(None);
        }
        let town = tables
            .towns
            .get(faction)
            .ok_or_else(|| SceneError::Key(faction.into()))?;
        let creature = tables
            .creatures
            .values()
            .find(|c| c.original == town.turrets.shooter)
            .ok_or_else(|| SceneError::Key(alloc::format!("creature {}", town.turrets.shooter)))?;
        let shift = i32::from(town.turrets.shift);
        let standing = u32::from(tables.battle.groups.standing);
        let shooters = [Turret::Keep, Turret::Lower, Turret::Upper].map(|turret| {
            turret.built(siege.fort).then(|| {
                let [x, y] = turret.point(town);
                // Its record's pose (E17 M4 T2b): facing right it is drawn unmirrored, its canvas
                // at x − 196 less the shift (`siege_parts_at_hex` 0x494EB3).
                let pose = context.turrets[turret as usize].unwrap_or(TurretPose {
                    group: standing,
                    frame: 0,
                    right: false,
                });
                let x = i32::from(x);
                let x = if pose.right { x - shift } else { x + shift };
                // The turret's stack outlined while it is the defender's active stack
                // (0x494E98 to 0x494EAC): its id at the turret's position.
                let id = context
                    .stacks
                    .iter()
                    .find(|s| s.hex == turret.hex() && s.alive)
                    .map(|s| s.id);
                // Outlined only while it stands (0x494E98).
                Shooter {
                    def: creature.def.clone(),
                    group: pose.group,
                    frame: pose.frame,
                    mirrored: !pose.right,
                    anchor: [x, i32::from(y)],
                    outline: context
                        .active
                        .filter(|&(active, _)| Some(active) == id && pose.group == standing)
                        .map(|(_, entry)| entry),
                }
            })
        });
        let moat = siege.moat != 0 && !town.parts[MOAT].pictures[0].is_empty();
        Ok(Some(Self {
            town,
            walls: context.walls,
            moat,
            fortress: siege
                .moat_hexes
                .iter()
                .any(|h| FORTRESS_MOAT_HEXES.contains(h)),
            shooters,
            width,
        }))
    }

    /// A part's picture `slot` at its point, cut to `clip` (left, top, right, bottom; none: whole).
    fn picture(&self, part: usize, slot: u8, clip: Option<[i32; 4]>) -> Option<Draw> {
        let p = &self.town.parts[part];
        let picture = p.pictures.get(usize::from(slot))?;
        (!picture.is_empty()).then(|| Draw::Wall {
            picture: picture.clone(),
            x: i32::from(p.at[0]),
            y: i32::from(p.at[1]),
            clip,
        })
    }

    /// The background buffer (`compose_battle_background` 0x493CF0): the town's picture, then
    /// with the moat flag the moat and its lip.
    pub fn background(&self) -> Vec<Draw> {
        let mut list = alloc::vec![Draw::Background {
            picture: self.town.background.clone(),
            x: 0,
            y: 0,
        }];
        if self.moat {
            list.extend(self.picture(MOAT, 0, None));
            list.extend(self.picture(LIP, 0, None));
        }
        list
    }

    /// The back wall, after the layer-1 obstacles (0x4946BB).
    pub fn back_wall(&self) -> Option<Draw> {
        self.picture(BACK_WALL, 0, None)
    }

    /// The columns of `row` in the order its passes visit them, and their direction: right to
    /// left from row 6 (0x49481F).
    pub fn columns(&self, row: u8) -> (Vec<u8>, i8) {
        let columns = 0..self.width;
        if row >= LEFTWARD_FROM {
            (columns.rev().collect(), -1)
        } else {
            (columns.collect(), 1)
        }
    }

    /// The drawbridge before the gate row's passes while the gate is not closed, its picture
    /// the gate's state (0x4947D2).
    pub fn drawbridge(&self) -> Option<Draw> {
        (self.walls.gate != GATE_CLOSED)
            .then(|| self.picture(DRAWBRIDGE, self.walls.gate, None))
            .flatten()
    }

    /// The drawbridge's rope after the gate row's passes while the gate is open (0x494A14).
    pub fn rope(&self) -> Option<Draw> {
        (self.walls.gate == GATE_OPEN)
            .then(|| self.picture(ROPE, 1, None))
            .flatten()
    }

    /// The parts drawn at `hex` in a row whose direction is `d` (`siege_parts_at_hex` 0x494C20),
    /// in index order, each with a picture for its state and a hex: a tower, the keep, the arch
    /// or a cover whole at its own hex, a cover after its turret's shooter; a wall or a buttress
    /// on hex h (row r, o = r mod 2, left edge L) in three strips of its whole height, the part
    /// left of L at h + 17d − o (or at h itself on row 0), the 44 columns from L at h, the rest
    /// at h − 17d − o + 1.
    pub fn parts_at(&self, hex: u8, d: i8, layout: &BattleLayout) -> Vec<Draw> {
        let mut list = Vec::new();
        let width = i32::from(self.width);
        for (i, part) in self.town.parts.iter().enumerate() {
            let state = self.walls.parts[i];
            let Some(own) = part.hex else {
                continue;
            };
            if part
                .pictures
                .get(usize::from(state))
                .is_none_or(String::is_empty)
            {
                continue;
            }
            if WHOLE.contains(&i) {
                if hex != own {
                    continue;
                }
                let turret = COVERS.iter().find(|c| c.0 == i).map(|c| c.2);
                let shooter = turret.and_then(|t| self.shooters[t as usize].as_ref());
                if let Some(s) = shooter {
                    list.push(Draw::Shooter {
                        def: s.def.clone(),
                        group: s.group,
                        frame: s.frame,
                        anchor: s.anchor,
                        mirrored: s.mirrored,
                        outline: s.outline,
                        rows: SHOOTER_ROWS,
                    });
                }
                list.extend(self.picture(i, state, None));
                continue;
            }
            let (h, at, d) = (i32::from(own), i32::from(hex), i32::from(d));
            let row = h / width;
            let odd = row % 2;
            let (r, c) = (u8::try_from(row).unwrap_or(0), own % self.width);
            let left = layout.hex_box(r, c)[0];
            if at == h + 17 * d - odd || (at == h && row == 0) {
                list.extend(self.picture(i, state, Some([i32::MIN, i32::MIN, left, i32::MAX])));
            } else if at == h - 17 * d - odd + 1 {
                list.extend(self.picture(
                    i,
                    state,
                    Some([left + STRIP, i32::MIN, i32::MAX, i32::MAX]),
                ));
            }
            if at == h {
                list.extend(self.picture(i, state, Some([left, i32::MIN, left + STRIP, i32::MAX])));
            }
        }
        list
    }

    /// What `draw_hex_stack` (0x494F40) draws after a stack on `hex` drawn in `pass` with the
    /// moat flag: none on the gate row while the gate is not closed, nor in the last pass, nor
    /// for a flier between its hexes (`pass` past the last: `draw_battle_stack` draws it by
    /// itself, off its hexes, so no `draw_hex_stack`; E17 M4 T3a, R1-2); else
    /// the moat's strip at the stack's hex and at a two-hex stack's other hex (its position + 1
    /// facing right, − 1 facing left) when it is its row's moat hex, the Fortress's second row's
    /// too ([`MOAT_HEXES`]; E17 M5 T6b2, R1-3) (`moat_over_stack` 0x495A10: the
    /// moat picture's pixels at the hex's columns L to L + 43 and its box's rows 36 to 41, inside
    /// the battlefield), then the count box again (`box`).
    pub fn over_stack(
        &self,
        stack: (u8, bool, bool),
        pass: u8,
        layout: &BattleLayout,
        count: Option<&Draw>,
    ) -> Vec<Draw> {
        let (hex, two_hex, mirrored) = stack;
        let row = hex / self.width.max(1);
        let gated = row == GATE_ROW && self.walls.gate != GATE_CLOSED;
        if !self.moat || gated || pass >= LAST_PASS {
            return Vec::new();
        }
        let other = two_hex.then(|| {
            if mirrored {
                hex.wrapping_sub(1)
            } else {
                hex.wrapping_add(1)
            }
        });
        let [fw, fh] = layout.mouse.field;
        // Both hexes are tried against the stack's own row's moat hexes (0x494FFE, 0x495039).
        let rows = [MOAT_HEXES, FORTRESS_MOAT_HEXES];
        let moat_hexes: Vec<u8> = (rows.iter().take(1 + usize::from(self.fortress)))
            .filter_map(|r| r.get(usize::from(row)).copied())
            .collect();
        let mut list = Vec::new();
        for h in core::iter::once(hex).chain(other) {
            if !moat_hexes.contains(&h) {
                continue;
            }
            let [x, y] = layout.hex_box(h / self.width, h % self.width);
            let clip = [
                x.max(0),
                (y + MOAT_ROWS[0]).max(0),
                (x + STRIP).min(fw),
                (y + MOAT_ROWS[1]).min(fh),
            ];
            list.extend(self.picture(MOAT, 0, Some(clip)));
        }
        list.extend(count.cloned());
        list
    }
}

#[cfg(test)]
mod tests;
