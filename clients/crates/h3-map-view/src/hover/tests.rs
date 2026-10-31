use super::*;
use alloc::string::ToString;
use alloc::vec;

use h3_ui::text::pack::Pack;

use crate::art::TileArt;
use crate::live::{Boat, Facing, KingdomHero, KingdomTown, LiveBoat, LiveHero, Selected};
use crate::sound::tests::{map_with, template};
use crate::tables::{self, Tables};

fn tables() -> Tables {
    let text = include_str!("../../../../map/assets/base.toml");
    tables::load(&[tables::parse(text).unwrap()], "base").unwrap()
}

fn layout() -> AdventureLayout {
    AdventureLayout::parse(include_str!("../../../../map/layouts/adventure.toml")).unwrap()
}

fn pos(x: u8, y: u8) -> Pos {
    Pos { x, y, z: 0 }
}

fn hero(x: u8, y: u8, owner: u8, class: u8) -> LiveHero {
    LiveHero {
        at: pos(x, y),
        class,
        owner,
        facing: Facing::Down,
        boat: None,
    }
}

/// The facts' table: object words and armies, Red's town, three heroes (A and B Red's, C Blue's;
/// B with its own name), colour 2 Red's ally, guarded tiles and costs.
const FACTS: &str = r#"
words = [0, 2, 0, 7, 0, 0, -1, -1, 0, 0, 0, 0]
armed = [1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]
towns = [[5, 0]]
hero_ids = [[16, 0], [20, 1], [32, 0]]
hero_list = [[16, 2, 0], [20, 3, 1]]
town_list = [0]
teams = [0, 1, 0, 3, 4, 5, 6, 7]
guarded = ["00000000", "00000000", "00000000", "00000000", "00001010", "01000000", "00000100", "00000000"]
reach = [[4, 4, 100], [6, 4, 241], [1, 5, 341], [5, 4, 141], [3, 4, 141], [7, 3, 300], [3, 2, 141], [0, 7, 1700], [5, 6, 3500], [2, 0, 500], [1, 0, 600], [7, 7, 400], [0, 0, 200]]
day_points = 1500
"#;

fn facts() -> Facts {
    let value: toml::Value = toml::from_str(FACTS).unwrap();
    Facts::read(&value, 12, 3, 8, 1).unwrap()
}

/// An 8 × 8 map (water at (0, 0), a shore picture, (1, 0) and (2, 0); coastal land at (7, 7))
/// with Red's town (its entrance at (3, 2), Blue's hero C on it, its body at (2, 2)), colour 2's
/// shipyard (entrance (6, 2) with a boat on it, body (5, 2)), Blue's garrison (entrance (3, 4),
/// body (2, 4)), a monster (5, 4), a chest (6, 4), a campfire (1, 5), a tree (2, 5), the Magic
/// Plains (4, 6), a shipwreck (1, 0), an arena (5, 6), a keymaster's tent (entrance (7, 6) under a
/// chest drawn over it, body (6, 6)); Red's heroes A (4, 3) and B (7, 3); a boat at (2, 0).
pub(crate) fn fixture(boat: bool) -> (MapArt, Live, Arc<Hover>) {
    let objects = [
        (template(0, 98, 0, &[(1, 0)], &[(0, 0)]), (3, 2)),
        (template(1, 87, 0, &[(1, 0)], &[(0, 0)]), (6, 2)),
        (template(2, 33, 0, &[(1, 0)], &[(0, 0)]), (3, 4)),
        (template(3, 54, 13, &[], &[(0, 0)]), (5, 4)),
        (template(4, 101, 0, &[], &[(0, 0)]), (6, 4)),
        (template(5, 12, 0, &[], &[(0, 0)]), (1, 5)),
        (template(6, 155, 0, &[(0, 0)], &[]), (2, 5)),
        (template(7, 46, 0, &[], &[]), (4, 6)),
        (template(8, 85, 0, &[], &[(0, 0)]), (1, 0)),
        (template(9, 4, 0, &[], &[(0, 0)]), (5, 6)),
        (template(10, 10, 0, &[(1, 0)], &[(0, 0)]), (7, 6)),
        (template(11, 101, 0, &[], &[(0, 0)]), (7, 6)),
    ];
    let (art, mut live, shapes) = map_with(&objects, &[0, 1, 2, 10]);
    live.objects[0].owner = Some(0);
    live.objects[1].owner = Some(2);
    live.objects[2].owner = Some(1);
    let mut a = hero(4, 3, 0, 2);
    if boat {
        a.boat = Some(Boat { kind: 0, colour: 0 });
    }
    live.heroes = vec![a, hero(7, 3, 0, 3), hero(3, 2, 1, 5)];
    live.boats = vec![
        LiveBoat {
            at: pos(2, 0),
            kind: 0,
            facing: Facing::Down,
        },
        LiveBoat {
            at: pos(6, 2),
            kind: 0,
            facing: Facing::Down,
        },
    ];
    live.explored.set(0, 6, 0, false);
    live.hover = Some(facts());
    let t = tables();
    let lists = [DrawLists::build(&art, &live, &shapes, 0).unwrap()];
    let hover = Hover::new(&art, &live, &t.records, &lists).unwrap();
    (art, live, hover)
}

