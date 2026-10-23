//! The adventure screen's scene (E24 M1 T3b1, *provisional*): what it shows at a view and a
//! moment, as a draw list in the original's order (`docs/epics/E24-design.md` §1.1, §1.9). The
//! frame's picture in the viewer's colours, then the view's passes, each over the whole view, rows
//! outer and columns inner: the terrain with the map's edge (§1.3, §1.4), the rivers, the roads,
//! the unexplored tiles (§1.5), the corner gems. The objects' passes (§2.2: the flat objects, the
//! shadows, the bodies) come between the roads and the unexplored tiles with T4; the path's with
//! M3. [`crate::compose`] draws the list.
//!
//! Each piece of the view carries its clip: the 32 × 32 rectangle of its tile (a road's 16 pixels
//! lower) inside the view's clip (§1.2), as the passes cut their source rectangles.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use h3_render::Bugs;

use crate::art::{Layer, MapArt};
use crate::layout::AdventureLayout;
use crate::live::Live;
use crate::tables::{ShroudPiece, Tables};

/// The view's top-left tile and level (the manager's +0xE4, design §1.1).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ViewAt {
    /// The column of the view's tile (0, 0).
    pub x: i32,
    /// Its row.
    pub y: i32,
    /// The level: 0 the surface, 1 the underground.
    pub z: u8,
}

/// Which pass a piece of the view comes from (design §1.1); it decides the drawer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Pass {
    /// A tile's terrain (pass 1).
    Terrain,
    /// A tile off the map, in the terrain's pass: the map's edge (§1.4).
    Edge,
    /// A tile's river (pass 2).
    River,
    /// A tile's road (pass 3).
    Road,
    /// An unexplored tile (pass 10), drawn twice: its pixels, then its runs as shadows (§1.5).
    Shroud,
}

/// One thing drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Draw {
    /// The frame's picture, opaque at (0, 0), its palette's entries 224–255 the viewer's 32
    /// colours of the players' palette (adventure_full_update 0x417420, design §1.9).
    Frame {
        /// The picture (`AdvMap.pcx`).
        picture: String,
        /// The players' palette (`PLAYERS.PAL`).
        players: String,
        /// The viewer's colour (0–7).
        viewer: u8,
    },
    /// A piece of the view: frame `frame` of `def`'s group 0, its canvas's top-left corner at
    /// (`x`, `y`), mirrored, drawn only inside `clip` (left, top, right, bottom; the last two
    /// exclusive).
    Piece {
        /// The pass.
        pass: Pass,
        /// The DEF.
        def: String,
        /// The frame in group 0.
        frame: u32,
        /// The canvas's left edge.
        x: i32,
        /// The canvas's top edge.
        y: i32,
        /// Mirrored left to right.
        mirror_x: bool,
        /// Mirrored top to bottom.
        mirror_y: bool,
        /// The rectangle drawn into.
        clip: [i32; 4],
    },
    /// A corner gem: frame `frame` (the viewer's colour) of `def`'s group 0 at (`x`, `y`), through
    /// the plain drawer with runs skipped, not mirrored, not clipped (0x40F2F0, design §1.1).
    Gem {
        /// The DEF.
        def: String,
        /// The frame.
        frame: u32,
        /// The canvas's left edge.
        x: i32,
        /// The canvas's top edge.
        y: i32,
    },
}

/// A DEF's palette cycles at the moment: each range rotated `steps` times (design §1.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cycle {
    /// The DEF.
    pub def: String,
    /// Its ranges of palette entries, first and last.
    pub ranges: Vec<[u8; 2]>,
    /// The steps (the moment's cycles).
    pub steps: u32,
}

/// What the screen shows: the draw list in order, and the palette cycles its DEFs are drawn with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scene {
    /// The draws, in order.
    pub draws: Vec<Draw>,
    /// Every cycling DEF of the tables, with the moment's steps.
    pub cycles: Vec<Cycle>,
}

/// Why a view has no scene.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneError {
    /// The view's level is not on the map.
    Level(u8),
    /// The viewer is not a colour (0–7).
    Viewer(u8),
    /// A tile of the view has a terrain, river or road code the tables lack.
    Code {
        /// The layer.
        layer: Layer,
        /// The code.
        code: u8,
        /// The tile.
        at: [i32; 2],
    },
}

impl fmt::Display for SceneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Level(z) => write!(f, "level {z} is not on the map"),
            Self::Viewer(c) => write!(f, "viewer {c} is not a colour (0-7)"),
            Self::Code { layer, code, at } => write!(
                f,
                "tile ({}, {}): {layer:?} code {code} is not in the tables",
                at[0], at[1]
            ),
        }
    }
}

