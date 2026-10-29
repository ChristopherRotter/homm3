use super::*;
use alloc::string::ToString;

use h3_map::model::Pos;

use crate::art::ObjectArt;
use crate::live::{Drawn, Explored, LiveObject, Moment};
use h3_render::Bugs;

/// A passability mask with every cell passable but `blocked` (dx, dy).
fn passable_except(blocked: &[(u8, u8)]) -> [u8; 6] {
    let mut mask = [0xff; 6];
    for &(dx, dy) in blocked {
        mask[usize::from(5 - dy)] &= !(1 << (7 - dx));
    }
    mask
}

fn tmpl(def: &str, class: u32, passable: [u8; 6], priority: u8) -> TemplateArt {
    TemplateArt {
        def: def.to_string(),
        passable,
        visitable: [0; 6],
        class,
        subclass: 0,
        priority,
    }
}

fn drawn(def: &str, class: u32) -> Drawn {
    Drawn {
        def: def.to_string(),
        own: None,
        class,
        subclass: 2,
    }
}

/// A `width` × `height` shape whose masks cover every cell, with `frames` frames.
fn shape(width: u8, height: u8, frames: u32) -> Shape {
    Shape {
        width,
        height,
        colour: [0xff; 6],
        shadow: [0xff; 6],
        frames,
    }
}

/// A 10 × 10 map (one level) with `templates` and objects (anchor x, y, level, template).
fn art(templates: Vec<TemplateArt>, objects: &[(u8, u8, u8, usize)]) -> MapArt {
    MapArt {
        size: 10,
        levels: 2,
        tiles: Vec::new(),
        templates,
        objects: objects
            .iter()
            .map(|&(x, y, z, template)| ObjectArt {
                anchor: Pos { x, y, z },
                template,
            })
            .collect(),
        hero_names: alloc::collections::BTreeMap::new(),
    }
}

fn live(objects: usize) -> Live {
    Live {
        objects: (0..objects)
            .map(|_| LiveObject {
                standing: true,
                drawn: None,
                owner: None,
                phase: 0,
            })
            .collect(),
        heroes: Vec::new(),
        boats: Vec::new(),
        viewer: 0,
        explored: Explored::all(10, 2, true),
        path: Vec::new(),
        moment: Moment::default(),
        walking: crate::live::Walking::default(),
        kingdom: crate::live::Kingdom::default(),
        buttons: None,
        minimap: None,
    }
}

fn shapes(entries: &[(&str, Shape)]) -> Shapes {
    let mut shapes = Shapes::default();
    for (name, shape) in entries {
        shapes.insert(name, *shape);
    }
    shapes
}

/// The (object, layer) of each entry of tile (x, y).
fn list(lists: &DrawLists, x: i32, y: i32) -> Vec<(usize, u8)> {
    lists.at(x, y).iter().map(|e| (e.object, e.layer)).collect()
}

#[test]
fn a_shape_is_its_msk_and_its_frames() {
    let mask = Mask {
        width: 3,
        height: 2,
        masks: [[0, 0, 0, 0, 0x01, 0x80], [0, 0, 0, 0, 0, 0x40]],
    };
    let s = Shape::new(&mask, 4).unwrap();
    assert_eq!((s.width, s.height, s.frames), (3, 2, 4));
    // The colour mask: (7, 1) and the anchor's cell (0, 0); the shadow mask: (1, 0).
    assert!(s.body(0, 0) && s.body(7, 1) && !s.body(1, 0) && !s.body(8, 0));
    assert!(s.shadow(1, 0) && !s.shadow(0, 0) && !s.shadow(0, 6));
    let wide = Mask { width: 9, ..mask };
    assert_eq!(Shape::new(&wide, 1), Err(ShapeError::Size(9, 2)));
    let high = Mask { height: 7, ..mask };
    assert_eq!(Shape::new(&high, 1), Err(ShapeError::Size(3, 7)));
    assert_eq!(Shape::new(&mask, 0), Err(ShapeError::NoFrames));
    assert_eq!(
        ShapeError::Size(9, 2).to_string(),
        "a 9 x 2 .msk is larger than 8 x 6"
    );
    assert_eq!(
        ShapeError::NoFrames.to_string(),
        "its DEF's first group has no frame"
    );
}