fn at(x: i32, y: i32) -> [i32; 3] {
    [x, y, 0]
}

/// Hero A selected with 100 points.
fn a(lifted: bool) -> Option<Selection> {
    Some(Selection {
        id: 16,
        at: pos(4, 3),
        lifted,
        points: 100,
    })
}

#[test]
fn days_count_beyond_the_points_left() {
    assert_eq!(days(300, 300, 1500), 0);
    assert_eq!(days(301, 300, 1500), 1);
    assert_eq!(days(1800, 300, 1500), 1);
    assert_eq!(days(1801, 300, 1500), 2);
    assert_eq!(days(9999, 0, 1500), 3);
    assert_eq!(days(5, 0, 0), 1);
}

#[test]
fn the_facts_read_from_the_live_parts_table() {
    let f = facts();
    assert_eq!(f.words[3], Some(7));
    assert_eq!(f.words[6], None);
    assert_eq!(f.armed[..3], [true, false, true]);
    assert_eq!(f.towns, [(5, 0)]);
    assert_eq!(f.heroes, [(16, false), (20, true), (32, false)]);
    assert_eq!(f.hero_list, [(16, 2, false), (20, 3, true)]);
    assert_eq!(f.town_list, [0]);
    assert_eq!(f.teams[2], 0);
    assert!(f.guarded.explored(4, 4, 0) && f.guarded.explored(1, 5, 0));
    assert!(!f.guarded.explored(5, 4, 0));
    assert_eq!(f.reach.get(&(5, 6)), Some(&3500));
    assert_eq!(f.day_points, 1500);
    let read = |text: &str, objects| {
        let value: toml::Value = toml::from_str(text).unwrap();
        Facts::read(&value, objects, 3, 8, 1)
    };
    // The counts: words and armies per object, ids per hero, a row per level's row of the size.
    assert!(read(FACTS, 11).is_none());
    assert!(read(&FACTS.replace("[[16, 0], [20, 1], ", "["), 12).is_none());
    assert!(read(&FACTS.replace(", \"00000000\"]\nreach", "]\nreach"), 12).is_none());
    assert!(read(&FACTS.replace("\"00001010\"", "\"0000101\""), 12).is_none());
    // Each value as given: a word of -1 or a count, flags 0 or 1, rows of 0 and 1.
    for (from, to) in [
        ("words = [0, 2,", "words = [-2, 2,"),
        ("armed = [1,", "armed = [2,"),
        ("[[16, 0], [20, 1]", "[[16, 2], [20, 1]"),
        ("[[16, 2, 0]", "[[16, 2, 3]"),
        ("\"00001010\"", "\"00002010\""),
        ("day_points", "days"),
    ] {
        assert!(read(&FACTS.replace(from, to), 12).is_none(), "{to}");
    }
}

/// The facts of [`visits_fixture`]'s five objects, `visits` their `[hover]` line.
fn visit_facts(visits: &str) -> String {
    let guarded = ["\"00000000\""; 8].join(", ");
    alloc::format!(
        "words = [0, 0, 1, 2, 3]\narmed = [0, 0, 0, 0, 0]\ntowns = []\nhero_ids = []\n\
         hero_list = []\ntown_list = []\nteams = [0, 1, 2, 3, 4, 5, 6, 7]\n\
         guarded = [{guarded}]\nreach = []\nday_points = 0\n{visits}\n"
    )
}

