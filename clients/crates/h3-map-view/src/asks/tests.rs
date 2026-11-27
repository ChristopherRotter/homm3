use super::*;

use alloc::string::ToString;
use h3_map::model::Rewards;
use h3_ui::text::pack::Pack;

/// A catalogue of the questions' keys and names (the recorded boxes' texts where they show, in
/// short).
fn texts() -> Catalogue {
    let json = concat!(
        r#"{ "map": { "ask": { "size": { "0": "a few", "1": "several", "2": "a pack of", "#,
        r#""8": "a legion of" }, "#,
        r#""artifact": "{{size}} {{creature}} keep it. Attack the {{creature}}?", "#,
        r#""guarded": "{{size}} {{creature}} keep the {{dwelling}}.", "#,
        r#""mine": "MINE", "bank": "A {{bank}} is here.", "#,
        r#""bank-looted": "once a {{bank}}", "derelict-ship": "SHIP", "crypt": "CRYPT", "#,
        r#""shipwreck": "WRECK", "tree-gold": "2000 gold", "tree-gems": "10 gems", "#,
        r#""tree-visited": "TREE0", "tree-poor-gold": "TREE POOR: gold", "#,
        r#""tree-poor-gems": "TREE POOR: gems", "pyramid": "PYRAMID", "tomb": "TOMB", "#,
        r#""tomb-found": "TOMB: {{artifact}}", "tomb-empty": "TOMB0", "#,
        r#""hire": "[{{dwelling}}] recruit {{creature}}?", "#,
        r#""joined": "{{amount}} {{creature}} come along.", "#,
        r#""none": "No {{creature}} to hire.", "#,
        r#""no-room": "No room for the {{creature}}", "#,
        r#""join": "{Followers} {{creature}} ask to come.", "#,
        r#""join-one": "One {{creature}}: {{price}} gold.", "#,
        r#""join-gold": "An offer:\n\n", "#,
        r#""join-price": "{{amount}} {{creature}}: {{price}} gold.", "#,
        r#""flee": "{{creature}} run.", "pass": "PASS", "maps-sea": "SEA", "#,
        r#""maps-land": "LAND", "maps-underground": "TUNNELS", "open-box": "OPEN?", "#,
        r#""or": "or", "chest": "CHEST", "arena": "ARENA", "school": "SCHOOL" }, "#,
        r#""find": { "reward-treasure": "{{hero}}: treasure." } }, "#,
        r#""creature": { "0_one": "Pikeman", "0_other": "Pikemen", "8_one": "Monk", "#,
        r#""8_other": "Monks", "139_one": "Peasant", "139_other": "Peasants" }, "#,
        r#""artifact": { "70": "Equestrian's Gloves" }, "#,
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
        power: 1977,
        diplomacy: 0,
        hero: "Mephala".to_string(),
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

/// An answer's one box and its gain at the end (A1's answers: nothing else).
fn said(reply: Option<Reply>) -> (Told, Gain) {
    let reply = reply.unwrap();
    assert_eq!(reply.boxes.len(), 1);
    assert_eq!(
        (
            reply.granted,
            reply.chooses,
            &reply.then,
            reply.taken,
            reply.reveal
        ),
        (false, false, &None, None, 0)
    );
    (reply.boxes[0].told.clone(), reply.gain)
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
        "a pack of Pikemen keep it. Attack the Pikemen?"
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
    assert_eq!(bank.told(&texts()).text, "A Cyclops Stockpile is here.");
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
        assert_eq!(box_.told(&texts()).text, "TREE0");
    }
    // Too poor: a box alone (the kingdom without gold, without gems; a resource past the bar's
    // never held).
    let poor = Visitor {
        resources: [0; 7],
        ..red()
    };
    let poor_gold = Ask::of(&seen(102, 0, 1 << 13), &poor, &art()).unwrap();
    assert!(!poor_gold.asks());
    assert_eq!(poor_gold.told(&texts()).text, "TREE POOR: gold");
    let poor_gems = Ask::of(&seen(102, 0, 2 << 13), &poor, &art()).unwrap();
    assert_eq!(poor_gems.told(&texts()).text, "TREE POOR: gems");
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
    let (found, gain) = said(tomb.yes(&finds, &art(), &texts()));
    assert_eq!(found.text, "TOMB: Equestrian's Gloves");
    assert_eq!(found.pictures, vec![(8, 70)]);
    assert_eq!((gain.morale, gain.skills), ([-3, -3], [0; 4]));
    // An artifact of primary skills gives them (the Centaur's Axe, 7: +2 Attack).
    let (_, axe) = said(Ask::Tomb(Some(7)).yes(&finds, &art(), &texts()));
    assert_eq!(axe.skills, [2, 0, 0, 0]);
    // Emptied: three pictures of bad morale, the morale lost.
    let empty = ask(&seen(108, 0, 0xFF88_C01E)).unwrap();
    assert_eq!(empty, Ask::Tomb(None));
    let (told, gain) = said(empty.yes(&finds, &art(), &texts()));
    assert_eq!(told.text, "TOMB0");
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
        "several Monks keep the Monastery."
    );
    assert!(guarded.yes(&tables().map.finds, &art(), &texts()).is_none());
    // Another player's too; a creature below level 5 guards nothing (its flag, then its hire:
    // below); no creature known, none.
    assert!(matches!(
        ask(&dwelling(35, 8, 3, Some(1))),
        Some(Ask::Guarded { .. })
    ));
    assert!(!guarded.flags());
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
            room: true,
            flag: false
        }
    );
    assert!(hire.asks() && hire.chooses() && !hire.beside());
    assert_eq!(hire.told(&texts()).text, "[Guardhouse] recruit Pikemen?");
    let finds = tables().map.finds;
    let (joined, gain) = said(hire.yes(&finds, &art(), &texts()));
    assert_eq!(joined.text, "14 Pikemen come along.");
    assert_eq!(gain.army[0], Some(pikemen(14)));
    assert_eq!(gain.army[1..], [None; 6]);
    // One joins by its singular; none to hire; no room (seven other stacks), the army as it was.
    let one = Ask::Hire {
        dwelling: 56,
        joins: pikemen(1),
        free: true,
        room: true,
        flag: false,
    };
    assert_eq!(
        said(one.yes(&finds, &art(), &texts())).0.text,
        "1 Pikeman come along."
    );
    let none = Ask::Hire {
        dwelling: 56,
        joins: pikemen(0),
        free: true,
        room: true,
        flag: false,
    };
    let (told, gain) = said(none.yes(&finds, &art(), &texts()));
    assert_eq!(told.text, "No Pikemen to hire.");
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
    let (told, gain) = said(crowded.yes(&finds, &art(), &texts()));
    assert_eq!(told.text, "No room for the Pikemen");
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
    // A colour past the teams is no ally: its flag taken (below).
    let stranger = Ask::of(&dwelling(56, 0, 14, Some(9)), &red(), &art());
    assert!(matches!(stranger, Some(Ask::Hire { flag: true, .. })));
    assert!(!hire.flags() && !ally.flags());
}