#[test]
fn shapes_are_found_by_name_ignoring_case() {
    let s = shapes(&[("AvLTree0.def", shape(2, 2, 1))]);
    assert_eq!(s.get("avltree0.DEF"), Some(&shape(2, 2, 1)));
    assert_eq!(s.get("avltree1.def"), None);
}

#[test]
fn the_pictures_are_the_templates_and_the_drawn_ones_once() {
    let a = art(
        vec![
            tmpl("a.def", 1, [0xff; 6], 0),
            tmpl("A.DEF", 1, [0xff; 6], 0),
            tmpl("b.def", 1, [0xff; 6], 0),
        ],
        &[(1, 1, 0, 0)],
    );
    let mut l = live(1);
    l.objects[0].drawn = Some(Drawn {
        own: Some("d.def".to_string()),
        ..drawn("c.def", 1)
    });
    assert_eq!(pictures(&a, &l), ["a.def", "b.def", "c.def", "d.def"]);
}

#[test]
fn a_flat_template_has_every_layer_0() {
    let t = tmpl("cursed.def", 223, passable_except(&[(0, 0)]), 1);
    assert_eq!(layers(&t, &shape(2, 2, 1)), [[0; 6]; 8]);
}

#[test]
fn layers_count_rows_from_the_base() {
    // Column 0: passable, passable, blocked (on a passable cell: layer 1 again), blocked.
    // Column 1: blocked, blocked, passable beside column 0's blocked cell (its layer 1), passable.
    let t = tmpl(
        "x.def",
        1,
        passable_except(&[(0, 2), (0, 3), (1, 0), (1, 1)]),
        0,
    );
    let l = layers(&t, &shape(2, 4, 1));
    assert_eq!(l[0][..4], [1, 2, 1, 2]);
    assert_eq!(l[1][..4], [1, 2, 1, 2]);
    // Only a blocked neighbour gives its layer: with (0, 3) passable, (1, 3) counts on from
    // (1, 2); with no blocked cell in column 0, column 1 counts 1 to 4.
    let t = tmpl("x.def", 1, passable_except(&[(0, 2), (1, 0), (1, 1)]), 0);
    let l = layers(&t, &shape(2, 4, 1));
    assert_eq!(l[1][..4], [1, 2, 1, 2]);
    let t = tmpl("x.def", 1, passable_except(&[(1, 0), (1, 1)]), 0);
    let l = layers(&t, &shape(2, 4, 1));
    assert_eq!(l[1][..4], [1, 2, 3, 4]);
    // Cells outside the footprint stay 0.
    assert_eq!(l[2], [0; 6]);
    assert_eq!(l[0][4..], [0, 0]);
}

#[test]
fn at_equal_layers_the_later_object_is_on_top() {
    let a = art(
        vec![tmpl("a.def", 1, passable_except(&[(0, 0)]), 0)],
        &[(4, 4, 0, 0), (4, 4, 0, 0)],
    );
    let lists = DrawLists::build(&a, &live(2), &shapes(&[("a.def", shape(1, 1, 1))]), 0).unwrap();
    assert_eq!(list(&lists, 4, 4), [(0, 1), (1, 1)]);
    assert!(lists.at(3, 4).is_empty());
    let e = lists.at(4, 4)[1];
    assert_eq!((e.dx, e.dy), (0, 0));
}

#[test]
fn a_higher_layer_stays_on_top_whatever_the_order() {
    // Object 0, two blocked cells high: layer 2 on (4, 3). Object 1, one cell on (4, 3): layer 1,
    // so it goes under object 0 there; on (4, 4) object 0 alone, layer 1.
    let a = art(
        vec![
            tmpl("tall.def", 1, passable_except(&[(0, 0), (0, 1)]), 0),
            tmpl("low.def", 1, passable_except(&[(0, 0)]), 0),
        ],
        &[(4, 4, 0, 0), (4, 3, 0, 1)],
    );
    let s = shapes(&[("tall.def", shape(1, 2, 1)), ("low.def", shape(1, 1, 1))]);
    let lists = DrawLists::build(&a, &live(2), &s, 0).unwrap();
    assert_eq!(list(&lists, 4, 3), [(1, 1), (0, 2)]);
    assert_eq!(list(&lists, 4, 4), [(0, 1)]);
    // The other way round: the higher one placed later goes on top of the lower one.
    let a = art(a.templates.clone(), &[(4, 3, 0, 1), (4, 4, 0, 0)]);
    let lists = DrawLists::build(&a, &live(2), &s, 0).unwrap();
    assert_eq!(list(&lists, 4, 3), [(0, 1), (1, 2)]);
}