/// The scene of the adventure screen at view `at`, with the original's drawing bugs fixed or,
/// for the comparisons, reproduced (`bugs`; the one so far: MCL-Q2's road on the map's last row).
///
/// # Errors
/// [`SceneError`]: the level or the viewer out of range, a code the tables lack.
pub fn scene(
    art: &MapArt,
    live: &Live,
    tables: &Tables,
    layout: &AdventureLayout,
    at: ViewAt,
    bugs: Bugs,
) -> Result<Scene, SceneError> {
    if usize::from(at.z) >= art.levels {
        return Err(SceneError::Level(at.z));
    }
    if live.viewer > 7 {
        return Err(SceneError::Viewer(live.viewer));
    }
    let view = View {
        art,
        live,
        tables,
        layout,
        at,
        bugs,
    };
    view.check_codes()?;
    let mut draws = Vec::new();
    draws.push(Draw::Frame {
        picture: layout.picture.clone(),
        players: layout.player_palette.clone(),
        viewer: live.viewer,
    });
    let passes: [TilePass<'_>; 4] = [View::terrain, View::river, View::road, View::shroud];
    for pass in passes {
        view.pass(&mut draws, pass);
    }
    for gem in &layout.view.gems {
        draws.push(Draw::Gem {
            def: gem.def.clone(),
            frame: u32::from(live.viewer),
            x: gem.at[0],
            y: gem.at[1],
        });
    }
    let cycles = [Layer::Terrain, Layer::River]
        .into_iter()
        .flat_map(|layer| tables.layer(layer).values())
        .filter(|art| !art.cycles.is_empty())
        .map(|art| Cycle {
            def: art.def.clone(),
            ranges: art.cycles.clone(),
            steps: live.moment.cycles,
        })
        .collect();
    Ok(Scene { draws, cycles })
}

/// The view being drawn.
struct View<'a> {
    art: &'a MapArt,
    live: &'a Live,
    tables: &'a Tables,
    layout: &'a AdventureLayout,
    at: ViewAt,
    bugs: Bugs,
}

/// One pass's work on one tile: its map coordinates, its place, the list.
type TilePass<'v> = fn(&View<'v>, i32, i32, [i32; 2], &mut Vec<Draw>);

impl<'a> View<'a> {
    /// Calls `visit` with each tile of the view: rows outer, columns inner (design §1.1).
    fn tiles(&self, mut visit: impl FnMut(i32, i32, [i32; 2])) {
        let view = &self.layout.view;
        for j in view.rows[0]..=view.rows[1] {
            for i in view.columns[0]..=view.columns[1] {
                let (x, y) = (self.at.x.saturating_add(i), self.at.y.saturating_add(j));
                visit(x, y, view.place(i, j));
            }
        }
    }

    /// One pass over the whole view.
    fn pass(&self, out: &mut Vec<Draw>, pass: TilePass<'a>) {
        self.tiles(|x, y, place| pass(self, x, y, place, out));
    }

    /// Every code of the view's tiles is in the tables (the passes then find each).
    fn check_codes(&self) -> Result<(), SceneError> {
        let mut missing = None;
        self.tiles(|x, y, _| {
            let Some(tile) = self.art.tile(x, y, self.at.z) else {
                return;
            };
            let layers = [
                (Layer::Terrain, Some(tile.terrain)),
                (Layer::River, tile.river),
                (Layer::Road, tile.road),
            ];
            for (layer, piece) in layers {
                let code = piece.map(|p| p.code);
                if let Some(code) = code.filter(|c| !self.tables.layer(layer).contains_key(c)) {
                    missing.get_or_insert(SceneError::Code {
                        layer,
                        code,
                        at: [x, y],
                    });
                }
            }
        });
        missing.map_or(Ok(()), Err)
    }

    /// The map's side: every map is square (the bounds' height and width; MCL-Q1).
    fn side(&self) -> i32 {
        i32::try_from(self.art.size).unwrap_or(i32::MAX)
    }

    /// The tile's rectangle `down` pixels below its place, inside the view's clip; `None` if
    /// nothing of it shows.
    fn clip(&self, place: [i32; 2], down: i32) -> Option<[i32; 4]> {
        let [left, top, right, bottom] = self.layout.view.clip;
        let tile = self.layout.view.tile;
        let (x, y) = (place[0], place[1].saturating_add(down));
        let rect = [
            x.max(left),
            y.max(top),
            x.saturating_add(tile).min(right),
            y.saturating_add(tile).min(bottom),
        ];
        (rect[0] < rect[2] && rect[1] < rect[3]).then_some(rect)
    }

    /// The DEF the tables give a layer's code (`check_codes` found it).
    fn def(&self, layer: Layer, code: u8) -> &str {
        self.tables
            .layer(layer)
            .get(&code)
            .map_or("", |art| art.def.as_str())
    }

    /// Pass 1: the tile's terrain on the map, the edge's piece off it (0x412900).
    fn terrain(&self, x: i32, y: i32, place: [i32; 2], out: &mut Vec<Draw>) {
        let Some(clip) = self.clip(place, 0) else {
            return;
        };
        out.push(match self.art.tile(x, y, self.at.z) {
            Some(tile) => {
                let p = tile.terrain;
                Draw::Piece {
                    pass: Pass::Terrain,
                    def: self.def(Layer::Terrain, p.code).into(),
                    frame: u32::from(p.picture),
                    x: place[0],
                    y: place[1],
                    mirror_x: p.mirror_x,
                    mirror_y: p.mirror_y,
                    clip,
                }
            }
            None => Draw::Piece {
                pass: Pass::Edge,
                def: self.tables.map.edge.clone(),
                frame: edge_frame(x, y, self.side()),
                x: place[0],
                y: place[1],
                mirror_x: false,
                mirror_y: false,
                clip,
            },
        });
    }