/// An 8 × 8 map with a Swan Pond (entrance (4, 1), body (3, 1)), a Hill Fort (6, 1), a Wagon
/// (3, 5), a Water Wheel (5, 5) and an Arena (6, 6), the facts' `visits` line `visits`.
fn visits_fixture(visits: &str) -> Arc<Hover> {
    let objects = [
        (template(0, 14, 0, &[(1, 0)], &[(0, 0)]), (4, 1)),
        (template(1, 35, 0, &[], &[(0, 0)]), (6, 1)),
        (template(2, 105, 0, &[], &[(0, 0)]), (3, 5)),
        (template(3, 109, 0, &[], &[(0, 0)]), (5, 5)),
        (template(4, 4, 0, &[], &[(0, 0)]), (6, 6)),
    ];
    let (art, mut live, shapes) = map_with(&objects, &[0]);
    let value: toml::Value = toml::from_str(&visit_facts(visits)).unwrap();
    live.hover = Some(Facts::read(&value, 5, 0, 8, 1).unwrap());
    let lists = [DrawLists::build(&art, &live, &shapes, 0).unwrap()];
    Hover::new(&art, &live, &tables().records, &lists).unwrap()
}

#[test]
fn the_visits_read_from_the_live_parts_table() {
    let read = |visits: &str| {
        let value: toml::Value = toml::from_str(&visit_facts(visits)).unwrap();
        Facts::read(&value, 5, 0, 8, 1)
    };
    let f = read("visits = [[1, 1], [1, -1], [0, 0], [0, -1], [1, -1]]").unwrap();
    assert_eq!(
        f.visits[..3],
        [
            Visit {
                known: true,
                visited: Some(true)
            },
            Visit {
                known: true,
                visited: None
            },
            Visit {
                known: false,
                visited: Some(false)
            },
        ]
    );
    // Left out (a live part written before T6a2a): none known.
    assert!(read("").unwrap().visits.is_empty());
    // One per object; known 0 or 1, the mark -1, 0 or 1.
    for wrong in [
        "visits = [[1, 1]]",
        "visits = [[2, 1], [1, -1], [0, 0], [0, -1], [1, -1]]",
        "visits = [[1, 2], [1, -1], [0, 0], [0, -1], [1, -1]]",
        "visits = [[1, -2], [1, -1], [0, 0], [0, -1], [1, -1]]",
    ] {
        assert!(read(wrong).is_none(), "{wrong}");
    }
}

#[test]
fn the_hint_of_a_visited_object() {
    let h = visits_fixture("visits = [[1, 1], [1, -1], [0, 0], [0, -1], [1, -1]]");
    let hint = |x, y| h.hint(at(x, y), false, None);
    // Known and visited, on the entrance and on the body (the entrance's hint).
    let pond = Hint::Visits {
        class: 14,
        bonus: Some(1),
        visited: Some(true),
    };
    assert_eq!((hint(4, 1), hint(3, 1)), (pond.clone(), pond));
    // Known without a mark; a mark without a bonus; neither; known with no bonus to show.
    assert_eq!(
        hint(6, 1),
        Hint::Visits {
            class: 35,
            bonus: Some(14),
            visited: None
        }
    );
    assert_eq!(
        hint(3, 5),
        Hint::Visits {
            class: 105,
            bonus: None,
            visited: Some(false)
        }
    );
    assert_eq!(hint(5, 5), Hint::Object(109));
    assert_eq!(hint(6, 6), Hint::Object(4));
    // A live part without them: the names.
    let h = visits_fixture("");
    assert_eq!(h.hint(at(4, 1), false, None), Hint::Object(14));
}