#[test]
fn at_equal_layers_an_earlier_object_higher_on_a_shared_tile_stays_on_top() {
    // Both two cells high on (4, 5) and (4, 4). Object 0: blocked, blocked (layers 1, 2).
    // Object 1: passable, blocked on it (layers 1, 1). On (4, 5) both are layer 1, but object 0
    // is higher on (4, 4): object 1 goes under it on both tiles.
    let a = art(
        vec![
            tmpl("a.def", 1, passable_except(&[(0, 0), (0, 1)]), 0),
            tmpl("b.def", 1, passable_except(&[(0, 1)]), 0),
        ],
        &[(4, 5, 0, 0), (4, 5, 0, 1)],
    );
    let s = shapes(&[("a.def", shape(1, 2, 1)), ("b.def", shape(1, 2, 1))]);
    let lists = DrawLists::build(&a, &live(2), &s, 0).unwrap();
    assert_eq!(list(&lists, 4, 5), [(1, 1), (0, 1)]);
    assert_eq!(list(&lists, 4, 4), [(1, 1), (0, 2)]);
    // Placed the other way round, object 1 (now 0) is nowhere higher: object 0 (now 1) on top.
    let a = art(a.templates.clone(), &[(4, 5, 0, 1), (4, 5, 0, 0)]);
    let lists = DrawLists::build(&a, &live(2), &s, 0).unwrap();
    assert_eq!(list(&lists, 4, 5), [(0, 1), (1, 1)]);
    assert_eq!(list(&lists, 4, 4), [(0, 1), (1, 2)]);
}

#[test]
fn the_comparison_looks_only_at_tiles_on_the_map() {
    // Object 0, 3 × 2 at (1, 1): its column 2 is off the map, layers 1 and 2 there. Object 1,
    // 2 × 2 at (0, 1): its column 1 is off the map too, layers 1 and 1. On the map both are
    // layers 1 and 2 in column 0: object 0 is higher only off the map, so object 1 goes on top.
    let a = art(
        vec![
            tmpl(
                "wide.def",
                1,
                passable_except(&[(0, 0), (0, 1), (1, 0), (1, 1)]),
                0,
            ),
            tmpl("b.def", 1, passable_except(&[(0, 0), (0, 1), (1, 1)]), 0),
        ],
        &[(1, 1, 0, 0), (0, 1, 0, 1)],
    );
    let s = shapes(&[("wide.def", shape(3, 2, 1)), ("b.def", shape(2, 2, 1))]);
    assert_eq!(layers(&a.templates[0], &shape(3, 2, 1))[2][..2], [1, 2]);
    assert_eq!(layers(&a.templates[1], &shape(2, 2, 1))[1][..2], [1, 1]);
    let lists = DrawLists::build(&a, &live(2), &s, 0).unwrap();
    assert_eq!(list(&lists, 0, 1), [(0, 1), (1, 1)]);
    assert_eq!(list(&lists, 0, 0), [(0, 2), (1, 2)]);
    assert_eq!(list(&lists, 1, 0), [(0, 2)]);
}

#[test]
fn cells_off_the_map_have_no_entries() {
    // A 3 × 3 object anchored at (1, 0): only (0, 0) and (1, 0) are on the map. One anchored past
    // the bottom-right corner, at (11, 11): only (9, 9) is.
    let a = art(
        vec![tmpl("a.def", 1, passable_except(&[]), 0)],
        &[(1, 0, 0, 0), (11, 11, 0, 0)],
    );
    let lists = DrawLists::build(&a, &live(2), &shapes(&[("a.def", shape(3, 3, 1))]), 0).unwrap();
    assert_eq!(list(&lists, 0, 0), [(0, 1)]);
    assert_eq!(list(&lists, 1, 0), [(0, 1)]);
    assert_eq!(list(&lists, 9, 9), [(1, 3)]);
    let e = lists.at(9, 9)[0];
    assert_eq!((e.dx, e.dy), (2, 2));
    let all: usize = (0..10)
        .flat_map(|y| (0..10).map(move |x| (x, y)))
        .map(|(x, y)| lists.at(x, y).len())
        .sum();
    assert_eq!(all, 3);
    assert!(lists.at(-1, 0).is_empty() && lists.at(0, 10).is_empty());
}

