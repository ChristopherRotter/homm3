//! The hero's info window's tests (E17 M6 T3b1), on an invented pack (the original's text never
//! enters git, ADR-0009).

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::{Affects, Hero, MoraleLuck, Observation, Side, Source};
use h3_ui::text::Catalogue;
use h3_ui::text::pack::Pack;
use h3_ui::ui::Item;

use super::{
    BACKGROUND, Info, LUCK, LUCK_LABEL, MORALE, MORALE_LABEL, POINTS, PORTRAIT, SKILLS, at, values,
    window,
};
use crate::record::tests::observation;
use crate::stats::Field;

const PACK: &str = r#"{
    "battle": {
        "hero-info": {
            "label": "{{word}}:",
            "attack": "A",
            "defense": "D",
            "power": "P",
            "knowledge": "K",
            "morale": "M",
            "luck": "L",
            "spell-points": "SP",
            "points": "{{word}}\n{{points}}/{{most}}"
        }
    }
}"#;

fn catalogue() -> Catalogue {
    Catalogue {
        packs: vec![Pack::from_json("en", PACK).unwrap()],
    }
}

fn hero(hero_type: u16, skills: [u8; 4], mana: [u16; 2]) -> Hero {
    Hero {
        hero_type,
        attack: skills[0],
        defense: skills[1],
        power: skills[2],
        knowledge: skills[3],
        mana: mana[0],
        max_mana: mana[1],
        casts: 0,
        commands_ballista: false,
        commands_tent: false,
        commands_catapult: false,
        spell_book: true,
        sex: None,
    }
}

/// The attacker led by hero type 3 in Green's colours, the defender by type 7 with no colour
/// named.
fn battle() -> Observation {
    let mut o = observation();
    o.sides[0].colour = Some(2);
    o.sides[0].hero = Some(hero(3, [2, 1, 1, 1], [10, 10]));
    o.sides[1].colour = None;
    o.sides[1].hero = Some(hero(7, [120, 0, 0, 150], [5, 1500]));
    o
}

fn portrait(hero_type: u16) -> Option<String> {
    (hero_type == 3).then(|| String::from("HPL003.pcx"))
}

#[test]
fn the_layout_is_the_constructors() {
    // 0x46C370: 78 x 202 at (1, 135) and (721, 135), its 15 items in the constructor's order.
    let layout = crate::scene::tests::layout().hero_info;
    assert_eq!(layout.places, [[1, 135], [721, 135]]);
    assert_eq!(layout.dialog.shadow, None);
    let ids: Vec<u16> = layout.dialog.items.iter().map(Item::id).collect();
    assert_eq!(ids, (2100..=2114).collect::<Vec<_>>());
    let Some(Item::Picture(back)) = layout.dialog.item(BACKGROUND) else {
        panic!("no background");
    };
    assert_eq!(
        (back.picture.as_str(), back.size),
        ("CHrPop.pcx", Some([78, 202]))
    );
    // Each row's label and value share a box, 12 below the last; morale's and luck's labels are
    // colour 1, the rest 4.
    let label = |id| match layout.dialog.item(id) {
        Some(Item::Label(l)) => (l.at, l.size, l.colour, l.align),
        _ => panic!("no label {id}"),
    };
    assert_eq!(label(SKILLS), ([9, 75], [60, 12], 4, 0));
    assert_eq!(label(SKILLS + 1), ([9, 75], [60, 12], 4, 2));
    assert_eq!(label(SKILLS + 7), ([9, 111], [60, 12], 4, 2));
    assert_eq!(label(MORALE_LABEL), ([9, 131], [60, 12], 1, 0));
    assert_eq!(label(LUCK_LABEL), ([9, 143], [60, 12], 1, 0));
    assert_eq!(label(POINTS), ([7, 165], [64, 30], 4, 5));
}

