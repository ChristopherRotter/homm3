use super::*;

use alloc::string::ToString;
use h3_ui::text::pack::Pack;

use crate::hover::Content;

/// A catalogue of the finds' keys and names (the recorded boxes' texts where they show).
fn texts() -> Catalogue {
    let json = concat!(
        r#"{ "map": { "resources": { "amount": "{{amount}}" }, "find": { "#,
        r#""per-day": "{{amount}}/day", "level": "+1 Level", "count": "{{amount}} {{name}}", "#,
        r#""plus": "+{{amount}} {{name}}", "spell-points": "Spell Points", "#,
        r#""windmill": "MILL", "windmill-empty": "MILL0", "stone": "STONE", "#,
        r#""stone-visited": "STONE0", "axis": "AXIS", "axis-visited": "AXIS0", "#,
        r#""temple": "TEMPLE", "temple-visited": "TEMPLE0", "#,
        r#""witch-hut": "teaching you {{skill}}", "witch-hut-known": "a tome on {{skill}}", "#,
        r#""witch-hut-empty": "deserted", "shrine": "{{text}}'{{spell}}'.", "#,
        r#""shrine-1": "a simple spell - ", "shrine-2": "a spell - ", "#,
        r#""shrine-3": "a sophisticated spell - ", "shrine-known": " KNOWN", "#,
        r#""shrine-no-book": " NO BOOK", "shrine-wisdom": " WISDOM" }, "#,
        r#""turn": { "town-lost": "{{player}}, you have lost your last town.", "#,
        r#""days-left": "{{player}}, you only have {{days}} days left.", "#,
        r#""last-day": "{{player}}, this is your last day." } }, "#,
        r#""artifact": { "7": "Centaur's Axe" }, "spell": { "53": "Haste" }, "#,
        r#""player-colour": { "0": "red", "1": "blue" }, "#,
        r#""skill-level": { "0": "Basic", "1": "Advanced", "2": "Expert" }, "#,
        r#""secondary-skill": { "2": "Logistics" }, "#,
        r#""primary-skill": { "0": "Attack Skill", "2": "Spell Power" }, "#,
        r#""creature": { "4_one": "Griffin", "4_other": "Griffins" }, "#,
        r#""sign": { "5": "Proceed at Your Own Risk", "0": "Do Not Enter" } }"#,
    );
    Catalogue {
        packs: vec![Pack::from_json("en", json).unwrap()],
    }
}

fn art() -> FindArt {
    FindArt {
        resources: "resour82.def".to_string(),
        artifacts: "artifact.def".to_string(),
        spells: "spellscr.def".to_string(),
        crests: "crest58.def".to_string(),
        luck: "ilck82.def".to_string(),
        morale: "imrl82.def".to_string(),
        skills: "pskill.def".to_string(),
        secondary: "secsk82.def".to_string(),
        creatures: "twcrport.def".to_string(),
        gold: "resource.def".to_string(),
        experience: 1000,
    }
}

/// A picture's DEF, frame and caption.
fn shown(kind: u32, subtype: i32) -> (String, u32, String) {
    let s = picture(&art(), &texts(), kind, subtype).unwrap();
    (s.def, s.frame, s.caption)
}

fn of(def: &str, frame: u32, caption: &str) -> (String, u32, String) {
    (def.to_string(), frame, caption.to_string())
}

