use super::*;
use alloc::collections::BTreeSet;
use alloc::string::ToString;
use alloc::vec;

use h3_map::model::Pos;

use h3_map::Limits;
use h3_map::writer::{self, AnyObject, Extras, GivenTile, RiverKind, RoadKind, Terrain};

use crate::art::tests::{map, raw, template};
use crate::live::{Arrow, Boat, Moment};
use crate::objects::Shape;
use crate::tables;

/// The committed tables and layout (`clients/map/`).
fn tables() -> Tables {
    let text = include_str!("../../../../map/assets/base.toml");
    tables::load(&[tables::parse(text).unwrap()], "base").unwrap()
}

fn layout() -> AdventureLayout {
    AdventureLayout::parse(include_str!("../../../../map/layouts/adventure.toml")).unwrap()
}

/// The art test's map (36 × 36 grass, a pool at the top-left, a dirt road on row 5 from column 3
/// to 6) with the given tiles.
fn art_with(tiles: Vec<GivenTile>) -> (MapArt, Live) {
    let extras = Extras {
        tiles,
        objects: vec![],
    };
    let bytes = writer::write_with(&map(), &extras).unwrap();
    let raw = h3_map::parse(&bytes, &Limits::default()).unwrap();
    let art = MapArt::from_raw(&raw).unwrap();
    let mut live = Live::at_start(&raw, &art, 0, &tables().map.towns);
    // The map's heroes are the heroes' tests' (below), which give their own.
    live.heroes.clear();
    (art, live)
}

/// A one-cell shape with both masks and one frame for every template of `art` and every town
/// picture of the tables (a town is drawn by its buildings), one frame for every DEF of the
/// tables' heroes.
fn shapes_of(art: &MapArt) -> Shapes {
    let mut shapes = Shapes::default();
    let one = Shape {
        width: 1,
        height: 1,
        colour: [0xff; 6],
        shadow: [0xff; 6],
        frames: 1,
    };
    for t in &art.templates {
        shapes.insert(&t.def, one);
    }
    let tables = tables();
    for (def, _) in tables.map.towns.defs() {
        shapes.insert(&def, one);
    }
    for (def, _) in tables.map.heroes.defs() {
        shapes.insert_frames(&def, 1);
    }
    shapes
}

fn at(x: i32, y: i32) -> ViewAt {
    ViewAt { x, y, z: 0 }
}

fn still(art: &MapArt, live: &Live, view: ViewAt, bugs: Bugs) -> Scene {
    scene(art, live, &tables(), &layout(), &shapes_of(art), view, bugs).unwrap()
}

/// A piece as (def, frame, its place, its mirrors, its clip).
type Seen = (String, u32, [i32; 2], [bool; 2], [i32; 4]);

