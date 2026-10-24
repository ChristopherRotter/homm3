//! The adventure screen's scene (E24 M1 T3b1, T4a, *provisional*): what it shows at a view and a
//! moment, as a draw list in the original's order (`docs/epics/E24-design.md` §1.1, §1.9, §2.2).
//! The frame's picture in the viewer's colours, then the view's passes, each over the whole view,
//! rows outer and columns inner: the terrain with the map's edge (§1.3, §1.4), the rivers, the
//! roads, the flat objects, the objects' shadows, their bodies by layer (§2.2: one 32 × 32 piece
//! per entry of a tile's draw list, [`crate::objects`]), the unexplored tiles (§1.5), the corner
//! gems; the heroes and boats as two-row objects among them (E24 M1 T5a, §2.3, §2.8): their
//! shadows after the objects', their pictures after the bodies of layers 1 and 2; the path's
//! arrows (E24 M1 T5b2, §2.10): their shadows after every other shadow, their pixels after the
//! bodies. [`crate::compose`] draws the list.
//!
//! Each piece of the view carries its clip: the 32 × 32 rectangle of its tile (a road's 16 pixels
//! lower) inside the view's clip (§1.2), as the passes cut their source rectangles.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::ops::Range;

use h3_render::Bugs;

use crate::art::{Layer, MapArt};
use crate::layout::AdventureLayout;
use crate::live::{Facing, Live, LiveBoat, LiveHero};
use crate::objects::{self, DrawLists, Entry, ObjectsError, Shapes};
use crate::tables::{ShroudPiece, Tables};

/// The highest layer a cell can have (a template is at most 6 rows high, design §2.1).
pub const MAX_LAYER: u8 = 6;

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
    /// A piece of a flat object (pass 4): its pixels, its flag runs in its owner's colour for
    /// the owned classes, skipped for the rest (E24 M1 T5b1, design §2.9).
    Flat,
    /// A piece of an object's, a hero's or a boat's shadow (pass 5).
    Shadow,
    /// A path arrow's shadow (pass 6): its runs as shadows alone (0x47BFA0 into 0x47E820,
    /// design §2.2).
    PathShadow,
    /// A piece of an object's body (pass 7), drawn as the flat objects', or of a hero's or boat's
    /// pictures (through 0x47C080: no colour, its flag runs skipped).
    Body,
    /// A path arrow (pass 8): its pixels, runs skipped (def_draw_raw 0x47BF50).
    Path,
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
    /// A piece of the view: frame `frame` of `def`'s group `group` (0 but a hero's or a boat's,
    /// §2.8), its canvas's top-left corner at (`x`, `y`), mirrored, drawn only inside `clip`
    /// (left, top, right, bottom; the last two exclusive).
    Piece {
        /// The pass.
        pass: Pass,
        /// The DEF.
        def: String,
        /// The group (by id).
        group: u32,
        /// The frame in the group.
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
        /// The entry of the scene's flag palette ([`Scene::flags`]) the piece's flag pixels (its
        /// runs of index 5) take in the flagged drawer: an owned object's owner (E24 M1 T5b1,
        /// design §2.9); `None` skips them.
        flag: Option<u8>,
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
    /// The palette the pieces' flag entries index (the tables' owners' palette, `game.pal`).
    pub flags: String,
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
    /// The level's objects have no draw lists.
    Objects(ObjectsError),
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
            Self::Objects(e) => e.fmt(f),
        }
    }
}