#[test]
fn the_window_shows_the_side_s_hero() {
    // 0x46CC10: the background in the hero's colours, the portrait, the rows' words and values,
    // the spell points; morale and luck 0 (frame 3) without a source.
    let layout = crate::scene::tests::layout().hero_info;
    let o = battle();
    let info = Info::of(&o, Side::Attacker, Field::Other, &portrait).unwrap();
    let w = window(&layout, &catalogue(), Side::Attacker, &info);
    assert_eq!(w.dialog.place, [1, 135]);
    assert_eq!(w.shown.players.get(&BACKGROUND), Some(&2));
    assert_eq!(
        w.shown.pictures.get(&PORTRAIT).map(String::as_str),
        Some("HPL003.pcx")
    );
    assert!(!w.shown.state(PORTRAIT).hidden);
    let text = |id| w.shown.texts.get(&id).map_or("", String::as_str);
    let rows: Vec<&str> = (SKILLS..SKILLS + 8).map(text).collect();
    assert_eq!(rows, ["A:", "2", "D:", "1", "P:", "1", "K:", "1"]);
    assert_eq!(text(MORALE_LABEL), "M:");
    assert_eq!(text(LUCK_LABEL), "L:");
    assert_eq!(text(POINTS), "SP\n10/10");
    assert_eq!(w.shown.frames.get(&MORALE), Some(&3));
    assert_eq!(w.shown.frames.get(&LUCK), Some(&3));
}

#[test]
fn the_values_are_held_and_a_hero_without_portrait_shows_none() {
    // The defender's: blue without a colour named (CLT-Q6), at (721, 135); attack and defense held
    // to 0..=99, power and knowledge to 1..=99 (0x46CC10); no portrait: the item hidden.
    let layout = crate::scene::tests::layout().hero_info;
    let o = battle();
    let mut info = Info::of(&o, Side::Defender, Field::Other, &portrait).unwrap();
    assert_eq!((info.player, info.portrait.as_ref()), (1, None));
    info.morale = 5;
    info.luck = -2;
    let w = window(&layout, &catalogue(), Side::Defender, &info);
    assert_eq!(w.dialog.place, [721, 135]);
    assert!(w.shown.state(PORTRAIT).hidden);
    let text = |id| w.shown.texts.get(&id).map_or("", String::as_str);
    let values: Vec<&str> = [1, 3, 5, 7].map(|k| text(SKILLS + k)).to_vec();
    assert_eq!(values, ["99", "0", "1", "99"]);
    assert_eq!(text(POINTS), "SP\n5/1500");
    // Morale's and luck's frames: the value + 3, held to −3..=3.
    assert_eq!(w.shown.frames.get(&MORALE), Some(&6));
    assert_eq!(w.shown.frames.get(&LUCK), Some(&1));
}

#[test]
fn the_right_button_finds_a_hero_s_rectangle_with_a_hero() {
    // "Hexes" 0xFC and 0xFD: over a side's hero's rectangle while it has a hero (0x4769D9,
    // 0x476A4F); nothing over a hex or without a hero.
    let layout = crate::scene::tests::layout();
    let mut o = battle();
    let [attacker, defender] = layout.mouse.heroes;
    let inside = |r: [i32; 4]| [(r[0] + r[2]) / 2, (r[1] + r[3]) / 2];
    assert_eq!(at(&layout, &o, inside(attacker)), Some(Side::Attacker));
    assert_eq!(at(&layout, &o, inside(defender)), Some(Side::Defender));
    assert_eq!(at(&layout, &o, [400, 300]), None);
    o.sides[1].hero = None;
    assert_eq!(at(&layout, &o, inside(defender)), None);
    assert_eq!(Info::of(&o, Side::Defender, Field::Other, &portrait), None);
}

fn morale(amount: i8, affects: Affects, source: Source) -> MoraleLuck {
    MoraleLuck::Morale {
        amount,
        affects,
        source,
    }
}

fn luck(amount: i8, affects: Affects, source: Source) -> MoraleLuck {
    MoraleLuck::Luck {
        amount,
        affects,
        source,
    }
}

