//! The adventure screen's scene (E24 M1 T3b1, T4a, *provisional*): what it shows at a view and a
//! moment, as a draw list in the original's order (`docs/epics/E24-design.md` §1.1, §1.9, §2.2).
//! The frame's picture in the viewer's colours, the adventure screen's dialog and resource bar
//! (E24 M3 T2a, §4.1: [`crate::panels`]), then the view's passes, each over the whole view,
//! rows outer and columns inner: the terrain with the map's edge (§1.3, §1.4), the rivers, the
//! roads, the flat objects, the objects' shadows, their bodies by layer (§2.2: one 32 × 32 piece
//! per entry of a tile's draw list, [`crate::objects`]), the unexplored tiles (§1.5), the corner
//! gems; the heroes and boats as two-row objects among them (E24 M1 T5a, §2.3, §2.8): their
//! shadows after the objects', their pictures after the bodies of layers 1 and 2; the path's
//! arrows (E24 M1 T5b2, §2.10): their shadows after every other shadow, their pixels after the
//! bodies. Then the minimap's tiles and box (E24 M3 T3a, §4.4: [`crate::minimap`]). A walk's
//! draw (E24 M2 T3b, §3.5, §3.8; [`crate::live::Walking`]) adds the view's pixel offset to
//! every pass's places and draws the lifted hero at the view's middle instead of on its tile: its
//! shadow's and pictures' cells in passes 5 and 7 among the tiles', then the whole of it once
//! more, blended, after the arrows (pass 9). [`crate::compose`] draws the list.
//!
//! Each piece of the view carries its clip: the 32 × 32 rectangle of its tile (a road's 16 pixels
//! lower) inside the view's clip (§1.2), as the passes cut their source rectangles.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::ops::Range;

use h3_map::model::Pos;
use h3_render::Bugs;
use h3_ui::ui;

use crate::art::{Layer, MapArt};
use crate::layout::AdventureLayout;
use crate::live::{Facing, Lifted, Live, LiveBoat, LiveHero};
use crate::minimap::{self, Base};
use crate::objects::{self, DrawLists, Entry, ObjectsError, Shape, Shapes};
use crate::panels;
use crate::status;
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
    /// The lifted hero drawn whole once more (pass 9, `view_moving_hero_step` 0x47FC80, E24 M2
    /// T3b): its literal pixels blended half and half with the screen, its runs skipped
    /// (`draw_frame_blend` 0x47D4F0 with no colour, through `def_draw_group_blend` 0x47C120).
    Blend,
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
    /// One of the dialog's or the resource bar's draws (E24 M3 T2a, design §4.1), drawn as the
    /// executable draws its item (`h3_ui::paint`).
    Dialog(ui::Draw),
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
    /// The minimap's tiles (E24 M3 T3a, design §4.4): each tile's colour in the pixels its
    /// column and row take, straight into the screen.
    Minimap(minimap::Tiles),
    /// The minimap's box (E24 M3 T3a, design §4.4): frame `frame` of `def`'s group 0 through the
    /// plain drawer with runs skipped, from its canvas's column and row `cut` at `at`
    /// (`def_draw_button_frame` 0x47BF00), as wide as the DEF less the columns cut, less what lies
    /// past `area`'s right edge, as high likewise.
    Radar {
        /// The DEF.
        def: String,
        /// The frame.
        frame: u32,
        /// Where its canvas's part is drawn.
        at: [i32; 2],
        /// The canvas's columns and rows cut at its left and top.
        cut: [i32; 2],
        /// The minimap's area (left, top, right, bottom; the last two exclusive).
        area: [i32; 4],
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

/// Where the view lies on the screen (E24 M3 T2b2, *provisional*; design §4.3): its clip, the
/// place of the map's tile (0, 0) with the walk's offset, and a tile's side. A view moved by some
/// pixels moves each of its pieces by as many (the lifted hero and the gems aside), so a frame
/// keeps what stays ([`crate::compose::Composer`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewPlace {
    /// The view's clip (left, top, right, bottom; the last two exclusive).
    pub clip: [i32; 4],
    /// The screen's point of the map's tile (0, 0)'s top-left corner.
    pub origin: [i32; 2],
    /// A tile's side.
    pub tile: i32,
}

