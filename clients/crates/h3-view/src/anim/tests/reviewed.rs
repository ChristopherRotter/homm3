//! E17 M3 T9b, the reviewer's findings on the timeline: what the engine emits between a strike's
//! hits and before a turn's rolls, the turns a command plays on, a death's spells, the damage
//! spells the timeline draws no other way and Magic Mirror's redirect.

use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::{Event, Side, Target};

use super::{
    FAERIE_DRAGON, GOBLIN, PIKEMAN, SWORDSMAN, TENT, WOLF_RIDER, applied, at, cast, flags, hurt,
    machine, moved, play_area, play_casts, play_strikes, snapshot, spell_damage, stack_cast,
    strike_casts, strike_spell, struck,
};
use crate::anim::{Logged, freezes};

/// A stack's last pose: among the corpses (layer 1) in its death group (5)?
fn dead(t: &crate::anim::Timeline, stack: u8) -> bool {
    let pose = t.at(t.end)[&stack];
    pose.group == 5 && pose.layer == 1
}

#[test]
fn what_a_hit_ends_before_the_next_hit_waits_for_the_strike() {
    // The engine's `strike`: the first hit's Blind (here Slow, 54) ends before the second hit;
    // both stacks react, the second dies, and the effect ends after the strike.
    let o = snapshot(vec![
        at(0, Side::Attacker, WOLF_RIDER, 52, true),
        at(1, Side::Defender, GOBLIN, 36, false),
        at(3, Side::Defender, GOBLIN, 54, false),
    ]);
    let area = flags(false, false, true);
    let melee = flags(false, false, false);
    let commands = [
        (None, vec![applied(1, 54)]),
        (
            Some(9000),
            vec![
                struck(0, 1, 1, melee),
                Event::EffectEnded {
                    target: 1,
                    spell: 54,
                },
                Event::Vanished {
                    stack: 9,
                    creatures: 1,
                },
                struck(0, 3, 40, area),
            ],
        ),
    ];
    let t = play_strikes(&o, &commands, &strike_casts());
    assert!(dead(&t, 3));
    assert_eq!(t.counts(t.end)[&1], 39);
    assert!(!t.hues(t.end).contains_key(&1));
    let around = t
        .log
        .iter()
        .any(|(_, l)| matches!(l, Logged::Strike { hits, .. } if hits.len() == 2));
    assert!(around, "{:?}", t.log);
    // A pending spell after what the last hit ended is still cast at the strike's end.
    let commands = [(
        None,
        vec![
            struck(0, 1, 0, melee),
            Event::EffectEnded {
                target: 1,
                spell: 54,
            },
            strike_spell(0, 1, 71),
            applied(1, 71),
        ],
    )];
    let t = play_strikes(&o, &commands, &strike_casts());
    assert!(
        t.log
            .iter()
            .any(|(_, l)| matches!(l, Logged::Poisoned { stack: 1, .. })),
        "{:?}",
        t.log
    );
}

#[test]
fn a_turn_the_command_plays_on_strikes_as_an_attack_of_its_own() {
    // A berserk stack's turn inside the command: its strike takes the striker's layer (6), not a
    // retaliation's (5).
    let o = snapshot(vec![
        at(0, Side::Attacker, WOLF_RIDER, 52, true),
        at(1, Side::Defender, GOBLIN, 36, false),
        at(2, Side::Defender, WOLF_RIDER, 70, true),
    ]);
    let melee = flags(false, false, false);
    let commands = [(
        None,
        vec![
            struck(0, 1, 0, melee),
            Event::TurnStarted { stack: 2 },
            struck(2, 1, 0, melee),
        ],
    )];
    let t = play_strikes(&o, &commands, &strike_casts());
    let layers: Vec<u8> = t
        .keys
        .iter()
        .filter(|k| k.stack == 2 && (11..=13).contains(&k.pose.group))
        .map(|k| k.pose.layer)
        .collect();
    assert!(
        !layers.is_empty() && layers.iter().all(|&l| l == 6),
        "{layers:?}"
    );
}

#[test]
fn a_regeneration_or_an_ended_effect_comes_before_the_turns_roll() {
    let regenerated = Event::Regenerated {
        stack: 1,
        hit_points: 5,
    };
    let ended = Event::EffectEnded {
        target: 1,
        spell: 54,
    };
    let bad = Event::BadMorale { stack: 1 };
    assert!(freezes(
        &[regenerated.clone(), ended.clone(), bad.clone()],
        1
    ));
    assert!(freezes(
        &[applied(1, 54), Event::Frightened { stack: 1 }],
        1
    ));
    assert!(!freezes(&[regenerated, moved(1, 0, 1), bad.clone()], 1));
    assert!(!freezes(&[ended, Event::BadMorale { stack: 2 }], 1));
    assert!(!freezes(&[], 1));
}

