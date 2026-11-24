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
        r#""witch-hut": "HUT: {{skill}}", "witch-hut-known": "HUT KNOWN: {{skill}}", "#,
        r#""witch-hut-full": "HUT FULL: {{skill}}", "pile": "Some {{resource}} lies here.", "#,
        r#""well": "WELL", "well-again": "WELL2", "well-full": "WELL0", "#,
        r#""reward-experience": "{{hero}}: experience.", "#,
        r#""reward-mana-lost": "{{hero}}: mana down.", "#,
        r#""reward-mana": "{{hero}}: mana up.", "#,
        r#""reward-morale-lost": "{{hero}}: morale down.", "#,
        r#""reward-morale": "{{hero}}: morale up.", "#,
        r#""reward-luck-lost": "{{hero}}: luck down.", "#,
        r#""reward-luck": "{{hero}}: luck up.", "#,
        r#""reward-treasure-lost": "{{hero}}: treasure lost.", "#,
        r#""reward-treasure": "{{hero}}: treasure.", "#,
        r#""reward-spell": "{{hero}}: a spell.", "reward-spells": "{{hero}}: spells.", "#,
        r#""reward-creature": "{{hero}} + one {{creature}}.", "#,
        r#""reward-creatures": "{{hero}} + {{creatures}}.", "#,
        r#""witch-hut-empty": "HUT0", "shrine": "{{text}}'{{spell}}'.", "#,
        r#""shrine-1": "SHRINE 1: ", "shrine-2": "SHRINE 2: ", "#,
        r#""shrine-3": "SHRINE 3: ", "shrine-known": " KNOWN", "#,
        r#""shrine-no-book": " NO BOOK", "shrine-wisdom": " WISDOM", "lighthouse": "LIGHT" }, "#,
        r#""turn": { "town-lost": "{{player}}: no town left.", "#,
        r#""days-left": "{{player}}: {{days}} days to take a town.", "#,
        r#""last-day": "{{player}}: the last day to take a town." } }, "#,
        r#""artifact": { "7": "Centaur's Axe" }, "spell": { "53": "Haste" }, "#,
        r#""player-colour": { "0": "red", "1": "blue" }, "#,
        r#""skill-level": { "0": "Basic", "1": "Advanced", "2": "Expert" }, "#,
        r#""secondary-skill": { "2": "Logistics" }, "#,
        r#""primary-skill": { "0": "Attack Skill", "2": "Spell Power" }, "#,
        r#""creature": { "4_one": "Griffin", "4_other": "Griffins", "1_one": "Halberdier", "#,
        r#""1_other": "Halberdiers" }, "resource": { "6": "Gold" }, "#,
        r#""artifact-event": { "7": "AXE." }, "#,
        r#""mine-event": { "0": "SAWMILL", "6": "GOLD MINE" }, "#,
        r#""sign": { "5": "SIGN 5", "0": "SIGN 0" } }"#,
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
        found_ms: 5000,
        dissolve: [192, 160, 224, 224],
        dissolve_ms: 65,
        pickups: vec!["pickup01.wav".to_string(), "pickup02.wav".to_string()],
        killfade: "killfade.wav".to_string(),
        artifact_skills: vec![[7, 2, 0, 0, 0], [12, 12, -3, 0, 0]],
        mine_income: [2, 1, 2, 1, 1, 1, 1000],
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
    find.told(&art(), &texts()).unwrap()
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
        Find::of([112, 0], Some(0xFFFE_8011), None, None, sign),
        Some(Find::Windmill {
            resource: 1,
            amount: 4
        })
    );
    // A word the find needs not given: none.
    for class in [112, 113, 88] {
        assert_eq!(Find::of([class, 0], None, None, None, sign), None);
    }
    assert_eq!(
        Find::of([113, 0], Some(0x4000), None, None, sign),
        Some(Find::WitchHut {
            skill: Some(2),
            known: false,
            full: false
        })
    );
    assert_eq!(
        Find::of([113, 0], Some(0xFFFF_E000), None, None, sign),
        Some(Find::WitchHut {
            skill: None,
            known: false,
            full: false
        })
    );
    assert_eq!(
        Find::of([88, 0], Some(0xFF86_A01F), None, None, sign),
        Some(Find::Shrine {
            level: 1,
            spell: 53,
            refusal: None
        })
    );
    assert_eq!(Find::of([90, 0], Some(0xFFFF_E000), None, None, sign), None);
    // The hero's marks and what the contents say it knows.
    assert_eq!(
        Find::of([100, 0], None, visited, None, sign),
        Some(Find::Stone { visited: true })
    );
    assert_eq!(
        Find::of([61, 0], None, None, None, sign),
        Some(Find::Axis { visited: false })
    );
    assert_eq!(
        Find::of([96, 0], None, visited, None, sign),
        Some(Find::Temple { visited: true })
    );
    let hut = Content::WitchHut {
        known: true,
        skill: Some((2, true)),
    };
    assert_eq!(
        Find::of([113, 0], Some(0x4000), None, Some(&hut), sign),
        Some(Find::WitchHut {
            skill: Some(2),
            known: true,
            full: false
        })
    );
    let other = Content::WitchHut {
        known: true,
        skill: Some((3, true)),
    };
    assert_eq!(
        Find::of([113, 0], Some(0x4000), None, Some(&other), sign),
        Some(Find::WitchHut {
            skill: Some(2),
            known: false,
            full: false
        })
    );
    let shrine = Content::Shrine {
        known: true,
        spell: Some((53, true)),
    };
    assert_eq!(
        Find::of([90, 0], Some(0xFF86_A01F), None, Some(&shrine), sign),
        Some(Find::Shrine {
            level: 3,
            spell: 53,
            refusal: Some(Refusal::Known)
        })
    );
    // A sign's text, an empty one the default; another class none.
    assert_eq!(
        Find::of([91, 0], None, None, None, (Some("Hi"), 30)),
        Some(Find::Sign {
            text: Some("Hi".to_string()),
            column: 30
        })
    );
    assert_eq!(
        Find::of([91, 0], None, None, None, (Some(""), 5)),
        Some(Find::Sign {
            text: None,
            column: 5
        })
    );
    assert_eq!(Find::of([80, 0], Some(0), None, None, sign), None);
    // Part B: a pile's resource by its subclass and amount by its word, gold's in hundreds; an
    // artifact by its subclass; a Well by the hero's day mark, its fullness given apart.
    assert_eq!(
        Find::of([79, 6], Some(700), None, None, sign),
        Some(Find::Pile {
            resource: 6,
            amount: 70_000
        })
    );
    assert_eq!(
        Find::of([79, 1], Some(5), None, None, sign),
        Some(Find::Pile {
            resource: 1,
            amount: 5
        })
    );
    assert_eq!(Find::of([79, 6], None, None, None, sign), None);
    assert_eq!(Find::of([79, 6], Some(u32::MAX), None, None, sign), None);
    assert_eq!(Find::of([79, u32::MAX], Some(5), None, None, sign), None);
    assert_eq!(
        Find::of([5, 7], None, None, None, sign),
        Some(Find::Artifact { artifact: 7 })
    );
    assert_eq!(Find::of([5, 70_000], None, None, None, sign), None);
    let well = Find::of([49, 0], None, visited, None, sign).unwrap();
    assert_eq!(
        well,
        Find::Well {
            visited: true,
            full: false
        }
    );
    assert_eq!(
        well.full(true),
        Find::Well {
            visited: true,
            full: true
        }
    );
    let axis = Find::Axis { visited: false };
    assert_eq!(axis.clone().full(true), axis);
}