#[test]
fn the_tiles_are_the_maps_with_the_heroes_and_boats_on_them() {
    let (_, _, h) = fixture(false);
    // Off the map, nothing.
    assert_eq!(h.base(at(8, 0)), None);
    assert_eq!(h.base(at(-1, 0)), None);
    assert_eq!(h.base([0, 0, 1]), None);
    assert_eq!(h.base([0, 0, -1]), None);
    assert_eq!(h.current(at(0, 8), None), None);
    assert!(!h.explored(at(0, 6)) && !h.explored(at(9, 0)) && h.explored(at(0, 5)));
    // Water never records the coast; coastal land does.
    assert_eq!(
        (
            h.base(at(0, 0)).unwrap().class,
            h.base(at(7, 7)).unwrap().class
        ),
        (0, 3)
    );
    assert!(h.base(at(1, 0)).unwrap().water);
    // The town's tiles: its entrance and its body, its owner by the view's owners.
    let town = h.base(at(3, 2)).unwrap();
    assert_eq!(
        (town.class, town.entrance, town.word, town.owner),
        (98, true, Some(0), Some(0))
    );
    assert!(h.base(at(2, 2)).unwrap().blocked);
    // Hero C stands on the town's entrance; hero A on (4, 3) unless lifted; boats on theirs.
    let c = h.current(at(3, 2), None).unwrap();
    assert_eq!(
        (c.class, c.hero, c.word, c.owner),
        (34, Some(32), Some(32), Some(1))
    );
    assert_eq!(h.current(at(4, 3), a(false)).unwrap().hero, Some(16));
    assert_eq!(h.current(at(4, 3), a(true)).unwrap().class, 0);
    let moved = Selection {
        at: pos(1, 1),
        ..a(false).unwrap()
    };
    assert_eq!(h.current(at(4, 3), Some(moved)).unwrap().class, 0);
    assert_eq!(h.current(at(1, 1), Some(moved)).unwrap().hero, Some(16));
    assert_eq!(h.current(at(2, 0), None).unwrap().class, 8);
    assert_eq!(h.hero_at(pos(7, 3)), Some(20));
    assert_eq!(h.hero_at(pos(0, 0)), None);
    assert_eq!(h.facts().day_points, 1500);
    // The special ground under a tile.
    assert_eq!(h.base(at(4, 6)).unwrap().ground, 46);
}

#[test]
fn the_hint_by_the_tile_its_object_records() {
    let (_, _, h) = fixture(false);
    let hint = |x, y| h.hint(at(x, y), false, a(false));
    // No object, the coast: the special ground's name, else nothing.
    assert_eq!(hint(0, 1), Hint::Empty);
    assert_eq!(hint(7, 7), Hint::Empty);
    assert_eq!(hint(4, 6), Hint::Object(46));
    // A hero, a town, a monster on its entrance.
    assert_eq!(
        hint(3, 2),
        Hint::Hero {
            id: 32,
            class: 5,
            own_name: false
        }
    );
    assert_eq!(hint(2, 2), Hint::Town { name: 5, kind: 0 });
    assert_eq!(
        hint(5, 4),
        Hint::Monster {
            creature: 13,
            count: 7
        }
    );
    // A body of the class of its entrance's (the garrison); a body whose entrance a boat holds
    // (the shipyard, the boat taken off); one whose entrance records another (the tent's under
    // the chest).
    assert_eq!(hint(2, 4), Hint::Object(33));
    assert_eq!(hint(5, 2), Hint::Object(87));
    assert_eq!(hint(6, 6), Hint::Object(101));
    // A class without a case of its own, and an Arena of which the viewer knows nothing (no
    // visits in these facts): its name alone (T7b2, F3-5).
    assert_eq!(hint(6, 4), Hint::Object(101));
    assert_eq!(hint(5, 6), Hint::Object(4));
    // Off the map, nothing.
    assert_eq!(h.hint(at(8, 8), false, None), Hint::Empty);
    // The selected hero lifted: its tile shows what lies under it, but at the view's middle.
    assert_eq!(h.hint(at(4, 3), false, a(true)), Hint::Empty);
    assert_eq!(
        h.hint(at(4, 3), true, a(true)),
        Hint::Hero {
            id: 16,
            class: 2,
            own_name: false
        }
    );
}