/// The scene of the adventure screen at view `at`, with the original's drawing bugs fixed or,
/// for the comparisons, reproduced (`bugs`; the one so far: MCL-Q2's road on the map's last row).
/// `shapes` gives the objects' templates their `.msk` and frames ([`objects::pictures`] names
/// them).
///
/// # Errors
/// [`SceneError`]: the level or the viewer out of range, a code the tables lack, an object's
/// picture without a shape.
pub fn scene(
    art: &MapArt,
    live: &Live,
    tables: &Tables,
    layout: &AdventureLayout,
    shapes: &Shapes,
    at: ViewAt,
    bugs: Bugs,
) -> Result<Scene, SceneError> {
    if usize::from(at.z) >= art.levels {
        return Err(SceneError::Level(at.z));
    }
    if live.viewer > 7 {
        return Err(SceneError::Viewer(live.viewer));
    }
    let lists = DrawLists::build(art, live, shapes, at.z).map_err(SceneError::Objects)?;
    let view = View {
        art,
        live,
        tables,
        layout,
        shapes,
        lists,
        standing: standing(live, at.z),
        arrows: arrows(live, at.z),
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
    let passes: [TilePass<'_>; 9] = [
        View::terrain,
        View::river,
        View::road,
        View::flat,
        View::shadows,
        View::path_shadow,
        View::bodies,
        View::path,
        View::shroud,
    ];
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
    let moment = live.moment;
    let cycles = [
        (Layer::Terrain, moment.cycles),
        (Layer::River, moment.river_cycles),
    ]
    .into_iter()
    .flat_map(|(layer, steps)| tables.layer(layer).values().map(move |art| (art, steps)))
    .filter(|(art, _)| !art.cycles.is_empty())
    .map(|(art, steps)| Cycle {
        def: art.def.clone(),
        ranges: art.cycles.clone(),
        steps,
    })
    .collect();
    Ok(Scene {
        draws,
        cycles,
        flags: tables.map.owners.palette.clone(),
    })
}

/// The view being drawn.
struct View<'a> {
    art: &'a MapArt,
    live: &'a Live,
    tables: &'a Tables,
    layout: &'a AdventureLayout,
    shapes: &'a Shapes,
    lists: DrawLists,
    standing: BTreeMap<(i32, i32), Standing<'a>>,
    arrows: BTreeMap<(i32, i32), u16>,
    at: ViewAt,
    bugs: Bugs,
}

/// A hero or a boat without one on its tile (design §2.3: the view finds them by the tiles' class
/// and word, one per tile).
#[derive(Debug, Clone, Copy)]
enum Standing<'a> {
    Hero(&'a LiveHero),
    Boat(&'a LiveBoat),
}

/// The heroes and boats on level `z` by their tiles (the first on a tile, should two be given).
fn standing(live: &Live, z: u8) -> BTreeMap<(i32, i32), Standing<'_>> {
    let heroes = live.heroes.iter().map(|h| (h.at, Standing::Hero(h)));
    let boats = live.boats.iter().map(|b| (b.at, Standing::Boat(b)));
    let mut out = BTreeMap::new();
    for (at, s) in heroes.chain(boats).filter(|(at, _)| at.z == z) {
        out.entry((i32::from(at.x), i32::from(at.y))).or_insert(s);
    }
    out
}

/// The path's arrows on level `z` by their tiles (the last given on a tile; an arrow 0 is none,
/// as the passes skip it).
fn arrows(live: &Live, z: u8) -> BTreeMap<(i32, i32), u16> {
    live.path
        .iter()
        .filter(|a| a.at.z == z && a.arrow > 0)
        .map(|a| ((i32::from(a.at.x), i32::from(a.at.y)), a.arrow))
        .collect()
}