#[test]
fn a_dwelling_of_another_team_without_guards_takes_its_flag_then_asks_its_hire() {
    // E24 M4 T2d1 (design §5.6): a neutral Guardhouse (`flag-dwelling`): Red's flag raised first,
    // then its hire as Red's own, its Pikemen free (level 1), told as `ask-hire`'s.
    let hire = ask(&dwelling(56, 0, 14, None)).unwrap();
    assert_eq!(
        hire,
        Ask::Hire {
            dwelling: 56,
            joins: pikemen(14),
            free: true,
            room: true,
            flag: true
        }
    );
    assert!(hire.flags() && hire.asks() && hire.chooses() && !hire.beside());
    assert_eq!(hire.told(&texts()).text, "[Guardhouse] recruit Pikemen?");
    let (joined, gain) = said(hire.yes(&tables().map.finds, &art(), &texts()));
    assert_eq!(joined.text, "14 Pikemen come along.");
    assert_eq!(gain.army[0], Some(pikemen(14)));
    // Blue's alike; a level-2 creature's flag taken, then the recruit dialog (not built).
    assert_eq!(ask(&dwelling(56, 0, 14, Some(1))), Some(hire));
    let archers = ask(&dwelling(57, 2, 9, None)).unwrap();
    assert!(matches!(
        archers,
        Ask::Hire {
            free: false,
            flag: true,
            ..
        }
    ));
    assert!(archers.yes(&tables().map.finds, &art(), &texts()).is_none());
}