#[test]
fn the_hint_of_a_hero_word_town_and_monster_without_them() {
    let (art, live, _) = fixture(false);
    let mut facts = facts();
    facts.towns.clear();
    facts.words[3] = None;
    facts.heroes[2].0 = 40;
    let t = tables();
    let mut live = Live {
        hover: Some(facts),
        ..live
    };
    // A hero's index nowhere in the facts: from the list, else none.
    live.heroes.truncate(3);
    let lists = [DrawLists::build(&art, &live, &shapes_of(&art), 0).unwrap()];
    let h = Hover::new(&art, &live, &t.records, &lists).unwrap();
    assert_eq!(
        h.hint(at(3, 2), false, None),
        Hint::Hero {
            id: 40,
            class: 5,
            own_name: false
        }
    );
    assert_eq!(h.hint(at(2, 2), false, None), Hint::Empty);
    assert_eq!(
        h.hint(at(5, 4), false, None),
        Hint::Monster {
            creature: 13,
            count: 0
        }
    );
    assert_eq!(h.hero(20), Some((20, 3, true)));
    assert_eq!(h.hero(99), None);
    // Without facts, no mouse-over.
    let bare = Live {
        hover: None,
        ..live
    };
    assert!(Hover::new(&art, &bare, &t.records, &lists).is_none());
}

/// The shapes `map_with` gives `art`'s objects.
fn shapes_of(art: &MapArt) -> objects::Shapes {
    let wide = [0, 1, 2, 10];
    let mut shapes = objects::Shapes::default();
    for (i, t) in art.templates.iter().enumerate() {
        shapes.insert(
            &t.def,
            objects::Shape {
                width: if wide.contains(&i) { 2 } else { 1 },
                height: 1,
                colour: [0xff; 6],
                shadow: [0xff; 6],
                frames: 1,
            },
        );
    }
    shapes
}

#[test]
fn a_monsters_body_without_its_entrance_has_no_hint() {
    // The client's fix (MCL-Q27 (a)): the original's case 54 writes nothing there, its bar
    // keeping the buffer's last text (0x40C342).
    let objects = [(template(0, 54, 1, &[(0, 0)], &[]), (2, 2))];
    let (art, mut live, shapes) = map_with(&objects, &[]);
    let mut f = facts();
    f.words = vec![Some(3)];
    f.armed = vec![false];
    f.heroes.clear();
    live.hover = Some(f);
    let lists = [DrawLists::build(&art, &live, &shapes, 0).unwrap()];
    let h = Hover::new(&art, &live, &tables().records, &lists).unwrap();
    assert_eq!(h.hint(at(2, 2), false, None), Hint::Empty);
}

#[test]
fn the_cursor_without_a_hero_selected() {
    let (_, _, h) = fixture(false);
    let cursor = |x, y| {
        let c = h.cursor(at(x, y), None, true);
        (c.frame, c.action)
    };
    // Red's town (its body), Red's hero B, an ally's shipyard.
    assert_eq!(cursor(2, 2), (frame::TOWN, 3));
    assert_eq!(cursor(7, 3), (frame::HERO, 2));
    assert_eq!(cursor(5, 2), (frame::BOARD, 8));
    // Blue's hero, Blue's garrison, the tree, a tile not explored, off the map: the arrow.
    for (x, y) in [(3, 2), (2, 4), (2, 5), (0, 6), (8, 0)] {
        assert_eq!(cursor(x, y), (frame::ARROW, -1), "({x}, {y})");
    }
}

#[test]
fn the_cursor_of_a_hero_on_land() {
    let (_, _, h) = fixture(false);
    let cursor = |x, y| {
        let c = h.cursor(at(x, y), a(false), true);
        (c.frame, c.action)
    };
    let day = |kind: u32, days: u32| kind + frame::DAY * days;
    // Itself; the bodies of Red's town and an ally's shipyard; the tree's and Blue's garrison's.
    assert_eq!(cursor(4, 3), (frame::HERO, 2));
    assert_eq!(cursor(2, 2), (frame::TOWN, 5));
    assert_eq!(cursor(5, 2), (frame::BOARD, 8));
    assert_eq!(cursor(2, 5), (frame::ARROW, -1));
    assert_eq!(cursor(2, 4), (frame::ARROW, -1));
    // Water: only a hero, a boat or a shipwreck there.
    assert_eq!(cursor(0, 0), (frame::ARROW, -1));
    assert_eq!(cursor(2, 0), (day(frame::BOARD, 1), 1));
    assert_eq!(cursor(1, 0), (day(frame::VISIT, 1), 1));
    // Guarded: empty land and an object entered (the arena) attack, one visited from beside it
    // (the chest, the campfire) not.
    assert_eq!(cursor(4, 4), (frame::ATTACK, 1));
    assert_eq!(cursor(5, 6), (day(frame::ATTACK, 3), 1));
    assert_eq!(cursor(6, 4), (day(frame::VISIT, 1), 1));
    assert_eq!(cursor(1, 5), (day(frame::VISIT, 1), 1));
    // The monster, Blue's garrison's armed entrance, Blue's hero, Red's hero B.
    assert_eq!(cursor(5, 4), (day(frame::ATTACK, 1), 1));
    assert_eq!(cursor(3, 4), (day(frame::ATTACK, 1), 1));
    assert_eq!(cursor(3, 2), (day(frame::ATTACK, 1), 1));
    assert_eq!(cursor(7, 3), (day(frame::EXCHANGE, 1), 1));
    // The coast from land; empty land two days away.
    assert_eq!(cursor(7, 7), (day(frame::MOVE, 1), 1));
    assert_eq!(cursor(0, 7), (day(frame::MOVE, 2), 1));
    // No cost given, or the reach stale after a walk: the arrow.
    assert_eq!(cursor(1, 1), (frame::ARROW, -1));
    assert_eq!(h.cursor(at(4, 4), a(false), false).frame, frame::ARROW);
}

