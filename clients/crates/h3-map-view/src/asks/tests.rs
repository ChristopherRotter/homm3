use super::*;

use alloc::string::ToString;
use h3_ui::text::pack::Pack;

/// A catalogue of the questions' keys and names (the recorded boxes' texts where they show, in
/// short).
fn texts() -> Catalogue {
    let json = concat!(
        r#"{ "map": { "ask": { "size": { "0": "a few", "1": "several", "2": "a pack of", "#,
        r#""8": "a legion of" }, "#,
        r#""artifact": "guarded by {{size}} {{creature}}. Fight the {{creature}}?", "#,
        r#""guarded": "the {{dwelling}} is guarded by {{size}} {{creature}}.", "#,
        r#""mine": "MINE", "bank": "You have found a {{bank}}.", "#,
        r#""bank-looted": "once a {{bank}}", "derelict-ship": "SHIP", "crypt": "CRYPT", "#,
        r#""shipwreck": "WRECK", "tree-gold": "2000 gold", "tree-gems": "10 gems", "#,
        r#""tree-visited": "my student", "tree-poor-gold": "I need 2000 gold.", "#,
        r#""tree-poor-gems": "I need 10 gems.", "pyramid": "PYRAMID", "tomb": "TOMB", "#,
        r#""tomb-found": "finding the {{artifact}}", "tomb-empty": "finding nothing", "#,
        r#""hire": "[{{dwelling}}] recruit {{creature}}?", "#,
        r#""joined": "{{amount}} {{creature}} join your army.", "#,
        r#""none": "There are no {{creature}} here to recruit.", "#,
        r#""no-room": "The {{creature}} would join your hero" } }, "#,
        r#""creature": { "0_one": "Pikeman", "0_other": "Pikemen", "8_one": "Monk", "#,
        r#""8_other": "Monks" }, "artifact": { "70": "Equestrian's Gloves" }, "#,
        r#""bank": { "0": "Cyclops Stockpile" }, "#,
        r#""dwelling1": { "35": "Monastery", "56": "Guardhouse" } }"#,
    );
    Catalogue {
        packs: vec![Pack::from_json("en", json).unwrap()],
    }
}

/// The committed tables' questions' numbers and finds' art.
fn tables() -> crate::tables::Tables {
    let text = include_str!("../../../../map/assets/base.toml");
    crate::tables::load(&[crate::tables::parse(text).unwrap()], "base").unwrap()
}

fn art() -> AskArt {
    tables().map.asks
}

/// Red, its own team's alone, the kingdom's resources a new game's and the hero's army one stack
/// of Centaurs (the recordings' live parts).
fn red() -> Visitor {
    let mut army = [None; 7];
    army[0] = Some(Stack {
        creature: 14,
        count: 18,
    });
    Visitor {
        colour: 0,
        teams: [0, 1, 2, 3, 4, 5, 6, 7],
        resources: [20, 10, 20, 10, 10, 10, 20000],
        army,
    }
}

fn seen(class: u32, subclass: u32, word: u32) -> Seen {
    Seen {
        class: [class, subclass],
        word: Some(word),
        ..Seen::default()
    }
}

fn ask(seen: &Seen) -> Option<Ask> {
    Ask::of(seen, &red(), &art())
}

fn pikemen(count: i32) -> Stack {
    Stack { creature: 0, count }
}

#[test]
fn the_tables_levels() {
    // Pikemen level 1, Monks 5, Angels 7; past the table none.
    let art = art();
    assert_eq!(art.levels.len(), 150);
    assert_eq!((art.level(0), art.level(8), art.level(12)), (1, 5, 7));
    assert_eq!(art.level(150), 0);
    assert_eq!((art.guards, art.tomb_morale), ([5, 3], -3));
    assert_eq!(art.tree_prices, vec![[6, 2000], [5, 10]]);
}