#[test]
fn every_picture_type_by_its_subtype() {
    // The resources: the frame the type, the caption the amount, none for 0, below -100 000 the
    // subtype + 100 000, else a mine's income (`find-windmill`'s Mercury 4).
    assert_eq!(shown(1, 4), of("resour82.def", 1, "4"));
    assert_eq!(shown(6, 0), of("resour82.def", 6, ""));
    assert_eq!(shown(6, -100_250), of("resour82.def", 6, "-250"));
    assert_eq!(shown(0, -2), of("resour82.def", 0, "2/day"));
    for kind in [7, 18, 19] {
        assert_eq!(shown(kind, 5), of("resour82.def", kind, ""));
    }
    // An artifact, a spell (`find-shrine`'s Haste), a crest (`turn-no-town`'s Red).
    assert_eq!(shown(8, 7), of("artifact.def", 7, "Centaur's Axe"));
    assert_eq!(shown(9, 53), of("spellscr.def", 53, "Haste"));
    assert_eq!(shown(10, 0), of("crest58.def", 0, "Red"));
    assert_eq!(shown(10, 9), of("crest58.def", 9, ""));
    // The luck's and the morale's frames 4, 3, 2 without a caption (`find-temple`'s 14).
    assert_eq!(shown(11, 1), of("ilck82.def", 4, ""));
    assert_eq!(shown(13, 1), of("ilck82.def", 2, ""));
    assert_eq!(shown(14, 1), of("imrl82.def", 4, ""));
    assert_eq!(shown(16, 1), of("imrl82.def", 2, ""));
    // The experience (`find-stone`'s 1 000), a level, none.
    assert_eq!(shown(17, 1000), of("pskill.def", 4, "1000"));
    assert_eq!(shown(17, -1), of("pskill.def", 4, "+1 Level"));
    assert_eq!(shown(17, 0), of("pskill.def", 4, ""));
    // A secondary skill: its level and name (`find-witch-hut`'s Basic Logistics, frame 9).
    assert_eq!(shown(20, 9), of("secsk82.def", 9, "Basic Logistics"));
    assert_eq!(shown(20, 11), of("secsk82.def", 11, "Expert Logistics"));
    // A creature: its portrait's frame + 2, the plural alone, one, several; none past 150.
    assert_eq!(shown(21, 4), of("twcrport.def", 6, "Griffins"));
    assert_eq!(shown(21, 4 | 1 << 16), of("twcrport.def", 6, "1 Griffin"));
    assert_eq!(
        shown(21, 4 | 12 << 16),
        of("twcrport.def", 6, "12 Griffins")
    );
    assert_eq!(shown(21, 151), of("twcrport.def", 153, ""));
    // The primary skills (`find-axis`'s +1 Spell Power), the count's high word 1, none.
    assert_eq!(shown(33, 1), of("pskill.def", 2, "+1 Spell Power"));
    assert_eq!(
        shown(31, 3 | 1 << 16),
        of("pskill.def", 0, "3 Attack Skill")
    );
    assert_eq!(shown(31, 0), of("pskill.def", 0, ""));
    // The spell points, type 36.
    assert_eq!(shown(35, 10), of("pskill.def", 5, "10 Spell Points"));
    assert_eq!(shown(36, 500), of("resource.def", 6, "500"));
    // A town's building, a type past 36: not built.
    assert_eq!(picture(&art(), &texts(), 22, 0), None);
    assert_eq!(picture(&art(), &texts(), 37, 0), None);
}

fn told(find: &Find) -> Told {
    find.told(&art(), &texts())
}

fn told_text(text: &str, pictures: &[(u32, i32)]) -> Told {
    Told {
        text: text.to_string(),
        pictures: pictures.to_vec(),
    }
}

#[test]
fn a_finds_facts_from_its_class_its_word_and_its_marks() {
    let visited = Some(Visit {
        known: true,
        visited: Some(true),
    });
    let sign = (None, 5);
    // `find-windmill`'s word 0xFFFE8011: Mercury, 4; `find-witch-hut`'s 0x4000: Logistics;
    // `find-shrine`'s 0xFF86A01F: Haste.
    assert_eq!(
        Find::of(112, Some(0xFFFE_8011), None, None, sign),
        Some(Find::Windmill {
            resource: 1,
            amount: 4
        })
    );
    // A word the find needs not given: none.
    for class in [112, 113, 88] {
        assert_eq!(Find::of(class, None, None, None, sign), None);
    }
    assert_eq!(
        Find::of(113, Some(0x4000), None, None, sign),
        Some(Find::WitchHut {
            skill: Some(2),
            known: false
        })
    );
    assert_eq!(
        Find::of(113, Some(0xFFFF_E000), None, None, sign),
        Some(Find::WitchHut {
            skill: None,
            known: false
        })
    );
    assert_eq!(
        Find::of(88, Some(0xFF86_A01F), None, None, sign),
        Some(Find::Shrine {
            level: 1,
            spell: 53,
            refusal: None
        })
    );
    assert_eq!(Find::of(90, Some(0xFFFF_E000), None, None, sign), None);
    // The hero's marks and what the contents say it knows.
    assert_eq!(
        Find::of(100, None, visited, None, sign),
        Some(Find::Stone { visited: true })
    );
    assert_eq!(
        Find::of(61, None, None, None, sign),
        Some(Find::Axis { visited: false })
    );
    assert_eq!(
        Find::of(96, None, visited, None, sign),
        Some(Find::Temple { visited: true })
    );
    let hut = Content::WitchHut {
        known: true,
        skill: Some((2, true)),
    };
    assert_eq!(
        Find::of(113, Some(0x4000), None, Some(&hut), sign),
        Some(Find::WitchHut {
            skill: Some(2),
            known: true
        })
    );
    let other = Content::WitchHut {
        known: true,
        skill: Some((3, true)),
    };
    assert_eq!(
        Find::of(113, Some(0x4000), None, Some(&other), sign),
        Some(Find::WitchHut {
            skill: Some(2),
            known: false
        })
    );
    let shrine = Content::Shrine {
        known: true,
        spell: Some((53, true)),
    };
    assert_eq!(
        Find::of(90, Some(0xFF86_A01F), None, Some(&shrine), sign),
        Some(Find::Shrine {
            level: 3,
            spell: 53,
            refusal: Some(Refusal::Known)
        })
    );
    // A sign's text, an empty one the default; another class none.
    assert_eq!(
        Find::of(91, None, None, None, (Some("Hi"), 30)),
        Some(Find::Sign {
            text: Some("Hi".to_string()),
            column: 30
        })
    );
    assert_eq!(
        Find::of(91, None, None, None, (Some(""), 5)),
        Some(Find::Sign {
            text: None,
            column: 5
        })
    );
    assert_eq!(Find::of(79, Some(0), None, None, sign), None);
}