/// What the screen shows: the draw list in order, and the palette cycles its DEFs are drawn with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scene {
    /// The draws, in order.
    pub draws: Vec<Draw>,
    /// Where the view's pieces lie (none for a scene without a view).
    pub view: Option<ViewPlace>,
    /// Every cycling DEF of the tables, with the moment's steps.
    pub cycles: Vec<Cycle>,
    /// The palette the pieces' flag entries index (the tables' owners' palette, `game.pal`).
    pub flags: String,
    /// The original's drawing bugs reproduced or fixed (the dialog's texts' long word, E17-design
    /// §3.6).
    pub bugs: Bugs,
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
    /// The level's objects have no draw lists, or an object's picture has no shape.
    Objects(ObjectsError),
    /// A hero's or boat's DEF has no frames (`Shapes::insert_frames`).
    Frames(String),
    /// The live part does not fit the map: what does not.
    Live(String),
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
            Self::Frames(def) => write!(f, "{def}: no frames"),
            Self::Live(what) => write!(f, "the live part does not fit the map: {what}"),
        }
    }
}

/// The scene of the adventure screen at view `at`, with the original's drawing bugs fixed or,
/// for the comparisons, reproduced (`bugs`: MCL-Q2's road on the map's last row, MCL-Q3's random
/// artifact's picture, MCL-Q4's flag of an object without an entrance on the map, MCL-Q7's lifted
/// hero at the map's side during a step). `shapes` gives
/// the objects' templates their `.msk` and frames ([`objects::pictures`] names them), and the
/// heroes' and boats' DEFs their frames.
///
/// # Errors
/// [`SceneError`]: the level or the viewer out of range, a live part that does not fit the map, a
/// code the tables lack, an object's picture without a shape, a hero's or boat's DEF without
/// frames while one stands on the level.
pub fn scene(
    art: &MapArt,
    live: &Live,
    tables: &Tables,
    layout: &AdventureLayout,
    shapes: &Shapes,
    at: ViewAt,
    bugs: Bugs,
) -> Result<Scene, SceneError> {
    check_view(art, live, at)?;
    let index = TileIndex::build(art, live, tables, layout, shapes, at.z, bugs)?;
    scene_in(&index, art, live, tables, layout, shapes, at, bugs)
}

/// Every tile of level `z` of a terrain the tables give (the minimap draws them all, E24 M3
/// T3a): the first that is not, from the top left.
fn check_terrains(art: &MapArt, tables: &Tables, z: u8) -> Result<(), SceneError> {
    let side = i32::try_from(art.size).unwrap_or(0);
    for y in 0..side {
        for x in 0..side {
            let code = art.tile(x, y, z).map(|t| t.terrain.code);
            if let Some(code) = code.filter(|c| !tables.terrains.contains_key(c)) {
                return Err(SceneError::Code {
                    layer: Layer::Terrain,
                    code,
                    at: [x, y],
                });
            }
        }
    }
    Ok(())
}

/// The view's level on the map, the viewer a colour, the live part fitting the map.
fn check_view(art: &MapArt, live: &Live, at: ViewAt) -> Result<(), SceneError> {
    if usize::from(at.z) >= art.levels {
        return Err(SceneError::Level(at.z));
    }
    if live.viewer > 7 {
        return Err(SceneError::Viewer(live.viewer));
    }
    check_live(art, live)
}

/// A level's objects as the view's passes look them up (E24 M3 T2b1, *provisional*; design
/// §4.2): the level's spatial index, each tile's draw list in a dense grid of the map's tiles (a
/// tile's list found at once whatever the map's size), and each listed object's look (its DEF,
/// its shape, whether it is flat, its flag's entry, its phase), worked out once for a live part's
/// objects, the level's map-wide checks done as it is built. A view then visits only its own
/// tiles ([`scene_in`]). It holds while the live part's objects stay as they were (their
/// pictures, owners, phases and whether they stand; none change before M4): built again when they
/// change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileIndex {
    z: u8,
    lists: DrawLists,
    looks: Vec<Option<Look>>,
    /// The level's minimap but the explored tiles and the heroes (E24 M3 T3a); none without the
    /// layout's area.
    minimap: Option<Base>,
}