// Part A2: a monster's offers, a Border Guard's, a Cartographer's and Pandora's Box's questions.

/// The recordings' creatures' numbers (`CRTRAITS.TXT`'s gold and AI value): the Pikemen's 60 and
/// 80, the Peasants' 10 and 15.
const PIKEMEN: Traits = Traits {
    gold: 60,
    value: 80,
};
const PEASANTS: Traits = Traits {
    gold: 10,
    value: 15,
};

/// A wandering monster of `creature` with `traits` and the tile `word`.
fn monster(creature: u32, traits: Traits, word: u32) -> Seen {
    Seen {
        monster: Some(traits),
        ..seen(54, creature, word)
    }
}

/// A monster's word: its count and aggression (−16 to 15), never flees, a message.
fn word(count: u32, aggression: i32, never_flees: bool, message: bool) -> u32 {
    let aggression = u32::try_from(aggression.rem_euclid(32)).unwrap();
    count | aggression << 12 | u32::from(never_flees) << 17 | u32::from(message) << 31
}

#[test]
fn a_heros_power_against_a_monster() {
    // `ask-join`'s hero: 18 Centaurs (AI value 100), Attack 1, Defense 3: 1800 × √(1.05 × 1.15).
    let value = |c: u16| if c == 14 { 100 } else { 0 };
    assert_eq!(power(&red().army, value, [1, 3]), 1977);
    // The skills held in 0 to 99; no army no power; a huge army saturates, no overflow.
    assert_eq!(power(&red().army, value, [-5, 0]), 1800);
    assert_eq!(
        power(&red().army, value, [127, 127]),
        power(&red().army, value, [99, 99])
    );
    assert_eq!(power(&[None; 7], value, [1, 3]), 0);
    let huge = [Some(Stack {
        creature: 0,
        count: i32::MAX,
    }); 7];
    assert!(power(&huge, |_| i32::MAX, [99, 99]) > 0);
    assert_eq!(power(&huge, |_| -1, [0, 0]), 0);
}

#[test]
fn the_strength_steps_and_the_sympathy() {
    // r of 7 or more 11 (no strength too); 1 to 7 ⌊2r − 2⌋; above 0.5 −1, above 0.333 −2; −3.
    assert_eq!(step(700, 100), 11);
    assert_eq!(step(5, 0), 11);
    assert_eq!(step(1977, 150), 11);
    assert_eq!(step(1977, 400), 7);
    assert_eq!(step(100, 100), 0);
    assert_eq!(step(149, 100), 0);
    assert_eq!(step(150, 100), 1);
    assert_eq!(step(51, 100), -1);
    assert_eq!(step(50, 100), -2);
    assert_eq!(step(334, 1000), -2);
    assert_eq!(step(333, 1000), -3);
    // None of its type or its pair's: 0; some 1; more than half 2 (the upgrade counts).
    let army = |stacks: &[(u16, i32)]| {
        let mut army = [None; 7];
        for (slot, &(creature, count)) in army.iter_mut().zip(stacks) {
            *slot = Some(Stack { creature, count });
        }
        army
    };
    assert_eq!(sympathy(&army(&[(14, 18)]), 0), 0);
    assert_eq!(sympathy(&army(&[(14, 18), (0, 5)]), 0), 1);
    assert_eq!(sympathy(&army(&[(14, 18), (1, 19)]), 0), 2);
    assert_eq!(sympathy(&army(&[(14, 18), (1, 18)]), 0), 1);
    // A neutral creature has no pair: its own type alone.
    assert_eq!(sympathy(&army(&[(138, 3)]), 139), 0);
    assert_eq!(sympathy(&army(&[(139, 3)]), 139), 2);
    // The aggression's sign.
    assert_eq!(aggression(word(10, -4, false, false)), -4);
    assert_eq!(aggression(word(5, 3, false, false)), 3);
    assert_eq!(aggression(word(5, 15, false, false)), 15);
    assert_eq!(aggression(word(5, -16, false, false)), -16);
}

