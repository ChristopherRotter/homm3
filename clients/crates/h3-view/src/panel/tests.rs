use super::*;
use alloc::vec::Vec;
use h3_protocol::wire::{Event, Hero, Siege, Tactics, Target};

use crate::record::tests::observation;

const RED_BLUE: [u8; 2] = [0, 1];

/// Auto Combat neither playing nor selected.
const NO_AUTO: Autoplay = Autoplay {
    plays: false,
    selected: false,
};

/// What the panel keeps before any human turn: `side`'s colours, the scroll buttons as built.
fn by(side: Side) -> Kept {
    Kept {
        colours: side,
        scroll: false,
    }
}

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
        sex: None,
    })
}

/// The disabled ones of the seven buttons and the two scroll buttons, in id order.
fn disabled(shown: &Shown) -> Vec<u16> {
    BUTTONS
        .iter()
        .chain(&[LOG_UP, LOG_DOWN])
        .copied()
        .filter(|id| shown.states.get(id).is_some_and(|s| s.disabled))
        .collect::<BTreeSet<_>>()
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
    let red = shown(
        &o,
        Some(Side::Attacker),
        (by(Side::Attacker), NO_AUTO),
        "a\nb",
        RED_BLUE,
    );
    // Every button enabled and the scroll buttons too (0x4786CA), none selected.
    assert!(disabled(&red).is_empty());
    assert!(red.states.values().all(|s| !s.selected && !s.hidden));
    assert_eq!(red.players.get(&PICTURE), Some(&0));
    assert_eq!(red.texts.get(&LOG).map(String::as_str), Some("a\nb"));
    // Blue human too, its turn: Auto Combat disabled, the picture in Blue's colours, its hero
    // without a book.
    let mut both = o.clone();
    both.sides[1].human = true;
    let blue = shown(
        &both,
        Some(Side::Defender),
        (by(Side::Defender), NO_AUTO),
        "",
        RED_BLUE,
    );
    assert_eq!(disabled(&blue), [AUTO_COMBAT, SPELL_BOOK]);
    assert_eq!(blue.players.get(&PICTURE), Some(&1));
    // A hero that cast this round; the wait phase.
    let mut cast = o.clone();
    cast.sides[0].hero = hero(true, 1);
    cast.wait_phase = true;
    assert_eq!(
        disabled(&shown(
            &cast,
            Some(Side::Attacker),
            (by(Side::Attacker), NO_AUTO),
            "",
            RED_BLUE
        )),
        [SPELL_BOOK, WAIT]
    );
    // No hero on the other side: no surrender; none on Red's: no retreat, no book either.
    let mut alone = o.clone();
    alone.sides[1].hero = None;
    assert_eq!(
        disabled(&shown(
            &alone,
            Some(Side::Attacker),
            (by(Side::Attacker), NO_AUTO),
            "",
            RED_BLUE
        )),
        [SURRENDER]
    );
    alone.sides[0].hero = None;
    assert_eq!(
        disabled(&shown(
            &alone,
            Some(Side::Attacker),
            (by(Side::Attacker), NO_AUTO),
            "",
            RED_BLUE
        )),
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
    let defender = shown(
        &o,
        Some(Side::Defender),
        (by(Side::Defender), NO_AUTO),
        "",
        RED_BLUE,
    );
    assert_eq!(
        disabled(&defender),
        [SURRENDER, RETREAT, AUTO_COMBAT, SPELL_BOOK]
    );
    o.sides[1].may_retreat = true;
    let tunnel = shown(
        &o,
        Some(Side::Defender),
        (by(Side::Defender), NO_AUTO),
        "",
        RED_BLUE,
    );
    assert_eq!(disabled(&tunnel), [SURRENDER, AUTO_COMBAT, SPELL_BOOK]);
    // The attacker may do both.
    let attacker = shown(
        &o,
        Some(Side::Attacker),
        (by(Side::Attacker), NO_AUTO),
        "",
        RED_BLUE,
    );
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
        disabled(&shown(
            &o,
            Some(Side::Attacker),
            (by(Side::Attacker), NO_AUTO),
            "",
            RED_BLUE
        )),
        [LOG_UP, LOG_DOWN, SPELL_BOOK, WAIT, DEFEND]
    );
    // E17 M5 T4c2a: its own panel's two enabled on a human's turn (0x4786AB), disabled on
    // another's with the seven (0x46C290); as built, before any turn, the spell book, Wait and
    // Defend disabled (0x472428) and the two enabled.
    let state = |shown: &Shown, id| shown.states.get(&id).map(|s| s.disabled);
    let red = shown(
        &o,
        Some(Side::Attacker),
        (by(Side::Attacker), NO_AUTO),
        "",
        RED_BLUE,
    );
    assert_eq!(
        [NEXT_CREATURE, START_COMBAT].map(|id| state(&red, id)),
        [Some(false); 2]
    );
    let blue = shown(
        &o,
        Some(Side::Defender),
        (by(Side::Defender), NO_AUTO),
        "",
        RED_BLUE,
    );
    assert_eq!(
        [NEXT_CREATURE, START_COMBAT, SURRENDER].map(|id| state(&blue, id)),
        [Some(true); 3]
    );
    let built = shown(&o, None, (by(Side::Defender), NO_AUTO), "", RED_BLUE);
    assert_eq!(
        [SPELL_BOOK, WAIT, DEFEND, SURRENDER, NEXT_CREATURE].map(|id| state(&built, id)),
        [Some(true), Some(true), Some(true), None, None]
    );
    assert_eq!(built.players.get(&PICTURE), Some(&1));
    // Outside the phase nothing is set before a turn.
    o.tactics = None;
    let outside = shown(&o, None, (by(Side::Defender), NO_AUTO), "", RED_BLUE);
    assert_eq!(state(&outside, SPELL_BOOK), None);
}

