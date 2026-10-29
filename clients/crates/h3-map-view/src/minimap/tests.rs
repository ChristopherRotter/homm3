use super::*;
use alloc::string::ToString;
use alloc::vec;

use h3_map::model::Pos;

use crate::live::{Facing, LiveHero};
use crate::objects::DrawLists;
use crate::sound::tests::{map_with, template};
use crate::tables;

/// The committed tables and layout (`clients/map/`).
fn tables() -> Tables {
    let text = include_str!("../../../../map/assets/base.toml");
    tables::load(&[tables::parse(text).unwrap()], "base").unwrap()
}

fn layout() -> AdventureLayout {
    AdventureLayout::parse(include_str!("../../../../map/layouts/adventure.toml")).unwrap()
}

fn hero(x: u8, y: u8, z: u8, owner: u8) -> LiveHero {
    LiveHero {
        at: Pos { x, y, z },
        class: 0,
        owner,
        facing: Facing::Down,
        boat: None,
    }
}

#[test]
fn the_tables_minimap_is_the_originals() {
    let t = tables();
    let m = &t.map.minimap;
    assert_eq!(
        (m.radar.as_str(), m.terrain, m.frames()),
        ("radar.def", [8, 9], 5)
    );
    assert_eq!(m.owned, [17, 20, 33, 42, 53, 87, 98]);
    assert_eq!(m.blocking.len(), 21);
    let sizes: Vec<(usize, u32, i32)> = m
        .sizes
        .iter()
        .map(|s| (s.size, s.radar.frame, scaled(100, s.radar.scale)))
        .collect();
    assert_eq!(
        sizes,
        [(36, 4, 400), (72, 3, 200), (108, 2, 133), (144, 1, 100)]
    );
    // Each size's tiles fill the area's 144 pixels both ways.
    for s in &m.sizes {
        for list in [&s.columns, &s.rows] {
            let sum: usize = (0..s.size).map(|i| usize::from(list[i % list.len()])).sum();
            assert_eq!(sum, 144, "size {}", s.size);
        }
    }
    assert_eq!((m.other.frame, scaled(7, m.other.scale)), (1, 7));
    assert_eq!(m.check(), Ok(()));
    // The tables name the box's frames.
    let need = t
        .defs()
        .into_iter()
        .find(|n| n.what == "minimap box")
        .unwrap();
    assert_eq!((need.name.as_str(), need.frames), ("radar.def", 5));
}

#[test]
fn the_scale_truncates_the_exact_product() {
    // 1.33 as an f32 is 11156849 / 2^23: 3 × 1.33 is just below 4, 100 × 1.33 just above 133.
    let third = 0x3FAA_3D71;
    assert_eq!(
        (scaled(3, third), scaled(100, third), scaled(0, third)),
        (3, 133, 0)
    );
    assert_eq!((scaled(9, 0x4080_0000), scaled(9, 0x3F00_0000)), (36, 4));
    // A scale of 2^24 shifts left; one far past saturates.
    assert_eq!(scaled(3, 0x4B80_0000), 3 << 24);
    assert_eq!(scaled(1000, 0x7E80_0000), i32::MAX);
    // No product of a view's corner (−10 to 255) with a table's scale lies within 2^-10 of the
    // next integer, so the x87's roundings (at most 2^-14 here) never carry it across (design
    // §4.4).
    for s in &tables().map.minimap.sizes {
        let bits = s.radar.scale;
        let mantissa = u64::from(bits & 0x7F_FFFF | 0x80_0000);
        let shift = 150 - u64::from((bits >> 23) & 0xFF);
        let one = 1u64 << shift;
        for v in 0..=255u64 {
            let fraction = v * mantissa % one;
            assert!(
                fraction == 0 || one - fraction > one >> 10,
                "{v} × {bits:#x}"
            );
        }
    }
}

