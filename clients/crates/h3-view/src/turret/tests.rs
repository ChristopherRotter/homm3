//! The turret's info's tests (E17 M6 T2b2), on an invented pack (the original's text never
//! enters git, ADR-0009).

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::{Observation, Side, Siege};
use h3_ui::text::Catalogue;
use h3_ui::text::pack::Pack;

use super::{ATTACK, Line, lines, text};
use crate::record::tests::observation;
use crate::scene::tests::stack;

const PACK: &str = r#"{
    "battle": {
        "turret": {
            "standing": "|{{part}} {{attack}} {{low}}-{{high}}",
            "destroyed": "|{{part}} gone"
        }
    },
    "wall": { "t:castle": { "5": "Up", "13": "Down", "14": "Keep" } }
}"#;

fn catalogue() -> Catalogue {
    Catalogue {
        packs: vec![Pack::from_json("en", PACK).unwrap()],
    }
}

/// A siege of a town of `fort` level, every part standing, with the keep's turret of 5 and the
/// towers' of 3 (BAT-SIE-016).
fn siege(fort: u8) -> Observation {
    let mut o = observation();
    let turret = |id, hex, count| {
        let mut s = stack(id, Side::Defender, 0, hex, false);
        s.war_machine = true;
        s.count = count;
        s
    };
    o.stacks = vec![
        stack(0, Side::Attacker, 0, 86, false),
        turret(1, 254, 5),
        turret(2, 255, 3),
        turret(3, 251, 3),
    ];
    o.siege = Some(Siege {
        faction: 0,
        fort,
        moat: 0,
        gate_open: false,
        walls: Vec::new(),
        moat_hexes: Vec::new(),
        parts: vec![2; 8],
        bonuses: Default::default(),
    });
    o
}

fn named(faction: u16, part: usize) -> Option<String> {
    (faction == 0).then(|| format!("wall.t:castle.{part}"))
}

#[test]
fn a_castle_tells_the_keep_and_both_towers() {
    let o = siege(3);
    assert_eq!(
        lines(&o),
        vec![
            Line {
                part: 14,
                count: Some(5)
            },
            Line {
                part: 5,
                count: Some(3)
            },
            Line {
                part: 13,
                count: Some(3)
            },
        ]
    );
    assert_eq!(ATTACK, 10);
    assert_eq!(
        text(&o, &catalogue(), &named).as_deref(),
        Some("|Keep 10 10-15|Up 10 6-9|Down 10 6-9")
    );
}

#[test]
fn a_citadel_tells_the_keep_alone_and_a_fort_nothing() {
    assert_eq!(
        text(&siege(2), &catalogue(), &named).as_deref(),
        Some("|Keep 10 10-15")
    );
    assert_eq!(lines(&siege(1)), Vec::new());
    assert_eq!(text(&siege(1), &catalogue(), &named), None);
    let mut field = siege(3);
    field.siege = None;
    assert_eq!(lines(&field), Vec::new());
    assert_eq!(text(&field, &catalogue(), &named), None);
}

#[test]
fn a_part_without_hit_points_is_destroyed() {
    // The upper tower destroyed (its turret's count 0, BAT-SIE-017): its line says so; a part
    // missing from the observation's list too. A standing part whose turret the observation lacks
    // counts 0; a name the catalogue lacks is empty.
    let mut o = siege(3);
    // The parts by wall-shot target: the upper tower's 0, the lower tower's 6, the keep's 7.
    let parts = &mut o.siege.as_mut().unwrap().parts;
    parts[0] = 0;
    parts.truncate(7);
    o.stacks[2].count = 0;
    o.stacks[2].alive = false;
    o.stacks.retain(|s| s.hex != 251);
    assert_eq!(
        text(&o, &catalogue(), &named).as_deref(),
        Some("|Keep gone|Up gone|Down 10 0-0")
    );
    o.siege.as_mut().unwrap().faction = 1;
    assert_eq!(
        text(&o, &catalogue(), &named).as_deref(),
        Some("| gone| gone| 10 0-0")
    );
}