    /// Pass 2: the tile's river (0x411B80).
    fn river(&self, x: i32, y: i32, place: [i32; 2], out: &mut Vec<Draw>) {
        self.layer(Layer::River, x, y, place, out);
    }

    /// Pass 3: the tile's road (0x411D60).
    fn road(&self, x: i32, y: i32, place: [i32; 2], out: &mut Vec<Draw>) {
        self.layer(Layer::Road, x, y, place, out);
    }

    /// The tile's river or road, if it has one and is on the map. A road lies `road_offset`
    /// pixels lower, so its lower half covers the next row; on the map's last row its rows past
    /// the tile are cut (MCL-Q2: the original cuts them from what the view's clip left, so at the
    /// view's last row it draws nothing).
    fn layer(&self, layer: Layer, x: i32, y: i32, place: [i32; 2], out: &mut Vec<Draw>) {
        let Some(tile) = self.art.tile(x, y, self.at.z) else {
            return;
        };
        let (piece, pass, down) = match layer {
            Layer::Road => (tile.road, Pass::Road, self.tables.map.road_offset),
            _ => (tile.river, Pass::River, 0),
        };
        let clip = piece.zip(self.clip(place, down)).and_then(|(_, mut clip)| {
            if y == self.side() - 1 && down > 0 {
                clip[3] = match self.bugs {
                    Bugs::Fixed => clip[3].min(place[1].saturating_add(self.layout.view.tile)),
                    Bugs::Original => clip[3].saturating_sub(down),
                };
            }
            (clip[1] < clip[3]).then_some(clip)
        });
        if let Some((piece, clip)) = piece.zip(clip) {
            out.push(Draw::Piece {
                pass,
                def: self.def(layer, piece.code).into(),
                frame: u32::from(piece.picture),
                x: place[0],
                y: place[1].saturating_add(down),
                mirror_x: piece.mirror_x,
                mirror_y: piece.mirror_y,
                clip,
            });
        }
    }

    /// Pass 10: an unexplored tile of the map, its piece by its unexplored neighbours (0x412220,
    /// 0x40F8C0).
    fn shroud(&self, x: i32, y: i32, place: [i32; 2], out: &mut Vec<Draw>) {
        let (z, explored) = (self.at.z, &self.live.explored);
        let hidden = self.art.tile(x, y, z).is_some() && !explored.explored(x, y, z);
        let Some(clip) = self.clip(place, 0).filter(|_| hidden) else {
            return;
        };
        // North, east, south, west, then north-east, south-east, south-west, north-west.
        let neighbours = [
            (0, -1),
            (1, 0),
            (0, 1),
            (-1, 0),
            (1, -1),
            (1, 1),
            (-1, 1),
            (-1, -1),
        ];
        let mask = neighbours
            .iter()
            .enumerate()
            .filter(|(_, (dx, dy))| !explored.explored(x + dx, y + dy, z))
            .fold(0u8, |mask, (bit, _)| mask | 1 << bit);
        let shroud = &self.tables.map.shroud;
        let (tx, ty) = (u8::try_from(x).unwrap_or(0), u8::try_from(y).unwrap_or(0));
        let (def, frame, mirror_x) = match shroud.piece(mask, tx, ty) {
            ShroudPiece::Full(frame) => (&shroud.full, frame, false),
            ShroudPiece::Edge { frame, mirror } => (&shroud.edge, frame, mirror),
            ShroudPiece::None => return,
        };
        out.push(Draw::Piece {
            pass: Pass::Shroud,
            def: def.clone(),
            frame: frame.into(),
            x: place[0],
            y: place[1],
            mirror_x,
            mirror_y: false,
            clip,
        });
    }
}

/// The map's edge's frame for a tile off a map of `side` (0x412900, design §1.4): the corners 16
/// to 19, the edges' four frames each by the coordinate along them, every tile further out by
/// both.
#[must_use]
pub fn edge_frame(x: i32, y: i32, side: i32) -> u32 {
    let along = |v: i32| (0..side).contains(&v);
    let frame = match (x, y) {
        (-1, -1) => 16,
        _ if x == side && y == -1 => 17,
        _ if x == side && y == side => 18,
        _ if x == -1 && y == side => 19,
        _ if y == -1 && along(x) => 20 + (x & 3),
        _ if x == side && along(y) => 24 + (y & 3),
        _ if y == side && along(x) => 28 + (x & 3),
        _ if x == -1 && along(y) => 32 + (y & 3),
        _ => (x.wrapping_add(16) & 3) + 4 * (y.wrapping_add(16) & 3),
    };
    u32::try_from(frame).unwrap_or(0)
}

#[cfg(test)]
mod tests;