#[test]
fn heroes_boats_events_the_fallen_and_the_other_level_have_no_entries() {
    let templates: Vec<TemplateArt> = NO_ENTRIES
        .iter()
        .map(|&class| tmpl("a.def", class, passable_except(&[(0, 0)]), 0))
        .chain([tmpl("a.def", 1, passable_except(&[(0, 0)]), 0)])
        .collect();
    let mut objects: Vec<(u8, u8, u8, usize)> =
        (0..NO_ENTRIES.len()).map(|t| (2, 2, 0, t)).collect();
    // A standing object of the other level, and one picked up.
    objects.extend([(2, 2, 1, NO_ENTRIES.len()), (2, 2, 0, NO_ENTRIES.len())]);
    let a = art(templates, &objects);
    let mut l = live(objects.len());
    l.objects[objects.len() - 1].standing = false;
    let s = shapes(&[("a.def", shape(1, 1, 1))]);
    assert!(DrawLists::build(&a, &l, &s, 0).unwrap().at(2, 2).is_empty());
    assert_eq!(
        list(&DrawLists::build(&a, &l, &s, 1).unwrap(), 2, 2),
        [(6, 1)]
    );
}

#[test]
fn a_fallen_object_leaves_the_others_in_the_order_it_gave_them() {
    // E24 M1 T6b (the reviewer's R1-1): B, A, C on (5, 5); B 1 x 2 blocked, A 1 x 1 blocked, C 1 x 2
    // with its anchor passable. The original builds the lists with every object and takes a
    // fallen one's entries out (0x4AABB0): C stays over B, where lists built without A put it
    // under B.
    let templates = vec![
        tmpl("b.def", 1, passable_except(&[(0, 0), (0, 1)]), 0),
        tmpl("a.def", 1, passable_except(&[(0, 0)]), 0),
        tmpl("c.def", 1, passable_except(&[(0, 1)]), 0),
    ];
    let s = shapes(&[
        ("b.def", shape(1, 2, 1)),
        ("a.def", shape(1, 1, 1)),
        ("c.def", shape(1, 2, 1)),
    ]);
    let all = art(
        templates.clone(),
        &[(5, 5, 0, 0), (5, 5, 0, 1), (5, 5, 0, 2)],
    );
    let mut l = live(3);
    let standing = DrawLists::build(&all, &l, &s, 0).unwrap();
    l.objects[1].standing = false;
    let fallen = DrawLists::build(&all, &l, &s, 0).unwrap();
    let objects =
        |lists: &DrawLists| -> Vec<usize> { lists.at(5, 5).iter().map(|e| e.object).collect() };
    assert_eq!(objects(&standing), [0, 1, 2]);
    assert_eq!(objects(&fallen), [0, 2]);
    assert_eq!(list(&fallen, 5, 4), list(&standing, 5, 4));
    // Without A from the start: C (now object 1) under B.
    let never = art(templates, &[(5, 5, 0, 0), (5, 5, 0, 2)]);
    assert_eq!(
        objects(&DrawLists::build(&never, &live(2), &s, 0).unwrap()),
        [1, 0]
    );
}

#[test]
fn a_shape_wider_than_a_template_is_cut_to_its_cells() {
    // `Shape`'s fields are public: one of 9 x 7 cells draws only the template's 8 x 6.
    let a = art(
        vec![tmpl("a.def", 1, passable_except(&[]), 0)],
        &[(9, 9, 0, 0)],
    );
    let lists = DrawLists::build(&a, &live(1), &shapes(&[("a.def", shape(9, 7, 1))]), 0).unwrap();
    let all: usize = (0..10)
        .flat_map(|y| (0..10).map(move |x| (x, y)))
        .map(|(x, y)| lists.at(x, y).len())
        .sum();
    assert_eq!(all, 48);
}

