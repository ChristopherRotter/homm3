use super::*;

use crate::objects::DrawLists;
use crate::sound::tests::{map_with, rules, template};

#[test]
fn a_tile_records_its_topmost_entrance_else_its_blockers_else_a_hole() {
    let records = rules().records;
    assert_eq!(
        (
            records.game_class(202),
            records.game_class(220),
            records.game_class(5)
        ),
        (158, 53, 5)
    );
    let objects = [
        // 0: a mine blocking (1, 1), a mountain over it blocking it too (the mine first: ownable).
        (template(0, 53, 2, &[(0, 0)], &[]), (1, 1)),
        (template(1, 134, 0, &[(0, 0)], &[]), (1, 1)),
        // 2: a mountain alone at (2, 1), an AB volcano at (3, 1) (loaded as 158).
        (template(2, 134, 0, &[(0, 0)], &[]), (2, 1)),
        (template(3, 202, 0, &[(0, 0)], &[]), (3, 1)),
        // 4: an arena's entrance at (4, 1) under a blocking mountain: the entrance wins.
        (template(4, 4, 0, &[], &[(0, 0)]), (4, 1)),
        (template(5, 134, 0, &[(0, 0)], &[]), (4, 1)),
        // 6: a hole at (5, 1), blocking nothing; flowers (passable) at (6, 1) and on the coast.
        (template(6, 124, 0, &[], &[]), (5, 1)),
        (template(7, 120, 0, &[], &[]), (6, 1)),
        (template(8, 120, 0, &[], &[]), (7, 7)),
    ];
    let (art, live, shapes) = map_with(&objects, &[]);
    let lists = DrawLists::build(&art, &live, &shapes, 0).unwrap();
    let at = |x: i32, y: i32| {
        let tile = art.tile(x, y, 0).unwrap();
        let r = records.record(&art, &live, tile, lists.at(x, y));
        (r.object, r.class, r.map_class, r.entrance, r.blocked)
    };
    assert_eq!(at(1, 1), (Some(0), 53, 53, false, true));
    assert_eq!(at(2, 1), (Some(2), 134, 134, false, true));
    assert_eq!(at(3, 1), (Some(3), 158, 202, false, true));
    assert_eq!(at(4, 1), (Some(4), 4, 4, true, false));
    assert_eq!(at(5, 1), (Some(6), 124, 124, false, false));
    assert_eq!(at(6, 1), (None, NO_CLASS, NO_CLASS, false, false));
    assert_eq!(at(7, 7), (None, COAST_CLASS, COAST_CLASS, false, false));
    // An entry whose object has no template is passed over.
    let mut broken = art.clone();
    broken.objects[3].template = 99;
    let tile = broken.tile(3, 1, 0).unwrap();
    let r = records.record(&broken, &live, tile, lists.at(3, 1));
    assert_eq!((r.object, r.class), (None, NO_CLASS));
}
