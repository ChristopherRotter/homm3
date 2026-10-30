use super::*;
use alloc::string::ToString;
use alloc::vec;

use h3_ui::text::pack::Pack;

use crate::hover::{Facts, Hint, Hover};
use crate::objects::DrawLists;
use crate::sound::tests::{map_with, template};
use crate::tables::{self, Tables};

fn tables() -> Tables {
    let text = include_str!("../../../../../map/assets/base.toml");
    tables::load(&[tables::parse(text).unwrap()], "base").unwrap()
}

/// The facts of `objects` objects with the contents' lines `lines` (no words, nothing known).
fn facts(lines: &str, objects: usize) -> Option<Facts> {
    let guarded = ["\"00000000\""; 8].join(", ");
    let zeros = vec!["0"; objects].join(", ");
    let text = alloc::format!(
        "words = [{zeros}]\narmed = [{zeros}]\ntowns = []\nhero_ids = []\nhero_list = []\n\
         town_list = []\nteams = [0, 1, 2, 3, 4, 5, 6, 7]\nguarded = [{guarded}]\nreach = []\n\
         day_points = 0\n{lines}\n"
    );
    let value: toml::Value = toml::from_str(&text).unwrap();
    Facts::read(&value, objects, 0, 8, 1)
}

const LINES: &str = r#"
mines = [[0, 3, [[46, 120]]], [1, 0, []]]
banks = [[2, 1, [[0, 10], [0, 5]]], [3, 0, []]]
shrines = [[4, 1, -1, 0], [5, 0, 3, 1]]
trees = [[6, 1, -1, -1], [7, 0, 1, 0]]
witch_huts = [[8, 0, 2, 1], [9, 1, -1, 0]]
quests = [
    [10, 1, 4, 1, 0, [10], []],
    [11, 1, 4, 2, 1, [1, 0, -2, 3], []],
    [12, 1, 4, 3, 2, [16, 1], []],
    [13, 0, 4, 4, 0, [13], []],
    [14, 1, 4, 5, 0, [7, 8], []],
    [15, 1, 4, 6, 0, [], [[0, 1], [13, 2]]],
    [16, 1, 4, 7, 0, [5, 0, 0, 0, 0, 0, 1000], []],
    [17, 1, 4, 8, 0, [16, 0], []],
    [18, 1, 4, 9, 0, [1], []],
    [19, 1, 0, 0, 0, [], []],
]
"#;