#[test]
fn the_tables_minimap_breaks_no_rule() {
    let fails = |change: fn(&mut MinimapArt)| {
        let mut m = tables().map.minimap;
        change(&mut m);
        m.check().unwrap_err()
    };
    assert_eq!(
        fails(|m| m.radar = "radar".to_string()),
        "`radar` is not a .DEF name"
    );
    assert_eq!(fails(|m| m.sizes[1].size = 36), "the size 36 twice");
    assert_eq!(
        fails(|m| m.sizes[0].columns.clear()),
        "the size 36: a tile's pixels empty or 0"
    );
    assert_eq!(
        fails(|m| m.sizes[2].rows[1] = 0),
        "the size 108: a tile's pixels empty or 0"
    );
    for scale in [0xBF80_0000, 0x0000_0001, 0x7F80_0000] {
        let mut m = tables().map.minimap;
        m.other.scale = scale;
        assert_eq!(
            m.check().unwrap_err(),
            alloc::format!("the scale {scale:#x} is not a positive normal f32")
        );
    }
    // The tables refuse it.
    let text = include_str!("../../../../map/assets/base.toml")
        .replace("radar = \"radar.def\"", "radar = \"radar\"");
    assert_eq!(
        tables::load(&[tables::parse(&text).unwrap()], "base").unwrap_err(),
        tables::TableError::Entry("minimap".into(), "`radar` is not a .DEF name".into())
    );
}

#[test]
fn a_tiles_colour_by_its_terrain_its_object_and_the_live_part() {
    let objects = [
        // 0: a mine blocking (1, 1), its entrance (2, 1); a neutral mine at (3, 1).
        (template(0, 53, 0, &[(1, 0)], &[(0, 0)]), (2, 1)),
        (template(1, 53, 0, &[(0, 0)], &[]), (3, 1)),
        // 2: a mountain (listed) at (4, 1), a brush's blocked tile (not listed) at (5, 1), a hole
        // at (6, 1).
        (template(2, 134, 0, &[(0, 0)], &[]), (4, 1)),
        (template(3, 114, 0, &[(0, 0)], &[]), (5, 1)),
        (template(4, 124, 0, &[], &[]), (6, 1)),
        // 5: a town whose entrance lies off the map: its blocked tile at (0, 2).
        (template(5, 98, 0, &[(0, 0)], &[(1, 0)]), (0, 2)),
        // 6: a pile (not owned) blocking (1, 2).
        (template(6, 79, 0, &[(0, 0)], &[]), (1, 2)),
    ];
    let (art, mut live, shapes) = map_with(&objects, &[0, 5]);
    live.objects[0].owner = Some(2);
    live.objects[5].owner = Some(1);
    let lists = DrawLists::build(&art, &live, &shapes, 0).unwrap();
    let (t, l) = (tables(), layout());
    let base = Base::build(&art, &live, &t, &l, &lists, 0, Bugs::Fixed).unwrap();
    let tiles = base.tiles(&live, &t);
    let at = |tiles: &Tiles, x: usize, y: usize| tiles.colours[y * 8 + x];
    assert_eq!(at(&tiles, 2, 1), Colour::Owner(66));
    assert_eq!(at(&tiles, 1, 1), Colour::Owner(66));
    assert_eq!(at(&tiles, 3, 1), Colour::Owner(63));
    assert_eq!(at(&tiles, 4, 1), Colour::Blocked(2));
    assert_eq!(at(&tiles, 5, 1), Colour::Open(2));
    assert_eq!(at(&tiles, 6, 1), Colour::Open(2));
    assert_eq!(at(&tiles, 1, 2), Colour::Open(2));
    assert_eq!(at(&tiles, 0, 0), Colour::Open(8));
    // The town without an entrance on the map: its owner's colour, the terrain's in the original.
    assert_eq!(at(&tiles, 0, 2), Colour::Owner(65));
    let original = Base::build(&art, &live, &t, &l, &lists, 0, Bugs::Original).unwrap();
    assert_eq!(at(&original.tiles(&live, &t), 0, 2), Colour::Open(2));
    // Unexplored tiles black, heroes in their owners' colours on explored tiles of the level.
    live.explored.set(4, 1, 0, false);
    live.explored.set(5, 5, 0, false);
    live.heroes = vec![hero(4, 4, 0, 3), hero(5, 5, 0, 3), hero(6, 6, 1, 3)];
    let tiles = base.tiles(&live, &t);
    assert_eq!(at(&tiles, 4, 1), Colour::Black);
    assert_eq!(at(&tiles, 4, 4), Colour::Owner(67));
    assert_eq!(at(&tiles, 5, 5), Colour::Black);
    assert_eq!(at(&tiles, 6, 6), Colour::Open(2));
    // What they draw: the grass's and the water's DEFs, the flag palette.
    let (defs, owner) = tiles.drawn();
    assert_eq!((defs, owner), (vec!["grastl.def", "watrtl.def"], true));
    let dirt = Tiles {
        look: Arc::clone(&tiles.look),
        colours: vec![Colour::Black, Colour::Blocked(0)],
    };
    let (defs, owner) = dirt.drawn();
    assert_eq!((defs, owner), (vec!["dirttl.def"], false));
    // An 8-tile map is no size of the tables': no tile drawn; the box frame 1 at 1.0.
    assert_eq!((tiles.columns(), tiles.rows()), (vec![], vec![]));
    assert_eq!(
        base.radar(ViewAt { x: 3, y: -2, z: 0 }),
        Draw::Radar {
            def: "radar.def".to_string(),
            frame: 1,
            at: [633, 26],
            cut: [0, 2],
            area: [630, 26, 774, 170],
        }
    );
    // A layout without the minimap's area: none.
    let mut bare = l.clone();
    bare.dialog
        .items
        .retain(|i| i.area().is_none_or(|(at, _)| at != [630, 26]));
    assert_eq!(
        Base::build(&art, &live, &t, &bare, &lists, 0, Bugs::Fixed),
        None
    );
}

