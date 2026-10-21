//! The battle log's tests (E17 M3 T4b-2a), on an invented pack only: the original's text never
//! enters git (ADR-0009).

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use super::{
    Hit, Log, Logged, NEW_FOR, Name, Named, Spread, Stack, defend, entries, heal, luck_good,
    morale_bad, morale_good, shown, strike, text_at, texts, wait, word,
};
use crate::text::Catalogue;
use crate::text::pack::Pack;

const PACK: &str = r#"{
    "battle": { "log": {
        "damage_one": "{{attacker}} hits for {{damage}}.",
        "damage_other": "{{attacker}} hit for {{damage}}.",
        "perish_one": "\nA {{creature}} falls.",
        "perish_other": "\n{{count}} {{creature}} fall.",
        "destroyed": "\n{{creature}} wrecked.",
        "wait_one": "{{creature}} holds.",
        "wait_other": "{{creature}} hold.",
        "defend_one": "{{creature}} braces,\n+{{defense}}.",
        "defend_other": "{{creature}} brace,\n+{{defense}}.",
        "morale-good": "{{creature}}: again.",
        "morale-bad": "{{creature}}: frozen.",
        "luck-good": "{{creature}}: lucky.",
        "heal": "{{healer}} mends {{creature}}\nfor {{hit_points}}.",
        "round": "Round.",
        "cast-on": "{{hero}}: {{spell}} on {{creature}}.",
        "resisted": "{{creature}} {{verb}} it.",
        "disrupted": "Ray takes {{defense}}."
    } },
    "word": {
        "creature_one": "beast", "creature_other": "beasts",
        "resist_one": "shrugs", "resist_other": "shrug"
    },
    "spell": { "t:arrow": "Dart" },
    "hero": { "t:mage": "Mage" },
    "creature": {
        "t:ant_one": "Ant", "t:ant_other": "Ants",
        "t:bee_one": "Bee", "t:bee_other": "Bees",
        "t:cart_one": "Cart", "t:cart_other": "Carts"
    }
}"#;

fn catalogue() -> Catalogue {
    Catalogue {
        packs: vec![Pack::from_json("en", PACK).unwrap()],
    }
}

fn stack(creature: &str, count: u32) -> Stack<'_> {
    Stack { creature, count }
}

fn hit(creature: &str, damage: u64, kills: u32) -> Hit<'_> {
    Hit {
        creature,
        war_machine: false,
        damage,
        kills,
    }
}

fn line(attacker: Stack<'_>, spread: Spread, hits: &[Hit<'_>]) -> String {
    strike(&catalogue(), attacker, spread, hits)
}

#[test]
fn a_strike_names_its_attacker_and_target_by_their_numbers() {
    let ants = stack("creature.t:ant", 4);
    assert_eq!(
        line(ants, Spread::Target, &[hit("creature.t:bee", 37, 0)]),
        "Ants hit for 37."
    );
    assert_eq!(
        line(
            stack("creature.t:ant", 1),
            Spread::Target,
            &[hit("creature.t:bee", 9, 1)]
        ),
        "Ant hits for 9.\nA Bee falls."
    );
    assert_eq!(
        line(ants, Spread::Target, &[hit("creature.t:bee", 37, 3)]),
        "Ants hit for 37.\n3 Bees fall."
    );
}

#[test]
fn a_war_machine_is_destroyed() {
    let cart = Hit {
        war_machine: true,
        ..hit("creature.t:cart", 120, 1)
    };
    assert_eq!(
        line(stack("creature.t:ant", 20), Spread::Target, &[cart]),
        "Ants hit for 120.\nCart wrecked."
    );
}