#[test]
fn the_contents_read_from_the_live_parts_table() {
    let f = facts(LINES, 20).unwrap();
    let c = |o: usize| f.contents[&o].clone();
    assert_eq!(
        c(0),
        Content::Mine {
            resource: 3,
            guards: vec![(46, 120)]
        }
    );
    assert_eq!(
        c(2),
        Content::Bank {
            visited: true,
            guards: vec![(0, 10), (0, 5)]
        }
    );
    assert_eq!(
        c(5),
        Content::Shrine {
            known: false,
            spell: Some((3, true))
        }
    );
    assert_eq!(
        (c(6), c(7)),
        (
            Content::Tree {
                known: true,
                price: None,
                visited: None
            },
            Content::Tree {
                known: false,
                price: Some(1),
                visited: Some(false)
            }
        )
    );
    assert_eq!(
        (c(8), c(9)),
        (
            Content::WitchHut {
                known: false,
                skill: Some((2, true))
            },
            Content::WitchHut {
                known: true,
                skill: None
            }
        )
    );
    let mission = |o: usize| match c(o) {
        Content::Quest { quest: Some(q), .. } => q.mission.clone(),
        other => panic!("{other:?}"),
    };
    assert_eq!(mission(11), Mission::Skills([1, 0, -2, 3]));
    assert_eq!(mission(12), Mission::DefeatHero(16, true));
    assert_eq!(mission(15), Mission::Creatures(vec![(0, 1), (13, 2)]));
    assert_eq!(
        c(19),
        Content::Quest {
            visited: true,
            seer: 0,
            quest: None
        }
    );
    // Left out (a live part written before T6a2b): none.
    assert!(facts("", 20).unwrap().contents.is_empty());
    // Each row's object below the objects' count and once; its flags 0 or 1; -1 or a value.
    for wrong in [
        "mines = [[20, 0, []]]",
        "mines = [[0, 0, []], [0, 1, []]]",
        "banks = [[20, 0, []]]",
        "banks = [[0, 2, []]]",
        "shrines = [[20, 0, -1, 0]]",
        "shrines = [[0, 0, -2, 0]]",
        "shrines = [[0, 0, 3, 2]]",
        "shrines = [[0, 2, -1, 0]]",
        "trees = [[20, 0, -1, -1]]",
        "trees = [[0, 0, -2, -1]]",
        "trees = [[0, 0, -1, -2]]",
        "trees = [[0, 0, -1, 2]]",
        "trees = [[0, 2, -1, -1]]",
        "witch_huts = [[20, 0, -1, 0]]",
        "witch_huts = [[0, 0, -2, 0]]",
        "witch_huts = [[0, 0, 2, 2]]",
        "witch_huts = [[0, 2, -1, 0]]",
        "quests = [[20, 1, 0, 0, 0, [], []]]",
        "quests = [[0, 2, 0, 0, 0, [], []]]",
        "quests = [[0, 1, 0, 1, 3, [1], []]]",
        "quests = [[0, 1, 0, 1, 0, [1, 2], []]]",
        "quests = [[0, 1, 0, 1, 0, [-1], []]]",
        "quests = [[0, 1, 0, 2, 0, [1, 2, 3], []]]",
        "quests = [[0, 1, 0, 2, 0, [1, 2, 3, 200], []]]",
        "quests = [[0, 1, 0, 3, 0, [1], []]]",
        "quests = [[0, 1, 0, 3, 0, [300, 0], []]]",
        "quests = [[0, 1, 0, 3, 0, [1, 2], []]]",
        "quests = [[0, 1, 0, 4, 0, [], []]]",
        "quests = [[0, 1, 0, 4, 0, [-1], []]]",
        "quests = [[0, 1, 0, 5, 0, [-1], []]]",
        "quests = [[0, 1, 0, 7, 0, [1, 2, 3, 4, 5, 6], []]]",
        "quests = [[0, 1, 0, 7, 0, [1, 2, 3, 4, 5, 6, 3000000000], []]]",
        "quests = [[0, 1, 0, 8, 0, [], []]]",
        "quests = [[0, 1, 0, 9, 0, [], []]]",
        "quests = [[0, 1, 0, 9, 0, [256], []]]",
        "quests = [[0, 1, 0, 10, 0, [], []]]",
    ] {
        assert!(facts(wrong, 20).is_none(), "{wrong}");
    }
}

#[test]
fn guards_merge_by_creature() {
    assert_eq!(Guards::of(&[]), None);
    assert_eq!(
        Guards::of(&[(13, 3), (13, 4)]),
        Some(Guards {
            creature: Some(13),
            count: 7
        })
    );
    assert_eq!(
        Guards::of(&[(13, 3), (0, 4)]),
        Some(Guards {
            creature: None,
            count: 7
        })
    );
}

