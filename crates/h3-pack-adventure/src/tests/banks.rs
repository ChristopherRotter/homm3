//! E08 T123a: increment 13's creature banks in the pack format and the base pack (increment 13's
//! design note §1, §2; ADV-BNK-001).

use h3_adventure::{BANK_LEVELS, Bank, BankQuestion, ObjectType};

use super::*;

/// A type with this bank sub-table (`[object_type.bank]`), visited as `visit`.
fn bank_type(visit: &str, table: &str) -> Result<AdventurePack, AdventurePackError> {
    parse(&format!(
        "[[object_type]]\nid = \"o\"\nname = \"O\"\nvisit = \"{visit}\"\nfrom_above = false\n\n\
         [object_type.bank]\n{table}"
    ))
}

// covers: ADV-BNK-001
#[test]
fn parses_the_creature_bank_kind() {
    let kind =
        |visit: &str, table: &str| bank_type(visit, table).unwrap().object_types.remove(0).kind;
    // Every field; the guards in the list's order, the resources by key, the artifacts by class.
    assert_eq!(
        kind(
            "neighbour",
            "question = \"search\"\nlevels = [\n    { chance = 60, guards = [{ creature = \
             \"wight\", count = 10 }, { creature = \"base:skeleton\", count = 5 }], upgrade = \
             50, resources = { gold = 2000, wood = 3 }, creature = { creature = \"angel\", count \
             = 2 }, artifacts = { treasure = 1, minor = 2, major = 3, relic = 4 } },\n    { \
             chance = 40, guards = [{ creature = \"wight\", count = 15 }] },\n]\nlooted = { \
             morale = -1, mark = 14 }\ndeck = true\ncoast = true\n"
        ),
        ObjectKindDef::Bank(BankDef {
            question: BankQuestion::Search,
            levels: vec![
                BankLevelDef {
                    chance: 60,
                    guards: vec![(key_of("p:wight"), 10), (key_of("base:skeleton"), 5)],
                    upgrade: 50,
                    resources: vec![(key_of("p:gold"), 2000), (key_of("p:wood"), 3)],
                    creature: Some((key_of("p:angel"), 2)),
                    artifacts: [1, 2, 3, 4],
                },
                // Left out: no upgrade, no resources, no creature, no artifacts.
                BankLevelDef {
                    chance: 40,
                    guards: vec![(key_of("p:wight"), 15)],
                    upgrade: 0,
                    resources: vec![],
                    creature: None,
                    artifacts: [0; 4],
                },
            ],
            looted: Some(OnceBonus {
                morale: -1,
                luck: 0,
                mark: 14,
            }),
            deck: true,
            coast: true,
        })
    );
    // An attack bank: no looted bonus, no deck, no coast when left out.
    assert_eq!(
        kind(
            "enter",
            "question = \"attack\"\nlevels = [{ chance = 100, guards = [{ creature = \"imp\", \
             count = 100 }], artifacts = { relic = 1 } }]\n"
        ),
        ObjectKindDef::Bank(BankDef {
            question: BankQuestion::Attack,
            levels: vec![BankLevelDef {
                chance: 100,
                guards: vec![(key_of("p:imp"), 100)],
                upgrade: 0,
                resources: vec![],
                creature: None,
                artifacts: [0, 0, 0, 1],
            }],
            looted: None,
            deck: false,
            coast: false,
        })
    );
    // An unknown question, field or class, a missing chance or guards, and a guard without a count
    // are syntax errors; a malformed key a key error.
    let level = "{ chance = 100, guards = [{ creature = \"imp\", count = 1 }] }";
    for bad in [
        format!("question = \"fight\"\nlevels = [{level}]"),
        format!("question = \"attack\"\nlevels = [{level}]\nrefill = true"),
        "question = \"attack\"\nlevels = [{ chance = 100, guards = [{ creature = \"imp\", count = \
         1 }], artifacts = { special = 1 } }]"
            .to_owned(),
        "question = \"attack\"\nlevels = [{ guards = [{ creature = \"imp\", count = 1 }] }]"
            .to_owned(),
        "question = \"attack\"\nlevels = [{ chance = 100 }]".to_owned(),
        "question = \"attack\"\nlevels = [{ chance = 100, guards = [{ creature = \"imp\" }] }]"
            .to_owned(),
        format!("levels = [{level}]"),
    ] {
        assert!(
            matches!(bank_type("enter", &bad), Err(AdventurePackError::Syntax(_))),
            "{bad}"
        );
    }
    for bad in [
        "question = \"attack\"\nlevels = [{ chance = 100, guards = [{ creature = \"a:b:c\", count \
         = 1 }] }]",
        "question = \"attack\"\nlevels = [{ chance = 100, guards = [{ creature = \"imp\", count = \
         1 }], resources = { \"a:b:c\" = 1 } }]",
        "question = \"attack\"\nlevels = [{ chance = 100, guards = [{ creature = \"imp\", count = \
         1 }], creature = { creature = \"a:b:c\", count = 1 } }]",
    ] {
        assert!(
            matches!(bank_type("enter", bad), Err(AdventurePackError::Key(_))),
            "{bad}"
        );
    }
}