#[test]
fn a_breath_adds_in_a_second_creature_but_not_its_targets_own() {
    let ants = stack("creature.t:ant", 2);
    let first = hit("creature.t:bee", 10, 1);
    // Another creature behind: both added up, named by the word for creatures.
    assert_eq!(
        line(ants, Spread::Target, &[first, hit("creature.t:cart", 5, 1)]),
        "Ants hit for 15.\n2 beasts fall."
    );
    // The same creature behind: the target's alone (the original's).
    assert_eq!(
        line(ants, Spread::Target, &[first, hit("creature.t:bee", 5, 2)]),
        "Ants hit for 10.\nA Bee falls."
    );
    // A third hit is never the breath's.
    assert_eq!(
        line(
            ants,
            Spread::Target,
            &[
                first,
                hit("creature.t:cart", 5, 0),
                hit("creature.t:ant", 7, 1)
            ]
        ),
        "Ants hit for 15.\nA beast falls."
    );
}

#[test]
fn hits_around_add_up() {
    let ants = stack("creature.t:ant", 3);
    let bees = [hit("creature.t:bee", 4, 0), hit("creature.t:bee", 6, 2)];
    assert_eq!(
        line(ants, Spread::Around, &bees),
        "Ants hit for 10.\n2 Bees fall."
    );
    let mixed = [
        hit("creature.t:bee", 4, 1),
        hit("creature.t:cart", 6, 0),
        hit("creature.t:bee", 1, 0),
    ];
    assert_eq!(
        line(ants, Spread::Around, &mixed),
        "Ants hit for 11.\nA beast falls."
    );
    assert_eq!(line(ants, Spread::Around, &[]), "Ants hit for 0.");
    assert_eq!(line(ants, Spread::Target, &[]), "Ants hit for 0.");
}

#[test]
fn huge_numbers_saturate() {
    let ants = stack("creature.t:ant", 3);
    let big = [
        hit("creature.t:bee", u64::MAX, u32::MAX),
        hit("creature.t:bee", 1, 1),
    ];
    let text = line(ants, Spread::Around, &big);
    assert_eq!(
        text,
        alloc::format!("Ants hit for {}.\n{} Bees fall.", i64::MAX, u32::MAX)
    );
}

#[test]
fn turns_morale_and_luck() {
    let c = catalogue();
    let one = stack("creature.t:bee", 1);
    let many = stack("creature.t:bee", 12);
    assert_eq!(wait(&c, one), "Bee holds.");
    assert_eq!(wait(&c, many), "Bees hold.");
    assert_eq!(defend(&c, one, 1), "Bee braces,\n+1.");
    assert_eq!(defend(&c, many, 4), "Bees brace,\n+4.");
    assert_eq!(morale_good(&c, many), "Bees: again.");
    assert_eq!(morale_bad(&c, one), "Bee: frozen.");
    assert_eq!(luck_good(&c, many), "Bees: lucky.");
}

#[test]
fn a_line_becomes_one_entry_or_two() {
    let one_line = |_: &str| 1;
    let two_lines = |_: &str| 2;
    let split = |list: &[&str]| list.iter().map(|s| String::from(*s)).collect::<Vec<_>>();
    assert_eq!(entries("Plain.", &two_lines), split(&["Plain."]));
    assert_eq!(entries("Hit.\nFall.", &one_line), split(&["Hit. Fall."]));
    assert_eq!(
        entries("Hit.\nFall.", &two_lines),
        split(&["Hit.", "Fall."])
    );
    assert_eq!(entries("A,\nb\nc", &one_line), split(&["A, b\nc"]));
    assert_eq!(entries("A,\nb\nc", &two_lines), split(&["A,", "b\nc"]));
    // The joined line is what is measured.
    assert_eq!(
        entries("ab\ncd", &|s: &str| i64::from(s != "ab cd") + 1),
        split(&["ab cd"])
    );
}

#[test]
fn the_item_shows_two_entries_from_the_first() {
    let log: Vec<String> = ["One.", "Two.", "Three."].map(String::from).into();
    assert_eq!(shown(&log, 0), "One.\nTwo.");
    assert_eq!(shown(&log, 2), "Three.");
    assert_eq!(shown(&log, 3), "");
    assert_eq!(shown(&[], 0), "");
}