#[test]
fn a_pile_an_artifact_and_a_well() {
    let (art, texts) = (art(), texts());
    // `find-pile`'s 70 000 gold: no box, kind 6 with ADVEVENT 113 and the name lower-cased,
    // the resource bar's gain, taken off the map.
    let pile = Find::Pile {
        resource: 6,
        amount: 70_000,
    };
    assert_eq!(pile.told(&art, &texts), None);
    assert_eq!(
        pile.found(&texts),
        Some(FoundStatus {
            resource: 6,
            amount: 70_000,
            text: "Some gold lies here.".to_string()
        })
    );
    assert_eq!(pile.gain(&art).resources[6], 70_000);
    assert!(pile.taken() && pile.visited() == pile);
    // A resource without a name: an empty one.
    let other = Find::Pile {
        resource: 3,
        amount: 2,
    };
    assert_eq!(other.found(&texts).unwrap().text, "Some  lies here.");
    // `find-artifact`'s Centaurs Axe: ARTEVENT's record and the artifact's picture, +2 Attack
    // shown at the pickup's end; one outside the table gives none.
    let axe = Find::Artifact { artifact: 7 };
    assert_eq!(told(&axe), told_text("AXE.", &[(8, 7)]));
    assert_eq!(axe.gain(&art).skills, [2, 0, 0, 0]);
    assert_eq!(art.skills_of(12), [12, -3, 0, 0]);
    assert_eq!(Find::Artifact { artifact: 9 }.gain(&art), Gain::default());
    assert!(axe.taken() && axe.found(&texts).is_none() && axe.visited() == axe);
    // The Magic Well: 77 below the maximum, 79 at it (no mark: the same again), 78 drunk today.
    let well = Find::Well {
        visited: false,
        full: false,
    };
    assert_eq!(told(&well), told_text("WELL", &[]));
    assert_eq!(told(&well.visited()), told_text("WELL2", &[]));
    let full = Find::Well {
        visited: false,
        full: true,
    };
    assert_eq!(told(&full), told_text("WELL0", &[]));
    assert_eq!(full.visited(), full);
    assert!(!well.taken() && well.gain(&art) == Gain::default());
    // The maximum: 10 times the Knowledge, 1 to 99.
    assert!(full_mana(10, 0) && !full_mana(9, 1) && full_mana(20, 2) && !full_mana(989, 120));
    // The pickup's sounds in turn; none without.
    assert_eq!(pickup(&art, 0), Some("pickup01.wav"));
    assert_eq!(pickup(&art, 3), Some("pickup02.wav"));
    let silent = FindArt {
        pickups: Vec::new(),
        ..art
    };
    assert_eq!(pickup(&silent, 0), None);
}