#[test]
fn a_monsters_free_join() {
    // `ask-join`: ten compliant Peasants (the recorded word 0x1C00A), its question, its OK the
    // join (the status window built again, the monster faded out), its Cancel the flight's.
    assert_eq!(word(10, -4, false, false), 114_698);
    let join = ask(&monster(139, PEASANTS, 114_698)).unwrap();
    let peasants = Stack {
        creature: 139,
        count: 10,
    };
    assert_eq!(
        join,
        Ask::Monster {
            stack: peasants,
            offer: Offer::Join,
            room: true,
            flees: true,
        }
    );
    assert!(join.asks() && !join.chooses() && join.beside());
    assert_eq!(join.attacker(), Some(278));
    let told = join.told(&texts());
    assert_eq!(told.text, "{Followers} Peasants ask to come.");
    assert!(told.pictures.is_empty());
    let yes = join.yes(&tables().map.finds, &art(), &texts()).unwrap();
    let mut gain = Gain::default();
    gain.army[0] = Some(peasants);
    assert_eq!(
        yes,
        Reply {
            gain,
            chooses: true,
            taken: Some(Fade::Killfade),
            ..Reply::default()
        }
    );
    // `ask-join-no`: declined, the flight asked; its OK the battle (not built), its Cancel the
    // monster faded out without its treasure.
    let flight = *join.no().unwrap().then.unwrap();
    assert_eq!(
        flight,
        Ask::Monster {
            stack: peasants,
            offer: Offer::Flee,
            room: true,
            flees: true,
        }
    );
    assert_eq!(flight.told(&texts()).text, "Peasants run.");
    assert_eq!(flight.yes(&tables().map.finds, &art(), &texts()), None);
    assert_eq!(
        flight.no(),
        Some(Reply {
            taken: Some(Fade::Killfade),
            ..Reply::default()
        })
    );
    // No room in the army: the army dialog (not built).
    let full = Visitor {
        army: [Some(Stack {
            creature: 14,
            count: 1,
        }); 7],
        ..red()
    };
    let crowded = Ask::of(&monster(139, PEASANTS, 114_698), &full, &art()).unwrap();
    assert!(matches!(crowded, Ask::Monster { room: false, .. }));
    assert_eq!(crowded.yes(&tables().map.finds, &art(), &texts()), None);
}

#[test]
fn a_monsters_join_for_gold() {
    // `ask-join-gold`: five Pikemen of aggression 3 (0x3005), the hero's Diplomacy Basic: 89 then
    // 90 with the price, the gold's picture; its OK pays and joins.
    let diplomat = Visitor {
        diplomacy: 1,
        ..red()
    };
    let gold = Ask::of(&monster(0, PIKEMEN, 12_293), &diplomat, &art()).unwrap();
    assert_eq!(
        gold,
        Ask::Monster {
            stack: pikemen(5),
            offer: Offer::Gold(300),
            room: true,
            flees: true,
        }
    );
    let told = gold.told(&texts());
    assert_eq!(told.text, "An offer:\n\n5 Pikemen: 300 gold.");
    assert_eq!(told.pictures, vec![(6, 300)]);
    let yes = gold.yes(&tables().map.finds, &art(), &texts()).unwrap();
    assert_eq!(yes.gain.resources, [0, 0, 0, 0, 0, 0, -300]);
    assert_eq!(yes.gain.army[0], Some(pikemen(5)));
    assert_eq!((yes.chooses, yes.taken), (true, Some(Fade::Killfade)));
    // One creature: 88 with its singular.
    let one = Ask::of(
        &monster(0, PIKEMEN, word(1, 3, false, false)),
        &diplomat,
        &art(),
    );
    assert_eq!(one.unwrap().told(&texts()).text, "One Pikeman: 60 gold.");
    // Too poor, or no Diplomacy: no offer, the flight at once.
    let poor = Visitor {
        resources: [0; 7],
        ..diplomat.clone()
    };
    let flight = Ask::of(&monster(0, PIKEMEN, 12_293), &poor, &art()).unwrap();
    assert!(matches!(
        flight,
        Ask::Monster {
            offer: Offer::Flee,
            ..
        }
    ));
    let flight = Ask::of(&monster(0, PIKEMEN, 12_293), &red(), &art()).unwrap();
    assert!(matches!(
        flight,
        Ask::Monster {
            offer: Offer::Flee,
            ..
        }
    ));
}