#[test]
fn a_guarded_pile_and_artifact() {
    // The map's text asked (`ask-pile`), a pile's guards without one fight at once (not built), a
    // message without guards is the plain pickup (not built).
    let text = |text: &str, guards: Vec<Stack>| Seen {
        message: Some((text.to_string(), guards)),
        ..seen(79, 6, 0x8000_0005)
    };
    let pile = ask(&text("Fight them?", vec![pikemen(10)])).unwrap();
    assert_eq!(pile, Ask::Text("Fight them?".to_string()));
    assert!(pile.asks() && pile.chooses() && pile.beside());
    assert_eq!(pile.told(&texts()).text, "Fight them?");
    assert_eq!(ask(&text("", vec![pikemen(10)])), None);
    assert_eq!(ask(&text("Here.", Vec::new())), None);
    assert_eq!(ask(&seen(79, 6, 5)), None);
    // A scroll's the same.
    let scroll = Seen {
        class: [93, 0],
        ..text("A scroll.", vec![pikemen(1)])
    };
    assert_eq!(ask(&scroll), Some(Ask::Text("A scroll.".to_string())));
    // An artifact without a text: GENRLTXT 421 by its one stack (`ask-artifact`: a pack of ten
    // Pikemen); several stacks not built.
    let artifact = |guards| Seen {
        class: [5, 7],
        ..text("", guards)
    };
    let asked = ask(&artifact(vec![pikemen(10)])).unwrap();
    assert!(asked.beside() && asked.chooses());
    assert_eq!(
        asked.told(&texts()).text,
        "guarded by a pack of Pikemen. Fight the Pikemen?"
    );
    assert_eq!(ask(&artifact(vec![pikemen(10), pikemen(5)])), None);
    assert!(asked.yes(&tables().map.finds, &art(), &texts()).is_none());
}

#[test]
fn the_size_words() {
    // Below 5 the first, 1 000 and more the ninth; a count below 0 the first.
    let words: Vec<String> = [1, 5, 10, 1000, -1]
        .into_iter()
        .map(|n| size(&texts(), n))
        .collect();
    assert_eq!(
        words,
        ["a few", "several", "a pack of", "a legion of", "a few"].map(String::from)
    );
}

#[test]
fn a_mine_a_bank_and_the_search_kinds() {
    // A neutral abandoned mine (`ask-mine`); an owned one, another kind's, none.
    let mine = ask(&seen(53, 7, 0)).unwrap();
    assert_eq!(mine.told(&texts()).text, "MINE");
    assert!(mine.asks() && mine.chooses() && !mine.beside());
    let owned = Seen {
        owner: Some(1),
        ..seen(53, 7, 0)
    };
    assert_eq!(ask(&owned), None);
    assert_eq!(ask(&seen(53, 0, 0)), None);
    // A bank (`ask-bank`'s Cyclops Stockpile, its word 0xFC00001F), looted (bit 25: a box).
    let bank = ask(&seen(16, 0, 0xFC00_001F)).unwrap();
    assert!(bank.asks() && bank.chooses());
    assert_eq!(
        bank.told(&texts()).text,
        "You have found a Cyclops Stockpile."
    );
    let looted = ask(&seen(16, 0, 1 << 25)).unwrap();
    assert!(!looted.asks());
    assert_eq!(looted.told(&texts()).text, "once a Cyclops Stockpile");
    // The search kinds, looted or not (`ask-crypt`); no status window chosen first.
    let told = |class| {
        let asked = ask(&seen(class, 0, 0xFC00_001F)).unwrap();
        assert!(asked.asks() && !asked.chooses());
        asked.told(&texts()).text
    };
    assert_eq!([told(24), told(84), told(85)], ["SHIP", "CRYPT", "WRECK"]);
    assert_eq!(ask(&seen(1, 0, 0)), None);
}