/// A map's rewards (the writer's model).
fn given() -> Rewards {
    Rewards {
        luck: 1,
        mana: 5,
        resources: [0, 0, 0, 0, 0, 0, 500],
        creatures: vec![h3_map::model::Stack {
            creature: 1,
            count: 5,
        }],
        ..Rewards::default()
    }
}

#[test]
fn an_events_boxes_as_find_event_recorded() {
    // Luck 1, mana 5, 500 gold, 5 Halberdiers: three boxes, the mana pushed first, the gold
    // shown as its box opens, the rest after the last.
    let (boxes, gain) = rewards(&given(), &texts(), "Mephala", "");
    let told: Vec<(&str, &[(u32, i32)])> = boxes
        .iter()
        .map(|b| (b.told.text.as_str(), b.told.pictures.as_slice()))
        .collect();
    assert_eq!(
        told,
        [
            ("Mephala: mana up.", &[(35, 5), (11, 1)][..]),
            ("Mephala: treasure.", &[(6, 500)][..]),
            ("Mephala + Halberdiers.", &[(21, 5 << 16 | 1)][..]),
        ]
    );
    assert_eq!(boxes[0].shows, Gain::default());
    assert_eq!(boxes[1].shows.resources, [0, 0, 0, 0, 0, 0, 500]);
    assert_eq!((gain.luck, gain.mana, gain.morale), (1, 5, [0, 0]));
    assert_eq!(
        gain.army[0],
        Some(Stack {
            creature: 1,
            count: 5
        })
    );
    // The map's text in the first box alone, every later box its first part's.
    let (boxes, _) = rewards(&given(), &texts(), "Mephala", "An event.");
    assert_eq!(boxes[0].told.text, "An event.");
    assert_eq!(boxes[1].told.text, "Mephala: treasure.");
}