/// How a listed object is drawn (what [`View::object_piece`] took from the map and the live part
/// for each piece).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Look {
    def: String,
    shape: Shape,
    flat: bool,
    flag: Option<u8>,
    phase: u8,
}

impl TileIndex {
    /// Level `z`'s index for `live`'s objects, with the original's drawing bugs fixed or
    /// reproduced (`bugs`: MCL-Q3's random artifact's picture, MCL-Q4's flag and minimap tile),
    /// the minimap's into `layout`'s area.
    ///
    /// # Errors
    /// [`SceneError::Objects`]: an object of the level with entries whose picture has no shape.
    pub fn build(
        art: &MapArt,
        live: &Live,
        tables: &Tables,
        layout: &AdventureLayout,
        shapes: &Shapes,
        z: u8,
        bugs: Bugs,
    ) -> Result<Self, SceneError> {
        let lists = DrawLists::build(art, live, shapes, z).map_err(SceneError::Objects)?;
        check_object_shapes(art, live, shapes, z, bugs)?;
        let looks = (0..art.objects.len())
            .map(|index| {
                let on_level = art.objects.get(index).is_some_and(|o| o.anchor.z == z);
                let listed = objects::template(art, index)
                    .filter(|t| on_level && !objects::NO_ENTRIES.contains(&t.class));
                let ((template, shape), def) = listed
                    .zip(objects::shape(art, live, shapes, index, bugs))
                    .zip(objects::def(art, live, index, bugs))?;
                let flag = objects::class(art, live, index).and_then(|c| {
                    let owner = objects::flag_owner(art, live, index, bugs);
                    tables.map.owners.entry(c, owner)
                });
                Some(Look {
                    def: String::from(def),
                    shape,
                    flat: template.flat(),
                    flag,
                    phase: live.objects.get(index).map_or(0, |o| o.phase),
                })
            })
            .collect();
        check_terrains(art, tables, z)?;
        let minimap = Base::build(art, live, tables, layout, &lists, z, bugs);
        Ok(Self {
            z,
            lists,
            looks,
            minimap,
        })
    }

    /// Its level.
    #[must_use]
    pub fn level(&self) -> u8 {
        self.z
    }
}

