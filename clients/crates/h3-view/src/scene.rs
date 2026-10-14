//! The scene: from a record's snapshot and the asset tables to the draw list of one frame (E17 M1
//! T7: the battle at time 0, `docs/epics/E17-design.md` §4; animations come with M2).
//!
//! The list names what is drawn (a picture, an obstacle's sprite, a creature's sprite with the
//! shadow its frame holds and its outline) in the executable's order; [`crate::compose`] decides
//! how each is drawn, so a later graphics option changes only the compositor (the user,
//! 2026-10-13).

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use h3_protocol::wire::{Manifest, Observation, Side, Stack};

use crate::anim::{Pose, Projectile, Timeline};
use crate::layout::BattleLayout;
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
}

impl Moment {
    /// The battle at `t` of `timeline` (E17 M2 T4b-2a).
    pub fn at(timeline: &Timeline, t: u32) -> Self {
        Self {
            poses: timeline.at(t),
            active: timeline.outlined(t),
            pulse: timeline.pulse(t),
            projectile: timeline.projectile(t),
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

/// A usual obstacle, placed.
struct Placed {
    def: String,
    before_stacks: bool,
    at: (u8, u8),
    x: i32,
    y: i32,
}

/// The background variant 0x41ADB0 returns when no tree, mountain or dead vegetation stands
/// around the battle (§4.4).
const EMPTY_SURROUNDINGS: usize = 2;

/// The draw list of the battle `observation` shows, at time 0: the background with the absolute
/// obstacle, the layer-1 obstacles, then each row's passes with its layer-0 obstacles and its
/// living stacks (§4.6). A natural obstacle without a type is not drawn; a removed one is no
/// longer in the observation's list (the engine drops it), so every listed one is drawn, whatever
/// hexes a removal cleared. Left out until later tasks: the
/// heroes and flags, the count boxes (M3, with `tiny.fnt`), corpses, spell obstacles, the grid.
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
    let keys = &manifest.keys;
    let (width, height) = (manifest.field.width, manifest.field.height);
    let at = |hex: u8| (hex / width.max(1), hex % width.max(1));
    let mut list = Vec::new();
    // The background buffer (§4.4): the battlefield's picture, then the absolute obstacle. Which
    // of a terrain's three pictures a battle gets depends on the adventure map around it (0x41ADB0,
    // §4.4), which a battle's record does not carry: an empty surrounding gives the third, which
    // the reference captures confirm (*provisional*, E17 M2 T1); a field with one picture has it.
    if let Some(handle) = observation.terrain {
        let key = key(&keys.terrains, "terrains", handle)?;
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
            before_stacks: *before_stacks,
            at: (row, column),
            x,
            y: y - layout.hex_step[1] * (i32::from(*rows) - 1),
        });
    }
    let obstacle_draw = |o: &Placed| Draw::Obstacle {
        def: o.def.clone(),
        group: 0,
        frame: 0,
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
    let stacks = observation
        .stacks
        .iter()
        .filter(|s| s.alive || moment.poses.contains_key(&s.id))
        .map(|s| creature(s, moment, manifest, tables, layout, at))
        .collect::<Result<Vec<_>, _>>()?;
    for row in 0..height {
        for pass in 0..PASSES {
            for column in 0..width {
                if pass == OBSTACLE_PASS {
                    let here = placed
                        .iter()
                        .filter(|o| !o.before_stacks && o.at == (row, column));
                    list.extend(here.map(obstacle_draw));
                }
                let here = stacks
                    .iter()
                    .filter(|(at, layer, _)| *layer == pass && *at == (row, column));
                list.extend(here.map(|(_, _, draw)| draw.clone()));
            }
        }
    }
    // A flier between its hexes: off them, drawn by itself over everything (anim::FLYING).
    list.extend(
        stacks
            .iter()
            .filter(|(_, layer, _)| *layer >= PASSES)
            .map(|(_, _, draw)| draw.clone()),
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
    Ok(list)
}

/// A stack's position (row, column), its draw layer and its draw, in its pose at `moment`.
fn creature(
    stack: &Stack,
    moment: &Moment,
    manifest: &Manifest,
    tables: &Tables,
    layout: &BattleLayout,
    at: impl Fn(u8) -> (u8, u8),
) -> Result<((u8, u8), u8, Draw), SceneError> {
    let key = key(&manifest.keys.creatures, "creatures", stack.creature)?;
    let entry = tables
        .creatures
        .get(key)
        .ok_or_else(|| SceneError::Key(key.to_string()))?;
    // Drawn once, from its position (§4.2): the engine's position is the executable's, a two-hex
    // stack's rear hex (BAT-GEO-002 since PR #150, protocol v23); moved by its pose's offset.
    let groups = tables.battle.groups;
    let pose = moment.poses.get(&stack.id).copied().unwrap_or(Pose {
        group: u32::from(groups.standing),
        frame: 0,
        hex: stack.hex,
        offset: [0, 0],
        mirrored: stack.side == Side::Defender,
        layer: STANDING,
    });
    let (row, column) = at(pose.hex);
    let [x, y] = layout.anchor(row, column);
    let draw = Draw::Creature {
        def: entry.def.clone(),
        group: pose.group,
        frame: pose.frame,
        anchor: [x + pose.offset[0], y + pose.offset[1]],
        mirrored: pose.mirrored,
        outline: (moment.active == Some(stack.id)).then(|| layout.outline_after(moment.pulse)),
    };
    Ok(((row, column), pose.layer, draw))
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