#[test]
fn a_monster_that_fights() {
    // An aggression above C fights (not built); a monster that never flees, or of aggression C,
    // fights after its offer declined (the insult: not built); its message (bit 31) not built;
    // no numbers, no question.
    let weak = Visitor {
        power: 100,
        ..red()
    };
    // 10 Peasants of strength 150 against 100: −1, C = −1.
    assert_eq!(
        Ask::of(
            &monster(139, PEASANTS, word(10, 0, false, false)),
            &weak,
            &art()
        ),
        None
    );
    let stays = ask(&monster(139, PEASANTS, word(10, -4, true, false))).unwrap();
    assert!(matches!(
        stays,
        Ask::Monster {
            offer: Offer::Join,
            flees: false,
            ..
        }
    ));
    assert_eq!(stays.no(), None);
    // C = 11 (r ≥ 7): aggression 11 offers nothing and never flees there.
    assert_eq!(
        ask(&monster(139, PEASANTS, word(10, 11, false, false))),
        None
    );
    let insulted = Ask::of(
        &monster(139, PEASANTS, word(10, -1, false, false)),
        &weak,
        &art(),
    );
    assert!(matches!(
        insulted,
        Some(Ask::Monster {
            offer: Offer::Join,
            flees: false,
            ..
        })
    ));
    assert_eq!(
        ask(&monster(139, PEASANTS, word(10, -4, false, true))),
        None
    );
    assert_eq!(ask(&seen(54, 139, 114_698)), None);
}

#[test]
fn a_border_guards_passage_and_a_cartographers_maps() {
    // `ask-guard`: its colour's key held, its question, its OK the guard taken with a pickup's
    // sound; no key, no question (not built).
    let guard = Seen {
        key: true,
        ..seen(9, 0, 0)
    };
    let pass = ask(&guard).unwrap();
    assert_eq!(pass, Ask::Pass);
    assert!(pass.asks() && !pass.chooses() && pass.beside());
    assert_eq!(pass.attacker(), None);
    assert_eq!(pass.told(&texts()).text, "PASS");
    assert_eq!(
        pass.yes(&tables().map.finds, &art(), &texts()),
        Some(Reply {
            taken: Some(Fade::Pickup),
            ..Reply::default()
        })
    );
    assert_eq!(pass.no(), Some(Reply::default()));
    assert_eq!(ask(&seen(9, 0, 0)), None);
    // `ask-cartographer`: a land one, 1 000 gold paid and the land's terrains revealed; entered.
    let maps = ask(&seen(13, 1, 0)).unwrap();
    assert_eq!(maps, Ask::Maps(1));
    assert!(maps.asks() && !maps.chooses() && !maps.beside());
    assert_eq!(maps.told(&texts()).text, "LAND");
    let yes = maps.yes(&tables().map.finds, &art(), &texts()).unwrap();
    assert_eq!(yes.gain.resources, [0, 0, 0, 0, 0, 0, -1000]);
    assert_eq!((yes.reveal, yes.taken, yes.chooses), (0xBF, None, false));
    assert_eq!(maps.no(), Some(Reply::default()));
    let texts = texts();
    assert_eq!(ask(&seen(13, 0, 0)).unwrap().told(&texts).text, "SEA");
    assert_eq!(ask(&seen(13, 2, 0)).unwrap().told(&texts).text, "TUNNELS");
    // Visited, too poor or of a kind past the tables: nothing asked (not built).
    let visited = Seen {
        visited: true,
        ..seen(13, 1, 0)
    };
    assert_eq!(ask(&visited), None);
    let poor = Visitor {
        resources: [0; 7],
        ..red()
    };
    assert_eq!(Ask::of(&seen(13, 1, 0), &poor, &art()), None);
    assert_eq!(ask(&seen(13, 3, 0)), None);
    // A price of an unknown resource is paid by nothing, a kind past the masks reveals nothing.
    let mut odd = art();
    odd.maps_price = [9, 0];
    let free = Ask::of(&seen(13, 1, 0), &red(), &odd).unwrap();
    assert_eq!(
        free.yes(&tables().map.finds, &odd, &texts).unwrap().gain,
        Gain::default()
    );
    assert_eq!(
        Ask::Maps(7)
            .yes(&tables().map.finds, &art(), &texts)
            .unwrap()
            .reveal,
        0
    );
}