/// [`scene`] with level `at.z`'s objects looked up in `index` ([`TileIndex::build`] for the same
/// map, live part's objects and bugs): only the view's tiles are visited (E24 M3 T2b1).
///
/// # Errors
/// As [`scene`] but the objects' shapes, which `index` checked; [`SceneError::Level`] as well for
/// an index of another level.
#[expect(clippy::too_many_arguments, reason = "scene's, with the index")]
pub fn scene_in(
    index: &TileIndex,
    art: &MapArt,
    live: &Live,
    tables: &Tables,
    layout: &AdventureLayout,
    shapes: &Shapes,
    at: ViewAt,
    bugs: Bugs,
) -> Result<Scene, SceneError> {
    check_view(art, live, at)?;
    if index.z != at.z {
        return Err(SceneError::Level(at.z));
    }
    check_hero_frames(live, tables, shapes, at.z)?;
    let view = View {
        art,
        live,
        tables,
        layout,
        shapes,
        index,
        standing: standing(live, at.z),
        lifted: lifted(live),
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
    // The panels (`adventure_full_update` 0x417420 after its picture, design §4.1): the dialog's
    // items in their order, the lists' selections (0x4038C0, §4.5), then the resource bar's; the
    // view after them.
    let players = &layout.player_palette;
    let lists = &tables.map.lists;
    let shown = panels::dialog(layout, lists, live, art.levels, at.z)
        .ok_or_else(|| SceneError::Live("a hero's portrait is past the tables'".into()))?;
    let bar = panels::resource_bar(layout, live, &tables.texts);
    // The status window's panel (E24 M3 T5b1, design §4.8): its items join the dialog's after the
    // resource bar's (none overlaps another panel's).
    let status = status::window(
        layout,
        &tables.map.status,
        lists,
        live,
        &art.hero_names,
        &tables.texts,
    )
    .map_err(SceneError::Live)?;
    draws.extend(
        layout
            .dialog
            .draws(&shown, players)
            .into_iter()
            .chain(panels::selections(layout, lists, live))
            .chain(layout.resource_bar.draws(&bar, players))
            .chain(
                status
                    .map(|(panel, shown)| panel.draws(&shown, players))
                    .into_iter()
                    .flatten(),
            )
            .map(Draw::Dialog),
    );
    let passes: [TilePass<'_>; 8] = [
        View::terrain,
        View::river,
        View::road,
        View::flat,
        View::shadows,
        View::path_shadow,
        View::bodies,
        View::path,
    ];
    for pass in passes {
        view.pass(&mut draws, pass);
    }
    view.lifted_whole(&mut draws);
    view.pass(&mut draws, View::shroud);
    for gem in &layout.view.gems {
        draws.push(Draw::Gem {
            def: gem.def.clone(),
            frame: u32::from(live.viewer),
            x: gem.at[0],
            y: gem.at[1],
        });
    }
    // The minimap after the view (`adventure_full_update` 0x41753A, design §4.4), as it was last
    // drawn (T3b).
    if let Some(base) = &index.minimap {
        let (view, hero) = live.minimap.map_or((at, None), |m| (m.view, m.hero));
        draws.push(Draw::Minimap(base.tiles(live, tables, hero)));
        draws.push(base.radar(view));
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
    let [ox, oy] = live.walking.offset;
    let [x, y] = layout
        .view
        .place(at.x.saturating_neg(), at.y.saturating_neg());
    Ok(Scene {
        draws,
        view: Some(ViewPlace {
            clip: layout.view.clip,
            origin: [x.saturating_add(ox), y.saturating_add(oy)],
            tile: layout.view.tile,
        }),
        cycles,
        flags: tables.map.owners.palette.clone(),
        bugs,
    })
}

/// The view being drawn.
struct View<'a> {
    art: &'a MapArt,
    live: &'a Live,
    tables: &'a Tables,
    layout: &'a AdventureLayout,
    shapes: &'a Shapes,
    index: &'a TileIndex,
    standing: BTreeMap<(i32, i32), Standing<'a>>,
    /// The lifted hero and its group and frame (`check_live` found it).
    lifted: Option<(&'a LiveHero, Lifted)>,
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
    // The lifted hero is off its tile (`hero_select` hides it from its tile's list, +0x20C).
    let lifted = live.walking.lifted.map(|l| l.hero);
    let heroes = live
        .heroes
        .iter()
        .enumerate()
        .filter(|&(i, _)| Some(i) != lifted)
        .map(|(_, h)| (h.at, Standing::Hero(h)));
    let boats = live.boats.iter().map(|b| (b.at, Standing::Boat(b)));
    let mut out = BTreeMap::new();
    for (at, s) in heroes.chain(boats).filter(|(at, _)| at.z == z) {
        out.entry((i32::from(at.x), i32::from(at.y))).or_insert(s);
    }
    out
}

/// The lifted hero with its group and frame, if one is lifted (`check_live` found it).
fn lifted(live: &Live) -> Option<(&LiveHero, Lifted)> {
    let lifted = live.walking.lifted?;
    live.heroes.get(lifted.hero).map(|h| (h, lifted))
}

/// The live part fits the map (E24 M1 T6b, the reviewer's R1-3): one entry per object, the
/// explored tiles of the map's size and levels, every hero, boat and arrow on the map, every hero
/// owned by a colour (the original draws no hero on land without one, design §2.8).
fn check_live(art: &MapArt, live: &Live) -> Result<(), SceneError> {
    let wrong = |what: String| Err(SceneError::Live(what));
    if live.objects.len() != art.objects.len() {
        return wrong(format!(
            "{} objects for the map's {}",
            live.objects.len(),
            art.objects.len()
        ));
    }
    if live.explored.size() != (art.size, art.levels) {
        return wrong(String::from(
            "the explored tiles are not of the map's size and levels",
        ));
    }
    let on_map = |at: Pos| {
        usize::from(at.x) < art.size
            && usize::from(at.y) < art.size
            && usize::from(at.z) < art.levels
    };
    let heroes = live.heroes.iter().map(|h| ("a hero", h.at, h.owner < 8));
    let boats = live.boats.iter().map(|b| ("a boat", b.at, true));
    let arrows = live.path.iter().map(|a| ("an arrow", a.at, true));
    for (what, at, owned) in heroes.chain(boats).chain(arrows) {
        let tile = (at.x, at.y, at.z);
        if !on_map(at) {
            return wrong(format!("{what} at {tile:?} is off the map"));
        }
        if !owned {
            return wrong(format!("{what} at {tile:?} has no owner"));
        }
    }
    if let Some(Lifted { hero, .. }) = live.walking.lifted
        && hero >= live.heroes.len()
    {
        return wrong(format!(
            "the lifted hero {hero} of {} heroes",
            live.heroes.len()
        ));
    }
    Ok(())
}

/// Every object level `z` draws has its shape (E24 M1 T6b, the reviewer's R1-4): each object
/// with entries the one it is drawn with ([`objects::shape`]: a random object's drawn or own DEF
/// too; [`DrawLists::build`] checked its template's).
fn check_object_shapes(
    art: &MapArt,
    live: &Live,
    shapes: &Shapes,
    z: u8,
    bugs: Bugs,
) -> Result<(), SceneError> {
    for (index, object) in art.objects.iter().enumerate() {
        let listed = objects::template(art, index)
            .is_some_and(|t| object.anchor.z == z && !objects::NO_ENTRIES.contains(&t.class));
        if listed && objects::shape(art, live, shapes, index, bugs).is_none() {
            let def = objects::def(art, live, index, bugs).unwrap_or_default();
            return Err(SceneError::Objects(ObjectsError::Shape(index, def.into())));
        }
    }
    Ok(())
}

/// While a hero or boat stands on level `z`, every DEF of the tables' heroes has its frames
/// (the reviewer's R1-4).
fn check_hero_frames(
    live: &Live,
    tables: &Tables,
    shapes: &Shapes,
    z: u8,
) -> Result<(), SceneError> {
    let standing = live
        .heroes
        .iter()
        .map(|h| h.at)
        .chain(live.boats.iter().map(|b| b.at));
    if standing.into_iter().any(|at| at.z == z) {
        let defs = tables.map.heroes.defs();
        if let Some((def, _)) = defs.into_iter().find(|(d, _)| shapes.frames(d).is_none()) {
            return Err(SceneError::Frames(def));
        }
    }
    Ok(())
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

/// The lifted hero's picture in tiles, columns and rows (96 × 64: design §3.5).
const LIFTED: [i32; 2] = [3, 2];

/// The rectangle two rectangles share, if it is not empty.
fn meet(a: [i32; 4], b: [i32; 4]) -> Option<[i32; 4]> {
    let rect = [
        a[0].max(b[0]),
        a[1].max(b[1]),
        a[2].min(b[2]),
        a[3].min(b[3]),
    ];
    (rect[0] < rect[2] && rect[1] < rect[3]).then_some(rect)
}

/// A cell of the lifted hero's picture over a tile: the hero, its group and frame, the cell's
/// column (0 to 2) and row (0 or 1).
#[derive(Debug, Clone, Copy)]
struct Cell<'h> {
    hero: &'h LiveHero,
    lifted: Lifted,
    at: [i32; 2],
}

/// Which of the lifted hero's draws: pass 5's cell, pass 7's cell, pass 9's whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Picture {
    Shadow,
    Cell,
    Whole,
}

/// One pass's work on one tile: its map coordinates, its place, the list.
type TilePass<'v> = fn(&View<'v>, i32, i32, [i32; 2], &mut Vec<Draw>);

