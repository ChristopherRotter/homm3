use super::*;

use alloc::string::ToString;
use alloc::vec;
use h3_ui::text::pack::Pack;

fn texts() -> Catalogue {
    let json = concat!(
        r#"{ "map": { "turn": { "end": "End?", "month": "Month of the {{name}}.", "#,
        r#""month-creature": "Month of the {{creature}}: {{creature}}!", "plague": "PLAGUE", "#,
        r#""week": "Week of the {{name}}.", "#,
        r#""week-creature": "Week of the {{creature}}: {{creature}} +5", "#,
        r#""week-deity": "Week of the {{creature}}: {{creature}} {{growth}}, {{other}} {{growth}}" } }, "#,
        r#""week-name": { "4": "Rat" }, "month-name": { "6": "Locust" }, "#,
        r#""creature": { "4_one": "griffin", "4_other": "griffins", "42_one": "Imp", "#,
        r#""43_one": "Familiar", "127_one": "Storm Elemental" } }"#,
    );
    Catalogue {
        packs: vec![Pack::from_json("en", json).unwrap()],
    }
}

fn art() -> TurnArt {
    TurnArt {
        asked: 3000,
        week: "newweek.wav".to_string(),
        month: "newmonth.wav".to_string(),
        deity: [42, 43],
        town_days: 7,
        computer: Vec::new(),
    }
}

fn date(month: u16, week: u16, day: u16) -> Date {
    Date { month, week, day }
}

#[test]
fn a_weeks_first_day_but_the_games_first() {
    assert!(!starts_week(date(1, 1, 1)));
    assert!(starts_week(date(1, 2, 1)));
    assert!(starts_week(date(2, 1, 1)));
    assert!(!starts_week(date(2, 1, 2)));
    assert_eq!(sound(&art(), date(1, 2, 1)), "newweek.wav");
    assert_eq!(sound(&art(), date(2, 1, 1)), "newmonth.wav");
    assert_eq!(week_key(4), "week-name.4");
    assert_eq!(month_key(6), "month-name.6");
    assert_eq!(question(&texts()), "End?");
}

#[test]
fn the_weeks_message_by_its_kind() {
    let (t, a) = (texts(), art());
    let week = |w, m, d| message(&t, &a, (d, w, m), 15);
    // Not a week's first day: none.
    assert_eq!(
        week(Calendar::Named(4), Calendar::Named(0), date(1, 1, 7)),
        None
    );
    // A normal week: its name; a creature's: its name upper-cased first, twice (`turn-week`,
    // `turn-week-creature`).
    let second = date(1, 2, 1);
    assert_eq!(
        week(Calendar::Named(4), Calendar::Named(0), second).as_deref(),
        Some("Week of the Rat.")
    );
    assert_eq!(
        week(Calendar::Creature(4), Calendar::Named(0), second).as_deref(),
        Some("Week of the Griffin: Griffin +5")
    );
    assert_eq!(
        week(Calendar::Creature(127), Calendar::Named(0), second).as_deref(),
        Some("Week of the Storm Elemental: Storm Elemental +5")
    );
    // The Deity's (no recording has it): the Imp's name twice, the growth signed, the
    // Familiar's, the growth again.
    assert_eq!(
        week(Calendar::Special, Calendar::Named(0), second).as_deref(),
        Some("Week of the Imp: Imp +15, Familiar +15")
    );
    assert_eq!(
        message(&t, &a, (second, Calendar::Special, Calendar::Named(0)), -2).as_deref(),
        Some("Week of the Imp: Imp -2, Familiar -2")
    );
    // A record's (E24 M5 T4c): the engine's creatures and growth, neither the tables' nor the
    // growth given; an upgrade the level lacks named as its creature; none known, empty names.
    let deity = |creatures| Calendar::Deity {
        creatures,
        bonus: 7,
    };
    let played = |creatures| message(&t, &a, (second, deity(creatures), Calendar::Named(0)), -2);
    assert_eq!(
        played([Some(43), Some(42)]).as_deref(),
        Some("Week of the Familiar: Familiar +7, Imp +7")
    );
    assert_eq!(
        played([Some(42), None]).as_deref(),
        Some("Week of the Imp: Imp +7, Imp +7")
    );
    assert_eq!(
        played([None, None]).as_deref(),
        Some("Week of the :  +7,  +7")
    );
}