fn texts() -> Catalogue {
    let json = r#"{
        "object": { "9": "Border Guard", "10": "Tent", "42": "Lighthouse", "88": "Shrine",
            "102": "Tree", "113": "Witch Hut", "83": "Seer's Hut", "215": "Quest Guard" },
        "keymaster": { "0": "Light blue" },
        "player-owned": { "0": "Owned by red", "1": "Owned by blue" },
        "dwelling1": { "56": "Guardhouse" }, "dwelling4": { "1": "Golem Factory" },
        "resource": { "0": "Wood", "5": "Gems", "6": "Gold" },
        "mine": { "0": "Sawmill", "7": "Abandoned Mine" },
        "bank": { "0": "Cyclops Stockpile" },
        "spell": { "3": "Haste" }, "secondary-skill": { "2": "Logistics" },
        "tree-price": { "1": " (+1 level for gold)" },
        "seer-name": { "4": "Engle" },
        "primary-skill": { "0": "Attack", "3": "Knowledge" },
        "artifact": { "7": "Axe", "8": "Shard", "9": "Flail" },
        "player-colour": { "1": "Blue" },
        "creature": { "0_one": "Pikeman", "0_other": "Pikemen", "13_other": "Archangels" },
        "hero": { "16": "Mephala" },
        "map-bonus": { "12": "(skill)", "18": "(level)", "19": "(spell)" },
        "map": {
            "hint": { "more": " {{text}}", "visited": "(seen)", "not-visited": "(unseen)",
                "keymaster": "{{colour}} {{name}}", "owned": "{{name}} - {{owner}}",
                "owner": " - {{owner}}", "open": "(", "close": ")", "guarded": "Guarded by",
                "creatures": "creatures", "learn-spell": "(Learn {{name}})",
                "spell-known": "(known)", "learn-skill": "(Teach {{name}})",
                "skill-known": "(had)", "seer": "{{name}} the Seer", "comma": ", ",
                "and": " and " },
            "guards": { "0": "a few", "2": "a pack of" },
            "quest": { "skill": "{{skill}} {{value}}", "count": "{{amount}} {{name}}",
                "1_seer-1": "(reach {{level}})", "2_guard-2": "( requires {{what}} )",
                "3_seer-1": "(defeat {{what}})", "4_seer-1": "(kill {{what}})",
                "5_seer-1": "(seeks {{what}})", "6_seer-1": "(wants {{what}})",
                "7_seer-1": "(pay {{what}})", "8_guard-1": "(be {{what}})",
                "9_seer-1": "(flag {{what}})" }
        }
    }"#;
    Catalogue {
        packs: vec![Pack::from_json("en", json).unwrap()],
    }
}