/// The pieces of one pass.
fn pieces(scene: &Scene, of: Pass) -> Vec<Seen> {
    scene
        .draws
        .iter()
        .filter_map(|d| match d {
            Draw::Piece {
                pass,
                def,
                group: _,
                frame,
                x,
                y,
                mirror_x,
                mirror_y,
                clip,
                flag: _,
            } if *pass == of => {
                Some((def.clone(), *frame, [*x, *y], [*mirror_x, *mirror_y], *clip))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn the_frame_then_the_passes_then_the_gems() {
    let (art, live) = art_with(vec![]);
    let scene = still(&art, &live, at(0, 0), Bugs::Fixed);
    assert_eq!(
        scene.draws[0],
        Draw::Frame {
            picture: "AdvMap.pcx".to_string(),
            players: "PLAYERS.PAL".to_string(),
            viewer: 0
        }
    );
    // Columns 0 to 18 and rows 0 to 16 show (design §1.2): 19 × 17 terrains, no edge.
    let terrain = pieces(&scene, Pass::Terrain);
    assert_eq!(terrain.len(), 19 * 17);
    assert!(pieces(&scene, Pass::Edge).is_empty());
    // Rows outer, columns inner: the first two are (0, 0) and (1, 0), clipped to the view.
    assert_eq!(terrain[0].0, "watrtl.def");
    assert_eq!((terrain[0].2, terrain[0].4), ([0, 8], [8, 8, 32, 40]));
    assert_eq!((terrain[1].2, terrain[1].4), ([32, 8], [32, 8, 64, 40]));
    // Column 18 shows 24 pixels; the last is (18, 16).
    let last = terrain.last().unwrap();
    assert_eq!(
        (last.0.as_str(), last.2, last.4),
        ("grastl.def", [576, 520], [576, 520, 600, 552])
    );
    // Every tile explored at the start: no shroud. The gems last, in the viewer's colour.
    assert!(pieces(&scene, Pass::Shroud).is_empty());
    let gems: Vec<&Draw> = scene.draws.iter().rev().take(4).collect();
    assert_eq!(
        gems[3],
        &Draw::Gem {
            def: "agemul.def".to_string(),
            frame: 0,
            x: 6,
            y: 6
        }
    );
    assert_eq!(
        gems[0],
        &Draw::Gem {
            def: "agemlr.def".to_string(),
            frame: 0,
            x: 556,
            y: 508
        }
    );
    // The order of the passes.
    let order: Vec<Option<Pass>> = scene
        .draws
        .iter()
        .map(|d| match d {
            Draw::Piece { pass, .. } => Some(*pass),
            _ => None,
        })
        .collect();
    let mut runs = order.clone();
    runs.dedup();
    assert_eq!(runs, [None, Some(Pass::Terrain), Some(Pass::Road), None]);
}

#[test]
fn the_terrains_pictures_and_mirrors() {
    let given = GivenTile {
        level: 0,
        tile: (2, 3),
        terrain: Terrain::Lava,
        picture: 41,
        river: None,
        road: None,
        flags: 0x03,
    };
    let (art, live) = art_with(vec![given]);
    let scene = still(&art, &live, at(0, 0), Bugs::Fixed);
    let lava = pieces(&scene, Pass::Terrain)
        .into_iter()
        .find(|p| p.2 == [64, 8 + 96])
        .unwrap();
    assert_eq!(
        (lava.0.as_str(), lava.1, lava.3),
        ("lavatl.def", 41, [true, true])
    );
}

#[test]
fn the_edge_around_the_map() {
    let (art, live) = art_with(vec![]);
    // The view's tile (0, 0) at (−2, −1): the edge's row −1 at view row 0, columns −2 and −1.
    let scene = still(&art, &live, at(-2, -1), Bugs::Fixed);
    let edge = pieces(&scene, Pass::Edge);
    // Row −1 (19 columns) and column −2, −1 of rows 0 to 15.
    assert_eq!(edge.len(), 19 + 2 * 16);
    let frames: Vec<u32> = edge.iter().take(4).map(|p| p.1).collect();
    // (−2, −1) further out, (−1, −1) the corner, then (0, −1), (1, −1) the top edge.
    assert_eq!(frames, [edge_frame(-2, -1, 36), 16, 20, 21]);
    // The bottom-right corner seen from (20, 20).
    let scene = still(&art, &live, at(20, 20), Bugs::Fixed);
    let corner = pieces(&scene, Pass::Edge)
        .into_iter()
        .find(|p| p.2 == [32 * 16, 8 + 32 * 16])
        .unwrap();
    assert_eq!((corner.0.as_str(), corner.1), ("edg.def", 18));
}

#[test]
fn the_edges_frames() {
    let side = 36;
    for (x, y, frame) in [
        (-1, -1, 16),
        (36, -1, 17),
        (36, 36, 18),
        (-1, 36, 19),
        (0, -1, 20),
        (7, -1, 23),
        (36, 1, 25),
        (2, 36, 30),
        (-1, 35, 35),
        // Further out: ((x + 16) mod 4) + 4 ((y + 16) mod 4).
        (-2, -2, 2 + 4 * 2),
        (-3, 5, 1 + 4),
        (38, -4, 2),
        (40, 40, 0),
        (-1, -5, 3 + 4 * 3),
    ] {
        assert_eq!(edge_frame(x, y, side), frame, "({x}, {y})");
    }
}

#[test]
fn rivers_and_roads_and_the_roads_offset() {
    let river = GivenTile {
        level: 0,
        tile: (8, 4),
        terrain: Terrain::Grass,
        picture: 20,
        river: Some((RiverKind::Muddy, 7)),
        road: Some((RoadKind::Cobblestone, 2)),
        flags: 0x0C | 0x10,
    };
    let (art, live) = art_with(vec![river]);
    let scene = still(&art, &live, at(0, 0), Bugs::Fixed);
    let rivers = pieces(&scene, Pass::River);
    assert_eq!(
        rivers,
        [(
            "mudrvr.def".to_string(),
            7,
            [256, 136],
            [true, true],
            [256, 136, 288, 168]
        )]
    );
    // The roads: the writer's on row 5 (columns 3 to 6), the given one on (8, 4); 16 pixels down.
    let roads = pieces(&scene, Pass::Road);
    assert_eq!(roads.len(), 5);
    assert_eq!(
        roads[0],
        (
            "cobbrd.def".to_string(),
            2,
            [256, 152],
            [true, false],
            [256, 152, 288, 184]
        )
    );
    assert_eq!(roads[1].2, [96, 8 + 160 + 16]);
    // The rivers' pass comes before the roads'.
    let first = |of: Pass| {
        scene
            .draws
            .iter()
            .position(|d| matches!(d, Draw::Piece { pass, .. } if *pass == of))
            .unwrap()
    };
    assert!(first(Pass::Terrain) < first(Pass::River) && first(Pass::River) < first(Pass::Road));
}

#[test]
fn a_road_above_the_view_shows_its_lower_half() {
    let (art, live) = art_with(vec![]);
    // The road's row 5 at view row −1: its picture from y −8, clipped to 8 … 23.
    let scene = still(&art, &live, at(0, 6), Bugs::Fixed);
    let roads = pieces(&scene, Pass::Road);
    assert_eq!(roads.len(), 4);
    assert_eq!((roads[0].2, roads[0].4), ([96, -8], [96, 8, 128, 24]));
}

#[test]
fn a_road_on_the_maps_last_row() {
    let road = |x| GivenTile {
        level: 0,
        tile: (x, 35),
        terrain: Terrain::Grass,
        picture: 20,
        river: Some((RiverKind::Clear, 1)),
        road: Some((RoadKind::Dirt, 1)),
        flags: 0,
    };
    let (art, live) = art_with(vec![road(4)]);
    let clip = |view: ViewAt, bugs| {
        let roads = pieces(&still(&art, &live, view, bugs), Pass::Road);
        roads.into_iter().map(|p| p.4).collect::<Vec<_>>()
    };
    // Row 35 at the view's row 15: only the tile's own lower half, whatever the bugs.
    let row = 8 + 32 * 15;
    for bugs in [Bugs::Fixed, Bugs::Original] {
        assert_eq!(clip(at(0, 20), bugs), [[128, row + 16, 160, row + 32]]);
    }
    // At the view's row 16 the view's clip already cut it to 16 rows: the original then cuts 16
    // more and draws nothing (MCL-Q2).
    let row = 8 + 32 * 16;
    assert_eq!(
        clip(at(0, 19), Bugs::Fixed),
        [[128, row + 16, 160, row + 32]]
    );
    assert!(clip(at(0, 19), Bugs::Original).is_empty());
    // The river on the same tile is never cut.
    let scene = still(&art, &live, at(0, 19), Bugs::Original);
    assert_eq!(pieces(&scene, Pass::River)[0].4, [128, row, 160, row + 32]);
}

#[test]
fn the_unexplored_tiles_pieces() {
    let (art, mut live) = art_with(vec![]);
    // Hide the block (4, 4)–(8, 8) but its centre (6, 6).
    for y in 4..=8 {
        for x in 4..=8 {
            live.explored.set(x, y, 0, (x, y) == (6, 6));
        }
    }
    let scene = still(&art, &live, at(0, 0), Bugs::Fixed);
    let shroud = pieces(&scene, Pass::Shroud);
    assert_eq!(shroud.len(), 24);
    let piece = |x: i32, y: i32| {
        shroud
            .iter()
            .find(|p| p.2 == [32 * x, 8 + 32 * y])
            .map(|p| (p.0.as_str(), p.1, p.3[0]))
            .unwrap()
    };
    let tables = tables();
    let expect = |mask: u8, x: u8, y: u8| match tables.map.shroud.piece(mask, x, y) {
        ShroudPiece::Full(f) => ("tshrc.def", u32::from(f), false),
        ShroudPiece::Edge { frame, mirror } => ("tshre.def", u32::from(frame), mirror),
        ShroudPiece::None => ("", 0, false),
    };
    // (5, 5): every neighbour hidden but the explored centre to its south-east (0x20).
    assert_eq!(piece(5, 5), expect(!0x20, 5, 5));
    // (4, 4): its north, west and the three corners off the block are explored.
    assert_eq!(piece(4, 4), expect(0x02 | 0x04 | 0x20, 4, 4));
    // (6, 4): explored to the north, north-east and north-west.
    assert_eq!(piece(6, 4), expect(0x02 | 0x04 | 0x08 | 0x20 | 0x40, 6, 4));
    // A tile with no explored neighbour gets the full piece, its frame by x and y.
    let mut hidden = live.clone();
    hidden.explored = crate::live::Explored::all(36, 1, false);
    let scene = still(&art, &hidden, at(0, 0), Bugs::Fixed);
    let shroud = pieces(&scene, Pass::Shroud);
    assert_eq!(shroud.len(), 19 * 17);
    let (_, frame, ..) = &shroud[19 * 3 + 2];
    assert_eq!(shroud[19 * 3 + 2].2, [64, 8 + 96]);
    assert_eq!(
        (shroud[0].0.as_str(), *frame),
        ("tshrc.def", expect(0xFF, 2, 3).1)
    );
    // A mask whose entry is 0 draws the full piece, as the mask 255's (0x41237F).
    let mut none = tables.clone();
    none.map.shroud.pieces[0xDF] = 0;
    let drawn = super::scene(
        &art,
        &live,
        &none,
        &layout(),
        &shapes_of(&art),
        at(0, 0),
        Bugs::Fixed,
    )
    .unwrap();
    let drawn = pieces(&drawn, Pass::Shroud);
    assert_eq!(drawn.len(), 24);
    assert_eq!(drawn.iter().filter(|p| p.0 == "tshrc.def").count(), 1);
    // An entry that gives no frame (100) draws nothing.
    none.map.shroud.pieces[0xDF] = 100;
    let drawn = super::scene(
        &art,
        &live,
        &none,
        &layout(),
        &shapes_of(&art),
        at(0, 0),
        Bugs::Fixed,
    )
    .unwrap();
    assert_eq!(pieces(&drawn, Pass::Shroud).len(), 23);
}

#[test]
fn the_shroud_is_drawn_over_the_roads() {
    let (art, mut live) = art_with(vec![]);
    live.explored.set(3, 5, 0, false);
    let scene = still(&art, &live, at(0, 0), Bugs::Fixed);
    let road = scene
        .draws
        .iter()
        .position(|d| {
            matches!(
                d,
                Draw::Piece {
                    pass: Pass::Road,
                    ..
                }
            )
        })
        .unwrap();
    let shroud = scene
        .draws
        .iter()
        .position(|d| {
            matches!(
                d,
                Draw::Piece {
                    pass: Pass::Shroud,
                    ..
                }
            )
        })
        .unwrap();
    assert!(road < shroud);
    // A tile off the map is never shrouded: the edge shows.
    let mut none = live.clone();
    none.explored = crate::live::Explored::all(36, 1, false);
    let scene = still(&art, &none, at(-3, -3), Bugs::Fixed);
    assert_eq!(pieces(&scene, Pass::Shroud).len(), 16 * 14);
}

#[test]
fn the_cycles_at_the_moment_and_the_viewers_colour() {
    let (art, mut live) = art_with(vec![]);
    live.moment = Moment {
        cycles: 7,
        river_cycles: 6,
        frames: 3,
    };
    live.viewer = 5;
    let scene = still(&art, &live, at(0, 0), Bugs::Fixed);
    let cycling: Vec<(&str, u32)> = scene
        .cycles
        .iter()
        .map(|c| (c.def.as_str(), c.steps))
        .collect();
    assert_eq!(
        cycling,
        [
            ("lavatl.def", 7),
            ("watrtl.def", 7),
            // The rivers' own step (a tick between the loads in the original, design §1.6).
            ("clrrvr.def", 6),
            ("mudrvr.def", 6),
            ("lavrvr.def", 6)
        ]
    );
    assert_eq!(scene.cycles[1].ranges, [[229, 240], [242, 253]]);
    assert!(matches!(scene.draws[0], Draw::Frame { viewer: 5, .. }));
    assert!(matches!(
        scene.draws.last(),
        Some(Draw::Gem { frame: 5, .. })
    ));
}

#[test]
fn refuses_a_level_a_viewer_or_a_code_it_cannot_draw() {
    let (art, mut live) = art_with(vec![]);
    let (t, l) = (tables(), layout());
    let view = ViewAt { x: 0, y: 0, z: 1 };
    assert_eq!(
        scene(&art, &live, &t, &l, &shapes_of(&art), view, Bugs::Fixed),
        Err(SceneError::Level(1))
    );
    let mut lacking = t.clone();
    lacking.roads.remove(&1);
    assert_eq!(
        scene(
            &art,
            &live,
            &lacking,
            &l,
            &shapes_of(&art),
            at(0, 0),
            Bugs::Fixed
        ),
        Err(SceneError::Code {
            layer: Layer::Road,
            code: 1,
            at: [3, 5]
        })
    );
    // Off the view (its rows −1 to 17 are the map's 6 to 24), the code is not needed.
    assert!(
        scene(
            &art,
            &live,
            &lacking,
            &l,
            &shapes_of(&art),
            at(0, 7),
            Bugs::Fixed
        )
        .is_ok()
    );
    let mut lacking = t.clone();
    lacking.terrains.remove(&8);
    assert!(matches!(
        scene(
            &art,
            &live,
            &lacking,
            &l,
            &shapes_of(&art),
            at(0, 0),
            Bugs::Fixed
        ),
        Err(SceneError::Code {
            layer: Layer::Terrain,
            code: 8,
            at: [0, 0]
        })
    ));
    live.viewer = 8;
    assert_eq!(
        scene(&art, &live, &t, &l, &shapes_of(&art), at(0, 0), Bugs::Fixed),
        Err(SceneError::Viewer(8))
    );
    let messages = [
        SceneError::Level(1).to_string(),
        SceneError::Viewer(8).to_string(),
        SceneError::Code {
            layer: Layer::River,
            code: 9,
            at: [1, 2],
        }
        .to_string(),
    ];
    assert_eq!(
        messages,
        [
            "level 1 is not on the map",
            "viewer 8 is not a colour (0-7)",
            "tile (1, 2): River code 9 is not in the tables"
        ]
    );
    // The raw map's own art reads too (no given tiles).
    assert!(MapArt::from_raw(&raw(&map())).is_ok());
}

/// The art test's map with `objects` after its own (a hero, a flat ground at (20, 20), a town).
fn art_with_objects(objects: Vec<AnyObject>) -> (MapArt, Live) {
    let extras = Extras {
        tiles: vec![],
        objects,
    };
    let bytes = writer::write_with(&map(), &extras).unwrap();
    let raw = h3_map::parse(&bytes, &Limits::default()).unwrap();
    let art = MapArt::from_raw(&raw).unwrap();
    let mut live = Live::at_start(&raw, &art, 0, &tables().map.towns);
    // The map's heroes are the heroes' tests' (below), which give their own.
    live.heroes.clear();
    (art, live)
}

/// An object of the art test's template (`def`, one blocked cell at the anchor) at (x, y).
fn object(def: &str, x: u8, y: u8, priority: u8) -> AnyObject {
    AnyObject {
        level: 0,
        tile: (x, y),
        template: template(def, 155, 0, priority),
        owner: None,
    }
}

/// `shapes_of(art)` with `def` a 2 × 2 shape of `frames`, its masks as stored.
fn with_shape(art: &MapArt, def: &str, colour: [u8; 6], shadow: [u8; 6], frames: u32) -> Shapes {
    let mut shapes = shapes_of(art);
    let shape = Shape {
        width: 2,
        height: 2,
        colour,
        shadow,
        frames,
    };
    shapes.insert(def, shape);
    shapes
}

/// The first and last index of `of`'s draws.
fn span(scene: &Scene, of: Pass) -> (usize, usize) {
    let at: Vec<usize> = scene
        .draws
        .iter()
        .enumerate()
        .filter(|(_, d)| matches!(d, Draw::Piece { pass, .. } if *pass == of))
        .map(|(i, _)| i)
        .collect();
    (at[0], at[at.len() - 1])
}

#[test]
fn an_objects_shadow_and_body_pieces_in_their_masks_cells() {
    // A 2 × 2 tree anchored at (5, 5): its body in cells (0, 0), (1, 0), (0, 1), its shadow in
    // (1, 0) and (1, 1) (bit 7 − dx of byte 5 − dy). Phase 3, frame step 2, four frames: frame 1.
    let (art, mut live) = art_with_objects(vec![object("tree.def", 5, 5, 0)]);
    let tree = art.objects.len() - 1;
    live.objects[tree].phase = 3;
    live.moment.frames = 2;
    let colour = [0, 0, 0, 0, 0x80, 0xc0];
    let shadow = [0, 0, 0, 0, 0x40, 0x40];
    let shapes = with_shape(&art, "tree.def", colour, shadow, 4);
    let s = scene(
        &art,
        &live,
        &tables(),
        &layout(),
        &shapes,
        at(0, 0),
        Bugs::Fixed,
    )
    .unwrap();
    // The frame's canvas (64 × 64) has its top-left on tile (4, 4): (128, 136).
    let tile = |x: i32, y: i32| [32 * x, 8 + 32 * y, 32 * x + 32, 40 + 32 * y];
    let piece = |x, y| {
        (
            "tree.def".to_string(),
            1,
            [128, 136],
            [false, false],
            tile(x, y),
        )
    };
    assert_eq!(pieces(&s, Pass::Shadow), [piece(4, 4), piece(4, 5)]);
    assert_eq!(
        pieces(&s, Pass::Body),
        [piece(5, 4), piece(4, 5), piece(5, 5)]
    );
    assert!(pieces(&s, Pass::Flat).is_empty());
    // The passes' order: roads, flat objects, shadows, bodies, the unexplored tiles.
    let (_, roads) = span(&s, Pass::Road);
    let (shadows, last_shadow) = span(&s, Pass::Shadow);
    let (bodies, _) = span(&s, Pass::Body);
    assert!(roads < shadows && last_shadow < bodies);
}

/// The flag entries of one pass's pieces of `def`, in order.
fn flags(scene: &Scene, of: Pass, of_def: &str) -> Vec<Option<u8>> {
    scene
        .draws
        .iter()
        .filter_map(|d| match d {
            Draw::Piece {
                pass, def, flag, ..
            } if *pass == of && def == of_def => Some(*flag),
            _ => None,
        })
        .collect()
}

/// An object of class `class` owned by `owner` (`def`, the art test's template) at (x, y).
fn owned(def: &str, class: u32, x: u8, y: u8, owner: Option<u8>) -> AnyObject {
    AnyObject {
        owner,
        template: template(def, class, 0, 0),
        ..object(def, x, y, 0)
    }
}

#[test]
fn an_owned_objects_flag_pixels_take_its_owners_entry() {
    // A mine of Blue's, made Tan's (the map's players are Red and Blue), a neutral dwelling, a
    // tree: the first two of the owned classes.
    let (art, mut live) = art_with_objects(vec![
        owned("mine.def", 53, 5, 5, Some(1)),
        owned("dwelling.def", 17, 8, 5, None),
        object("tree.def", 11, 5, 0),
    ]);
    let mine = art.objects.len() - 3;
    assert_eq!(live.objects[mine].owner, Some(1));
    live.objects[mine].owner = Some(2);
    let s = still(&art, &live, at(0, 0), Bugs::Fixed);
    // game.pal's entry 64 + the colour, 63 without an owner; a tree's flag runs are skipped.
    assert_eq!(flags(&s, Pass::Body, "mine.def"), [Some(66)]);
    assert_eq!(flags(&s, Pass::Body, "dwelling.def"), [Some(63)]);
    assert_eq!(flags(&s, Pass::Body, "tree.def"), [None]);
    // Shadows never take one.
    assert_eq!(flags(&s, Pass::Shadow, "mine.def"), [None]);
    assert_eq!(s.flags, "game.pal");
    // A flat object of an owned class takes it in pass 4.
    let (art, live) = art_with_objects(vec![AnyObject {
        template: template("ground.def", 53, 0, 1),
        ..owned("ground.def", 53, 5, 5, Some(0))
    }]);
    let s = still(&art, &live, at(0, 0), Bugs::Fixed);
    assert_eq!(flags(&s, Pass::Flat, "ground.def"), [Some(64)]);
}

#[test]
fn an_object_without_an_entrance_on_the_map_takes_tile_zeros_owner_with_the_bugs() {
    // Green's mine anchored two columns past the right edge, its entrance (one left of the
    // anchor) off the map, its body's third column on it (x 35); Purple's mine at (1, 0), whose
    // entrance is tile (0, 0); Red's mine below it.
    let (art, mut live) = art_with_objects(vec![
        owned("far.def", 53, 37, 5, Some(1)),
        owned("corner.def", 53, 1, 0, Some(1)),
        owned("mine.def", 53, 5, 8, Some(0)),
    ]);
    let far = art.objects.len() - 3;
    live.objects[far].owner = Some(3);
    live.objects[far + 1].owner = Some(5);
    assert_eq!(objects::entrance(&art, far), None);
    assert_eq!(objects::entrance(&art, far + 1), Some((0, 0)));
    assert_eq!(objects::entrance(&art, far + 2), Some((4, 8)));
    let mut shapes = shapes_of(&art);
    let wide = Shape {
        width: 3,
        height: 1,
        colour: [0, 0, 0, 0, 0, 0x20],
        shadow: [0; 6],
        frames: 1,
    };
    shapes.insert("far.def", wide);
    let view = |bugs| scene(&art, &live, &tables(), &layout(), &shapes, at(20, 0), bugs).unwrap();
    assert_eq!(flags(&view(Bugs::Fixed), Pass::Body, "far.def"), [Some(67)]);
    assert_eq!(
        flags(&view(Bugs::Original), Pass::Body, "far.def"),
        [Some(69)]
    );
    assert_eq!(
        objects::flag_owner(&art, &live, far + 2, Bugs::Original),
        Some(0)
    );
    // Without an object whose entrance is tile (0, 0), none.
    let (art, mut live) = art_with_objects(vec![owned("far.def", 53, 37, 5, None)]);
    let far = art.objects.len() - 1;
    live.objects[far].owner = Some(3);
    assert_eq!(objects::flag_owner(&art, &live, far, Bugs::Original), None);
    assert_eq!(objects::flag_owner(&art, &live, far, Bugs::Fixed), Some(3));
    // An object not in the map has no class, entrance or owner.
    assert_eq!(objects::class(&art, &live, art.objects.len()), None);
    assert_eq!(objects::entrance(&art, art.objects.len()), None);
}

#[test]
fn a_flat_object_in_every_cell_without_masks() {
    // A flat 2 × 2 with empty masks at (6, 3), under the tree at (5, 5) placed after it.
    let (art, mut live) = art_with_objects(vec![
        object("plain.def", 6, 3, 1),
        object("tree.def", 5, 5, 0),
    ]);
    live.explored.set(0, 0, 0, false);
    let mut shapes = with_shape(&art, "plain.def", [0; 6], [0; 6], 1);
    shapes.insert(
        "tree.def",
        *with_shape(&art, "tree.def", [0xff; 6], [0xff; 6], 1)
            .get("tree.def")
            .unwrap(),
    );
    let s = scene(
        &art,
        &live,
        &tables(),
        &layout(),
        &shapes,
        at(0, 0),
        Bugs::Fixed,
    )
    .unwrap();
    let flat = pieces(&s, Pass::Flat);
    assert_eq!(flat.len(), 4);
    assert!(flat.iter().all(|p| p.0 == "plain.def" && p.2 == [160, 72]));
    assert!(pieces(&s, Pass::Shadow).iter().all(|p| p.0 == "tree.def"));
    assert!(pieces(&s, Pass::Body).iter().all(|p| p.0 == "tree.def"));
    let (_, roads) = span(&s, Pass::Road);
    let (flats, last_flat) = span(&s, Pass::Flat);
    let (shadows, _) = span(&s, Pass::Shadow);
    let (_, last_body) = span(&s, Pass::Body);
    let (shroud, _) = span(&s, Pass::Shroud);
    assert!(roads < flats && last_flat < shadows && last_body < shroud);
}

#[test]
fn an_object_off_the_view_and_its_level_are_left() {
    // The tree at (25, 5) lies right of the view at (0, 0); on the underground nothing is drawn.
    let (art, live) = art_with_objects(vec![object("tree.def", 25, 5, 0)]);
    let shapes = with_shape(&art, "tree.def", [0xff; 6], [0xff; 6], 1);
    let s = scene(
        &art,
        &live,
        &tables(),
        &layout(),
        &shapes,
        at(0, 0),
        Bugs::Fixed,
    )
    .unwrap();
    assert!(pieces(&s, Pass::Body).is_empty());
    let s = scene(
        &art,
        &live,
        &tables(),
        &layout(),
        &shapes,
        at(7, 0),
        Bugs::Fixed,
    )
    .unwrap();
    // Columns 24 and 25 at the view's columns 17 and 18 (24 pixels shown), rows 4 and 5.
    let bodies = pieces(&s, Pass::Body);
    assert_eq!(bodies.len(), 4);
    assert_eq!(bodies[1].4, [576, 136, 600, 168]);
}

#[test]
fn an_object_without_a_shape_has_no_scene() {
    let (art, live) = art_with_objects(vec![object("tree.def", 5, 5, 0)]);
    let err = scene(
        &art,
        &live,
        &tables(),
        &layout(),
        &shapes_of(&art),
        at(0, 0),
        Bugs::Fixed,
    );
    assert!(err.is_ok());
    let mut lacking = Shapes::default();
    lacking.insert("tree.def", *shapes_of(&art).get("tree.def").unwrap());
    let err = scene(
        &art,
        &live,
        &tables(),
        &layout(),
        &lacking,
        at(0, 0),
        Bugs::Fixed,
    )
    .unwrap_err();
    assert!(matches!(
        err,
        SceneError::Objects(ObjectsError::Shape(1, _))
    ));
    assert_eq!(err.to_string(), "object 1: no .msk for cursed.def");
    // The picture a random object was drawn as needs its shape too (E24 M1 T6b, R1-4).
    let mut random = live.clone();
    let tree = art.objects.len() - 1;
    random.objects[tree].drawn = Some(crate::live::Drawn {
        def: "gold.def".to_string(),
        own: None,
        class: 79,
        subclass: 6,
    });
    let err = scene(
        &art,
        &random,
        &tables(),
        &layout(),
        &shapes_of(&art),
        at(0, 0),
        Bugs::Fixed,
    )
    .unwrap_err();
    assert_eq!(
        err.to_string(),
        format!("object {tree}: no .msk for gold.def")
    );
    // A hero's DEFs need their frames while a hero stands on the level.
    let mut hero = live;
    hero.heroes = vec![land_hero(9, 9, 2, 2, Facing::Right)];
    let err = scene(
        &art,
        &hero,
        &tables(),
        &layout(),
        &lacking,
        at(0, 0),
        Bugs::Fixed,
    );
    assert!(matches!(err, Err(SceneError::Objects(_))));
    let mut frames = shapes_of(&art);
    frames.insert("cursed.def", *shapes_of(&art).get("tree.def").unwrap());
    let all = scene(
        &art,
        &hero,
        &tables(),
        &layout(),
        &frames,
        at(0, 0),
        Bugs::Fixed,
    );
    assert!(all.is_ok());
    let mut no_frames = Shapes::default();
    let one = *shapes_of(&art).get("tree.def").unwrap();
    let towns = tables().map.towns.defs();
    for def in art
        .templates
        .iter()
        .map(|t| &t.def)
        .chain(towns.iter().map(|(d, _)| d))
    {
        no_frames.insert(def, one);
    }
    let first = tables().map.heroes.defs()[0].0.clone();
    let err = scene(
        &art,
        &hero,
        &tables(),
        &layout(),
        &no_frames,
        at(0, 0),
        Bugs::Fixed,
    )
    .unwrap_err();
    assert_eq!(err.to_string(), format!("{first}: no frames"));
}

// -- the heroes and boats (E24 M1 T5a; design §2.3, §2.8) -------------------------------------

/// A hero or boat piece: its DEF, group, frame, canvas, mirror and clip.
type HeroSeen = (String, u32, u32, [i32; 2], bool, [i32; 4]);

/// The pieces of one pass, with their groups.
fn grouped(scene: &Scene, of: Pass) -> Vec<HeroSeen> {
    scene
        .draws
        .iter()
        .filter_map(|d| match d {
            Draw::Piece {
                pass,
                def,
                group,
                frame,
                x,
                y,
                mirror_x,
                clip,
                ..
            } if *pass == of => Some((def.clone(), *group, *frame, [*x, *y], *mirror_x, *clip)),
            _ => None,
        })
        .collect()
}

/// The DEFs one pass draws on the view tile (x, y), in order (the view at (0, 0)).
fn on_tile(scene: &Scene, of: Pass, x: i32, y: i32) -> Vec<String> {
    let clip = [32 * x, 8 + 32 * y, 32 * x + 32, 40 + 32 * y];
    grouped(scene, of)
        .into_iter()
        .filter(|p| p.5 == clip)
        .map(|p| p.0)
        .collect()
}

fn pos(x: u8, y: u8) -> Pos {
    Pos { x, y, z: 0 }
}

fn land_hero(x: u8, y: u8, class: u8, owner: u8, facing: Facing) -> LiveHero {
    LiveHero {
        at: pos(x, y),
        class,
        owner,
        facing,
        boat: None,
    }
}

/// `shapes_of(art)` with group 0's frames of the hero and boat pictures the tests draw.
fn hero_shapes(art: &MapArt, frames: &[(&str, u32)]) -> Shapes {
    let mut shapes = shapes_of(art);
    for &(def, n) in frames {
        shapes.insert_frames(def, n);
    }
    shapes
}

fn drawn(art: &MapArt, live: &Live, shapes: &Shapes, view: ViewAt) -> Scene {
    scene(art, live, &tables(), &layout(), shapes, view, Bugs::Fixed).unwrap()
}

#[test]
fn a_hero_on_land_is_two_rows_of_three_pieces_its_flag_under_it() {
    // At (5, 5) facing down-left: group 3 mirrored; frame step 11, the flag's 8 frames: 3.
    let (art, mut live) = art_with(vec![]);
    live.heroes = vec![land_hero(5, 5, 3, 2, Facing::DownLeft)];
    live.moment.frames = 11;
    let shapes = hero_shapes(&art, &[("af02.def", 8), ("ah03_.def", 1)]);
    let s = drawn(&art, &live, &shapes, at(0, 0));
    // The 96 × 64 frame's canvas on tile (4, 4), a piece on each of the six tiles it covers.
    let tile = |x: i32, y: i32| [32 * x, 8 + 32 * y, 32 * x + 32, 40 + 32 * y];
    let tiles = [(4, 4), (5, 4), (6, 4), (4, 5), (5, 5), (6, 5)];
    let expected: Vec<HeroSeen> = tiles
        .iter()
        .flat_map(|&(x, y)| {
            [("af02.def", 3), ("ah03_.def", 0)]
                .map(|(def, frame)| (def.to_string(), 3, frame, [128, 136], true, tile(x, y)))
        })
        .collect();
    assert_eq!(grouped(&s, Pass::Shadow), expected);
    assert_eq!(grouped(&s, Pass::Body), expected);
    // Every facing's group and mirror.
    for (facing, group, mirrored) in [
        (Facing::Up, 0, false),
        (Facing::UpRight, 1, false),
        (Facing::Right, 2, false),
        (Facing::DownRight, 3, false),
        (Facing::Down, 4, false),
        (Facing::DownLeft, 3, true),
        (Facing::Left, 2, true),
        (Facing::UpLeft, 1, true),
    ] {
        live.heroes[0].facing = facing;
        let first = grouped(&drawn(&art, &live, &shapes, at(0, 0)), Pass::Body)[0].clone();
        assert_eq!((first.1, first.4), (group, mirrored), "{facing:?}");
    }
}

#[test]
fn heroes_lie_between_the_layers_1_and_2() {
    // The 2 × 2 tree anchored at (6, 6), blocked at its anchor: on row 6 its cells' layer is 1,
    // on row 5 2 (design §2.1). A hero's row is drawn after layer 1, the row above after layer 2.
    let (art, mut live) = art_with_objects(vec![object("tree.def", 6, 6, 0)]);
    let shapes = with_shape(&art, "tree.def", [0xff; 6], [0xff; 6], 1);
    let order =
        |live: &Live, x, y| on_tile(&drawn(&art, live, &shapes, at(0, 0)), Pass::Body, x, y);
    let (flag, hero, tree) = ("af00.def", "ah00_.def", "tree.def");
    // On the tree's passable cell of layer 2 the tree covers the hero.
    live.heroes = vec![land_hero(6, 5, 0, 0, Facing::Right)];
    assert_eq!(order(&live, 6, 5), [flag, hero, tree]);
    // On its cell of layer 1 the hero covers it, and on the row above it too (layer 2).
    live.heroes = vec![land_hero(5, 6, 0, 0, Facing::Right)];
    assert_eq!(order(&live, 5, 6), [tree, flag, hero]);
    assert_eq!(order(&live, 5, 5), [tree, flag, hero]);
    // The shadows: the objects' first, then the heroes'.
    let shadows = on_tile(&drawn(&art, &live, &shapes, at(0, 0)), Pass::Shadow, 5, 5);
    assert_eq!(shadows, [tree, flag, hero]);
}

#[test]
fn heroes_right_over_left_and_lower_over_upper() {
    // Two heroes side by side and one below: on a tile all three cover, the slots' order.
    let (art, mut live) = art_with(vec![]);
    live.heroes = vec![
        land_hero(6, 5, 1, 1, Facing::Right),
        land_hero(5, 5, 0, 0, Facing::Right),
        land_hero(5, 6, 2, 2, Facing::Right),
    ];
    let s = drawn(&art, &live, &hero_shapes(&art, &[]), at(0, 0));
    // Tile (5, 5): the left hero (k 1), the right one (k 2), then the lower one (k 4).
    assert_eq!(
        on_tile(&s, Pass::Body, 5, 5),
        [
            "af00.def",
            "ah00_.def",
            "af01.def",
            "ah01_.def",
            "af02.def",
            "ah02_.def"
        ]
    );
    // One frame each: frame 0.
    assert!(grouped(&s, Pass::Body).iter().all(|p| p.2 == 0));
}

#[test]
fn boats_their_waves_but_on_the_coast_and_a_heros_boat_flag() {
    // Water of an open picture (21) and of shore pictures (3, 0): the coast.
    let water = |x: u8, picture: u8| GivenTile {
        level: 0,
        tile: (x, 8),
        terrain: Terrain::Water,
        picture,
        river: None,
        road: None,
        flags: 0,
    };
    let (art, mut live) = art_with(vec![water(4, 21), water(8, 3), water(12, 25), water(16, 0)]);
    let sail = |x: u8, kind: u8, colour: u8| LiveHero {
        boat: Some(Boat { kind, colour }),
        ..land_hero(x, 8, 4, 0, Facing::Up)
    };
    live.heroes = vec![sail(4, 1, 3), sail(16, 2, 8)];
    live.boats = vec![
        LiveBoat {
            at: pos(8, 8),
            kind: 2,
            facing: Facing::Left,
        },
        LiveBoat {
            at: pos(12, 8),
            kind: 0,
            facing: Facing::Up,
        },
    ];
    let s = drawn(&art, &live, &hero_shapes(&art, &[]), at(0, 0));
    // A hero in a boat in open water: the waves, the boat's flag (type 1, colour 3), the boat;
    // no hero picture.
    assert_eq!(
        on_tile(&s, Pass::Body, 4, 8),
        ["abm02_.def", "abf02d.def", "ab02_.def"]
    );
    // On the coast without waves, a colour past 7 without a flag.
    assert_eq!(on_tile(&s, Pass::Body, 16, 8), ["ab03_.def"]);
    // An empty boat on the coast: the boat alone, its own facing's group, mirrored.
    let coast = grouped(&s, Pass::Body);
    let boat = coast
        .iter()
        .find(|p| p.0 == "ab03_.def" && p.3 == [224, 232]);
    assert_eq!(boat.map(|p| (p.1, p.4)), Some((2, true)));
    assert_eq!(on_tile(&s, Pass::Body, 8, 8), ["ab03_.def"]);
    // In open water its waves first.
    assert_eq!(on_tile(&s, Pass::Body, 12, 8), ["abm01_.def", "ab01_.def"]);
    // The shadows draw the same.
    assert_eq!(
        on_tile(&s, Pass::Shadow, 4, 8),
        ["abm02_.def", "abf02d.def", "ab02_.def"]
    );
}

#[test]
fn heroes_off_the_map_another_level_or_a_second_on_a_tile_are_left() {
    let (art, mut live) = art_with(vec![]);
    live.heroes = vec![
        // At the map's left edge: its frame's left column lies off the map.
        land_hero(0, 3, 0, 0, Facing::Right),
        // A second on the same tile: not drawn.
        land_hero(0, 3, 1, 1, Facing::Right),
        // A class past the tables' own: its flag alone.
        land_hero(9, 3, 18, 3, Facing::Right),
    ];
    let s = drawn(&art, &live, &hero_shapes(&art, &[]), at(-2, 0));
    let body = grouped(&s, Pass::Body);
    let defs: BTreeSet<&str> = body.iter().map(|p| p.0.as_str()).collect();
    assert_eq!(defs, BTreeSet::from(["af00.def", "ah00_.def", "af03.def"]));
    // The first hero's pieces on the map's tiles 0 and 1 of rows 2 and 3 only (the view's
    // column 2 is the map's 0).
    let first = body.iter().filter(|p| p.0 == "ah00_.def").count();
    assert_eq!(first, 4);
}

#[test]
fn a_live_part_that_does_not_fit_the_map_has_no_scene() {
    // E24 M1 T6b (the reviewer's R1-3): the map is 36 x 36, one level.
    let (art, live) = art_with(vec![]);
    let shapes = shapes_of(&art);
    let refused = |live: &Live| {
        scene(
            &art,
            live,
            &tables(),
            &layout(),
            &shapes,
            at(0, 0),
            Bugs::Fixed,
        )
        .unwrap_err()
        .to_string()
    };
    let mut fewer = live.clone();
    fewer.objects.pop();
    let n = art.objects.len();
    assert_eq!(
        refused(&fewer),
        format!(
            "the live part does not fit the map: {} objects for the map's {n}",
            n - 1
        )
    );
    let mut explored = live.clone();
    explored.explored = crate::live::Explored::all(36, 2, true);
    assert_eq!(
        refused(&explored),
        "the live part does not fit the map: the explored tiles are not of the map's size and levels"
    );
    let mut hero = live.clone();
    hero.heroes = vec![LiveHero {
        at: Pos { x: 9, y: 9, z: 1 },
        ..land_hero(9, 9, 2, 2, Facing::Right)
    }];
    assert_eq!(
        refused(&hero),
        "the live part does not fit the map: a hero at (9, 9, 1) is off the map"
    );
    hero.heroes = vec![land_hero(9, 36, 2, 2, Facing::Right)];
    assert!(refused(&hero).ends_with("a hero at (9, 36, 0) is off the map"));
    hero.heroes = vec![LiveHero {
        owner: 8,
        ..land_hero(9, 9, 2, 2, Facing::Right)
    }];
    assert_eq!(
        refused(&hero),
        "the live part does not fit the map: a hero at (9, 9, 0) has no owner"
    );
    let mut boat = live.clone();
    boat.boats = vec![LiveBoat {
        at: Pos { x: 36, y: 0, z: 0 },
        kind: 0,
        facing: Facing::Right,
    }];
    assert!(refused(&boat).ends_with("a boat at (36, 0, 0) is off the map"));
    let mut path = live;
    path.path = vec![Arrow {
        at: Pos { x: 190, y: 5, z: 0 },
        arrow: 3,
    }];
    assert!(refused(&path).ends_with("an arrow at (190, 5, 0) is off the map"));
}

#[test]
fn the_paths_shadows_under_the_bodies_its_arrows_over_them() {
    // The 2 × 2 tree anchored at (6, 6) covers (5, 5) to (6, 6); a path through (5, 5) and (6, 5)
    // ends at (7, 5) (E24 M1 T5b2, design §2.2: passes 6 and 8).
    let (art, mut live) = art_with_objects(vec![object("tree.def", 6, 6, 0)]);
    let shapes = with_shape(&art, "tree.def", [0xff; 6], [0xff; 6], 1);
    let arrow = |x, y, z, arrow| Arrow {
        at: Pos { x, y, z },
        arrow,
    };
    live.path = vec![
        arrow(5, 5, 0, 3),
        arrow(6, 5, 0, 29),
        arrow(7, 5, 0, 1),
        // Not drawn: an arrow 0 (none), one off the view.
        arrow(8, 5, 0, 0),
        arrow(30, 30, 0, 2),
    ];
    let s = drawn(&art, &live, &shapes, at(0, 0));
    let tile = |x: i32, y: i32| [32 * x, 8 + 32 * y];
    let clip = |x: i32, y: i32| [32 * x, 8 + 32 * y, 32 * x + 32, 40 + 32 * y];
    let expected: Vec<Seen> = [(5, 5, 2), (6, 5, 28), (7, 5, 0)]
        .into_iter()
        .map(|(x, y, frame)| {
            (
                "adag.def".to_string(),
                frame,
                tile(x, y),
                [false; 2],
                clip(x, y),
            )
        })
        .collect();
    assert_eq!(pieces(&s, Pass::PathShadow), expected);
    assert_eq!(pieces(&s, Pass::Path), expected);
    // Each pass over the whole view: every shadow, the arrows' shadows, the bodies, the arrows.
    let order = [Pass::Shadow, Pass::PathShadow, Pass::Body, Pass::Path].map(|p| span(&s, p));
    assert!(order.windows(2).all(|w| w[0].1 < w[1].0), "{order:?}");
    // No path shown: neither pass draws.
    live.path.clear();
    assert!(pieces(&drawn(&art, &live, &shapes, at(0, 0)), Pass::PathShadow).is_empty());
}

#[test]
fn the_underground_is_drawn_as_the_surface() {
    // E24 M1 T6b (the reviewer's R1-9): a two-level map, the view on level 1: its terrain, its
    // explored tiles, a hero and an arrow of that level; the surface's left.
    let mut two = map();
    two.underground = Some(writer::Underground {
        terrain: Terrain::Subterranean,
        patches: vec![],
        objects: vec![],
    });
    let bytes = writer::write_with(&two, &Extras::default()).unwrap();
    let raw = h3_map::parse(&bytes, &Limits::default()).unwrap();
    let art = MapArt::from_raw(&raw).unwrap();
    assert_eq!(art.levels, 2);
    let mut live = Live::at_start(&raw, &art, 0, &tables().map.towns);
    live.heroes = vec![
        LiveHero {
            at: Pos { x: 5, y: 5, z: 1 },
            ..land_hero(5, 5, 0, 0, Facing::Right)
        },
        land_hero(5, 5, 1, 1, Facing::Right),
    ];
    live.path = vec![
        Arrow {
            at: Pos { x: 6, y: 5, z: 1 },
            arrow: 3,
        },
        Arrow {
            at: Pos { x: 7, y: 5, z: 0 },
            arrow: 3,
        },
    ];
    live.explored.set(9, 9, 1, false);
    live.explored.set(12, 12, 0, false);
    let s = drawn(
        &art,
        &live,
        &hero_shapes(&art, &[]),
        ViewAt { x: 0, y: 0, z: 1 },
    );
    let tables = tables();
    let below = tables.terrains[&art.tile(0, 0, 1).unwrap().terrain.code]
        .def
        .clone();
    assert_ne!(
        below,
        tables.terrains[&art.tile(0, 0, 0).unwrap().terrain.code].def
    );
    let terrain = pieces(&s, Pass::Terrain);
    assert!(!terrain.is_empty() && terrain.iter().all(|p| p.0 == below));
    let body: BTreeSet<String> = grouped(&s, Pass::Body).into_iter().map(|p| p.0).collect();
    assert_eq!(
        body,
        BTreeSet::from(["af00.def".to_string(), "ah00_.def".to_string()])
    );
    assert_eq!(pieces(&s, Pass::Path).len(), 1);
    assert_eq!(pieces(&s, Pass::Path)[0].2, [32 * 6, 8 + 32 * 5]);
    let shroud = pieces(&s, Pass::Shroud);
    assert_eq!(shroud.len(), 1);
    assert_eq!(shroud[0].2, [32 * 9, 8 + 32 * 9]);
}

// The walk's draw (E24 M2 T3b, design §3.5, §3.8).

/// `live` with hero `index` lifted in `group` at `frame`, the view slid by `offset`.
fn walking(live: &Live, index: usize, group: u32, frame: u32, offset: [i32; 2]) -> Live {
    let mut live = live.clone();
    live.walking = crate::live::Walking {
        offset,
        lifted: Some(Lifted {
            hero: index,
            group,
            frame,
        }),
    };
    live
}

/// A pass's pieces of the heroes' and boats' DEFs (`ah…`, `af…`, `ab…`), the map's objects left.
fn heroes_of(scene: &Scene, of: Pass) -> Vec<HeroSeen> {
    grouped(scene, of)
        .into_iter()
        .filter(|p| p.0.starts_with('a'))
        .collect()
}

/// The lifted hero's cell (a, b) at its fixed place: the view's tile (8 + a, 7 + b)'s rectangle.
fn cell(a: i32, b: i32) -> [i32; 4] {
    [256 + 32 * a, 232 + 32 * b, 288 + 32 * a, 264 + 32 * b]
}

#[test]
fn a_lifted_hero_is_drawn_at_the_views_middle_and_once_more_blended() {
    // Hero class 3 of colour 2 at (20, 20), facing left (mirrored), lifted in the walking group 7
    // at frame 3; the objects' frame step 11: its flag's frame (11 + 3) mod 8. The view at
    // (11, 12) has the hero at its middle; at (0, 0) it is off the view and still drawn there.
    let (art, mut live) = art_with(vec![]);
    live.heroes = vec![land_hero(20, 20, 3, 2, Facing::Left)];
    live.moment.frames = 11;
    let lifted = walking(&live, 0, 7, 3, [0, 0]);
    let shapes = hero_shapes(&art, &[("af02.def", 8), ("ah03_.def", 8)]);
    let cells = [(0, 0), (1, 0), (2, 0), (0, 1), (1, 1), (2, 1)];
    for view in [at(11, 12), at(0, 0)] {
        let s = drawn(&art, &lifted, &shapes, view);
        let piece = |def: &str, frame: u32, clip: [i32; 4]| {
            (def.to_string(), 7, frame, [256, 232], true, clip)
        };
        // Pass 5: the hero's shadow alone, a cell on each tile it covers.
        let shadows: Vec<HeroSeen> = cells
            .iter()
            .map(|&(a, b)| piece("ah03_.def", 3, cell(a, b)))
            .collect();
        assert_eq!(heroes_of(&s, Pass::Shadow), shadows, "{view:?}");
        // Pass 7: its flag and the hero in each cell.
        let bodies: Vec<HeroSeen> = cells
            .iter()
            .flat_map(|&(a, b)| {
                [
                    piece("af02.def", 6, cell(a, b)),
                    piece("ah03_.def", 3, cell(a, b)),
                ]
            })
            .collect();
        assert_eq!(heroes_of(&s, Pass::Body), bodies, "{view:?}");
        // Pass 9: both whole, blended, after the arrows and before the unexplored tiles.
        let whole = [256, 232, 352, 296];
        assert_eq!(
            grouped(&s, Pass::Blend),
            [piece("af02.def", 6, whole), piece("ah03_.def", 3, whole)],
            "{view:?}"
        );
    }
    // Not lifted it stands on its tile: its facing's group 2, off the view at (0, 0).
    let s = drawn(&art, &live, &shapes, at(11, 12));
    assert!(grouped(&s, Pass::Blend).is_empty());
    assert_eq!(heroes_of(&s, Pass::Body).len(), 12);
    assert!(heroes_of(&s, Pass::Body).iter().all(|p| p.1 == 2));
    assert!(heroes_of(&drawn(&art, &live, &shapes, at(0, 0)), Pass::Body).is_empty());
}

#[test]
fn pass_9_comes_after_the_arrows_and_before_the_unexplored_tiles() {
    let (art, mut live) = art_with(vec![]);
    live.heroes = vec![land_hero(9, 8, 0, 0, Facing::Right)];
    live.path = vec![Arrow {
        at: pos(10, 8),
        arrow: 3,
    }];
    live.explored.set(0, 0, 0, false);
    let s = drawn(
        &art,
        &walking(&live, 0, 7, 0, [0, 0]),
        &hero_shapes(&art, &[]),
        at(0, 0),
    );
    let (_, path) = span(&s, Pass::Path);
    let (blend, last) = span(&s, Pass::Blend);
    let (shroud, _) = span(&s, Pass::Shroud);
    assert!(path < blend && last < shroud);
}

#[test]
fn the_walks_offset_moves_every_tiles_place_but_the_lifted_hero_and_the_gems() {
    let (art, mut live) = art_with(vec![]);
    live.heroes = vec![land_hero(9, 8, 0, 0, Facing::Right)];
    live.path = vec![Arrow {
        at: pos(12, 8),
        arrow: 3,
    }];
    let shapes = hero_shapes(&art, &[]);
    let still = drawn(&art, &walking(&live, 0, 7, 0, [0, 0]), &shapes, at(0, 0));
    let slid = drawn(&art, &walking(&live, 0, 7, 0, [10, -22]), &shapes, at(0, 0));
    // A terrain's or an arrow's canvas is its tile's place; its clip that tile inside the view's.
    let moved = |p: &Seen| {
        let [x, y] = [p.2[0] + 10, p.2[1] - 22];
        let clip = [x.max(8), y.max(8), (x + 32).min(600), (y + 32).min(552)];
        (p.0.clone(), p.1, [x, y], p.3, clip)
    };
    for pass in [Pass::Terrain, Pass::Path] {
        let expected: Vec<Seen> = pieces(&still, pass)
            .iter()
            .map(moved)
            .filter(|p| p.4[0] < p.4[2] && p.4[1] < p.4[3])
            .collect();
        // The rows and columns past the view's (−1 and 19, 17) fill what the offset uncovers.
        let got: Vec<Seen> = pieces(&slid, pass)
            .into_iter()
            .filter(|p| expected.iter().any(|e| e.2 == p.2))
            .collect();
        assert_eq!(got, expected, "{pass:?}");
    }
    // The pass 8 arrow on (12, 8): at its tile's place plus the offset.
    assert_eq!(pieces(&slid, Pass::Path)[0].2, [32 * 12 + 10, 8 + 32 * 8 - 22]);
    // The lifted hero and the gems keep their places (the pool's pieces move).
    for pass in [Pass::Shadow, Pass::Body, Pass::Blend] {
        assert_eq!(heroes_of(&slid, pass), heroes_of(&still, pass), "{pass:?}");
        assert!(!heroes_of(&still, pass).is_empty(), "{pass:?}");
    }
    let gems = |s: &Scene| -> Vec<Draw> {
        s.draws
            .iter()
            .filter(|d| matches!(d, Draw::Gem { .. }))
            .cloned()
            .collect()
    };
    assert_eq!(gems(&slid), gems(&still));
}

#[test]
fn the_lifted_heros_cells_lie_among_the_layers_as_a_heros() {
    // The 2 × 2 tree anchored at (6, 6): its cells' layer 1 on row 6, 2 on row 5. The lifted
    // hero's lower cells come after layer 1, its upper ones after layer 2.
    let (art, mut live) = art_with_objects(vec![object("tree.def", 6, 6, 0)]);
    let shapes = with_shape(&art, "tree.def", [0xff; 6], [0xff; 6], 1);
    let (flag, hero, tree) = ("af00.def", "ah00_.def", "tree.def");
    // The hero's row 5: on (6, 5) the tree's layer 2 covers its lower cell.
    live.heroes = vec![land_hero(6, 5, 0, 0, Facing::Right)];
    let s = drawn(&art, &walking(&live, 0, 7, 0, [0, 0]), &shapes, at(-3, -3));
    assert_eq!(on_tile(&s, Pass::Body, 9, 8), [flag, hero, tree]);
    // The hero's row 6: its lower cell over the tree's layer 1 on (6, 6), its upper one over
    // layer 2 on (6, 5).
    live.heroes = vec![land_hero(6, 6, 0, 0, Facing::Right)];
    let s = drawn(&art, &walking(&live, 0, 7, 0, [0, 0]), &shapes, at(-3, -2));
    assert_eq!(on_tile(&s, Pass::Body, 9, 8), [tree, flag, hero]);
    assert_eq!(on_tile(&s, Pass::Body, 9, 7), [tree, flag, hero]);
    // The shadows: the objects', then the lifted hero's (the hero alone), then the others'.
    assert_eq!(on_tile(&s, Pass::Shadow, 9, 7), [tree, hero]);
}

#[test]
fn an_empty_lists_heroes_come_before_the_lifted_heros_cell() {
    // The lifted hero at (10, 10), a standing one below it at (10, 11) whose upper row covers the
    // lifted hero's tile: with no object there, the standing hero first (0x411396); its shadow
    // after the lifted one's.
    let (art, mut live) = art_with(vec![]);
    live.heroes = vec![
        land_hero(10, 10, 0, 0, Facing::Right),
        land_hero(10, 11, 1, 1, Facing::Right),
    ];
    let s = drawn(
        &art,
        &walking(&live, 0, 7, 0, [0, 0]),
        &hero_shapes(&art, &[]),
        at(1, 2),
    );
    assert_eq!(
        on_tile(&s, Pass::Body, 9, 8),
        ["af01.def", "ah01_.def", "af00.def", "ah00_.def"]
    );
    assert_eq!(
        on_tile(&s, Pass::Shadow, 9, 8),
        ["ah00_.def", "af01.def", "ah01_.def"]
    );
}

#[test]
fn the_lifted_hero_beyond_the_maps_sides_is_cut() {
    // The map is 36 × 36. Its cells off the map are not drawn; pass 9's picture is cut by whole
    // tiles past a side, less the offset on the left and the top.
    let (art, mut live) = art_with(vec![]);
    live.heroes = vec![land_hero(0, 0, 0, 0, Facing::Right)];
    let shapes = hero_shapes(&art, &[]);
    let blend_clip = |view: ViewAt, offset: [i32; 2]| {
        let s = drawn(&art, &walking(&live, 0, 7, 0, offset), &shapes, view);
        grouped(&s, Pass::Blend).first().map(|p| p.5)
    };
    // The view at (−10, −8): the picture's first map column −2, its first row −1.
    let s = drawn(&art, &walking(&live, 0, 7, 0, [0, 0]), &shapes, at(-10, -8));
    let clips: BTreeSet<[i32; 4]> = heroes_of(&s, Pass::Body).iter().map(|p| p.5).collect();
    assert_eq!(clips, BTreeSet::from([cell(2, 1)]));
    assert_eq!(blend_clip(at(-10, -8), [0, 0]), Some([320, 264, 352, 296]));
    assert_eq!(blend_clip(at(-10, -8), [-10, 22]), Some([310, 286, 352, 296]));
    // Past the right and the bottom sides by a column and a row: no offset there.
    assert_eq!(blend_clip(at(26, 28), [0, 0]), Some([256, 232, 320, 264]));
    assert_eq!(blend_clip(at(26, 28), [10, -22]), Some([256, 232, 320, 264]));
    // Nothing left: no draw.
    assert_eq!(blend_clip(at(-11, -8), [0, 22]), None);
}

#[test]
fn a_lifted_hero_in_a_boat_its_waves_flags_and_boat() {
    // Water at (4, 8), open (21), and at (16, 8), the coast (0); the hero of colour 2 in a boat
    // of type 1 with flag colour 3, facing up, lifted in group 6 at frame 5, the objects' step 0.
    let water = |x: u8, picture: u8| GivenTile {
        level: 0,
        tile: (x, 8),
        terrain: Terrain::Water,
        picture,
        river: None,
        road: None,
        flags: 0,
    };
    let (art, mut live) = art_with(vec![water(4, 21), water(16, 0)]);
    let sail = |x: u8| LiveHero {
        boat: Some(Boat { kind: 1, colour: 3 }),
        ..land_hero(x, 8, 4, 2, Facing::Up)
    };
    let shapes = hero_shapes(&art, &[]);
    live.heroes = vec![sail(4)];
    let s = drawn(&art, &walking(&live, 0, 6, 5, [0, 0]), &shapes, at(-5, 0));
    // Pass 5: the boat's shadow alone; pass 7: the waves, the boat's flag by its colour, the
    // boat; pass 9: the boat's flag by the hero's colour and the boat, no waves.
    assert_eq!(on_tile(&s, Pass::Shadow, 9, 8), ["ab02_.def"]);
    assert_eq!(
        on_tile(&s, Pass::Body, 9, 8),
        ["abm02_.def", "abf02d.def", "ab02_.def"]
    );
    let whole: Vec<(String, u32, u32)> = grouped(&s, Pass::Blend)
        .into_iter()
        .map(|p| (p.0, p.1, p.2))
        .collect();
    assert_eq!(
        whole,
        [
            ("abf02r.def".to_string(), 6, 5),
            ("ab02_.def".to_string(), 6, 5)
        ]
    );
    // On the coast: no waves.
    live.heroes = vec![sail(16)];
    let s = drawn(&art, &walking(&live, 0, 6, 5, [0, 0]), &shapes, at(7, 0));
    assert_eq!(
        on_tile(&s, Pass::Body, 9, 8),
        ["abf02d.def", "ab02_.def"]
    );
}

#[test]
fn a_lifted_hero_that_is_none_of_the_heroes_has_no_scene() {
    let (art, live) = art_with(vec![]);
    let err = scene(
        &art,
        &walking(&live, 0, 7, 0, [0, 0]),
        &tables(),
        &layout(),
        &shapes_of(&art),
        at(0, 0),
        Bugs::Fixed,
    )
    .unwrap_err();
    assert_eq!(
        err,
        SceneError::Live("the lifted hero 0 of 0 heroes".to_string())
    );
}