#[test]
fn a_computers_turn_disables_the_seven_buttons() {
    let o = battle();
    // Red's colours kept from its turn (0x46BF90 leaves the picture).
    let blue = shown(
        &o,
        Some(Side::Defender),
        (by(Side::Attacker), NO_AUTO),
        "",
        RED_BLUE,
    );
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
    assert_eq!(blue.players.get(&PICTURE), Some(&0));
}

#[test]
fn before_a_turn_the_panel_is_as_built() {
    let mut o = battle();
    let built = shown(&o, None, (by(Side::Attacker), NO_AUTO), "", RED_BLUE);
    assert_eq!(disabled(&built), [LOG_UP, LOG_DOWN]);
    assert_eq!(built.players.get(&PICTURE), Some(&0));
    o.sides[1].human = true;
    assert_eq!(
        disabled(&shown(
            &o,
            None,
            (by(Side::Attacker), NO_AUTO),
            "",
            RED_BLUE
        )),
        [AUTO_COMBAT, LOG_UP, LOG_DOWN]
    );
}

#[test]
fn each_turn_finds_the_battle_its_events_left() {
    let mut o = battle();
    o.tactics = Some(Tactics {
        side: Side::Attacker,
        columns: 3,
    });
    let cast = Event::SpellCast {
        side: Side::Attacker,
        spell: 1,
        target: Target::None {},
        second: Target::None {},
    };
    let events = [
        Event::TacticsEnded {},
        Event::Waited { stack: 0 },
        Event::TurnStarted { stack: 1 },
        cast.clone(),
        Event::SpellCast {
            side: Side::Defender,
            spell: 1,
            target: Target::None {},
            second: Target::None {},
        },
        Event::TurnStarted { stack: 0 },
        Event::Defended { stack: 0 },
        Event::RoundStarted { round: 2 },
        Event::TurnStarted { stack: 0 },
    ];
    let turns = turns(&o, &events);
    assert_eq!(turns.len(), 3);
    // The tactics phase over; stack 1 did not wait.
    assert_eq!(
        (turns[0].active, turns[0].wait_phase, turns[0].tactics),
        (Some(1), false, None)
    );
    // Stack 0 waited this round: the wait phase; Red's hero cast (Blue's has none to count).
    assert_eq!((turns[1].active, turns[1].wait_phase), (Some(0), true));
    assert_eq!(turns[1].sides[0].hero.as_ref().map(|h| h.casts), Some(1));
    // A new round: no wait phase, no casts.
    assert_eq!((turns[2].round, turns[2].wait_phase), (2, false));
    assert_eq!(turns[2].sides[0].hero.as_ref().map(|h| h.casts), Some(0));
    let mut heroless = battle();
    heroless.sides[0].hero = None;
    let after = super::turns(
        &heroless,
        &[
            cast,
            Event::RoundStarted { round: 2 },
            Event::TurnStarted { stack: 0 },
        ],
    );
    assert!(after[0].sides[0].hero.is_none());
    // E17 M3 T7b-2b-2b: a turn its pick freezes is none, as the timeline's.
    let frozen = super::turns(
        &battle(),
        &[
            Event::TurnStarted { stack: 1 },
            Event::BadMorale { stack: 1 },
            Event::TurnStarted { stack: 0 },
            Event::Frightened { stack: 0 },
            Event::TurnStarted { stack: 1 },
        ],
    );
    assert_eq!(frozen.len(), 1);
    assert_eq!(frozen[0].active, Some(1));
    // E17 M5 T4c2b: a placement's walk moves its stack, hex by hex, for the next turn's shade.
    let mut o = battle();
    o.stacks = alloc::vec![
        crate::scene::tests::stack(0, Side::Attacker, 0, 18, false),
        crate::scene::tests::stack(1, Side::Attacker, 0, 52, false),
    ];
    let from = o.stacks[0].hex;
    let placed = super::turns(
        &o,
        &[
            Event::Moved {
                stack: 0,
                from,
                to: 20,
            },
            Event::Moved {
                stack: 0,
                from: 20,
                to: 21,
            },
            Event::Moved {
                stack: 9,
                from: 0,
                to: 1,
            },
            Event::TurnStarted { stack: 1 },
        ],
    );
    assert_eq!(placed[0].stacks[0].hex, 21);
    assert_eq!(placed[0].stacks[1].hex, o.stacks[1].hex);
}