#[test]
fn the_cursor_at_an_own_or_unarmed_entrance() {
    let (art, live, _) = fixture(false);
    let mut f = facts();
    // The garrison Red's ally's and unarmed; Red's town unarmed.
    f.armed = vec![false; 12];
    let mut live = Live {
        hover: Some(f),
        ..live
    };
    live.objects[2].owner = Some(2);
    live.heroes.truncate(2);
    live.hover.as_mut().unwrap().heroes.truncate(2);
    let lists = [DrawLists::build(&art, &live, &shapes_of(&art), 0).unwrap()];
    let h = Hover::new(&art, &live, &tables().records, &lists).unwrap();
    let frame = |x, y| h.cursor(at(x, y), a(false), true).frame;
    // An ally's garrison and Red's town: visited.
    assert_eq!(frame(3, 4), frame::VISIT + frame::DAY);
    assert_eq!(frame(3, 2), frame::VISIT + frame::DAY);
    // Blue's: unarmed, visited too.
    live.objects[2].owner = Some(1);
    let h = Hover::new(&art, &live, &tables().records, &lists).unwrap();
    assert_eq!(
        h.cursor(at(3, 4), a(false), true).frame,
        frame::VISIT + frame::DAY
    );
}

#[test]
fn the_cursor_of_a_hero_in_a_boat() {
    let (_, _, h) = fixture(true);
    let cursor = |x, y| {
        let c = h.cursor(at(x, y), a(false), true);
        (c.frame, c.action)
    };
    // Land but the coast: the arrow; the coast: a landing.
    assert_eq!(cursor(0, 7), (frame::ARROW, -1));
    assert_eq!(cursor(7, 7), (frame::DISEMBARK + frame::DAY, 1));
    // Water: the sail, a visit from the boat by days, another boat nothing.
    assert_eq!(cursor(0, 0), (frame::BOARD + frame::DAY, 1));
    assert_eq!(cursor(1, 0), (frame::SAIL_VISIT + 1, 1));
    assert_eq!(cursor(2, 0), (frame::DAY, -1));
}

#[test]
fn the_cursor_on_another_level_than_the_hero() {
    let (mut art, mut live, shapes) = map_with(&[], &[]);
    let water: Vec<TileArt> = art.tiles.clone();
    art.tiles.extend(water);
    art.levels = 2;
    live.explored = Explored::all(8, 2, true);
    let mut f = facts();
    f.words.clear();
    f.armed.clear();
    f.heroes.clear();
    f.guarded = Explored::all(8, 2, false);
    live.hover = Some(f);
    let lists = [
        DrawLists::build(&art, &live, &shapes, 0).unwrap(),
        DrawLists::build(&art, &live, &shapes, 1).unwrap(),
    ];
    let h = Hover::new(&art, &live, &tables().records, &lists).unwrap();
    assert_eq!(h.cursor([4, 4, 1], a(false), true).frame, frame::ARROW);
    assert_eq!(h.base([4, 4, 1]).unwrap().class, 0);
}