#[test]
fn a_random_object_keeps_its_templates_place_and_masks() {
    // The map's template is one cell, its shadow mask empty; the drawn picture's two cells high,
    // with three frames.
    let a = art(
        vec![tmpl("random.def", 76, passable_except(&[(0, 0)]), 0)],
        &[(3, 3, 0, 0)],
    );
    let mut l = live(1);
    l.objects[0].drawn = Some(drawn("gold.def", 79));
    let random = Shape {
        shadow: [0; 6],
        ..shape(1, 1, 1)
    };
    let s = shapes(&[("random.def", random), ("gold.def", shape(1, 2, 3))]);
    let lists = DrawLists::build(&a, &l, &s, 0).unwrap();
    assert_eq!(list(&lists, 3, 3), [(0, 1)]);
    assert_eq!(list(&lists, 3, 2), []);
    assert_eq!(template(&a, 0).map(|t| t.def.as_str()), Some("random.def"));
    for bugs in [Bugs::Fixed, Bugs::Original] {
        // Without its own picture the client draws it as the original does.
        assert_eq!(def(&a, &l, 0, bugs), Some("gold.def"));
        assert_eq!(def(&a, &live(1), 0, bugs), Some("random.def"));
        let drawn = super::shape(&a, &l, &s, 0, bugs);
        assert_eq!(
            drawn,
            Some(Shape {
                frames: 3,
                ..random
            })
        );
        assert_eq!(super::shape(&a, &live(1), &s, 0, bugs), Some(random));
        // No such object, no shape for the drawn picture or the template's.
        assert_eq!((template(&a, 1), def(&a, &l, 1, bugs)), (None, None));
        assert_eq!(super::shape(&a, &l, &s, 1, bugs), None);
        let only_random = shapes(&[("random.def", random)]);
        assert_eq!(super::shape(&a, &l, &only_random, 0, bugs), None);
        let only_gold = shapes(&[("gold.def", random)]);
        assert_eq!(super::shape(&a, &l, &only_gold, 0, bugs), None);
    }
}

#[test]
fn the_client_draws_a_random_objects_own_picture_and_masks() {
    // MCL-Q3: the original names AVA0129 for artifact 129, the editor's template AVA0130.
    let a = art(
        vec![tmpl("avarnd4.def", 69, passable_except(&[(0, 0)]), 0)],
        &[(3, 3, 0, 0)],
    );
    let mut l = live(1);
    l.objects[0].drawn = Some(Drawn {
        own: Some("AVA0130.def".to_string()),
        ..drawn("ava0129.def", 5)
    });
    let random = shape(2, 1, 1);
    let own = Shape {
        colour: [0, 0, 0, 0, 0, 0xC0],
        ..shape(2, 1, 8)
    };
    let s = shapes(&[
        ("avarnd4.def", random),
        ("ava0129.def", shape(2, 1, 4)),
        ("ava0130.def", own),
    ]);
    assert_eq!(def(&a, &l, 0, Bugs::Fixed), Some("AVA0130.def"));
    assert_eq!(super::shape(&a, &l, &s, 0, Bugs::Fixed), Some(own));
    assert_eq!(def(&a, &l, 0, Bugs::Original), Some("ava0129.def"));
    let original = super::shape(&a, &l, &s, 0, Bugs::Original);
    assert_eq!(
        original,
        Some(Shape {
            frames: 4,
            ..random
        })
    );
    // Placed by the map's template either way.
    let lists = DrawLists::build(&a, &l, &s, 0).unwrap();
    assert_eq!(list(&lists, 2, 3), [(0, 1)]);
}

#[test]
fn an_object_without_a_shape_is_refused() {
    let a = art(
        vec![tmpl("a.def", 1, passable_except(&[(0, 0)]), 0)],
        &[(3, 3, 0, 0)],
    );
    let err = DrawLists::build(&a, &live(1), &Shapes::default(), 0).unwrap_err();
    assert_eq!(err, ObjectsError::Shape(0, "a.def".to_string()));
    assert_eq!(err.to_string(), "object 0: no .msk for a.def");
}