#[test]
fn a_contents_hints_text() {
    let t = texts();
    let names = BTreeMap::from([(20, "Own".to_string())]);
    let text = |h: Hint| h.text(&names, &t);
    let keymaster = |class, colour, visited| Hint::Keymaster {
        class,
        colour,
        visited,
    };
    assert_eq!(text(keymaster(9, 0, None)), "Light blue Border Guard");
    assert_eq!(text(keymaster(10, 0, Some(true))), "Light blue Tent (seen)");
    assert_eq!(text(keymaster(232, 8, None)), " ");
    let dwelling = |class, kind, owner| Hint::Dwelling { class, kind, owner };
    assert_eq!(text(dwelling(17, 56, Some(0))), "Guardhouse - Owned by red");
    assert_eq!(text(dwelling(20, 1, None)), "Golem Factory");
    assert_eq!(text(dwelling(20, 2, None)), "");
    assert_eq!(
        text(Hint::Lighthouse { owner: Some(1) }),
        "Lighthouse - Owned by blue"
    );
    assert_eq!(text(Hint::Lighthouse { owner: Some(8) }), "Lighthouse - ");
    assert_eq!(text(Hint::Lighthouse { owner: None }), "Lighthouse");
    assert_eq!(text(Hint::Resource(6)), "Gold");
    assert_eq!(text(Hint::Resource(8)), "");
    let guards = |creature, count| Some(Guards { creature, count });
    assert_eq!(
        text(Hint::Mine {
            name: 0,
            owner: Some(0),
            produces: Some(0),
            guards: guards(Some(13), 4),
        }),
        "Sawmill Owned by red (Wood) Guarded by a few Archangels"
    );
    assert_eq!(
        text(Hint::Mine {
            name: 7,
            owner: None,
            produces: None,
            guards: guards(Some(151), 12),
        }),
        "Abandoned Mine Guarded by a pack of "
    );
    assert_eq!(
        text(Hint::Mine {
            name: 0,
            owner: Some(1),
            produces: None,
            guards: None,
        }),
        "Sawmill Owned by blue"
    );
    let bank = |mark| Hint::Bank { bank: 0, mark };
    assert_eq!(
        text(bank(BankMark::NotVisited)),
        "Cyclops Stockpile (unseen)"
    );
    assert_eq!(text(bank(BankMark::Visited)), "Cyclops Stockpile (seen)");
    assert_eq!(
        text(bank(BankMark::Guarded(Guards {
            creature: None,
            count: 15
        }))),
        "Cyclops Stockpile (Guarded by a pack of creatures)"
    );
    assert_eq!(
        text(Hint::Bank {
            bank: 11,
            mark: BankMark::Visited
        }),
        " (seen)"
    );
    let teaches = |class, bonus, teach, visited| Hint::Teaches {
        class,
        bonus,
        teach,
        visited,
    };
    assert_eq!(
        text(teaches(88, None, Some(Teach::Spell(3, true)), None)),
        "Shrine (Learn Haste) (known)"
    );
    assert_eq!(
        text(teaches(88, None, Some(Teach::Spell(3, false)), None)),
        "Shrine (Learn Haste)"
    );
    assert_eq!(text(teaches(88, Some(19), None, None)), "Shrine (spell)");
    assert_eq!(text(teaches(88, None, None, None)), "Shrine");
    assert_eq!(
        text(teaches(102, None, Some(Teach::Price(1)), Some(false))),
        "Tree  (+1 level for gold) (unseen)"
    );
    assert_eq!(
        text(teaches(113, None, Some(Teach::Skill(2, true)), None)),
        "Witch Hut (Teach Logistics) (had)"
    );
    assert_eq!(
        text(teaches(113, None, Some(Teach::Skill(2, false)), None)),
        "Witch Hut (Teach Logistics)"
    );
    let quest = |variant, mission| Some(Arc::new(Quest { variant, mission }));
    let seer = |mission| Hint::Quest {
        class: 83,
        seer: Some(4),
        quest: quest(0, mission),
    };
    assert_eq!(text(seer(Mission::Level(10))), "Engle the Seer (reach 10)");
    assert_eq!(
        text(seer(Mission::DefeatHero(16, false))),
        "Engle the Seer (defeat Mephala)"
    );
    assert_eq!(
        text(seer(Mission::DefeatHero(20, true))),
        "Engle the Seer (defeat Own)"
    );
    assert_eq!(
        text(seer(Mission::DefeatMonster(13))),
        "Engle the Seer (kill Archangels)"
    );
    assert_eq!(
        text(seer(Mission::Artifacts(vec![7, 8, 9]))),
        "Engle the Seer (seeks Axe, Shard and Flail)"
    );
    assert_eq!(
        text(seer(Mission::Creatures(vec![(0, 1), (0, 2), (151, 3)]))),
        "Engle the Seer (wants 1 Pikeman, 2 Pikemen and 3 )"
    );
    assert_eq!(
        text(seer(Mission::Resources([5, 0, 0, 0, 0, -1, 1000]))),
        "Engle the Seer (pay 5 Wood and 1000 Gold)"
    );
    assert_eq!(
        text(seer(Mission::BePlayer(1))),
        "Engle the Seer (flag blue)"
    );
    assert_eq!(text(seer(Mission::BePlayer(8))), "Engle the Seer (flag )");
    let guard = |variant, mission| Hint::Quest {
        class: 215,
        seer: None,
        quest: quest(variant, mission),
    };
    assert_eq!(
        text(guard(1, Mission::Skills([1, 0, -2, 3]))),
        "Quest Guard ( requires Attack 1 and Knowledge 3 )"
    );
    assert_eq!(
        text(guard(0, Mission::BeHero(16, false))),
        "Quest Guard (be Mephala)"
    );
    assert_eq!(
        text(Hint::Quest {
            class: 83,
            seer: Some(48),
            quest: None
        }),
        " the Seer"
    );
    // The other hints are not the contents'.
    assert_eq!(super::text(&Hint::Empty, &names, &t), "");
}