/// A bank's summary: its question, visit, from above, looted mark, deck, coast, the creatures
/// guarding its last level and its reward creature's key.
type Summary = (
    BankQuestion,
    Visit,
    bool,
    Option<u8>,
    bool,
    bool,
    Vec<String>,
    Option<String>,
);

// covers: ADV-BNK-001
#[test]
fn the_base_pack_types_the_creature_banks_of_increment_13() {
    // Base's eleven in the original's kind order, after the bonus visitables; ab and sod none.
    let (core, base) = compile(&ORIGINAL_PACKS[..1], PROFILE).unwrap();
    let (_, sod) = compile(&ORIGINAL_PACKS, PROFILE).unwrap();
    let banks = |part: &AdventureRuleset| {
        part.object_types()
            .iter()
            .filter_map(|t| match &t.kind {
                ObjectKind::Bank(bank) => Some((t.clone(), bank.clone())),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let keys = |part: &AdventureRuleset| {
        banks(part)
            .into_iter()
            .map(|(t, _)| t.key.as_str().to_owned())
            .collect::<Vec<_>>()
    };
    assert_eq!(keys(&sod), keys(&base));
    let creature = |c| core.creature_key(c).unwrap().as_str().to_owned();
    let summary = |t: &ObjectType, b: &Bank| -> Summary {
        let last = b.levels.last().unwrap();
        (
            b.question,
            t.visit,
            t.from_above,
            b.looted.map(|l| {
                assert_eq!((l.morale, l.luck), (-1, 0), "{}", t.key.as_str());
                l.mark
            }),
            b.deck,
            b.coast,
            last.guards.iter().map(|g| creature(g.0)).collect(),
            last.creature.map(|c| creature(c.0)),
        )
    };
    let (a, s) = (BankQuestion::Attack, BankQuestion::Search);
    let (e, n) = (Visit::Enter, Visit::Neighbour);
    let one = |c: &str| vec![format!("base:{c}")];
    let dragons = ["green", "red", "gold", "black"].map(|c| format!("base:{c}_dragon"));
    let undead = ["skeleton", "walking_dead", "wight", "vampire"].map(|c| format!("base:{c}"));
    let expected: [(&str, &str, Summary); 11] = [
        (
            "cyclops_stockpile",
            "Cyclops Stockpile",
            (a, e, false, None, false, false, one("cyclops"), None),
        ),
        (
            "dwarven_treasury",
            "Dwarven Treasury",
            (a, e, false, None, false, false, one("dwarf"), None),
        ),
        (
            "griffin_conservatory",
            "Griffin Conservatory",
            (
                a,
                e,
                false,
                None,
                false,
                false,
                one("griffin"),
                Some("base:angel".into()),
            ),
        ),
        (
            "imp_cache",
            "Imp Cache",
            (a, e, false, None, false, false, one("imp"), None),
        ),
        (
            "medusa_stores",
            "Medusa Stores",
            (a, e, false, None, false, false, one("medusa"), None),
        ),
        (
            "naga_bank",
            "Naga Bank",
            (a, e, false, None, false, false, one("naga"), None),
        ),
        (
            "dragon_fly_hive",
            "Dragon Fly Hive",
            (
                a,
                e,
                false,
                None,
                false,
                false,
                one("dragon_fly"),
                Some("base:wyvern".into()),
            ),
        ),
        (
            "shipwreck",
            "Shipwreck",
            (s, n, true, Some(14), true, true, one("wight"), None),
        ),
        (
            "derelict_ship",
            "Derelict Ship",
            (
                s,
                e,
                false,
                Some(16),
                true,
                false,
                one("water_elemental"),
                None,
            ),
        ),
        (
            "crypt",
            "Crypt",
            (s, e, false, Some(15), false, false, undead.to_vec(), None),
        ),
        (
            "dragon_utopia",
            "Dragon Utopia",
            (a, e, false, None, false, false, dragons.to_vec(), None),
        ),
    ];
    let found = banks(&base);
    assert_eq!(found.len(), expected.len());
    for ((t, b), (key, name, want)) in found.iter().zip(expected) {
        assert_eq!(t.key.as_str(), format!("base:{key}"));
        assert_eq!(t.name, name);
        assert_eq!(summary(t, b), want, "{key}");
        // Four levels, every chance column 30, 30, 30, 10; no kind mark; not ownable.
        assert_eq!(b.levels.len(), BANK_LEVELS, "{key}");
        let chances = b.levels.iter().map(|l| l.chance).collect::<Vec<_>>();
        assert_eq!(chances, [30, 30, 30, 10], "{key}");
        assert!(!t.kind_mark && !t.kind.ownable(), "{key}");
        // Class 16's upgrade chance 50 %, the others' none.
        let class_16 = b.question == a && key != "dragon_utopia";
        assert!(
            b.levels
                .iter()
                .all(|l| l.upgrade == if class_16 { 50 } else { 0 }),
            "{key}"
        );
    }
    // The levels of three kinds whole, as survey §2's table (the guards before the split).
    let by_key = |key: &str| {
        found
            .iter()
            .find(|(t, _)| t.key.as_str() == format!("base:{key}"))
            .map(|(_, b)| b.clone())
            .unwrap()
    };
    let resource = |k: &str| {
        let i = core.resources().iter().position(|r| r.key.as_str() == k);
        ResourceId::from_index(i.unwrap()).unwrap()
    };
    let level_values = |key: &str| {
        by_key(key)
            .levels
            .iter()
            .map(|l| {
                (
                    l.guards.iter().map(|g| g.1).collect::<Vec<_>>(),
                    l.resources
                        .iter()
                        .map(|&(r, a)| (core.resources()[r.index()].key.as_str().to_owned(), a))
                        .collect::<Vec<_>>(),
                    l.creature.map(|c| c.1),
                    l.artifacts,
                )
            })
            .collect::<Vec<_>>()
    };
    let res = |list: &[(&str, u32)]| {
        list.iter()
            .map(|&(k, a)| (format!("base:{k}"), a))
            .collect::<Vec<_>>()
    };
    // D-65 the original: the Imp Cache's level 1 with 2 mercury, the Naga Bank's sulfur.
    assert_eq!(
        level_values("imp_cache"),
        [
            (
                vec![100],
                res(&[("mercury", 2), ("gold", 1000)]),
                None,
                [0; 4]
            ),
            (
                vec![150],
                res(&[("mercury", 3), ("gold", 1500)]),
                None,
                [0; 4]
            ),
            (
                vec![200],
                res(&[("mercury", 4), ("gold", 2000)]),
                None,
                [0; 4]
            ),
            (
                vec![300],
                res(&[("mercury", 6), ("gold", 3000)]),
                None,
                [0; 4]
            ),
        ]
    );
    assert_eq!(
        by_key("naga_bank").levels[3].resources,
        [
            (resource("base:sulfur"), 24),
            (resource("base:gold"), 12000)
        ]
    );
    assert_eq!(
        level_values("crypt"),
        [
            (vec![30, 20], res(&[("gold", 1500)]), None, [0; 4]),
            (vec![25, 20, 5], res(&[("gold", 2000)]), None, [0; 4]),
            (
                vec![20, 20, 10, 5],
                res(&[("gold", 2500)]),
                None,
                [1, 0, 0, 0]
            ),
            (
                vec![20, 20, 10, 10],
                res(&[("gold", 5000)]),
                None,
                [1, 0, 0, 0]
            ),
        ]
    );
    assert_eq!(
        level_values("dragon_utopia"),
        [
            (
                vec![8, 5, 2, 1],
                res(&[("gold", 20000)]),
                None,
                [1, 1, 1, 1]
            ),
            (
                vec![8, 6, 3, 2],
                res(&[("gold", 30000)]),
                None,
                [0, 1, 1, 2]
            ),
            (
                vec![8, 6, 4, 3],
                res(&[("gold", 40000)]),
                None,
                [0, 0, 1, 3]
            ),
            (
                vec![8, 7, 6, 5],
                res(&[("gold", 50000)]),
                None,
                [0, 0, 0, 4]
            ),
        ]
    );
    // The reward creatures' counts: Angels 1 to 4, Wyverns 4 to 12.
    for (key, counts) in [
        ("griffin_conservatory", [1, 2, 3, 4]),
        ("dragon_fly_hive", [4, 6, 8, 12]),
    ] {
        let got = by_key(key)
            .levels
            .iter()
            .map(|l| l.creature.unwrap().1)
            .collect::<Vec<_>>();
        assert_eq!(got, counts, "{key}");
    }
}