/// The stacks [`word`] names: 1 an Ant, 2 a Bee, 3 a Cart (a war machine).
fn named(id: u8) -> Option<Named> {
    let (creature, war_machine) = match id {
        1 => ("creature.t:ant", false),
        2 => ("creature.t:bee", false),
        3 => ("creature.t:cart", true),
        _ => return None,
    };
    Some(Named {
        creature: creature.into(),
        war_machine,
    })
}

/// The spell 15 and the attacker's hero [`word`] names (E17 M3 T7b-1).
fn key(name: Name) -> Option<String> {
    match name {
        Name::Spell(15) => Some("spell.t:arrow".into()),
        Name::Hero(0) => Some("hero.t:mage".into()),
        _ => None,
    }
}

fn worded(logged: &Logged) -> Option<String> {
    word(&catalogue(), logged, &named, &key)
}

#[test]
fn a_cast_names_its_hero_spell_and_target() {
    let cast = |side, spell, target, count| Logged::Cast {
        side,
        spell,
        target,
        count,
    };
    assert_eq!(
        worded(&cast(0, 15, 1, 3)).as_deref(),
        Some("Mage: Dart on Ants.")
    );
    assert_eq!(
        worded(&cast(0, 15, 2, 1)).as_deref(),
        Some("Mage: Dart on Bee.")
    );
    for unnamed in [cast(1, 15, 1, 3), cast(0, 16, 1, 3), cast(0, 15, 9, 3)] {
        assert_eq!(worded(&unnamed), None);
    }
    let resisted = |stack, count| Logged::Resisted { stack, count };
    assert_eq!(worded(&resisted(2, 1)).as_deref(), Some("Bee shrugs it."));
    assert_eq!(worded(&resisted(1, 4)).as_deref(), Some("Ants shrug it."));
    assert_eq!(worded(&resisted(9, 4)), None);
    let damage = |spell, hit| Logged::SpellDamage { spell, hit };
    assert_eq!(
        worded(&damage(15, (1, 20, 0))).as_deref(),
        Some("Dart hits for 20.")
    );
    assert_eq!(
        worded(&damage(15, (3, 40, 1))).as_deref(),
        Some("Dart hits for 40.\nCart wrecked.")
    );
    assert_eq!(worded(&damage(16, (1, 20, 0))), None);
    assert_eq!(worded(&damage(15, (9, 20, 0))), None);
}

#[test]
fn a_heal_names_the_tent_and_its_patient() {
    let c = catalogue();
    assert_eq!(
        heal(
            &c,
            stack("creature.t:cart", 1),
            stack("creature.t:ant", 3),
            25
        ),
        "Cart mends Ants\nfor 25."
    );
}