#[test]
fn a_months_first_week_shows_the_months_with_a_normal_week() {
    let (t, a) = (texts(), art());
    let first = date(2, 1, 1);
    let month = |w, m| message(&t, &a, (first, w, m), 15);
    assert_eq!(
        month(Calendar::Named(4), Calendar::Named(6)).as_deref(),
        Some("Month of the Locust.")
    );
    assert_eq!(
        month(Calendar::Named(4), Calendar::Creature(4)).as_deref(),
        Some("Month of the Griffin: Griffin!")
    );
    assert_eq!(
        month(Calendar::Named(4), Calendar::Special).as_deref(),
        Some("PLAGUE")
    );
    // A creature's week at a month's start: the week's message.
    assert_eq!(
        month(Calendar::Creature(4), Calendar::Named(6)).as_deref(),
        Some("Week of the Griffin: Griffin +5")
    );
}

#[test]
fn the_first_letter_upper_cased() {
    assert_eq!(upper_first("imp"), "Imp");
    assert_eq!(upper_first("Éclair"), "Éclair");
    assert_eq!(upper_first(""), "");
}

#[test]
fn the_tables_section_reads_from_toml() {
    let section: TurnArt = toml::from_str(
        r#"
        asked = 3000
        week = "newweek.wav"
        month = "newmonth.wav"
        deity = [42, 43]
        town_days = 7
        "#,
    )
    .unwrap();
    assert_eq!(section, art());
}

/// A record's other player's turn (E24 M4 T4c): Blue's hero 7 seen from (5, 5) walking east,
/// then a step out of sight, then one back into sight; hero 9 not shown at the start. Its boats
/// (E24 M4 T4c2): a boarding kept by its step's place, one out of sight's left out, a landing.
#[test]
fn a_record_s_other_turn_as_walks() {
    use crate::engine::{OtherEnd, OtherStep, OtherTurn, OtherWalk};
    use crate::live::{Facing, LiveBoat};
    use crate::timeline::Sea;
    let at = |x, y| Pos { x, y, z: 0 };
    let hero = |x, owner| LiveHero {
        at: at(x, 5),
        class: 1,
        owner,
        facing: Facing::ALL[2],
        boat: None,
    };
    let map = [hero(1, 0), hero(5, 1)];
    let ids = record_heroes(&[(3, at(1, 5)), (7, at(5, 5)), (8, at(30, 30))], &map);
    assert_eq!(ids, vec![(3, 0), (7, 1)]);
    let step = |from: Option<Pos>, to: Option<Pos>| OtherStep {
        from,
        to,
        sea: None,
    };
    let walk = |hero, steps: Vec<OtherStep>| OtherWalk {
        hero,
        ends: vec![OtherEnd {
            after: steps.len(),
            taken: vec![at(9, 4)],
            ..OtherEnd::default()
        }],
        steps,
    };
    let mut other = OtherTurn {
        turn: 1,
        player: 1,
        open: false,
        walks: vec![
            walk(
                7,
                vec![
                    step(Some(at(5, 5)), Some(at(6, 5))),
                    step(Some(at(6, 5)), Some(at(7, 6))),
                    step(Some(at(7, 6)), None),
                    step(None, Some(at(9, 6))),
                ],
            ),
            walk(7, vec![step(Some(at(9, 6)), Some(at(9, 5)))]),
        ],
    };
    let boat = LiveBoat {
        at: at(9, 6),
        kind: 1,
        facing: Facing::ALL[0],
    };
    other.walks[0].steps[1].sea = Some(Sea::Board(1));
    other.walks[0].steps[2].sea = Some(Sea::Board(2));
    other.walks[1].steps[0].sea = Some(Sea::Land(boat));
    let walks = record_walks(&other, &ids, &map, &|at| (at.x == 9).then_some(4)).unwrap();
    let to = |x, y, facing| Hero {
        at: at(x, y),
        facing,
    };
    let taken = |after| End {
        after,
        taken: vec![(at(9, 4), 4)],
        seen: true,
        ..End::default()
    };
    assert_eq!(
        walks,
        vec![
            Steps {
                hero: 1,
                slot: 0,
                steps: vec![
                    to(6, 5, Direction::East),
                    to(7, 6, Direction::SouthEast),
                    to(9, 6, Direction::SouthEast),
                ],
                unseen: vec![2],
                sea: vec![(1, Sea::Board(1))],
                ends: vec![taken(3)],
                seen: Some(true),
            },
            Steps {
                hero: 1,
                slot: 0,
                steps: vec![to(9, 5, Direction::North)],
                unseen: Vec::new(),
                sea: vec![(0, Sea::Land(boat))],
                ends: vec![taken(1)],
                seen: Some(true),
            },
        ]
    );
    // A hero the start does not show, a step between tiles apart: refused.
    let unknown = OtherTurn {
        walks: vec![walk(9, vec![])],
        ..other.clone()
    };
    assert!(record_walks(&unknown, &ids, &map, &|_| None).is_err());
    let apart = OtherTurn {
        walks: vec![walk(7, vec![step(Some(at(5, 5)), Some(at(8, 5)))])],
        ..other.clone()
    };
    assert!(record_walks(&apart, &ids, &map, &|_| None).is_err());
    // A first step into sight faces as the map hero did.
    let into = OtherTurn {
        walks: vec![walk(7, vec![step(None, Some(at(6, 5)))])],
        ..other
    };
    let facing = hero_of(&map[1]).facing;
    assert_eq!(
        record_walks(&into, &ids, &map, &|_| None).unwrap()[0].steps,
        vec![to(6, 5, facing)]
    );
}