#[test]
fn a_death_ends_its_stacks_timed_spells_but_disrupting_ray() {
    // A stack under Haste (53) and Disrupting Ray (47) dies and is reborn: its hue Disrupting
    // Ray's alone, as a stack under that one only.
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 52, false),
        at(1, Side::Defender, GOBLIN, 53, false),
        at(2, Side::Defender, GOBLIN, 70, false),
    ]);
    let melee = flags(false, false, false);
    let died = |spells: &[u16]| {
        let mut events: Vec<Event> = spells.iter().map(|&s| applied(1, s)).collect();
        events.push(struck(0, 1, 40, melee));
        events.push(Event::Reborn {
            stack: 1,
            creatures: 10,
        });
        play_strikes(&o, &[(None, events)], &strike_casts())
    };
    let both = died(&[53, 47]);
    let ray = died(&[47]);
    let hue = |t: &crate::anim::Timeline| t.hues(t.end).get(&1).copied();
    assert!(hue(&both).is_some());
    assert_eq!(hue(&both), hue(&ray));
    // Haste alone: no hue once dead.
    assert_eq!(hue(&died(&[53])), None);
    assert_eq!(both.counts(both.end)[&1], 10);
}

#[test]
fn a_damage_spell_drawn_no_other_way_still_kills() {
    // A hero's spell with neither a projectile nor a bolt (Implosion, 18, unknown to the casts'
    // looks): the stack reacts and dies, the damage's line written.
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 52, false),
        at(1, Side::Defender, GOBLIN, 53, false),
    ]);
    let t = play_casts(
        &o,
        &[(Some(0), vec![cast(18, 1), spell_damage(1, 500, 40)])],
    );
    assert!(dead(&t, 1));
    let line = Logged::SpellDamage {
        spell: 18,
        hit: (1, 500, 40),
    };
    assert!(t.log.iter().any(|(_, l)| *l == line), "{:?}", t.log);
    // A Faerie Dragon's Ice Bolt (22) at a stack: its animation over it, the kills taken; an
    // effect beside it and a hit on no living stack change nothing else.
    let o = snapshot(vec![
        at(0, Side::Attacker, FAERIE_DRAGON, 81, true),
        at(1, Side::Defender, SWORDSMAN, 83, false),
    ]);
    let events = vec![
        stack_cast(0, 22, Target::Stack { stack: 1 }),
        applied(1, 54),
        hurt(1, 20, 20, 3),
        hurt(7, 20, 20, 1),
    ];
    let t = play_area(&o, &[(Some(0), events)]);
    assert_eq!(t.counts(t.end)[&1], 37);
    assert!(t.magics.iter().any(|k| k.animation == 9));
}

#[test]
fn magic_mirror_sends_a_spell_on_to_the_casters_side() {
    // Slow (54) aimed at the Goblins goes to the Pikemen: its line names them, its animation
    // over them, their hue; Magic Arrow (15) sent on kills among them.
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 52, false),
        at(1, Side::Defender, GOBLIN, 53, false),
    ]);
    let mirrored = Event::SpellRedirected { from: 1, to: 0 };
    let t = play_casts(
        &o,
        &[(Some(0), vec![cast(54, 1), mirrored.clone(), applied(0, 54)])],
    );
    assert!(
        t.log
            .iter()
            .any(|(_, l)| matches!(l, Logged::Cast { target: 0, .. })),
        "{:?}",
        t.log
    );
    assert!(t.hues(t.end).contains_key(&0));
    assert!(!t.hues(t.end).contains_key(&1));
    let t = play_casts(
        &o,
        &[(Some(0), vec![cast(15, 1), mirrored, spell_damage(0, 10, 2)])],
    );
    assert_eq!(t.counts(t.end)[&0], 38);
    assert!(!t.counts(t.end).contains_key(&1));
}

#[test]
fn a_war_machines_turn_that_drew_nothing_keeps_no_outline_into_the_next_command() {
    // The tent's tick outlines it; its turn draws nothing; the Faerie Dragon's next command keeps
    // the dragon outlined through its casting frames, as without the tent (E17 M3 T7c-1).
    let o = snapshot(vec![
        at(0, Side::Attacker, FAERIE_DRAGON, 81, true),
        machine(1, Side::Attacker, TENT, 153, 18),
        at(2, Side::Defender, SWORDSMAN, 83, false),
        at(3, Side::Defender, SWORDSMAN, 100, false),
    ]);
    let commands = [
        (
            Some(0),
            vec![
                moved(3, 100, 99),
                Event::TurnStarted { stack: 1 },
                Event::TurnStarted { stack: 0 },
            ],
        ),
        (
            Some(5000),
            vec![
                stack_cast(0, 21, Target::Hex { hex: 83 }),
                hurt(2, 20, 20, 0),
            ],
        ),
    ];
    let t = play_area(&o, &commands);
    let cast: Vec<u32> = t
        .keys
        .iter()
        .filter(|k| k.stack == 0 && k.pose.group == 17)
        .map(|k| k.shown)
        .collect();
    let first = cast.first().copied().unwrap_or_default();
    assert!(!cast.is_empty());
    assert_eq!(t.outlined(first), Some(0));
}