#[test]
fn each_happening_has_its_line() {
    let strike = Logged::Strike {
        attacker: 1,
        count: 2,
        spread: Spread::Target,
        hits: vec![(2, 30, 1), (3, 5, 1)],
    };
    // Two kinds hit: the word for creatures; the second a war machine that stays unnamed.
    assert_eq!(
        worded(&strike).as_deref(),
        Some("Ants hit for 35.\n2 beasts fall.")
    );
    let wrecked = Logged::Strike {
        attacker: 2,
        count: 1,
        spread: Spread::Around,
        hits: vec![(3, 9, 1)],
    };
    assert_eq!(
        worded(&wrecked).as_deref(),
        Some("Bee hits for 9.\nCart wrecked.")
    );
    let wait = Logged::Wait { stack: 1, count: 1 };
    assert_eq!(worded(&wait).as_deref(), Some("Ant holds."));
    // A defend's gain: max(1, defense × 20 / 100), 7 → 1, 0 → 1, 10 → 2 (E17 M3 T7b-2a: the
    // defense the line carries, Disrupting Ray's lowered one).
    let defend = Logged::Defend {
        stack: 2,
        count: 4,
        defense: 7,
    };
    assert_eq!(worded(&defend).as_deref(), Some("Bees brace,\n+1."));
    let zero = Logged::Defend {
        stack: 2,
        count: 1,
        defense: 0,
    };
    assert_eq!(worded(&zero).as_deref(), Some("Bee braces,\n+1."));
    let ten = Logged::Defend {
        stack: 1,
        count: 1,
        defense: 10,
    };
    assert_eq!(worded(&ten).as_deref(), Some("Ant braces,\n+2."));
    let disrupted = Logged::Disrupted {
        stack: 1,
        lowered: 3,
    };
    assert_eq!(worded(&disrupted).as_deref(), Some("Ray takes 3."));
    let heal = Logged::Heal {
        healer: 3,
        target: 1,
        counts: [1, 5],
        hit_points: 12,
    };
    assert_eq!(worded(&heal).as_deref(), Some("Cart mends Ants\nfor 12."));
    let good = Logged::Morale {
        stack: 1,
        count: 1,
        good: true,
    };
    let bad = Logged::Morale {
        stack: 2,
        count: 2,
        good: false,
    };
    assert_eq!(worded(&good).as_deref(), Some("Ant: again."));
    assert_eq!(worded(&bad).as_deref(), Some("Bees: frozen."));
    let luck = Logged::Luck { stack: 1, count: 3 };
    assert_eq!(worded(&luck).as_deref(), Some("Ants: lucky."));
    assert_eq!(worded(&Logged::Round).as_deref(), Some("Round."));
    // What writes no line, and a stack the names lack.
    assert_eq!(worded(&Logged::Action), None);
    assert_eq!(worded(&Logged::Turn(1)), None);
    for ghost in [
        Logged::Strike {
            attacker: 9,
            count: 1,
            spread: Spread::Target,
            hits: vec![],
        },
        Logged::Strike {
            attacker: 1,
            count: 1,
            spread: Spread::Target,
            hits: vec![(9, 1, 0)],
        },
        Logged::Wait { stack: 9, count: 1 },
        Logged::Defend {
            stack: 9,
            count: 1,
            defense: 1,
        },
        Logged::Heal {
            healer: 9,
            target: 1,
            counts: [1, 1],
            hit_points: 1,
        },
        Logged::Heal {
            healer: 3,
            target: 9,
            counts: [1, 1],
            hit_points: 1,
        },
        Logged::Morale {
            stack: 9,
            count: 1,
            good: true,
        },
        Logged::Luck { stack: 9, count: 1 },
    ] {
        assert_eq!(worded(&ghost), None);
    }
}

/// Each character a line: a text of more than one character takes two lines or more.
fn by_chars(text: &str) -> i64 {
    i64::try_from(text.chars().count()).unwrap()
}

/// Ten characters to a line.
fn tens(text: &str) -> i64 {
    i64::try_from(text.chars().count().div_ceil(10)).unwrap()
}

#[test]
fn new_entries_show_and_keep_the_status_line_out() {
    let mut log = Log::default();
    log.status("hint", 0, false);
    assert_eq!(log.text, "hint");
    // One entry, then a line split in two: at most two new, the log shown from the first new.
    log.add("one", 100, &tens);
    assert_eq!((log.fresh, log.first, log.text.as_str()), (1, 0, "one"));
    log.add("two\nthree", 200, &by_chars);
    assert_eq!(log.entries, ["one", "two", "three"]);
    assert_eq!(
        (log.fresh, log.first, log.text.as_str()),
        (2, 1, "two\nthree")
    );
    // The status line waits 3 000 ms from the last showing, unless forced.
    log.status("hint", 200 + NEW_FOR - 1, false);
    assert_eq!(log.text, "two\nthree");
    log.status("forced", 300, true);
    assert_eq!(log.text, "forced");
    log.status("hint", 200 + NEW_FOR, false);
    assert_eq!((log.fresh, log.text.as_str()), (2, "hint"));
}