#[test]
fn every_part_of_an_event() {
    let rewards_of = |r: &Rewards, text: &str| rewards(r, &texts(), "Orrin", text);
    // The experience, the primary and secondary skills share the first box's 175; the losses'
    // texts and pictures; a loss of a resource below -100 000.
    let lost = Rewards {
        experience: 500,
        primary: [1, 0, 2, 0],
        skills: vec![(2, 1), (2, 3)],
        mana: -3,
        morale: -1,
        luck: -2,
        resources: [-5, 0, 0, 0, 0, 0, 0],
        ..Rewards::default()
    };
    let (boxes, gain) = rewards_of(&lost, "");
    assert_eq!(
        boxes[0].told,
        told_text(
            "Orrin: experience.",
            &[
                (17, 500),
                (31, 1),
                (33, 2),
                (20, 9),
                (20, 11),
                (35, -3),
                (16, -1),
                (13, -2)
            ]
        )
    );
    assert_eq!(
        boxes[1].told,
        told_text("Orrin: treasure lost.", &[(0, -100_005)])
    );
    assert_eq!(boxes[1].shows.resources[0], -5);
    assert_eq!(
        (gain.skills, gain.morale, gain.luck, gain.mana),
        ([1, 0, 2, 0], [-1, -1], -2, -3)
    );
    // Each part's own text when it opens a box: the morale's, the luck's, the mana lost's.
    for (r, text) in [
        (
            Rewards {
                morale: 2,
                ..Rewards::default()
            },
            "Orrin: morale up.",
        ),
        (
            Rewards {
                morale: -2,
                ..Rewards::default()
            },
            "Orrin: morale down.",
        ),
        (
            Rewards {
                luck: -1,
                ..Rewards::default()
            },
            "Orrin: luck down.",
        ),
        (
            Rewards {
                mana: -1,
                ..Rewards::default()
            },
            "Orrin: mana down.",
        ),
    ] {
        assert_eq!(rewards_of(&r, "").0[0].told.text, text);
    }
    // Nine artifacts: a box of eight (183), the ninth in a box of its own with its text again.
    let many = Rewards {
        artifacts: (0..9).collect(),
        ..Rewards::default()
    };
    let (boxes, _) = rewards_of(&many, "Look.");
    assert_eq!(boxes.len(), 2);
    assert_eq!(boxes[0].told.text, "Look.");
    assert_eq!(boxes[0].told.pictures.len(), 8);
    assert_eq!(boxes[1].told, told_text("Orrin: treasure.", &[(8, 8)]));
    // One spell keeps the map's text; two tell 188 over it (the original's: a question).
    let one = Rewards {
        spells: vec![53],
        ..Rewards::default()
    };
    assert_eq!(rewards_of(&one, "Read.").0[0].told.text, "Read.");
    assert_eq!(rewards_of(&one, "").0[0].told.text, "Orrin: a spell.");
    let two = Rewards {
        spells: vec![53, 54],
        ..Rewards::default()
    };
    assert_eq!(
        rewards_of(&two, "Read.").0[0].told,
        told_text("Orrin: spells.", &[(9, 53), (9, 54)])
    );
    // One creature (185, the singular); a creature past 150 has no name; nothing given, no box.
    let one = Rewards {
        creatures: vec![
            h3_map::model::Stack {
                creature: 4,
                count: 1,
            },
            h3_map::model::Stack {
                creature: 151,
                count: 2,
            },
        ],
        ..Rewards::default()
    };
    let (boxes, gain) = rewards_of(&one, "");
    assert_eq!(
        boxes[0].told,
        told_text(
            "Orrin + one Griffin.",
            &[(21, 1 << 16 | 4), (21, 2 << 16 | 151)]
        )
    );
    assert_eq!(gain.army.iter().flatten().count(), 2);
    let none = rewards_of(&Rewards::default(), "Nothing.");
    assert!(none.0.is_empty() && none.1 == Gain::default());
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
        full: false,
    };
    assert_eq!(told(&hut), told_text("HUT: Logistics", &[(20, 9)]));
    assert_eq!(told(&hut.visited()), told_text("HUT KNOWN: Logistics", &[]));
    // Eight skills (173), the skill known first (172).
    let full = Find::WitchHut {
        skill: Some(2),
        known: false,
        full: true,
    };
    assert_eq!(told(&full), told_text("HUT FULL: Logistics", &[]));
    assert_eq!(
        told(&full.visited()),
        told_text("HUT KNOWN: Logistics", &[])
    );
    let empty = Find::WitchHut {
        skill: None,
        known: false,
        full: false,
    };
    assert_eq!(told(&empty), told_text("HUT0", &[]));
    assert_eq!(empty.visited(), empty);
    let shrine = Find::Shrine {
        level: 1,
        spell: 53,
        refusal: None,
    };
    assert_eq!(told(&shrine), told_text("SHRINE 1: 'Haste'.", &[(9, 53)]));
    assert_eq!(
        told(&shrine.visited()),
        told_text("SHRINE 1: 'Haste'. KNOWN", &[])
    );
    for (refusal, tail) in [(Refusal::NoBook, " NO BOOK"), (Refusal::Wisdom, " WISDOM")] {
        let refused = Find::Shrine {
            level: 2,
            spell: 53,
            refusal: Some(refusal),
        };
        let text = alloc::format!("SHRINE 2: 'Haste'.{tail}");
        assert_eq!(told(&refused), told_text(&text, &[]));
        assert_eq!(refused.visited(), refused);
    }
    // A sign: its own text, else RANDSIGN by its column mod 25 (`find-sign`'s column 5).
    let sign = Find::Sign {
        text: None,
        column: 30,
    };
    assert_eq!(told(&sign), told_text("SIGN 5", &[]));
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
        told_text(": no town left.", &[(10, 0)])
    );
    assert_eq!(
        town_lost(&texts(), 1, false),
        told_text("Blue: no town left.", &[(10, 1)])
    );
    assert_eq!(
        days_left(&texts(), 0, 6, true),
        told_text(": 6 days to take a town.", &[(10, 0)])
    );
    assert_eq!(
        days_left(&texts(), 0, 1, false),
        told_text("Red: the last day to take a town.", &[(10, 0)])
    );
}