impl<'a> View<'a> {
    /// Calls `visit` with each tile of the view: rows outer, columns inner (design §1.1), at its
    /// place moved by the walk's offset (`hero_walk_animation` 0x480380: every pass adds +0xF4
    /// and +0xF8 to every tile's place; design §3.5).
    fn tiles(&self, mut visit: impl FnMut(i32, i32, [i32; 2])) {
        let view = &self.layout.view;
        let [dx, dy] = self.live.walking.offset;
        for j in view.rows[0]..=view.rows[1] {
            for i in view.columns[0]..=view.columns[1] {
                let (x, y) = (self.at.x.saturating_add(i), self.at.y.saturating_add(j));
                let [px, py] = view.place(i, j);
                visit(x, y, [px.saturating_add(dx), py.saturating_add(dy)]);
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

    /// The map's rectangle on the screen: its tiles' places moved by the walk's offset (MCL-Q7's
    /// cut of the lifted hero).
    fn map_shown(&self) -> [i32; 4] {
        let view = &self.layout.view;
        let [ox, oy] = self.live.walking.offset;
        let side = self.side();
        let [left, top] = view.place(-self.at.x, -self.at.y);
        let [right, bottom] = view.place(side - self.at.x, side - self.at.y);
        [left + ox, top + oy, right + ox, bottom + oy]
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

    /// Pass 5: the tile's pieces of the objects' shadows, in list order, then the lifted hero's
    /// cell over the tile (0x411A2C), then the heroes' and boats' by slot (0x411590; design §2.2,
    /// §3.8).
    fn shadows(&self, x: i32, y: i32, place: [i32; 2], out: &mut Vec<Draw>) {
        self.objects(x, y, place, out, Pass::Shadow, None);
        if let Some(cell) = self.lifted_cell(x, y) {
            self.lifted_piece(cell, Pass::Shadow, out);
        }
        self.standing(x, y, place, out, Pass::Shadow, 0..SLOTS);
    }

    /// Pass 7: the tile's pieces of the objects' bodies, layer by layer, each in list order, the
    /// heroes and boats on the tile's row after layer 1 and those on the row below after layer 2,
    /// the lifted hero's lower cells after the first and its upper ones after the second (0x410C00;
    /// design §2.2, §2.3, §3.8). A tile with an empty list draws its heroes and boats, then the
    /// lifted hero's cell (0x411396).
    fn bodies(&self, x: i32, y: i32, place: [i32; 2], out: &mut Vec<Draw>) {
        let cell = self.lifted_cell(x, y);
        if self.index.lists.at(x, y).is_empty() {
            self.standing(x, y, place, out, Pass::Body, 0..SLOTS);
            if let Some(cell) = cell {
                self.lifted_piece(cell, Pass::Body, out);
            }
            return;
        }
        for layer in 0..=MAX_LAYER {
            self.objects(x, y, place, out, Pass::Body, Some(layer));
            let (slots, row) = match layer {
                1 => (0..3, 1),
                2 => (3..SLOTS, 0),
                _ => continue,
            };
            self.standing(x, y, place, out, Pass::Body, slots);
            if let Some(cell) = cell.filter(|c| c.at[1] == row) {
                self.lifted_piece(cell, Pass::Body, out);
            }
        }
    }

    /// The view's tile the lifted hero's 96 × 64 picture has its top-left corner on (the view's
    /// middle less one column and one row: (8, 7)), and that corner's place, which the walk's
    /// offset does not move (0x47F860, 0x47FC80: 256, 232).
    fn lifted_corner(&self) -> ([i32; 2], [i32; 2]) {
        let [mx, my] = self.layout.scroll.middle;
        let corner = [mx - 1, my - 1];
        (corner, self.layout.view.place(corner[0], corner[1]))
    }

    /// The cell (column 0 to 2, row 0 or 1) of the lifted hero's picture over the map's tile
    /// (x, y), if a hero is lifted: with the original's bugs only on the map (passes 5 and 7 draw
    /// no piece off it: 0x4115A1, 0x410C13); the client's on every tile, cut where it lies off the
    /// map ([`View::lifted_piece`], MCL-Q7).
    fn lifted_cell(&self, x: i32, y: i32) -> Option<Cell<'a>> {
        let (hero, lifted) = self.lifted?;
        if self.bugs == Bugs::Original {
            self.art.tile(x, y, self.at.z)?;
        }
        let ([ci, cj], _) = self.lifted_corner();
        let (a, b) = (x - self.at.x - ci, y - self.at.y - cj);
        ((0..LIFTED[0]).contains(&a) && (0..LIFTED[1]).contains(&b)).then_some(Cell {
            hero,
            lifted,
            at: [a, b],
        })
    }

    /// One cell of the lifted hero's picture (`view_moving_hero_shadow_piece` 0x47FB40 in pass 5,
    /// `view_moving_hero_piece` 0x47F860 in pass 7; design §3.5): the cell's square of each of its
    /// pictures at the fixed place, its group and frame the walk's, mirrored by its facing. The
    /// shadow is the hero's or the boat's alone; the pictures on land its flag and the hero, in
    /// a boat the waves (not on the coast), the boat's flag by its type and colour, the boat. A
    /// flag's frame is the objects' frame step plus the hero's, modulo the walk's frames. The cell
    /// lies inside the view's clip (the original clips it to the screen alone). The client cuts it
    /// at the map's side moved by the walk's offset, as a standing hero is cut at the map's side;
    /// the original draws the whole cell of a tile on the map, at a place the offset does not move
    /// (MCL-Q7: a step's frames at a side draw the hero over the map's edge or leave a strip of
    /// it undrawn).
    fn lifted_piece(&self, cell: Cell<'_>, pass: Pass, out: &mut Vec<Draw>) {
        let Cell {
            hero,
            lifted,
            at: [a, b],
        } = cell;
        let tile = self.layout.view.tile;
        let (_, [cx, cy]) = self.lifted_corner();
        let (left, top) = (cx + tile * a, cy + tile * b);
        let square = [left, top, left + tile, top + tile];
        let clip = match self.bugs {
            Bugs::Fixed => meet(square, self.map_shown()),
            Bugs::Original => Some(square),
        };
        let Some(clip) = clip else {
            return;
        };
        let defs = match pass {
            Pass::Shadow => self.lifted_defs(hero, lifted, Picture::Shadow),
            _ => self.lifted_defs(hero, lifted, Picture::Cell),
        };
        for (def, frame) in defs.into_iter().flatten() {
            out.push(Draw::Piece {
                pass,
                def: String::from(def),
                group: lifted.group,
                frame,
                x: cx,
                y: cy,
                mirror_x: hero.facing.mirrored(),
                mirror_y: false,
                clip,
                flag: None,
            });
        }
    }

    /// Pass 9 (`view_moving_hero_step` 0x47FC80; design §3.5, §3.8): the lifted hero's flag, then
    /// the hero or its boat, whole, blended over everything drawn, at the fixed place; a boat's
    /// flag by its type and the hero's owner. The client cuts it at the map's side moved by the
    /// walk's offset, as its cells (MCL-Q7). The original cuts it by whole tiles past a side, on
    /// the left and the top less the walk's offset there too, on the right and the bottom not:
    /// with passes 5 and 7's cells, cut at the view's tiles, a step's frames at a side leave a
    /// strip of the hero half transparent or undrawn, or draw it over the map's edge.
    fn lifted_whole(&self, out: &mut Vec<Draw>) {
        let Some((hero, lifted)) = self.lifted else {
            return;
        };
        let tile = self.layout.view.tile;
        let ([ci, cj], [cx, cy]) = self.lifted_corner();
        let picture = [cx, cy, cx + LIFTED[0] * tile, cy + LIFTED[1] * tile];
        let map = self.map_shown();
        let bounds = match self.bugs {
            Bugs::Fixed => map,
            Bugs::Original => {
                let [ox, oy] = self.live.walking.offset;
                let side = self.side();
                let (first_x, first_y) = (self.at.x + ci, self.at.y + cj);
                [
                    if first_x < 0 { map[0] } else { i32::MIN },
                    if first_y < 0 { map[1] } else { i32::MIN },
                    if first_x + LIFTED[0] >= side {
                        map[2] - ox
                    } else {
                        i32::MAX
                    },
                    if first_y + LIFTED[1] >= side {
                        map[3] - oy
                    } else {
                        i32::MAX
                    },
                ]
            }
        };
        let Some(clip) = meet(picture, bounds) else {
            return;
        };
        for (def, frame) in self
            .lifted_defs(hero, lifted, Picture::Whole)
            .into_iter()
            .flatten()
        {
            out.push(Draw::Piece {
                pass: Pass::Blend,
                def: String::from(def),
                group: lifted.group,
                frame,
                x: cx,
                y: cy,
                mirror_x: hero.facing.mirrored(),
                mirror_y: false,
                clip,
                flag: None,
            });
        }
    }

    /// The lifted hero's DEFs a draw of `picture` takes, in order, each with its frame.
    fn lifted_defs(
        &self,
        hero: &LiveHero,
        lifted: Lifted,
        picture: Picture,
    ) -> [Option<(&str, u32)>; 3] {
        let art = &self.tables.map.heroes;
        let pick = |list: &'a [String], i: usize| list.get(i).map(String::as_str);
        let frame = lifted.frame;
        let flag_frame = self
            .live
            .moment
            .frames
            .wrapping_add(frame)
            .checked_rem(self.tables.map.walk.frames)
            .unwrap_or(0);
        let with = |def: Option<&'a str>, f: u32| def.map(|d| (d, f));
        let owner = usize::from(hero.owner);
        match (hero.boat, picture) {
            (None, Picture::Shadow) => [
                with(pick(&art.classes, hero.class.into()), frame),
                None,
                None,
            ],
            (None, _) => [
                with(pick(&art.flags, owner), flag_frame),
                with(pick(&art.classes, hero.class.into()), frame),
                None,
            ],
            (Some(boat), Picture::Shadow) => {
                [with(pick(&art.boats, boat.kind.into()), frame), None, None]
            }
            (Some(boat), Picture::Cell) => {
                let kind = usize::from(boat.kind);
                let coast = self
                    .art
                    .tile(hero.at.x.into(), hero.at.y.into(), hero.at.z)
                    .is_some_and(|t| t.coast());
                let flag = (boat.colour < 8).then(|| kind * 8 + usize::from(boat.colour));
                [
                    with(pick(&art.waves, kind).filter(|_| !coast), frame),
                    with(flag.and_then(|i| pick(&art.boat_flags, i)), flag_frame),
                    with(pick(&art.boats, kind), frame),
                ]
            }
            (Some(boat), Picture::Whole) => {
                let kind = usize::from(boat.kind);
                [
                    with(pick(&art.boat_flags, kind * 8 + owner), flag_frame),
                    with(pick(&art.boats, kind), frame),
                    None,
                ]
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
        let waves = |at: Pos, kind: usize| {
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
        let entries = self.index.lists.at(x, y).iter();
        for entry in entries.filter(|e| layer.is_none_or(|l| e.layer == l)) {
            out.extend(self.object_piece(entry, place, clip, pass));
        }
    }

    /// An entry's piece in a pass, if the pass draws it (its object's look from the index).
    fn object_piece(
        &self,
        entry: &Entry,
        place: [i32; 2],
        clip: [i32; 4],
        pass: Pass,
    ) -> Option<Draw> {
        let look = self.index.looks.get(entry.object).and_then(Option::as_ref);
        look.and_then(|look| self.look_piece(look, entry, place, clip, pass))
    }

    /// A listed object's piece of `entry` in a pass, if the pass draws it.
    fn look_piece(
        &self,
        look: &Look,
        entry: &Entry,
        place: [i32; 2],
        clip: [i32; 4],
        pass: Pass,
    ) -> Option<Draw> {
        let (shape, dx, dy) = (&look.shape, entry.dx, entry.dy);
        let drawn = match pass {
            Pass::Flat => look.flat,
            Pass::Shadow => !look.flat && shape.shadow(dx, dy),
            _ => !look.flat && shape.body(dx, dy),
        };
        let flag = match pass {
            Pass::Flat | Pass::Body => look.flag,
            _ => None,
        };
        drawn.then(|| {
            let tile = self.layout.view.tile;
            let step = u64::from(look.phase) + u64::from(self.live.moment.frames);
            let frame = step.checked_rem(u64::from(shape.frames)).unwrap_or(0);
            let (columns, rows) = (i32::from(shape.width), i32::from(shape.height));
            Draw::Piece {
                pass,
                def: look.def.clone(),
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