#[test]
fn pandoras_box() {
    // `ask-pandora`: 1 500 gold, its question, its OK `grant_rewards`' gold box without the map's
    // text, the box taken with a pickup's sound.
    let gold = Rewards {
        resources: [0, 0, 0, 0, 0, 0, 1500],
        ..Rewards::default()
    };
    let boxed = Seen {
        rewards: Some(gold.clone()),
        ..seen(6, 0, 0)
    };
    let open = ask(&boxed).unwrap();
    assert_eq!(
        open,
        Ask::OpenBox {
            rewards: gold,
            hero: "Mephala".to_string(),
        }
    );
    assert!(open.asks() && !open.chooses() && open.beside());
    assert_eq!(open.told(&texts()).text, "OPEN?");
    let yes = open.yes(&tables().map.finds, &art(), &texts()).unwrap();
    assert_eq!(yes.boxes.len(), 1);
    assert_eq!(yes.boxes[0].told.text, "Mephala: treasure.");
    assert_eq!(yes.boxes[0].told.pictures, vec![(6, 1500)]);
    assert_eq!(yes.boxes[0].shows.resources, [0, 0, 0, 0, 0, 0, 1500]);
    assert_eq!((yes.granted, yes.taken), (true, Some(Fade::Pickup)));
    assert_eq!(open.no(), Some(Reply::default()));
    // A message or guards, no rewards, empty rewards: not built.
    let told = Seen {
        message: Some((String::from("A box."), Vec::new())),
        ..boxed.clone()
    };
    assert_eq!(ask(&told), None);
    assert_eq!(ask(&seen(6, 0, 0)), None);
    let empty = Seen {
        rewards: Some(Rewards::default()),
        ..boxed
    };
    assert_eq!(ask(&empty), None);
}

#[test]
fn part_a1s_cancel_ends_the_visit() {
    assert_eq!(ask(&seen(63, 0, 0)).unwrap().no(), Some(Reply::default()));
    assert_eq!(ask(&seen(63, 0, 0)).unwrap().attacker(), None);
}

#[test]
fn the_tables_choices() {
    let art = art();
    assert_eq!((art.chest, art.arena), ([500, 500], 2));
    assert_eq!((art.school, art.school_price), (1, [6, 1000]));
}

/// What a choice's answer gives that the panels show: `gold` more, `skills` gained.
fn gives(reply: &Reply) -> (i32, [i8; 4]) {
    assert!(reply.boxes.is_empty() && !reply.granted && !reply.chooses);
    assert_eq!((&reply.then, reply.reveal), (&None, 0));
    (reply.gain.resources[6], reply.gain.skills)
}

#[test]
fn a_treasure_chests_choice() {
    // `choose-chest`: 1 000 gold (its word's bits 11 to 14 two steps of 500), style 7 with the
    // "or", the gold's picture then the experience's, 500.
    let chest = ask(&seen(101, 0, 2 << 11)).unwrap();
    assert_eq!(
        chest,
        Ask::Chest {
            gold: 1000,
            experience: 500
        }
    );
    assert!(chest.asks() && chest.chooses() && chest.beside());
    assert_eq!((chest.style(), chest.attacker()), (Style::Choose, None));
    assert_eq!(chest.or(&texts()).as_deref(), Some("or"));
    let told = chest.told(&texts());
    assert_eq!(told.text, "CHEST");
    assert_eq!(told.pictures, vec![(6, 1000), (17, 500)]);
    // The gold's picture the gold, the experience's nothing the panels show; taken either way
    // with a pickup's sound. No Cancel: the default.
    let yes = chest.yes(&tables().map.finds, &art(), &texts()).unwrap();
    assert_eq!(
        (gives(&yes), yes.taken, yes.marks),
        ((1000, [0; 4]), Some(Fade::Pickup), false)
    );
    let second = chest.second().unwrap();
    assert_eq!(
        (gives(&second), second.taken),
        ((0, [0; 4]), Some(Fade::Pickup))
    );
    assert_eq!(chest.no(), Some(Reply::default()));
    // An artifact's chest (bit 10): not built.
    assert_eq!(ask(&seen(101, 0, 1 << 10 | 2 << 11)), None);
}