/// An 8 × 8 map of the contents' classes (each one tile, its entrance its own but the last
/// Shrine's, which has none) and Red's and Blue's owners, the facts' contents from [`LINES`]'s
/// kinds.
fn fixture() -> Arc<Hover> {
    let classes: [(u32, u32); 28] = [
        (9, 0),
        (10, 2),
        (42, 0),
        (79, 6),
        (17, 56),
        (20, 1),
        (53, 0),
        (53, 7),
        (53, 2),
        (16, 3),
        (24, 0),
        (84, 0),
        (85, 0),
        (25, 0),
        (88, 0),
        (90, 0),
        (102, 0),
        (102, 0),
        (113, 0),
        (113, 0),
        (83, 0),
        (83, 0),
        (215, 0),
        (215, 0),
        (53, 5),
        (212, 0),
        (88, 0),
        (102, 0),
    ];
    let objects: Vec<_> = classes
        .iter()
        .enumerate()
        .map(|(i, &(class, subclass))| {
            let cell: &[(u8, u8)] = &[(0, 0)];
            let (blocked, visited) = if i == 26 {
                (cell, &[][..])
            } else {
                (&[][..], cell)
            };
            let at = (
                u8::try_from(1 + i % 7).unwrap(),
                u8::try_from(1 + i / 7).unwrap(),
            );
            (template(i, class, subclass, blocked, visited), at)
        })
        .collect();
    let (art, mut live, shapes) = map_with(&objects, &[]);
    live.objects[2].owner = Some(0);
    live.objects[4].owner = Some(0);
    live.objects[6].owner = Some(0);
    live.objects[8].owner = Some(1);
    let lines = r#"
mines = [[6, 0, [[13, 2]]], [7, 3, [[46, 120]]], [8, 2, []]]
banks = [[9, 0, [[0, 3]]], [10, 1, []], [11, 1, [[56, 30]]], [12, 1, [[0, 3], [1, 4]]], [13, 0, []]]
shrines = [[14, 1, -1, 0], [15, 0, 3, 1], [26, 1, -1, 0]]
trees = [[16, 0, 1, 0], [17, 1, -1, -1], [27, 0, -1, -1]]
witch_huts = [[18, 0, 2, 0], [19, 1, -1, 0]]
quests = [
    [20, 1, 4, 1, 0, [10], []],
    [21, 0, 4, 1, 0, [10], []],
    [22, 1, 0, 9, 1, [1], []],
    [23, 1, 0, 0, 0, [], []],
]
visits = [[0, -1], [0, 1], [0, -1], [0, -1], [0, -1], [0, -1], [0, -1], [0, -1], [0, -1],
    [0, -1], [0, -1], [0, -1], [0, -1], [0, -1], [0, -1], [0, -1], [0, -1], [0, -1], [0, -1],
    [0, -1], [0, -1], [0, -1], [0, -1], [0, -1], [0, -1], [0, -1], [0, -1], [0, -1]]
"#;
    live.hover = Some(facts(lines, classes.len()).unwrap());
    let lists = [DrawLists::build(&art, &live, &shapes, 0).unwrap()];
    Hover::new(&art, &live, &tables().records, &lists).unwrap()
}

