//! E17 M5 T5b1: the stacks' palette effects by their spells (design §7.26): Bloodlust's glow on
//! one stack and on several together, Stone Gaze's glow after a strike and the petrified look
//! while it lasts.

use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::{Effect as Held, Event, Side, Target};
use h3_render::recolour::Effect;

use super::{
    PIKEMAN, SWORDSMAN, applied, at, cast, flags, play_casts_with, play_strikes, snapshot,
    strike_casts, strike_spell, struck,
};
use crate::anim::{BLOODLUST, Casts, Logged, STONE_GAZE, SpellLook, Timeline};

/// The casts' stage with Bloodlust (no animation of its own) and Stone Gaze, by their numbers.
fn glowing(mut casts: Casts) -> Casts {
    for (original, sign) in [(BLOODLUST, 1), (STONE_GAZE, -1)] {
        let look = SpellLook {
            original,
            sign,
            ..SpellLook::default()
        };
        casts.spells.insert(original, look);
    }
    casts
}

/// The looks `t` gives `stack` from each key on: (shown, effect).
fn looks_of(t: &Timeline, stack: u8) -> Vec<(u32, Option<Effect>)> {
    t.looks
        .iter()
        .filter(|k| k.stack == stack)
        .map(|k| (k.shown, k.look))
        .collect()
}

/// A glow's keys from `first`, a redraw each 100 ms, then none in the last one's millisecond.
fn glow(first: u32, steps: &[u8], effect: fn(u8) -> Effect) -> Vec<(u32, Option<Effect>)> {
    let mut keys: Vec<(u32, Option<Effect>)> = (0..)
        .zip(steps)
        .map(|(i, &s)| (first + 100 * i, Some(effect(s))))
        .collect();
    let last = keys.last().map_or(first, |k| k.0);
    keys.push((last, None));
    keys
}

const BLOODLUST_STEPS: [u8; 20] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];

#[test]
fn bloodlust_glows_after_its_line_and_before_the_hero_s_last_frames() {
    // A hero's Bloodlust (`cast_spell` 0x5A122B): his five frames, the effect and the line at
    // 500, then 20 redraws of trunc(100 F) from 600, its strength 0.0 to 1.0 and back to 0.1
    // (the last replaced in its millisecond by the redraw that clears the flag), then his
    // frames 5 to 7 and standing.
    let o = snapshot(vec![
        at(0, Side::Attacker, SWORDSMAN, 86, false),
        at(1, Side::Defender, PIKEMAN, 100, false),
    ]);
    let casts = glowing(super::casts());
    let commands = [(Some(0), vec![cast(BLOODLUST, 0), applied(0, BLOODLUST)])];
    let t = play_casts_with(&o, &commands, casts);
    assert_eq!(
        looks_of(&t, 0),
        glow(600, &BLOODLUST_STEPS, Effect::Bloodlust)
    );
    assert!(
        t.log
            .iter()
            .any(|(at, l)| *at == 500 && matches!(l, Logged::Cast { .. }))
    );
    assert_eq!(t.looks(1500).get(&0), Some(&Effect::Bloodlust(9)));
    assert_eq!(t.looks(1600).get(&0), Some(&Effect::Bloodlust(10)));
    assert!(t.looks(2500).is_empty());
    // His last three casting frames after the glow, then he stands.
    let heroes = t.heroes.as_ref().unwrap();
    let group = |p: [Option<crate::hero::HeroPose>; 2]| p[0].map(|h| (h.group, h.frame));
    let shown = [2500, 2600, 2700, 2800].map(|at| group(heroes.at(&t, at)));
    assert_eq!(
        shown,
        [Some((4, 4)), Some((4, 5)), Some((4, 6)), Some((0, 0))]
    );
}

