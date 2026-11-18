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