#[test]
fn each_finds_box_and_its_visited_one() {
    // The recorded boxes (`[[message]]`): the text and the picture.
    let windmill = Find::Windmill {
        resource: 1,
        amount: 4,
    };
    assert_eq!(told(&windmill), told_text("MILL", &[(1, 4)]));
    assert_eq!(told(&windmill.visited()), told_text("MILL0", &[]));
    let lost = Find::Windmill {
        resource: -1,
        amount: 4,
    };
    assert_eq!(told(&lost), told_text("MILL0", &[]));
    let stone = Find::Stone { visited: false };
    assert_eq!(told(&stone), told_text("STONE", &[(17, 1000)]));
    assert_eq!(told(&stone.visited()), told_text("STONE0", &[]));
    let axis = Find::Axis { visited: false };
    assert_eq!(told(&axis), told_text("AXIS", &[(33, 1)]));
    assert_eq!(told(&axis.visited()), told_text("AXIS0", &[]));
    let temple = Find::Temple { visited: false };
    assert_eq!(told(&temple), told_text("TEMPLE", &[(14, 1)]));
    assert_eq!(told(&temple.visited()), told_text("TEMPLE0", &[]));
    let hut = Find::WitchHut {
        skill: Some(2),
        known: false,
    };
    assert_eq!(told(&hut), told_text("teaching you Logistics", &[(20, 9)]));
    assert_eq!(told(&hut.visited()), told_text("a tome on Logistics", &[]));
    let empty = Find::WitchHut {
        skill: None,
        known: false,
    };
    assert_eq!(told(&empty), told_text("deserted", &[]));
    assert_eq!(empty.visited(), empty);
    let shrine = Find::Shrine {
        level: 1,
        spell: 53,
        refusal: None,
    };
    assert_eq!(
        told(&shrine),
        told_text("a simple spell - 'Haste'.", &[(9, 53)])
    );
    assert_eq!(
        told(&shrine.visited()),
        told_text("a simple spell - 'Haste'. KNOWN", &[])
    );
    for (refusal, tail) in [(Refusal::NoBook, " NO BOOK"), (Refusal::Wisdom, " WISDOM")] {
        let refused = Find::Shrine {
            level: 2,
            spell: 53,
            refusal: Some(refusal),
        };
        let text = alloc::format!("a spell - 'Haste'.{tail}");
        assert_eq!(told(&refused), told_text(&text, &[]));
        assert_eq!(refused.visited(), refused);
    }
    // A sign: its own text, else RANDSIGN by its column mod 25 (`find-sign`'s column 5).
    let sign = Find::Sign {
        text: None,
        column: 30,
    };
    assert_eq!(told(&sign), told_text("Proceed at Your Own Risk", &[]));
    assert_eq!(sign.visited(), sign);
    let own = Find::Sign {
        text: Some("Hi".to_string()),
        column: 0,
    };
    assert_eq!(told(&own), told_text("Hi", &[]));
}

#[test]
fn a_turn_without_a_town_names_the_player() {
    // `turn-no-town`: the original's empty name (the comparisons'), the viewer's the colour's
    // (CLT-Q100 (b)); the crest in both.
    assert_eq!(
        town_lost(&texts(), 0, true),
        told_text(", you have lost your last town.", &[(10, 0)])
    );
    assert_eq!(
        town_lost(&texts(), 1, false),
        told_text("Blue, you have lost your last town.", &[(10, 1)])
    );
    assert_eq!(
        days_left(&texts(), 0, 6, true),
        told_text(", you only have 6 days left.", &[(10, 0)])
    );
    assert_eq!(
        days_left(&texts(), 0, 1, false),
        told_text("Red, this is your last day.", &[(10, 0)])
    );
}

#[test]
fn what_a_visit_gives_the_panels() {
    let windmill = Find::Windmill {
        resource: 1,
        amount: 4,
    };
    assert_eq!(windmill.gain().resource, Some((1, 4)));
    assert_eq!(windmill.visited().gain(), Gain::default());
    let lost = Find::Windmill {
        resource: -1,
        amount: 4,
    };
    assert_eq!(lost.gain(), Gain::default());
    assert_eq!(Find::Axis { visited: false }.gain().skills, [0, 0, 1, 0]);
    assert_eq!(Find::Axis { visited: true }.gain(), Gain::default());
    assert_eq!(Find::Temple { visited: false }.gain().morale, [1, 2]);
    assert_eq!(Find::Stone { visited: false }.gain(), Gain::default());
}