#[test]
fn the_hint_of_an_owner_or_contents() {
    let h = fixture();
    let hint = |i: i32| h.hint([1 + i % 7, 1 + i / 7, 0], false, None);
    assert_eq!(
        hint(0),
        Hint::Keymaster {
            class: 9,
            colour: 0,
            visited: None
        }
    );
    assert_eq!(
        hint(1),
        Hint::Keymaster {
            class: 10,
            colour: 2,
            visited: Some(true)
        }
    );
    assert_eq!(hint(2), Hint::Lighthouse { owner: Some(0) });
    assert_eq!(hint(3), Hint::Resource(6));
    assert_eq!(
        hint(4),
        Hint::Dwelling {
            class: 17,
            kind: 56,
            owner: Some(0)
        }
    );
    assert_eq!(
        hint(5),
        Hint::Dwelling {
            class: 20,
            kind: 1,
            owner: None
        }
    );
    let guards = |creature, count| Some(Guards { creature, count });
    // Red's mine: what it produces (Red views); an abandoned one's name, its drawn resource
    // unseen; Blue's: no production shown.
    assert_eq!(
        hint(6),
        Hint::Mine {
            name: 0,
            owner: Some(0),
            produces: Some(0),
            guards: guards(Some(13), 2)
        }
    );
    assert_eq!(
        hint(7),
        Hint::Mine {
            name: 7,
            owner: None,
            produces: None,
            guards: guards(Some(46), 120)
        }
    );
    assert_eq!(
        hint(8),
        Hint::Mine {
            name: 2,
            owner: Some(1),
            produces: None,
            guards: None
        }
    );
    let bank = |bank, mark| Hint::Bank { bank, mark };
    assert_eq!(hint(9), bank(3, BankMark::NotVisited));
    assert_eq!(hint(10), bank(8, BankMark::Visited));
    assert_eq!(
        hint(11),
        bank(
            9,
            BankMark::Guarded(Guards {
                creature: Some(56),
                count: 30
            })
        )
    );
    assert_eq!(
        hint(12),
        bank(
            7,
            BankMark::Guarded(Guards {
                creature: None,
                count: 7
            })
        )
    );
    assert_eq!(hint(13), bank(10, BankMark::NotVisited));
    let teaches = |class, bonus, teach, visited| Hint::Teaches {
        class,
        bonus,
        teach,
        visited,
    };
    assert_eq!(hint(14), teaches(88, Some(19), None, None));
    assert_eq!(
        hint(15),
        teaches(90, None, Some(Teach::Spell(3, true)), None)
    );
    assert_eq!(
        hint(16),
        teaches(102, None, Some(Teach::Price(1)), Some(false))
    );
    assert_eq!(hint(17), teaches(102, Some(18), None, None));
    assert_eq!(
        hint(18),
        teaches(113, None, Some(Teach::Skill(2, false)), None)
    );
    // A hut's kind known (the harness writes none for a hut without a skill); a Tree with
    // nothing shown: its name.
    assert_eq!(hint(19), teaches(113, Some(12), None, None));
    assert_eq!(hint(27), Hint::Object(102));
    let level = Arc::new(Quest {
        variant: 0,
        mission: Mission::Level(10),
    });
    assert_eq!(
        hint(20),
        Hint::Quest {
            class: 83,
            seer: Some(4),
            quest: Some(level)
        }
    );
    // A Seer's Hut not visited, a Quest Guard without its quest: the names.
    assert_eq!(hint(21), Hint::Object(83));
    assert_eq!(
        hint(22),
        Hint::Quest {
            class: 215,
            seer: None,
            quest: Some(Arc::new(Quest {
                variant: 1,
                mission: Mission::BePlayer(1)
            }))
        }
    );
    assert_eq!(hint(23), Hint::Object(215));
    // A mine without its contents; a Border Gate; a Shrine without an entrance on the map.
    assert_eq!(hint(24), Hint::Object(53));
    assert_eq!(
        hint(25),
        Hint::Keymaster {
            class: 212,
            colour: 0,
            visited: None
        }
    );
    assert_eq!(hint(26), Hint::Object(88));
}
