//! The scene: from a record's snapshot and the asset tables to the draw list of one frame (E17 M1
//! T7: the battle at time 0, `docs/epics/E17-design.md` §4; animations come with M2).
//!
//! The list names what is drawn (a picture, an obstacle's sprite, a creature's sprite with the
//! shadow its frame holds and its outline) in the executable's order; [`crate::compose`] decides
//! how each is drawn, so a later graphics option changes only the compositor (the user,
//! 2026-10-13).

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use h3_protocol::wire::{Manifest, Observation, Side, Stack};

use crate::layout::BattleLayout;
use crate::tables::{ObstacleArt, Tables};

/// The row passes of the battle's redraw (0x494440, §4.6): for each row, passes 0–7, each over
/// the row's columns left to right.
const PASSES: u8 = 8;
/// The pass that draws the layer-0 obstacle anchored on a hex.
const OBSTACLE_PASS: u8 = 2;
/// A standing stack's draw layer, the pass it is drawn in (+0x8 = 4 from the setup, 0x43D750).
const STANDING: u8 = 4;

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
    let keys = &manifest.keys;
    let (width, height) = (manifest.field.width, manifest.field.height);
    let at = |hex: u8| (hex / width.max(1), hex % width.max(1));
    let mut list = Vec::new();
    // The background buffer (§4.4): the battlefield's picture, variant 0 (which variant a battle
    // gets, 0x41ADB0, is not read), then the absolute obstacle.
    if let Some(handle) = observation.terrain {
        let key = key(&keys.terrains, "terrains", handle)?;
        let field = tables
            .battlefields
            .get(key)
            .ok_or_else(|| SceneError::Key(key.to_string()))?;
        list.push(Draw::Background {
            picture: field.backgrounds[0].clone(),
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
        .filter(|s| s.alive)
        .map(|s| creature(s, manifest, tables, layout, observation.active, at))
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
    Ok(list)
}

/// A living stack's position (row, column), its draw layer and its draw.
fn creature(
    stack: &Stack,
    manifest: &Manifest,
    tables: &Tables,
    layout: &BattleLayout,
    active: Option<u8>,
    at: impl Fn(u8) -> (u8, u8),
) -> Result<((u8, u8), u8, Draw), SceneError> {
    let key = key(&manifest.keys.creatures, "creatures", stack.creature)?;
    let entry = tables
        .creatures
        .get(key)
        .ok_or_else(|| SceneError::Key(key.to_string()))?;
    // Drawn once, from its position (§4.2): the engine's position is the executable's, a two-hex
    // stack's rear hex (BAT-GEO-002 since PR #150, protocol v23).
    let (row, column) = at(stack.hex);
    let draw = Draw::Creature {
        def: entry.def.clone(),
        group: u32::from(tables.battle.groups.standing),
        frame: 0,
        anchor: layout.anchor(row, column),
        mirrored: stack.side == Side::Defender,
        outline: (active == Some(stack.id)).then_some(layout.outline_active),
    };
    Ok(((row, column), STANDING, draw))
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
