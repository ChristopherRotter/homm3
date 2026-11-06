//! Eagle Eye's message's tests (E17 M6 T5c), on an invented pack (the original's text never
//! enters git, ADR-0009).

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use h3_ui::text::Catalogue;
use h3_ui::text::pack::Pack;

use super::*;

const PACK: &str = r#"{
    "result": {
        "eagle-eye": {
            "learnt": "{{hero}} learns {{spell}}",
            "comma": "; ",
            "and": " & ",
            "stop": "!"
        }
    }
}"#;

fn catalogue() -> Catalogue {
    Catalogue {
        packs: vec![Pack::from_json("en", PACK).unwrap()],
    }
}

/// Picks numbered `numbers`, each named `S<number>`.
fn picks(numbers: &[u32]) -> Vec<Pick> {
    numbers
        .iter()
        .map(|&n| Pick {
            original: n,
            name: format!("S{n}"),
        })
        .collect()
}

fn told(numbers: &[u32]) -> Vec<Told> {
    boxes(
        &catalogue(),
        (true, Some("Orrin")),
        &picks(numbers),
        ("spellScr.def", [83, 61]),
    )
}

fn texts(boxes: &[Told]) -> Vec<String> {
    boxes.iter().map(|b| b.text.clone()).collect()
}

#[test]
fn one_spell_is_one_sentence_with_its_picture() {
    let b = told(&[53]);
    assert_eq!(texts(&b), ["Orrin learns S53!"]);
    assert_eq!(
        b[0].pictures,
        [Picture {
            def: "spellScr.def".to_string(),
            frame: 53,
            size: [83, 61],
            caption: "S53".to_string(),
        }]
    );
}

#[test]
fn the_spells_go_by_their_number_and_the_last_after_and() {
    // The set's order is the original's numbers (a sorted set), not the picks' order.
    let b = told(&[41, 3, 17]);
    assert_eq!(texts(&b), ["Orrin learns S3; S17 & S41!"]);
    let frames: Vec<u32> = b[0].pictures.iter().map(|p| p.frame).collect();
    assert_eq!(frames, [3, 17, 41]);
    assert_eq!(texts(&told(&[2, 1])), ["Orrin learns S1 & S2!"]);
}

#[test]
fn eight_spells_fill_a_box_the_eighth_after_and() {
    let eight: Vec<u32> = (1..=8).collect();
    assert_eq!(
        texts(&told(&eight)),
        ["Orrin learns S1; S2; S3; S4; S5; S6; S7 & S8!"]
    );
}

#[test]
fn a_ninth_spell_starts_a_box_of_its_own() {
    // The first box's eighth after " and ", then a new sentence of the box's own spells.
    let nine: Vec<u32> = (1..=9).collect();
    let b = told(&nine);
    assert_eq!(
        texts(&b),
        [
            "Orrin learns S1; S2; S3; S4; S5; S6; S7 & S8!",
            "Orrin learns S9!"
        ]
    );
    assert_eq!(b[0].pictures.len(), 8);
    assert_eq!(b[1].pictures.len(), 1);
    // Eleven: the second box's last after " and ".
    let eleven: Vec<u32> = (1..=11).collect();
    assert_eq!(texts(&told(&eleven))[1], "Orrin learns S9; S10 & S11!");
    // Sixteen and seventeen: two full boxes, then a third.
    let seventeen: Vec<u32> = (1..=17).collect();
    let b = told(&seventeen);
    assert_eq!(b.len(), 3);
    assert_eq!(
        b[1].text,
        "Orrin learns S9; S10; S11; S12; S13; S14; S15 & S16!"
    );
    assert_eq!(b[2].text, "Orrin learns S17!");
}

#[test]
fn nothing_without_picks_a_hero_or_the_winners_window() {
    let c = catalogue();
    let spells = ("spellScr.def", [83, 61]);
    assert!(told(&[]).is_empty());
    assert!(boxes(&c, (true, None), &picks(&[1]), spells).is_empty());
    // The loser's window (or no winner named) is followed by nothing.
    assert!(boxes(&c, (false, Some("Orrin")), &picks(&[1]), spells).is_empty());
}

#[test]
fn each_box_draws_a_pickup_sound_of_one_to_seven() {
    // The C library's sequence from the seed: deterministic, each 1 to 7.
    let drawn = pickups(12_345, 40);
    assert_eq!(drawn.len(), 40);
    assert!(drawn.iter().all(|n| (1..=7).contains(n)));
    assert_eq!(pickups(12_345, 3), drawn[..3]);
    assert!(pickups(7, 0).is_empty());
    // rand() from 0: 38, 7 719, 21 238 (the sequence's first draws), each 1 + its rest by 7.
    assert_eq!(pickups(0, 3), [4, 6, 1]);
}