#[test]
fn settled_entries_empty_the_status_line() {
    let mut log = Log::default();
    // Nothing new: nothing happens.
    log.status("hint", 0, false);
    log.settle(5000);
    assert_eq!(log.text, "hint");
    log.add("one", 100, &tens);
    log.settle(100 + NEW_FOR - 1);
    assert_eq!((log.fresh, log.text.as_str()), (1, "one"));
    log.settle(100 + NEW_FOR);
    assert_eq!((log.fresh, log.text.as_str()), (0, ""));
}

#[test]
fn the_scroll_keeps_two_entries_in_view() {
    let mut log = Log::default();
    // Nothing to show: the text stays.
    log.scroll(1, 0);
    assert_eq!((log.first, log.text.as_str()), (0, ""));
    for (i, line) in ["a", "b", "c", "d"].iter().enumerate() {
        log.add(line, u32::try_from(i).unwrap(), &tens);
    }
    // Two new at most: the last two show.
    assert_eq!((log.fresh, log.first, log.text.as_str()), (2, 2, "c\nd"));
    // Up and down by one, the next-to-last at most, the first at least.
    log.scroll(-1, 10);
    assert_eq!((log.first, log.text.as_str(), log.at), (1, "b\nc", 10));
    log.scroll(3, 11);
    assert_eq!((log.first, log.text.as_str()), (2, "c\nd"));
    log.scroll(-5, 12);
    assert_eq!((log.first, log.text.as_str()), (0, "a\nb"));
    // One entry: the first.
    let mut one = Log::default();
    one.add("a", 0, &tens);
    one.scroll(1, 1);
    assert_eq!((one.first, one.text.as_str()), (0, "a"));
}

#[test]
fn the_item_follows_the_lines_and_turns() {
    let human = |stack: u8| stack == 1;
    let logged = [
        (0, Logged::Turn(1)),
        (10, Logged::Wait { stack: 1, count: 1 }),
        (10, Logged::Turn(2)),
        (20, Logged::Action),
        (20, Logged::Wait { stack: 9, count: 1 }),
        (1000, Logged::Turn(1)),
        (3010, Logged::Turn(2)),
        (3010, Logged::Action),
        (3500, Logged::Round),
        (6500, Logged::Turn(1)),
    ];
    let got = texts(&logged, &worded, &tens, &human);
    assert_eq!(
        got,
        [
            (0, String::new()),
            (10, "Ant holds.".into()),
            (3010, String::new()),
            (3500, "Round.".into()),
            (6500, String::new()),
        ]
    );
    assert_eq!(text_at(&got, 9), "");
    assert_eq!(text_at(&got, 3499), "");
    assert_eq!(text_at(&got, 4000), "Round.");
    assert_eq!(text_at(&[], 4000), "");
}

#[test]
fn the_mouses_hints_and_the_scrolls_enter_the_item() {
    // E17 M3 T6c-2: a hint replaces the item unless new entries showed less than 3 000 ms ago
    // and it is not forced; a human's turn only settles; a scroll shows the entries again.
    let human = |stack: u8| stack == 1;
    let status = |text: &str, forced| Logged::Status {
        text: text.into(),
        forced,
    };
    let logged = [
        (0, Logged::Turn(1)),
        (0, status("Walk here.", false)),
        (10, Logged::Wait { stack: 1, count: 1 }),
        (20, status("Walk here.", false)),
        (30, status("Hit them", true)),
        (40, Logged::Scroll(-1)),
        (3100, Logged::Turn(1)),
        (3100, status("", false)),
    ];
    let got = texts(&logged, &worded, &tens, &human);
    assert_eq!(
        got,
        [
            (0, String::new()),
            (0, "Walk here.".into()),
            (10, "Ant holds.".into()),
            (30, "Hit them".into()),
            (40, "Ant holds.".into()),
            (3100, String::new()),
        ]
    );
    assert_eq!(worded(&status("x", true)), None);
    assert_eq!(worded(&Logged::Scroll(1)), None);
}
