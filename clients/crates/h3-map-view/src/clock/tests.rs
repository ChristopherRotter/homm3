use super::*;

use alloc::vec;
use h3_ui::text::pack::Pack;

fn texts() -> Catalogue {
    let json = concat!(
        r#"{ "map": { "clock": { "system": "<S>:", "seconds_one": "1 second.", "#,
        r#""seconds_other": "{{count}} seconds.", "minute": "1 minute.", "#,
        r#""minutes": "{{minutes}} minutes.", "expired": "Out." } } }"#,
    );
    Catalogue {
        packs: vec![Pack::from_json("en", json).unwrap()],
    }
}

#[test]
fn the_first_warning_comes_at_half_a_minute_s_turn_and_a_quarter_of_a_longer_one() {
    assert_eq!(first(60_000), 30_000);
    assert_eq!(first(120_000), 30_000);
    assert_eq!(first(900_000), 225_000);
}

#[test]
fn the_warnings_follow_the_survey_s_schedule() {
    // One minute (recorded, `seat-clock`): 30 s at 30 059 (29 941 left), then to 10 s left,
    // then each second, none below 2 s.
    assert_eq!(next(29_941, 60_000), 19_941);
    assert_eq!(next(9_961, 60_000), 1_000);
    assert_eq!(next(2_000, 60_000), 1_000);
    assert_eq!(next(1_801, 60_000), 0);
    // Two minutes (`seat-clock-two`): to a minute left, then to 10 s.
    assert_eq!(next(89_941, 120_000), 29_941);
    assert_eq!(next(59_941, 120_000), 49_941);
    // Four minutes and more: half the limit while more than half is left.
    assert_eq!(next(179_941, 240_000), 120_000);
    assert_eq!(next(674_941, 900_000), 450_000);
    assert_eq!(next(224_941, 900_000), 164_941);
}

#[test]
fn a_warning_shows_seconds_a_minute_or_minutes_to_a_tenth() {
    assert_eq!(warning(29_941), Warning::Seconds(30));
    assert_eq!(warning(9_961), Warning::Seconds(9));
    assert_eq!(warning(1_801), Warning::Seconds(1));
    assert_eq!(warning(60_000), Warning::Seconds(60));
    assert_eq!(warning(65_000), Warning::Minute);
    assert_eq!(warning(89_941), Warning::Minutes(15));
    assert_eq!(warning(179_941), Warning::Minutes(30));
    assert_eq!(warning(674_941), Warning::Minutes(112));
    assert_eq!(warning(224_941), Warning::Minutes(37));
}

#[test]
fn minutes_are_the_f32_s_printed_to_a_tenth() {
    assert_eq!(tenths(0), 0);
    assert_eq!(tenths(60_000), 10);
    // 1.45 is no f32: its nearest lies above (1.4500000477), so 1.5.
    assert_eq!(tenths(87_000), 15);
    // 1.35's nearest lies above too (1.3500000238): 1.4.
    assert_eq!(tenths(81_000), 14);
    // 1.25 is one: half away from zero, 1.3.
    assert_eq!(tenths(75_000), 13);
    // 0.05's nearest lies above (0.0500000007): 0.1.
    assert_eq!(tenths(3_000), 1);
    assert_eq!(tenths(u32::MAX), 715_828);
}

#[test]
fn a_warning_s_line_is_the_system_s_mark_and_its_text() {
    let texts = texts();
    assert_eq!(text(&texts, Warning::Seconds(30)), "<S>:30 seconds.");
    assert_eq!(text(&texts, Warning::Seconds(1)), "<S>:1 second.");
    assert_eq!(text(&texts, Warning::Minute), "<S>:1 minute.");
    assert_eq!(text(&texts, Warning::Minutes(15)), "<S>:1.5 minutes.");
    assert_eq!(expired(&texts), "Out.");
}

#[test]
fn a_chat_line_lives_from_its_first_draw_one_leaving_a_draw() {
    let mut chat = Chat::default();
    chat.add(Warning::Seconds(30), 3);
    assert_eq!(
        chat.lines(),
        [ChatLine {
            warning: Warning::Seconds(30),
            drawn: None
        }]
    );
    assert!(chat.draw(100, 1_000));
    assert!(!chat.draw(1_100, 1_000));
    chat.add(Warning::Seconds(9), 3);
    chat.add(Warning::Seconds(8), 3);
    chat.add(Warning::Seconds(7), 3);
    // A fourth past the ring's three drops the oldest.
    assert_eq!(chat.shown(), [9, 8, 7].map(Warning::Seconds));
    assert!(chat.draw(1_200, 1_000));
    assert!(!chat.draw(2_200, 1_000));
    // Older than their life: one a draw.
    assert!(chat.draw(2_201, 1_000));
    assert_eq!(chat.shown().len(), 2);
    assert!(chat.draw(2_202, 1_000));
    assert!(chat.draw(2_203, 1_000));
    assert!(chat.shown().is_empty());
    assert!(!chat.draw(2_204, 1_000));
    let carried = Chat::of(&[ChatLine {
        warning: Warning::Minute,
        drawn: Some(-5),
    }]);
    assert_eq!(carried.shown(), [Warning::Minute]);
}

#[test]
fn no_clock_s_numbers_keep_lines_for_ever() {
    let art = ClockArt::default();
    assert_eq!(
        (art.tick, art.life, art.entries, art.deadline),
        (1, u32::MAX, 0, 0)
    );
    assert!(art.sound.is_empty());
}