const LEADERSHIP: Source = Source::Skill { skill: 6, level: 2 };
const PENDANT: Source = Source::Artifact { artifact: 108 };

#[test]
fn the_hero_s_own_changes_make_its_morale_and_luck() {
    // hero_morale 0x4E39B0, hero_luck 0x4E36C0 (E17 M6 T3b2): the hero's own list, its changes
    // to its own stacks summed and held to -3..=3; its changes to the enemy's and the other
    // hero's changes are not read; the window's frames the values + 3.
    let layout = crate::scene::tests::layout().hero_info;
    let mut o = battle();
    o.morale_and_luck[0] = vec![
        morale(2, Affects::Own, LEADERSHIP),
        morale(3, Affects::Own, PENDANT),
        luck(3, Affects::Own, PENDANT),
        luck(-1, Affects::Own, Source::Temporary),
        luck(-2, Affects::Enemy, Source::Specialty),
    ];
    o.morale_and_luck[1] = vec![
        morale(-1, Affects::Both, Source::Visit { mark: 3 }),
        luck(1, Affects::Enemy, Source::Specialty),
    ];
    assert_eq!(values(&o, Side::Attacker, Field::Other), (3, 2));
    assert_eq!(values(&o, Side::Defender, Field::Other), (-1, 0));
    let info = Info::of(&o, Side::Attacker, Field::Holy, &portrait).unwrap();
    assert_eq!((info.morale, info.luck), (3, 2));
    let w = window(&layout, &catalogue(), Side::Attacker, &info);
    assert_eq!(w.shown.frames.get(&MORALE), Some(&6));
    assert_eq!(w.shown.frames.get(&LUCK), Some(&5));
    // Below -3: held.
    o.morale_and_luck[1] = vec![morale(-4, Affects::Own, Source::Temporary)];
    assert_eq!(values(&o, Side::Defender, Field::Other), (-3, 0));
}

#[test]
fn cursed_ground_and_the_hourglass_make_them_0() {
    // On Cursed Ground both are 0 (the flag +0x53C0 = 2 passed, 0x4769EE); the Hourglass of the
    // Evil Hour (a luck cap reaching the side, worn by either hero) makes luck the cap's value
    // whatever the changes, a negative one too; the Spirit of Oppression (a morale cap) is not
    // read (CLT-Q42).
    let mut o = battle();
    o.morale_and_luck[0] = vec![
        morale(2, Affects::Own, LEADERSHIP),
        luck(-1, Affects::Own, Source::Visit { mark: 1 }),
    ];
    assert_eq!(values(&o, Side::Attacker, Field::Cursed), (0, 0));
    assert_eq!(values(&o, Side::Attacker, Field::Other), (2, -1));
    let hourglass = Source::Artifact { artifact: 85 };
    let spirit = Source::Artifact { artifact: 84 };
    o.morale_and_luck[1] = vec![
        MoraleLuck::LuckAtMost {
            value: 0,
            affects: Affects::Both,
            source: hourglass,
        },
        MoraleLuck::MoraleAtMost {
            value: 0,
            affects: Affects::Both,
            source: spirit,
        },
    ];
    assert_eq!(values(&o, Side::Attacker, Field::Other), (2, 0));
    // The cap reaches its wearer's side too; one to its own side only does not reach the other.
    o.morale_and_luck[1].push(luck(2, Affects::Own, Source::Temporary));
    assert_eq!(values(&o, Side::Defender, Field::Other), (0, 0));
    o.morale_and_luck[1][0] = MoraleLuck::LuckAtMost {
        value: 1,
        affects: Affects::Own,
        source: hourglass,
    };
    assert_eq!(values(&o, Side::Defender, Field::Other), (0, 1));
    assert_eq!(values(&o, Side::Attacker, Field::Other), (2, -1));
    // An empty list (a side without a hero has one): 0.
    o.morale_and_luck[0].clear();
    assert_eq!(values(&o, Side::Attacker, Field::Other), (0, 0));
}