#[test]
fn each_size_lays_its_tiles_and_scales_its_box() {
    let t = tables();
    let m = &t.map.minimap;
    let look = |side: usize| {
        let (pattern, _) = m.of(side);
        Arc::new(Look {
            at: [630, 26],
            size: [144, 144],
            side,
            pattern,
            terrains: vec![],
            entries: [8, 9],
        })
    };
    let tiles = |side: usize| Tiles {
        look: look(side),
        colours: vec![],
    };
    let first = |spans: Vec<[i32; 2]>| spans.into_iter().take(4).collect::<Vec<_>>();
    assert_eq!(
        first(tiles(36).columns()),
        [[630, 634], [634, 638], [638, 642], [642, 646]]
    );
    assert_eq!(
        first(tiles(72).rows()),
        [[26, 28], [28, 30], [30, 32], [32, 34]]
    );
    assert_eq!(
        first(tiles(108).columns()),
        [[630, 632], [632, 633], [633, 634], [634, 636]]
    );
    assert_eq!(
        first(tiles(108).rows()),
        [[26, 27], [27, 28], [28, 30], [30, 31]]
    );
    assert_eq!(tiles(108).rows().last(), Some(&[168, 170]));
    assert_eq!(tiles(144).columns().last(), Some(&[773, 774]));
    // The box: the corner times the scale, a negative coordinate cut from the canvas.
    let radar = |side: usize, x: i32, y: i32| {
        let (_, radar) = m.of(side);
        let base = Base {
            look: look(side),
            radar: ("radar.def".to_string(), radar),
            z: 0,
            colours: vec![],
        };
        base.radar(ViewAt { x, y, z: 0 })
    };
    let want = |frame: u32, at: [i32; 2], cut: [i32; 2]| Draw::Radar {
        def: "radar.def".to_string(),
        frame,
        at,
        cut,
        area: [630, 26, 774, 170],
    };
    assert_eq!(radar(36, -9, -8), want(4, [630, 26], [36, 32]));
    assert_eq!(radar(72, 5, 3), want(3, [640, 32], [0, 0]));
    assert_eq!(radar(108, 3, 100), want(2, [633, 159], [0, 0]));
    assert_eq!(radar(144, 134, -1), want(1, [764, 26], [0, 1]));
}
