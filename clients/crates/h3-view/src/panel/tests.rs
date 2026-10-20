use super::*;
use alloc::vec::Vec;
use h3_protocol::wire::{Hero, Siege, Tactics};

use crate::record::tests::observation;

const RED_BLUE: [u8; 2] = [0, 1];

fn hero(spell_book: bool, casts: u8) -> Option<Hero> {
    Some(Hero {
        hero_type: 0,
        attack: 1,
        defense: 1,
        power: 1,
        knowledge: 1,
        mana: 10,
        max_mana: 10,
        casts,
        commands_ballista: false,
        commands_tent: false,
        commands_catapult: false,
        spell_book,
    })
}

/// The disabled ones of the seven buttons and the two scroll buttons, in id order.
fn disabled(shown: &Shown) -> Vec<u16> {
    BUTTONS
        .iter()
        .chain(&[LOG_UP, LOG_DOWN])
        .copied()
        .filter(|id| shown.states.get(id).is_some_and(|s| s.disabled))
        .collect::<alloc::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Red human against a computer, both led by heroes; Red's hero has a spell book.
fn battle() -> Observation {
    let mut o = observation();
    o.sides[0].human = true;
    o.sides[0].hero = hero(true, 0);
    o.sides[1].hero = hero(false, 0);
    o
}

#[test]
fn a_humans_turn_enables_what_the_battle_allows() {
    let o = battle();
    let red = shown(&o, Some(Side::Attacker), "a\nb", RED_BLUE);
    // Every button enabled and the scroll buttons too (0x4786CA), none selected.
    assert!(disabled(&red).is_empty());
    assert!(red.states.values().all(|s| !s.selected && !s.hidden));
    assert_eq!(red.players.get(&PICTURE), Some(&0));
    assert_eq!(red.texts.get(&LOG).map(String::as_str), Some("a\nb"));
    // Blue human too, its turn: Auto Combat disabled, the picture in Blue's colours, its hero
    // without a book.
    let mut both = o.clone();
    both.sides[1].human = true;
    let blue = shown(&both, Some(Side::Defender), "", RED_BLUE);
    assert_eq!(disabled(&blue), [AUTO_COMBAT, SPELL_BOOK]);
    assert_eq!(blue.players.get(&PICTURE), Some(&1));
    // A hero that cast this round; the wait phase.
    let mut cast = o.clone();
    cast.sides[0].hero = hero(true, 1);
    cast.wait_phase = true;
    assert_eq!(
        disabled(&shown(&cast, Some(Side::Attacker), "", RED_BLUE)),
        [SPELL_BOOK, WAIT]
    );
    // No hero on the other side: no surrender; none on Red's: no retreat, no book either.
    let mut alone = o.clone();
    alone.sides[1].hero = None;
    assert_eq!(
        disabled(&shown(&alone, Some(Side::Attacker), "", RED_BLUE)),
        [SURRENDER]
    );
    alone.sides[0].hero = None;
    assert_eq!(
        disabled(&shown(&alone, Some(Side::Attacker), "", RED_BLUE)),
        [SURRENDER, RETREAT, SPELL_BOOK]
    );
}

#[test]
fn at_a_town_the_defender_neither_surrenders_nor_retreats_without_its_tunnel() {
    let mut o = battle();
    o.siege = Some(Siege {
        faction: 0,
        fort: 1,
        moat: 0,
        gate_open: false,
        walls: Vec::new(),
        moat_hexes: Vec::new(),
        parts: Vec::new(),
    });
    o.sides[1].human = true;
    o.sides[1].may_retreat = false;
    let defender = shown(&o, Some(Side::Defender), "", RED_BLUE);
    assert_eq!(
        disabled(&defender),
        [SURRENDER, RETREAT, AUTO_COMBAT, SPELL_BOOK]
    );
    o.sides[1].may_retreat = true;
    let tunnel = shown(&o, Some(Side::Defender), "", RED_BLUE);
    assert_eq!(disabled(&tunnel), [SURRENDER, AUTO_COMBAT, SPELL_BOOK]);
    // The attacker may do both.
    let attacker = shown(&o, Some(Side::Attacker), "", RED_BLUE);
    assert_eq!(disabled(&attacker), [AUTO_COMBAT]);
}

#[test]
fn the_tactics_phase_disables_the_book_wait_and_defend() {
    let mut o = battle();
    o.tactics = Some(Tactics {
        side: Side::Attacker,
        columns: 3,
    });
    // The scroll buttons stay as built.
    assert_eq!(
        disabled(&shown(&o, Some(Side::Attacker), "", RED_BLUE)),
        [LOG_UP, LOG_DOWN, SPELL_BOOK, WAIT, DEFEND]
    );
}

#[test]
fn a_computers_turn_disables_the_seven_buttons() {
    let o = battle();
    let blue = shown(&o, Some(Side::Defender), "", RED_BLUE);
    assert_eq!(
        disabled(&blue),
        [
            SURRENDER,
            RETREAT,
            OPTIONS,
            AUTO_COMBAT,
            LOG_UP,
            LOG_DOWN,
            SPELL_BOOK,
            WAIT,
            DEFEND
        ]
    );
    // Built for the side whose turn comes (provisional).
    assert_eq!(blue.players.get(&PICTURE), Some(&1));
}

#[test]
fn before_a_turn_the_panel_is_as_built() {
    let mut o = battle();
    let built = shown(&o, None, "", RED_BLUE);
    assert_eq!(disabled(&built), [LOG_UP, LOG_DOWN]);
    assert_eq!(built.players.get(&PICTURE), Some(&0));
    o.sides[1].human = true;
    assert_eq!(
        disabled(&shown(&o, None, "", RED_BLUE)),
        [AUTO_COMBAT, LOG_UP, LOG_DOWN]
    );
}