fn texts() -> Catalogue {
    let json = concat!(
        r#"{ "map-hint": { "0": "World Map" }, "object": { "46": "Magic Plains" }, "#,
        r#""hero": { "16": "Mephala" }, "hero-class": { "2": "Ranger" }, "#,
        r#""town-name": { "5": "Arcton" }, "town-type": { "0": "Castle" }, "#,
        r#""creature": { "13_one": "Archangel", "13_other": "Archangels" }, "#,
        r#""map": { "hint": { "hero": "{{name}} the {{class}}", "town": "{{name}}, {{type}}", "#,
        r#""monster": "{{size}} {{creature}}", "more": " {{text}}", "visited": "(seen)", "#,
        r#""not-visited": "(unseen)" }, "#,
        r#""army": { "0": "A few", "1": "Several", "8": "A Legion of" } } , "#,
        r#""map-bonus": { "1": "(bonus)" } }"#,
    );
    Catalogue {
        packs: vec![Pack::from_json("en", json).unwrap()],
    }
}

#[test]
fn a_hints_text() {
    let t = texts();
    let names = BTreeMap::from([(20, "Own".to_string())]);
    let text = |h: Hint| h.text(&names, &t);
    assert_eq!(text(Hint::Empty), "");
    assert_eq!(text(Hint::Panel(0)), "World Map");
    assert_eq!(text(Hint::Object(46)), "Magic Plains");
    assert_eq!(text(Hint::Object(232)), "");
    let hero = |id, own_name| Hint::Hero {
        id,
        class: 2,
        own_name,
    };
    assert_eq!(text(hero(16, false)), "Mephala the Ranger");
    assert_eq!(text(hero(20, true)), "Own the Ranger");
    assert_eq!(text(Hint::Town { name: 5, kind: 0 }), "Arcton, Castle");
    let monster = |creature, count| Hint::Monster { creature, count };
    assert_eq!(text(monster(13, 4)), "A few Archangels");
    assert_eq!(text(monster(13, 5)), "Several Archangels");
    assert_eq!(text(monster(13, 1000)), "A Legion of Archangels");
    assert_eq!(text(monster(151, 1)), "A few ");
    // T7b2 (F3-3): an entry past the catalogue's family is empty, not its key.
    assert_eq!(text(monster(150, 1)), "A few ");
    assert_eq!(text(hero(200, false)), " the Ranger");
    assert_eq!(
        text(Hint::Town {
            name: 999,
            kind: 12
        }),
        ", "
    );
    let visits = |bonus, visited| Hint::Visits {
        class: 46,
        bonus,
        visited,
    };
    assert_eq!(
        text(visits(Some(1), Some(true))),
        "Magic Plains (bonus) (seen)"
    );
    assert_eq!(text(visits(None, Some(false))), "Magic Plains (unseen)");
    assert_eq!(text(visits(Some(1), None)), "Magic Plains (bonus)");
}

pub(crate) fn kingdom() -> Kingdom {
    Kingdom {
        heroes: vec![KingdomHero::default(); 2],
        towns: vec![KingdomTown::default()],
        selected: Selected::Hero(0),
        ..Kingdom::default()
    }
}

#[test]
fn the_panels_items_under_the_mouse() {
    let l = layout();
    let panels = PanelItems::new(&l);
    let all = |_| true;
    assert_eq!(panels.at([700, 98], all), Some(id::MINIMAP));
    assert_eq!(panels.at([607, 395], all), Some(2));
    // Inside the status window, the area under it (the window's items are its own panel's).
    assert_eq!(panels.at([700, 480], all), Some(2));
    // Over the status bar the chat's edit, added last.
    assert_eq!(panels.at([300, 565], all), Some(id::EDIT));
    // Over the resource bar, the frame between the panels: none.
    assert_eq!(panels.at([60, 580], all), None);
    assert_eq!(panels.at([604, 180], all), None);
    // A layout's picture without a size takes no mouse; a dialog without a status bar has no
    // edit.
    let mut bare = layout();
    bare.dialog.items.retain(|i| i.id() != id::STATUS_BAR);
    let picture = "kind = \"picture\"\nid = 999\npicture = \"x.pcx\"\nat = [0, 0]";
    bare.dialog.items.push(toml::from_str(picture).unwrap());
    let bare = PanelItems::new(&bare);
    assert_eq!(bare.at([300, 565], all), None);
    assert_eq!(bare.at([1, 1], all), None);
    // An item that takes no mouse is passed over.
    assert_eq!(panels.at([727, 212], |i| i != id::UNDERGROUND), None);
    // The hero list's slot: its frame, then its portrait.
    assert_eq!(panels.at([641, 228], all), Some(id::FRAMES));
    assert_eq!(
        panels.at([641, 228], |i| i != id::FRAMES),
        Some(id::PORTRAITS)
    );
}