/// The slots of the heroes and boats over a tile (design §2.3): k = 3 dy + dx + 1 for the one
/// standing dx columns right of and dy rows below it.
const SLOTS: usize = 6;

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
                    group: 0,
                    frame: u32::from(p.picture),
                    x: place[0],
                    y: place[1],
                    mirror_x: p.mirror_x,
                    mirror_y: p.mirror_y,
                    clip,
                    flag: None,
                }
            }
            None => Draw::Piece {
                pass: Pass::Edge,
                def: self.tables.map.edge.clone(),
                group: 0,
                frame: edge_frame(x, y, self.side()),
                x: place[0],
                y: place[1],
                mirror_x: false,
                mirror_y: false,
                clip,
                flag: None,
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
                group: 0,
                frame: u32::from(piece.picture),
                x: place[0],
                y: place[1].saturating_add(down),
                mirror_x: piece.mirror_x,
                mirror_y: piece.mirror_y,
                clip,
                flag: None,
            });
        }
    }

    /// Pass 4: the tile's pieces of flat objects, in list order (0x412470).
    fn flat(&self, x: i32, y: i32, place: [i32; 2], out: &mut Vec<Draw>) {
        self.objects(x, y, place, out, Pass::Flat, None);
    }

    /// Pass 5: the tile's pieces of the objects' shadows, in list order, then the heroes' and
    /// boats' by slot (0x411590; design §2.2).
    fn shadows(&self, x: i32, y: i32, place: [i32; 2], out: &mut Vec<Draw>) {
        self.objects(x, y, place, out, Pass::Shadow, None);
        self.standing(x, y, place, out, Pass::Shadow, 0..SLOTS);
    }

    /// Pass 7: the tile's pieces of the objects' bodies, layer by layer, each in list order, the
    /// heroes and boats on the tile's row after layer 1 and those on the row below after layer 2
    /// (0x410C00; design §2.2, §2.3: a tile with an empty list draws them all, in the same order).
    fn bodies(&self, x: i32, y: i32, place: [i32; 2], out: &mut Vec<Draw>) {
        for layer in 0..=MAX_LAYER {
            self.objects(x, y, place, out, Pass::Body, Some(layer));
            match layer {
                1 => self.standing(x, y, place, out, Pass::Body, 0..3),
                2 => self.standing(x, y, place, out, Pass::Body, 3..SLOTS),
                _ => {}
            }
        }
    }

    /// The pieces of the heroes and boats in `slots` over a tile of the map (view_tile_heroes_boats
    /// 0x40FB30), the hero's or boat's in each slot in order.
    fn standing(
        &self,
        x: i32,
        y: i32,
        place: [i32; 2],
        out: &mut Vec<Draw>,
        pass: Pass,
        slots: Range<usize>,
    ) {
        let on_map = self.art.tile(x, y, self.at.z).is_some();
        let Some(clip) = self.clip(place, 0).filter(|_| on_map) else {
            return;
        };
        for k in slots {
            let (dx, dy) = (k % 3, k / 3);
            let (dx, dy) = (
                i32::try_from(dx).unwrap_or(0) - 1,
                i32::try_from(dy).unwrap_or(0),
            );
            if let Some(&s) = self.standing.get(&(x + dx, y + dy)) {
                self.standing_pieces(s, [1 - dx, 1 - dy], place, clip, pass, out);
            }
        }
    }

    /// A hero's or boat's pictures in order (design §2.3, §2.8): on land the flag by the owner
    /// and the hero by its class (0x40FE30); in a boat the waves unless its tile is the coast,
    /// the boat's flag by its type and colour, the boat; an empty boat's waves and boat
    /// (0x410760). Each its facing's group, mirrored above facing 4, its frame the moment's
    /// frame step modulo group 0's frames (no phase); the 96 × 64 frame's square `square` (in
    /// tiles from its top-left) at the tile's place. The shadow pass draws the same through the
    /// shadow drawer (0x4102C0, 0x4109B0).
    fn standing_pieces(
        &self,
        standing: Standing<'_>,
        square: [i32; 2],
        place: [i32; 2],
        clip: [i32; 4],
        pass: Pass,
        out: &mut Vec<Draw>,
    ) {
        let art = &self.tables.map.heroes;
        let pick = |list: &'a [String], i: usize| list.get(i).map(String::as_str);
        let waves = |at: h3_map::model::Pos, kind: usize| {
            let coast = self
                .art
                .tile(at.x.into(), at.y.into(), at.z)
                .is_some_and(|t| t.coast());
            pick(&art.waves, kind).filter(|_| !coast)
        };
        let (facing, defs): (Facing, [Option<&str>; 3]) = match standing {
            Standing::Hero(h) => match h.boat {
                None => (
                    h.facing,
                    [
                        pick(&art.flags, h.owner.into()),
                        pick(&art.classes, h.class.into()),
                        None,
                    ],
                ),
                Some(boat) => {
                    let (kind, colour) = (usize::from(boat.kind), usize::from(boat.colour));
                    let flag = (colour < 8).then(|| kind * 8 + colour);
                    (
                        h.facing,
                        [
                            waves(h.at, kind),
                            flag.and_then(|i| pick(&art.boat_flags, i)),
                            pick(&art.boats, kind),
                        ],
                    )
                }
            },
            Standing::Boat(b) => {
                let kind = usize::from(b.kind);
                (b.facing, [waves(b.at, kind), pick(&art.boats, kind), None])
            }
        };
        let group = art
            .groups
            .get(usize::from(facing.number()))
            .copied()
            .unwrap_or(0);
        let tile = self.layout.view.tile;
        for def in defs.into_iter().flatten() {
            let frames = self.shapes.frames(def).unwrap_or(0);
            let frame = self.live.moment.frames.checked_rem(frames).unwrap_or(0);
            out.push(Draw::Piece {
                pass,
                def: String::from(def),
                group,
                frame,
                x: place[0] - tile * square[0],
                y: place[1] - tile * square[1],
                mirror_x: facing.mirrored(),
                mirror_y: false,
                clip,
                flag: None,
            });
        }
    }

    /// The pieces one object pass draws on a tile: each entry of its list (of `layer` only, if
    /// given) that the pass draws (§2.2: flat templates in pass 4, every cell; the others' shadow
    /// mask's cells in pass 5, their colour mask's in pass 7), the cell's square of the object's
    /// frame at the tile's place.
    fn objects(
        &self,
        x: i32,
        y: i32,
        place: [i32; 2],
        out: &mut Vec<Draw>,
        pass: Pass,
        layer: Option<u8>,
    ) {
        let Some(clip) = self.clip(place, 0) else {
            return;
        };
        let entries = self.lists.at(x, y).iter();
        for entry in entries.filter(|e| layer.is_none_or(|l| e.layer == l)) {
            out.extend(self.object_piece(entry, place, clip, pass));
        }
    }

    /// An entry's piece in a pass, if the pass draws it.
    fn object_piece(
        &self,
        entry: &Entry,
        place: [i32; 2],
        clip: [i32; 4],
        pass: Pass,
    ) -> Option<Draw> {
        let index = entry.object;
        let template = objects::template(self.art, index);
        let shape = objects::shape(self.art, self.live, self.shapes, index, self.bugs);
        let def = objects::def(self.art, self.live, index, self.bugs);
        let (dx, dy) = (entry.dx, entry.dy);
        let drawn = template
            .zip(shape)
            .zip(def)
            .filter(|((t, s), _)| match pass {
                Pass::Flat => t.flat(),
                Pass::Shadow => !t.flat() && s.shadow(dx, dy),
                _ => !t.flat() && s.body(dx, dy),
            });
        let flag = match pass {
            Pass::Flat | Pass::Body => objects::class(self.art, self.live, index).and_then(|c| {
                let owner = objects::flag_owner(self.art, self.live, index, self.bugs);
                self.tables.map.owners.entry(c, owner)
            }),
            _ => None,
        };
        drawn.map(|((_, shape), def)| {
            let tile = self.layout.view.tile;
            let phase = self.live.objects.get(index).map_or(0, |o| o.phase);
            let step = u64::from(phase) + u64::from(self.live.moment.frames);
            let frame = step.checked_rem(u64::from(shape.frames)).unwrap_or(0);
            let (columns, rows) = (i32::from(shape.width), i32::from(shape.height));
            Draw::Piece {
                pass,
                def: String::from(def),
                group: 0,
                frame: u32::try_from(frame).unwrap_or(0),
                x: place[0] - tile * (columns - 1 - i32::from(dx)),
                y: place[1] - tile * (rows - 1 - i32::from(dy)),
                mirror_x: false,
                mirror_y: false,
                clip,
                flag,
            }
        })
    }

    /// Pass 6: the shadow of the tile's path arrow (0x411F50).
    fn path_shadow(&self, x: i32, y: i32, place: [i32; 2], out: &mut Vec<Draw>) {
        self.arrow(x, y, place, out, Pass::PathShadow);
    }

    /// Pass 8: the tile's path arrow (0x4120B0).
    fn path(&self, x: i32, y: i32, place: [i32; 2], out: &mut Vec<Draw>) {
        self.arrow(x, y, place, out, Pass::Path);
    }

    /// The tile's path arrow a in `pass`: the path DEF's frame a − 1 on the tile, a tile of the
    /// map with an arrow only (design §2.2).
    fn arrow(&self, x: i32, y: i32, place: [i32; 2], out: &mut Vec<Draw>, pass: Pass) {
        let on_map = self.art.tile(x, y, self.at.z).is_some();
        let Some((&arrow, clip)) = self
            .arrows
            .get(&(x, y))
            .filter(|_| on_map)
            .zip(self.clip(place, 0))
        else {
            return;
        };
        out.push(Draw::Piece {
            pass,
            def: self.tables.map.path.clone(),
            group: 0,
            frame: u32::from(arrow).saturating_sub(1),
            x: place[0],
            y: place[1],
            mirror_x: false,
            mirror_y: false,
            clip,
            flag: None,
        });
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
            group: 0,
            frame: frame.into(),
            x: place[0],
            y: place[1],
            mirror_x,
            mirror_y: false,
            clip,
            flag: None,
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