#[test]
fn the_panel_keeps_the_last_human_turns_colours_and_scroll_buttons() {
    let o = battle();
    // Red human, Blue a computer.
    let red_then_blue = [(Side::Attacker, false), (Side::Defender, false)];
    assert_eq!(
        kept(&o, &red_then_blue, Side::Defender),
        Kept {
            colours: Side::Attacker,
            scroll: true
        }
    );
    // No human turn yet, or Red's in the tactics phase: the side it was built for; the scroll
    // buttons as built.
    assert_eq!(
        kept(&o, &[(Side::Defender, false)], Side::Defender),
        by(Side::Defender)
    );
    assert_eq!(
        kept(&o, &[(Side::Attacker, true)], Side::Defender),
        by(Side::Attacker)
    );
    assert_eq!(kept(&o, &[], Side::Attacker), by(Side::Attacker));
    // A computer's turn after Red's: the seven dimmed, the scroll buttons left enabled.
    let after = Kept {
        colours: Side::Attacker,
        scroll: true,
    };
    let blue = shown(&o, Some(Side::Defender), (after, NO_AUTO), "", RED_BLUE);
    assert_eq!(disabled(&blue).len(), 7);
    assert!(!disabled(&blue).contains(&LOG_UP));
}

#[test]
fn auto_combat_dims_the_turn_it_plays_and_keeps_its_button_selected() {
    // E17 M5 T4c1 (design §7.22): Red's turn played by Auto Combat: the seven dimmed but Auto
    // Combat, enabled and selected (0x4784F9); the scroll buttons as Red's turns left them.
    let o = battle();
    let red = (by(Side::Attacker), NO_AUTO);
    let playing = Autoplay {
        plays: true,
        selected: false,
    };
    let auto = shown(&o, Some(Side::Attacker), (red.0, playing), "", RED_BLUE);
    assert_eq!(
        disabled(&auto),
        [
            SURRENDER, RETREAT, OPTIONS, LOG_UP, LOG_DOWN, SPELL_BOOK, WAIT, DEFEND
        ]
    );
    let button = auto.state(AUTO_COMBAT);
    assert!(button.selected && !button.disabled);
    // The computer's turn after it: dimmed, Auto Combat still selected (0x46BF90 sets the
    // disabled state only, the selected frame drawn over it).
    let kept = Autoplay {
        plays: false,
        selected: true,
    };
    let blue = shown(&o, Some(Side::Defender), (red.0, kept), "", RED_BLUE);
    assert_eq!(disabled(&blue).len(), 9);
    assert!(blue.state(AUTO_COMBAT).selected);
    // Before a turn, as built with its selection; a human's turn it does not play unselects it
    // (0x47864A).
    assert!(
        shown(&o, None, (red.0, kept), "", RED_BLUE)
            .state(AUTO_COMBAT)
            .selected
    );
    let human = shown(&o, Some(Side::Attacker), (red.0, kept), "", RED_BLUE);
    assert!(!human.state(AUTO_COMBAT).selected && disabled(&human).is_empty());
    // E17 M5 T4c2b: in the tactics phase the panel's +0xC (0x46C290) dims its two as well.
    let mut phase = o.clone();
    phase.tactics = Some(Tactics {
        side: Side::Attacker,
        columns: 3,
    });
    let auto = shown(&phase, Some(Side::Attacker), (red.0, playing), "", RED_BLUE);
    assert!(
        [NEXT_CREATURE, START_COMBAT]
            .iter()
            .all(|&id| auto.state(id).disabled)
    );
    assert!(auto.state(AUTO_COMBAT).selected && !auto.state(AUTO_COMBAT).disabled);
    // Blue a computer with Auto Combat on: its turn as any computer's.
    assert_eq!(
        disabled(&shown(
            &o,
            Some(Side::Defender),
            (red.0, playing),
            "",
            RED_BLUE
        ))
        .len(),
        9
    );
}