#[test]
fn the_tree_of_knowledge() {
    // Its price in the word's bits 13 to 15 (`ask-tree`: 0xFFFF2000, 2 000 gold) and the
    // experience's "+1 Level"; ten gems; free (148: not built); no word, none.
    let gold = ask(&seen(102, 0, 0xFFFF_2000)).unwrap();
    assert!(gold.asks() && !gold.chooses() && !gold.beside());
    let told = gold.told(&texts());
    assert_eq!(
        (told.text.as_str(), told.pictures),
        ("2000 gold", vec![(17, -1)])
    );
    let gems = ask(&seen(102, 0, 2 << 13)).unwrap();
    assert_eq!(gems.told(&texts()).text, "10 gems");
    assert_eq!(ask(&seen(102, 0, 0)), None);
    assert_eq!(ask(&seen(102, 0, 3 << 13)), None);
    let wordless = Seen {
        word: None,
        ..seen(102, 0, 0)
    };
    assert_eq!(ask(&wordless), None);
    // Visited (the hero's mark, free or not): a box alone.
    for word in [0, 1 << 13] {
        let visited = Seen {
            visited: true,
            ..seen(102, 0, word)
        };
        let box_ = ask(&visited).unwrap();
        assert!(!box_.asks());
        assert_eq!(box_.told(&texts()).text, "my student");
    }
    // Too poor: a box alone (the kingdom without gold, without gems; a resource past the bar's
    // never held).
    let poor = Visitor {
        resources: [0; 7],
        ..red()
    };
    let poor_gold = Ask::of(&seen(102, 0, 1 << 13), &poor, &art()).unwrap();
    assert!(!poor_gold.asks());
    assert_eq!(poor_gold.told(&texts()).text, "I need 2000 gold.");
    let poor_gems = Ask::of(&seen(102, 0, 2 << 13), &poor, &art()).unwrap();
    assert_eq!(poor_gems.told(&texts()).text, "I need 10 gems.");
    let mut odd = art();
    odd.tree_prices = vec![[9, 1]];
    let never = Ask::of(&seen(102, 0, 1 << 13), &red(), &odd).unwrap();
    assert!(matches!(never, Ask::Tree { poor: true, .. }));
    assert!(gold.yes(&tables().map.finds, &art(), &texts()).is_none());
}

#[test]
fn the_pyramid_and_the_tomb() {
    let pyramid = ask(&seen(63, 0, 0)).unwrap();
    assert!(pyramid.asks() && pyramid.chooses());
    assert_eq!(pyramid.told(&texts()).text, "PYRAMID");
    assert!(pyramid.yes(&tables().map.finds, &art(), &texts()).is_none());
    // The Tomb's artifact while its bit 0 is set (`ask-tomb`'s 0xFF88C01F: the Equestrian's
    // Gloves, 70), its search the artifact's box and its morale lost.
    let tomb = ask(&seen(108, 0, 0xFF88_C01F)).unwrap();
    assert_eq!(tomb, Ask::Tomb(Some(70)));
    assert!(tomb.asks() && !tomb.chooses());
    assert_eq!(tomb.told(&texts()).text, "TOMB");
    let finds = tables().map.finds;
    let (found, gain) = tomb.yes(&finds, &art(), &texts()).unwrap();
    assert_eq!(found.text, "finding the Equestrian's Gloves");
    assert_eq!(found.pictures, vec![(8, 70)]);
    assert_eq!((gain.morale, gain.skills), ([-3, -3], [0; 4]));
    // An artifact of primary skills gives them (the Centaur's Axe, 7: +2 Attack).
    let (_, axe) = Ask::Tomb(Some(7)).yes(&finds, &art(), &texts()).unwrap();
    assert_eq!(axe.skills, [2, 0, 0, 0]);
    // Emptied: three pictures of bad morale, the morale lost.
    let empty = ask(&seen(108, 0, 0xFF88_C01E)).unwrap();
    assert_eq!(empty, Ask::Tomb(None));
    let (told, gain) = empty.yes(&finds, &art(), &texts()).unwrap();
    assert_eq!(told.text, "finding nothing");
    assert_eq!(told.pictures, vec![(16, 0); 3]);
    assert_eq!(gain.morale, [-3, -3]);
}

/// A dwelling of class 17 of `subclass` with `creature` of `growth`, owned by `owner`.
fn dwelling(subclass: u32, creature: u16, growth: i32, owner: Option<u8>) -> Seen {
    Seen {
        owner,
        dwelling: Some((creature, growth)),
        ..seen(17, subclass, 0)
    }
}