#[test]
fn a_mass_bloodlust_glows_every_stack_it_reached_together() {
    // At Expert (0x5A1359 to 0x5A1577): its line, then every stack it reached glows in the same
    // redraws.
    let o = snapshot(vec![
        at(0, Side::Attacker, SWORDSMAN, 86, false),
        at(2, Side::Attacker, PIKEMAN, 52, false),
        at(1, Side::Defender, PIKEMAN, 100, false),
    ]);
    let mass = Event::SpellCast {
        side: Side::Attacker,
        spell: BLOODLUST,
        target: Target::None {},
        second: Target::None {},
    };
    let commands = [(
        Some(0),
        vec![mass, applied(0, BLOODLUST), applied(2, BLOODLUST)],
    )];
    let t = play_casts_with(&o, &commands, glowing(super::casts()));
    let expected = glow(600, &BLOODLUST_STEPS, Effect::Bloodlust);
    assert_eq!(looks_of(&t, 0), expected);
    assert_eq!(looks_of(&t, 2), expected);
    assert!(looks_of(&t, 1).is_empty());
    assert!(
        t.log
            .iter()
            .any(|(at, l)| *at == 500 && matches!(l, Logged::CastAll { .. }))
    );
}

#[test]
fn stone_gaze_glows_after_its_line_and_leaves_the_stack_petrified_while_it_lasts() {
    // After a strike (`cast_spell` 0x5A1121): its sound, the effect and its line (GENRLTXT 559 or
    // 560), then 10 redraws of trunc(100 F), the saturation times 1.0 down to 0.1, then the
    // petrified look (`draw_battle_stack` 0x43E3CB) until the spell ends; a death ends it too.
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 99, false),
        at(1, Side::Defender, PIKEMAN, 100, false),
    ]);
    let melee = flags(false, false, false);
    let gaze = vec![
        struck(0, 1, 0, melee),
        strike_spell(0, 1, STONE_GAZE),
        applied(1, STONE_GAZE),
    ];
    // It ends at a round's start; the turn's start redraws the battle.
    let ended = vec![
        Event::RoundStarted { round: 2 },
        Event::EffectEnded {
            target: 1,
            spell: STONE_GAZE,
        },
        Event::TurnStarted { stack: 0 },
    ];
    let casts = glowing(strike_casts());
    let t = play_strikes(&o, &[(None, gaze.clone()), (Some(20_000), ended)], &casts);
    let keys = looks_of(&t, 1);
    let line = t
        .log
        .iter()
        .find(|(_, l)| {
            *l == Logged::Petrified {
                stack: 1,
                count: 40,
            }
        })
        .map(|(at, _)| *at)
        .unwrap();
    // The glow's ten steps a deadline apart from the line, then petrified in the last one's
    // millisecond, then its own colours again once the spell ends.
    let steps: Vec<Option<Effect>> = keys.iter().map(|k| k.1).collect();
    let mut expected: Vec<Option<Effect>> =
        (1..=10).rev().map(|s| Some(Effect::StoneGlow(s))).collect();
    expected.extend([Some(Effect::Petrified), None]);
    assert_eq!(steps, expected);
    assert!(keys[0].0 > line && keys[9].0 - keys[0].0 == 900 && keys[10].0 == keys[9].0);
    assert_eq!(t.looks(keys[9].0 + 1000).get(&1), Some(&Effect::Petrified));
    assert!(t.looks(keys[11].0).is_empty());
    assert!(
        t.cues
            .iter()
            .any(|c| c.sound == super::Sound::Spell(STONE_GAZE))
    );
    // A petrified stack that dies lies in its own colours.
    let killed = vec![struck(0, 1, 40, melee)];
    let t = play_strikes(&o, &[(None, gaze), (Some(20_000), killed)], &casts);
    assert_eq!(looks_of(&t, 1).last().map(|k| k.1), Some(None));
    // Where the engine did not apply it, nothing glows.
    let resisted = vec![struck(0, 1, 0, melee), strike_spell(0, 1, STONE_GAZE)];
    let t = play_strikes(&o, &[(None, resisted)], &casts);
    assert!(t.looks.is_empty());
}

#[test]
fn a_snapshot_s_petrified_stack_looks_so_from_the_start() {
    let mut stone = at(1, Side::Defender, PIKEMAN, 100, false);
    stone.effects = vec![Held {
        spell: STONE_GAZE,
        rounds: Some(2),
        mastery: 0,
        value: 0,
        attack: 0,
        defense: 0,
        speed: 0,
    }];
    let o = snapshot(vec![at(0, Side::Attacker, PIKEMAN, 99, false), stone]);
    let t = play_strikes(&o, &[], &glowing(strike_casts()));
    assert_eq!(looks_of(&t, 1), [(0, Some(Effect::Petrified))]);
    assert_eq!(t.looks(0).get(&1), Some(&Effect::Petrified));
}