#[test]
fn what_a_visit_gives_the_panels() {
    let windmill = Find::Windmill {
        resource: 1,
        amount: 4,
    };
    let art = art();
    assert_eq!(windmill.gain(&art).resources, [0, 4, 0, 0, 0, 0, 0]);
    assert_eq!(windmill.visited().gain(&art), Gain::default());
    let lost = Find::Windmill {
        resource: -1,
        amount: 4,
    };
    assert_eq!(lost.gain(&art), Gain::default());
    assert_eq!(
        Find::Axis { visited: false }.gain(&art).skills,
        [0, 0, 1, 0]
    );
    assert_eq!(Find::Axis { visited: true }.gain(&art), Gain::default());
    assert_eq!(Find::Temple { visited: false }.gain(&art).morale, [1, 2]);
    // The Learning Stone's experience (E24 M4 T2c: its levels' dialogs after its box).
    assert_eq!(
        Find::Stone { visited: false }.gain(&art),
        Gain {
            experience: art.experience,
            ..Gain::default()
        }
    );
    assert_eq!(Find::Stone { visited: true }.gain(&art), Gain::default());
}

#[test]
fn a_flag_raised_at_a_mine_and_a_lighthouse() {
    // E24 M4 T2d1 (design §5.6): a neutral Sawmill without guards (`flag-mine`; Blue's Ore Pit
    // alike, `flag-mine-blue`): MINEEVNT's record of its resource with the wood at its "2/day";
    // a Gold Mine's 1000/day.
    let mine = |resource: u8, guards: Vec<(u32, u32)>| Content::Mine { resource, guards };
    let sawmill = Find::flag_of([53, 0], Some(&mine(0, vec![])), false).unwrap();
    assert_eq!(sawmill, Find::Mine { resource: 0 });
    assert!(sawmill.flagged() && !sawmill.taken());
    assert_eq!(told(&sawmill), told_text("SAWMILL", &[(0, -2)]));
    assert_eq!(shown(0, -2), of("resour82.def", 0, "2/day"));
    let gold = Find::Mine { resource: 6 };
    assert_eq!(told(&gold), told_text("GOLD MINE", &[(6, -1000)]));
    assert_eq!(shown(6, -1000), of("resour82.def", 6, "1000/day"));
    // A resource past the incomes has none (no caption), its record's text none.
    assert_eq!(told(&Find::Mine { resource: 7 }), told_text("", &[(7, 0)]));
    // The panels show nothing of it; told once (`flagged`: the visitor's after it).
    assert_eq!(sawmill.visited(), sawmill);
    assert_eq!(sawmill.gain(&art()), Gain::default());
    assert_eq!(sawmill.found(&texts()), None);
    // The Lighthouse (`flag-lighthouse`): ADVEVENT 69 alone.
    let light = Find::flag_of([42, 0], None, false).unwrap();
    assert_eq!(light, Find::Lighthouse);
    assert!(light.flagged());
    assert_eq!(told(&light), told_text("LIGHT", &[]));
    assert_eq!(light.visited(), light);
    // Of the visitor's team, guarded, the abandoned mine, without contents, another class: none.
    assert_eq!(Find::flag_of([53, 0], Some(&mine(0, vec![])), true), None);
    assert_eq!(Find::flag_of([42, 0], None, true), None);
    let guarded = mine(2, vec![(56, 10)]);
    assert_eq!(Find::flag_of([53, 2], Some(&guarded), false), None);
    assert_eq!(Find::flag_of([53, 7], Some(&mine(6, vec![])), false), None);
    assert_eq!(Find::flag_of([53, 0], None, false), None);
    assert_eq!(Find::flag_of([112, 0], None, false), None);
    assert!(!Find::Stone { visited: false }.flagged());
}