#[test]
fn a_dwellings_guards() {
    // A neutral Monastery (`ask-dwelling-guards`): its Monks' three weeks of growth, 3 a week.
    let guarded = ask(&dwelling(35, 8, 3, None)).unwrap();
    assert_eq!(
        guarded,
        Ask::Guarded {
            dwelling: 35,
            guards: Stack {
                creature: 8,
                count: 9
            }
        }
    );
    assert!(guarded.asks() && guarded.chooses() && !guarded.beside());
    assert_eq!(
        guarded.told(&texts()).text,
        "the Monastery is guarded by several Monks."
    );
    assert!(guarded.yes(&tables().map.finds, &art(), &texts()).is_none());
    // Another player's too; a creature below level 5 guards nothing (its flag: not built); no
    // creature known, none.
    assert!(matches!(
        ask(&dwelling(35, 8, 3, Some(1))),
        Some(Ask::Guarded { .. })
    ));
    assert_eq!(ask(&dwelling(56, 0, 14, None)), None);
    assert_eq!(ask(&seen(17, 56, 0)), None);
}

#[test]
fn a_dwellings_hire() {
    // Red's own Guardhouse (`ask-hire`): its Pikemen's week of growth join free, told.
    let hire = ask(&dwelling(56, 0, 14, Some(0))).unwrap();
    assert_eq!(
        hire,
        Ask::Hire {
            dwelling: 56,
            joins: pikemen(14),
            free: true,
            room: true
        }
    );
    assert!(hire.asks() && hire.chooses() && !hire.beside());
    assert_eq!(hire.told(&texts()).text, "[Guardhouse] recruit Pikemen?");
    let finds = tables().map.finds;
    let (joined, gain) = hire.yes(&finds, &art(), &texts()).unwrap();
    assert_eq!(joined.text, "14 Pikemen join your army.");
    assert_eq!(gain.army[0], Some(pikemen(14)));
    assert_eq!(gain.army[1..], [None; 6]);
    // One joins by its singular; none to hire; no room (seven other stacks), the army as it was.
    let one = Ask::Hire {
        dwelling: 56,
        joins: pikemen(1),
        free: true,
        room: true,
    };
    assert_eq!(
        one.yes(&finds, &art(), &texts()).unwrap().0.text,
        "1 Pikeman join your army."
    );
    let none = Ask::Hire {
        dwelling: 56,
        joins: pikemen(0),
        free: true,
        room: true,
    };
    let (told, gain) = none.yes(&finds, &art(), &texts()).unwrap();
    assert_eq!(told.text, "There are no Pikemen here to recruit.");
    assert_eq!(gain, Gain::default());
    let full = Visitor {
        army: [Some(Stack {
            creature: 14,
            count: 1,
        }); 7],
        ..red()
    };
    let crowded = Ask::of(&dwelling(56, 0, 14, Some(0)), &full, &art()).unwrap();
    assert!(matches!(crowded, Ask::Hire { room: false, .. }));
    let (told, gain) = crowded.yes(&finds, &art(), &texts()).unwrap();
    assert_eq!(told.text, "The Pikemen would join your hero");
    assert_eq!(gain, Gain::default());
    // A stack of their kind is room.
    let mut kin = full.clone();
    kin.army[6] = Some(pikemen(2));
    assert!(matches!(
        Ask::of(&dwelling(56, 0, 14, Some(0)), &kin, &art()),
        Some(Ask::Hire { room: true, .. })
    ));
    // A level-2 creature's or an ally's dwelling: the recruit dialog (not built).
    let archers = ask(&dwelling(57, 2, 9, Some(0))).unwrap();
    assert!(matches!(archers, Ask::Hire { free: false, .. }));
    assert!(archers.yes(&finds, &art(), &texts()).is_none());
    let allied = Visitor {
        teams: [0, 0, 2, 3, 4, 5, 6, 7],
        ..red()
    };
    let ally = Ask::of(&dwelling(56, 0, 14, Some(1)), &allied, &art()).unwrap();
    assert!(matches!(ally, Ask::Hire { free: false, .. }));
    // A colour past the teams is no ally.
    let stranger = Ask::of(&dwelling(56, 0, 14, Some(9)), &red(), &art());
    assert_eq!(stranger, None);
}