#[test]
fn which_items_take_the_mouse() {
    let k = kingdom();
    let on = Buttons {
        move_hero: false,
        next_hero: true,
    };
    // Disabled on one level, enabled on two; the quest log takes it, disabled or not.
    assert!(!takes(id::UNDERGROUND, &k, on, 1));
    assert!(takes(id::UNDERGROUND, &k, on, 2));
    assert!(takes(id::QUESTS, &k, on, 1));
    assert!(!takes(id::MOVE_HERO, &k, on, 1));
    assert!(takes(id::NEXT_HERO, &k, on, 1));
    // The lists: a slot holding a hero or a town, the frames always.
    assert!(takes(id::PORTRAITS + 1, &k, on, 1));
    assert!(!takes(id::PORTRAITS + 2, &k, on, 1));
    assert!(takes(id::MOVEMENT + 1, &k, on, 1) && !takes(id::MANA + 2, &k, on, 1));
    assert!(takes(id::TOWN_PICTURES, &k, on, 1));
    assert!(!takes(id::TOWN_PICTURES + 1, &k, on, 1));
    assert!(takes(id::FRAMES + 4, &k, on, 1));
    assert!(takes(id::MINIMAP, &k, on, 1));
}

#[test]
fn the_panels_hint_by_its_item() {
    let k = kingdom();
    let f = facts();
    let hint = |item| panel_hint(item, &k, Some(&f));
    assert_eq!(hint(None), Hint::Empty);
    assert_eq!(hint(Some(id::MINIMAP)), Hint::Panel(0));
    assert_eq!(hint(Some(12)), Hint::Panel(11));
    assert_eq!(hint(Some(id::HEROES[1])), Hint::Panel(13));
    assert_eq!(hint(Some(id::TOWNS[0])), Hint::Panel(15));
    assert_eq!(hint(Some(id::TOWNS[1])), Hint::Panel(16));
    assert_eq!(hint(Some(id::STATUS_BAR)), Hint::Panel(26));
    for item in [0, id::EDIT, 37, 1001, 2000] {
        assert_eq!(hint(Some(item)), Hint::Empty, "{item}");
    }
    // The hero list's slots: portraits, bars and frames; past the list, nothing.
    let b = Hint::Hero {
        id: 20,
        class: 3,
        own_name: true,
    };
    for item in [16, 21, 26, 40] {
        assert_eq!(hint(Some(item)), b, "{item}");
    }
    assert_eq!(hint(Some(id::FRAMES + 2)), Hint::Empty);
    assert_eq!(
        hint(Some(id::TOWN_PICTURES)),
        Hint::Town { name: 5, kind: 0 }
    );
    assert_eq!(hint(Some(id::TOWN_PICTURES + 1)), Hint::Empty);
    // Without facts the slots name nothing.
    assert_eq!(panel_hint(Some(id::PORTRAITS), &k, None), Hint::Empty);
    assert_eq!(panel_hint(Some(id::TOWN_PICTURES), &k, None), Hint::Empty);
}

#[test]
fn a_stills_parked_hint() {
    let (_, mut live, _) = fixture(false);
    live.kingdom = kingdom();
    let l = layout();
    // The parking over the adventure options.
    assert_eq!(parked_hint(&l, &live, 1, [700, 300]), Hint::Panel(8));
    live.buttons = Some(Buttons {
        move_hero: true,
        next_hero: true,
    });
    assert_eq!(parked_hint(&l, &live, 1, [695, 276]), Hint::Panel(6));
}

#[test]
fn a_held_buttons_box() {
    // A button held follows the mouse in and out of its own box (E24 M3 T6b1).
    let panels = PanelItems::new(&layout());
    assert!(panels.holds(3, [679, 196]) && panels.holds(3, [710, 227]));
    assert!(!panels.holds(3, [711, 212]) && !panels.holds(3, [695, 228]));
    assert!(!panels.holds(999, [695, 212]));
}