#[test]
fn a_record_s_moves_ends_by_the_steps_kept() {
    // E24 M4 T6 (the reviewer's R1-2, R1-3, R1-5): each move's end by the steps kept (one out of
    // sight left out), what it takes, removes and flags by the objects' indexes and the heroes
    // it takes off the map by the map's (one the start does not show left out); the record's
    // judgement of the hero's tile at its choice (its first step's start told) and at each end
    // (its last step's end told).
    use crate::engine::{OtherEnd, OtherStep, OtherTurn, OtherWalk};
    use crate::live::Facing;
    let at = |x, y| Pos { x, y, z: 0 };
    let map = [LiveHero {
        at: at(5, 5),
        class: 1,
        owner: 1,
        facing: Facing::ALL[2],
        boat: None,
    }];
    let ids = [(7, 0)];
    let step = |from: Option<Pos>, to: Option<Pos>| OtherStep {
        from,
        to,
        sea: None,
    };
    let other = OtherTurn {
        turn: 0,
        player: 1,
        open: false,
        walks: vec![OtherWalk {
            hero: 7,
            steps: vec![
                step(Some(at(5, 5)), Some(at(6, 5))),
                step(Some(at(6, 5)), None),
                step(None, Some(at(8, 5))),
                step(Some(at(8, 5)), Some(at(9, 5))),
            ],
            ends: vec![
                OtherEnd {
                    after: 2,
                    taken: vec![at(7, 4)],
                    ..OtherEnd::default()
                },
                OtherEnd {
                    after: 4,
                    removed: vec![at(10, 5)],
                    flagged: vec![(at(11, 5), 3)],
                    lost: vec![7, 9],
                    ..OtherEnd::default()
                },
            ],
        }],
    };
    let index = |at: Pos| Some(usize::from(at.x));
    let [walk] = &record_walks(&other, &ids, &map, &index).unwrap()[..] else {
        panic!("one walk");
    };
    assert_eq!(walk.steps.len(), 3);
    assert_eq!((walk.unseen.clone(), walk.seen), (vec![1], Some(true)));
    assert_eq!(
        walk.ends,
        [
            End {
                after: 1,
                taken: vec![(at(7, 4), 7)],
                seen: false,
                ..End::default()
            },
            End {
                after: 3,
                removed: vec![10],
                flags: vec![(11, 3)],
                lost: vec![0],
                seen: true,
                ..End::default()
            },
        ]
    );
    assert_eq!(map_heroes(&[9, 7], &ids), [0]);
    // An end before any step (none from a record's events, the type allows it): seen as the
    // walk's choice is.
    let mut first = other;
    first.walks[0].ends = vec![OtherEnd::default()];
    let [walk] = &record_walks(&first, &ids, &map, &index).unwrap()[..] else {
        panic!("one walk");
    };
    assert_eq!((walk.ends[0].after, walk.ends[0].seen), (0, true));
}