#[test]
fn the_arenas_and_a_schools_choices() {
    // `choose-arena`: style 10, +2 Attack or +2 Defense, no "or"; its Cancel the Attack, marked
    // (MCL-Q33).
    let arena = ask(&seen(4, 0, 0)).unwrap();
    assert_eq!(arena, Ask::Arena(2));
    assert!(arena.asks() && arena.chooses() && !arena.beside());
    assert_eq!(
        (arena.style(), arena.or(&texts())),
        (Style::ChooseCancel, None)
    );
    let told = arena.told(&texts());
    assert_eq!(
        (told.text.as_str(), told.pictures),
        ("ARENA", vec![(31, 2), (32, 2)])
    );
    let yes = arena.yes(&tables().map.finds, &art(), &texts()).unwrap();
    assert_eq!((gives(&yes), yes.taken), ((0, [2, 0, 0, 0]), None));
    assert_eq!(gives(&arena.second().unwrap()), (0, [0, 2, 0, 0]));
    let no = arena.no().unwrap();
    assert_eq!((gives(&no), no.marks), ((0, [2, 0, 0, 0]), true));
    // Visited by the hero: not built.
    let visited = Seen {
        visited: true,
        ..seen(4, 0, 0)
    };
    assert_eq!(ask(&visited), None);
    // `choose-school`: +1 Spell Power or +1 Knowledge for 1 000 gold; its Cancel neither.
    let school = ask(&seen(47, 0, 0)).unwrap();
    assert_eq!(
        school,
        Ask::School {
            gain: 1,
            price: [6, 1000]
        }
    );
    assert_eq!(school.style(), Style::ChooseCancel);
    let told = school.told(&texts());
    assert_eq!(
        (told.text.as_str(), told.pictures),
        ("SCHOOL", vec![(33, 1), (34, 1)])
    );
    let yes = school.yes(&tables().map.finds, &art(), &texts()).unwrap();
    assert_eq!(gives(&yes), (-1000, [0, 0, 1, 0]));
    assert_eq!(gives(&school.second().unwrap()), (-1000, [0, 0, 0, 1]));
    assert_eq!(school.no(), Some(Reply::default()));
    // Visited, or the kingdom too poor: not built (ADVEVENT 72, 73).
    let visited = Seen {
        visited: true,
        ..seen(47, 0, 0)
    };
    assert_eq!(ask(&visited), None);
    let mut poor = red();
    poor.resources[6] = 999;
    assert_eq!(Ask::of(&seen(47, 0, 0), &poor, &art()), None);
    // A price past the resources: free, nothing paid.
    let odd = AskArt {
        school_price: [9, 0],
        ..art()
    };
    let free = Ask::of(&seen(47, 0, 0), &red(), &odd).unwrap();
    assert_eq!(gives(&free.second().unwrap()), (0, [0, 0, 0, 1]));
}

#[test]
fn the_styles_of_the_others() {
    // A question's style 2, a box that asks nothing style 1; neither has a second picture or an
    // "or".
    let pyramid = ask(&seen(63, 0, 0)).unwrap();
    assert_eq!((pyramid.style(), pyramid.second()), (Style::OkCancel, None));
    assert_eq!(pyramid.or(&texts()), None);
    let looted = ask(&seen(16, 0, 1 << 25)).unwrap();
    assert_eq!(looted.style(), Style::Ok);
}

#[test]
fn a_notice_s_box_and_its_answers() {
    // E24 M5 T4c (design §5.28): a box of style 1 in the client's words, no status window chosen
    // before it, not from beside of itself (its tile's, `engine::visits::lay`), no answer of the
    // client's (the host's closing answer), its Cancel's the default.
    use crate::notice::Notice;
    let texts = Catalogue {
        packs: vec![
            Pack::from_json(
                "en",
                r#"{ "map": { "stub": { "battle": "{Battle}\n\nNot yet." } } }"#,
            )
            .unwrap(),
        ],
    };
    let notice = Ask::Notice(Notice::Battle);
    let told = notice.told(&texts);
    assert_eq!(
        (told.text.as_str(), told.pictures.len()),
        ("{Battle}\n\nNot yet.", 0)
    );
    assert_eq!((notice.style(), notice.asks()), (Style::Ok, false));
    assert!(!notice.chooses() && !notice.beside() && !notice.flags());
    let tables = tables();
    assert_eq!(notice.yes(&tables.map.finds, &art(), &texts), None);
    assert_eq!(
        (notice.no(), notice.second()),
        (Some(Reply::default()), None)
    );
    assert_eq!((notice.or(&texts), notice.attacker()), (None, None));
}
