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
/// then a step out of sight, then one back into sight; hero 9 not shown at the start.
#[test]
fn a_record_s_other_turn_as_walks() {
    use crate::engine::{OtherStep, OtherTurn, OtherWalk};
    use crate::live::Facing;
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
    let walk = |hero, steps| OtherWalk { hero, steps };
    let other = OtherTurn {
        turn: 1,
        player: 1,
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
    let walks = record_walks(&other, &ids, &map).unwrap();
    let to = |x, y, facing| Hero {
        at: at(x, y),
        facing,
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
            },
            Steps {
                hero: 1,
                slot: 0,
                steps: vec![to(9, 5, Direction::North)],
                unseen: Vec::new(),
            },
        ]
    );
    // A hero the start does not show, a step between tiles apart: refused.
    let unknown = OtherTurn {
        walks: vec![walk(9, vec![])],
        ..other.clone()
    };
    assert!(record_walks(&unknown, &ids, &map).is_err());
    let apart = OtherTurn {
        walks: vec![walk(7, vec![step(Some(at(5, 5)), Some(at(8, 5)))])],
        ..other.clone()
    };
    assert!(record_walks(&apart, &ids, &map).is_err());
    // A first step into sight faces as the map hero did.
    let into = OtherTurn {
        walks: vec![walk(7, vec![step(None, Some(at(6, 5)))])],
        ..other
    };
    let facing = hero_of(&map[1]).facing;
    assert_eq!(
        record_walks(&into, &ids, &map).unwrap()[0].steps,
        vec![to(6, 5, facing)]
    );
}
