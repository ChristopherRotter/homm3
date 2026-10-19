//! The battle log's tests (E17 M3 T4b-2a), on an invented pack only: the original's text never
//! enters git (ADR-0009).

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use super::{
    Hit, Spread, Stack, defend, entries, luck_good, morale_bad, morale_good, strike, wait,
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
        "luck-good": "{{creature}}: lucky."
    } },
    "word": { "creature_one": "beast", "creature_other": "beasts" },
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
    assert_eq!(entries("Plain.", two_lines), split(&["Plain."]));
    assert_eq!(entries("Hit.\nFall.", one_line), split(&["Hit. Fall."]));
    assert_eq!(entries("Hit.\nFall.", two_lines), split(&["Hit.", "Fall."]));
    assert_eq!(entries("A,\nb\nc", one_line), split(&["A, b\nc"]));
    assert_eq!(entries("A,\nb\nc", two_lines), split(&["A,", "b\nc"]));
    // The joined line is what is measured.
    assert_eq!(
        entries("ab\ncd", |s: &str| i64::from(s != "ab cd") + 1),
        split(&["ab cd"])
    );
}
