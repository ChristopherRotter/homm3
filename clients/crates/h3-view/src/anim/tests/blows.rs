//! E17 M6 T5d2a: a Dread Knight's Death Blow (`damage_stage1_with_luck` 0x4439A7, design §12.23)
//! with its line, sound, animation over the stack struck and wait, before the strike's own
//! animation; an Ogre Mage's Bloodlust (`creature_cast_action` 0x4486A4) with the caster's frames,
//! its line and the glow over its target.

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::{Event, Observation, Side, Strike, Target};
use h3_render::Bugs;
use h3_render::recolour::Effect;

use super::{
    FAERIE_DRAGON, GOBLIN, PIKEMAN, SWORDSMAN, applied, area_looks, at, casts, cues, flags, layout,
    looks, roll_magic, snapshot, stack_cast, struck, tables, timeline,
};
use crate::anim::{
    BLOODLUST, Casts, Command, DEATH_BLOW, GOOD_LUCK, Logged, MagicLook, Sound, Speed, SpellLook,
    Stage, Timeline,
};
use crate::sound::Lengths;

/// The rolled effects' animations with Death Blow's (`sp03_`, 4 frames, centred).
fn blow_magic() -> BTreeMap<u8, MagicLook> {
    let mut all = roll_magic();
    let blow = MagicLook {
        def: "sp03_.def".into(),
        frames: 4,
        size: [80, 90],
        place: 1,
        translucent: false,
    };
    all.insert(DEATH_BLOW, blow);
    all
}

/// The casts' spells with Bloodlust (no animation of its own: it glows).
fn bloodlust_casts() -> Casts {
    let mut casts = casts();
    let look = SpellLook {
        original: BLOODLUST,
        sign: 1,
        ..SpellLook::default()
    };
    casts.spells.insert(BLOODLUST, look);
    casts
}

/// The timeline of `snapshot` and `commands` at speed 0 with the samples' `lengths` (none: no
/// sound) and the creatures' `looks`.
fn play_blows(
    snapshot: &Observation,
    commands: &[(Option<u32>, Vec<Event>)],
    lengths: Option<&Lengths>,
    looks: &BTreeMap<u16, crate::anim::Look>,
) -> Timeline {
    let layout = layout();
    let magic = blow_magic();
    let casts = bloodlust_casts();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic: &magic,
        casts: &casts,
        bugs: Bugs::Original,
        siege: None,
        lengths,
    };
    let commands: Vec<Command<'_>> = commands
        .iter()
        .map(|(at, events)| Command { at: *at, events })
        .collect();
    timeline(snapshot, &commands, looks, stage, Speed::default()).unwrap()
}

fn named(sound: Sound, ms: u32) -> Lengths {
    Lengths {
        named: [(sound, ms)].into_iter().collect(),
        ..Lengths::default()
    }
}

fn blow() -> Strike {
    Strike {
        death_blow: true,
        ..flags(false, false, false)
    }
}

/// The Pikemen (0, active) beside the Goblins (1).
fn two() -> Observation {
    snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 86, false),
        at(1, Side::Defender, GOBLIN, 87, false),
    ])
}

/// Animation `number`'s keys in `t`: (shown, frame, the stacks it is over).
fn over(t: &Timeline, number: u8) -> Vec<(u32, Option<u32>, Vec<u8>)> {
    t.magics
        .iter()
        .filter(|k| k.animation == number)
        .map(|k| (k.shown, k.frame, k.over.iter().map(|o| o.0).collect()))
        .collect()
}

/// When `stack` first takes `group` after `from`.
fn first_in(t: &Timeline, stack: u8, group: u32, from: u32) -> Option<u32> {
    t.keys
        .iter()
        .find(|k| k.stack == stack && k.pose.group == group && k.drawn >= from)
        .map(|k| k.drawn)
}

#[test]
fn a_death_blow_plays_over_the_stack_struck_before_the_strike() {
    // The line (by the striker's 40), `Deathblo.wav`, then 73 over the Goblins without their
    // hit, a redraw each 100 ms and one that erases it; then the Pikemen's strike.
    let events = vec![struck(0, 1, 0, blow()), Event::TurnStarted { stack: 1 }];
    let t = play_blows(&two(), &[(Some(0), events.clone())], None, &looks());
    let line = Logged::DeathBlow {
        stack: 0,
        count: 40,
    };
    let at = t.log.iter().find(|(_, l)| *l == line).map(|(at, _)| *at);
    let at = at.unwrap_or_else(|| panic!("{:?}", t.log));
    assert!(cues(&t).contains(&(at, 0, Sound::DeathBlow, false)));
    let frames = over(&t, DEATH_BLOW);
    assert_eq!(
        frames,
        [
            (at + 100, Some(0), vec![1]),
            (at + 200, Some(1), vec![1]),
            (at + 300, Some(2), vec![1]),
            (at + 400, Some(3), vec![1]),
            (at + 500, None, vec![1]),
        ]
    );
    // The Goblins stand through it (no hit), the Pikemen strike after it, the strike's line
    // after their animation.
    assert!((1..5).all(|f| t.at(at + 100 * f)[&1].group == 2));
    let strike = (11..=13)
        .filter_map(|g| first_in(&t, 0, g, 0))
        .min()
        .unwrap();
    assert!(strike >= at + 500, "{strike}");
    let struck_line = t
        .log
        .iter()
        .find(|(_, l)| matches!(l, Logged::Strike { .. }));
    assert!(struck_line.is_some_and(|(s, _)| *s > strike));
    // With sound the strike waits for `Deathblo`'s end, from its start; the frames the same.
    let long = named(Sound::DeathBlow, 2_000);
    let w = play_blows(&two(), &[(Some(0), events)], Some(&long), &looks());
    assert_eq!(over(&w, DEATH_BLOW), frames);
    let waited = (11..=13)
        .filter_map(|g| first_in(&w, 0, g, 0))
        .min()
        .unwrap();
    assert_eq!(waited, strike + (at + 2_000 - (at + 500)));
}

#[test]
fn good_luck_comes_before_a_death_blow_and_a_retaliation_may_blow_too() {
    // Lucky and a Death Blow: 18 over the striker, then 73 over the stack struck (0x4418B3,
    // 0x443B3C). The Goblins' retaliation with its own Death Blow: its line by the Goblins, 73
    // over the Pikemen.
    let both = Strike {
        lucky: true,
        ..blow()
    };
    let back = Strike {
        retaliation: true,
        ..blow()
    };
    let events = vec![
        struck(0, 1, 0, both),
        struck(1, 0, 0, back),
        Event::TurnStarted { stack: 1 },
    ];
    let t = play_blows(&two(), &[(Some(0), events)], None, &looks());
    let firsts: Vec<(u8, Vec<u8>)> = t
        .magics
        .iter()
        .filter(|k| k.frame == Some(0))
        .map(|k| (k.animation, k.over.iter().map(|o| o.0).collect()))
        .collect();
    assert_eq!(
        firsts,
        [
            (GOOD_LUCK, vec![0]),
            (DEATH_BLOW, vec![1]),
            (DEATH_BLOW, vec![0])
        ]
    );
    let blows: Vec<u8> = t
        .log
        .iter()
        .filter_map(|(_, l)| match *l {
            Logged::DeathBlow { stack, .. } => Some(stack),
            _ => None,
        })
        .collect();
    assert_eq!(blows, [0, 1]);
}

#[test]
fn a_death_blow_without_its_stacks_draws_what_it_can() {
    // A striker the timeline does not follow writes and plays nothing; a target it does not
    // follow takes no animation, the line and the sound still go.
    let events = vec![struck(9, 1, 0, blow()), struck(0, 8, 0, blow())];
    let t = play_blows(&two(), &[(Some(0), events)], None, &looks());
    assert!(over(&t, DEATH_BLOW).is_empty());
    let blows: Vec<u8> = t
        .log
        .iter()
        .filter_map(|(_, l)| match *l {
            Logged::DeathBlow { stack, .. } => Some(stack),
            _ => None,
        })
        .collect();
    assert_eq!(blows, [0]);
    assert!(cues(&t).iter().any(|c| c.1 == 0 && c.2 == Sound::DeathBlow));
}

/// The looks `t` gives `stack` from each key on: (shown, effect).
fn looks_of(t: &Timeline, stack: u8) -> Vec<(u32, Option<Effect>)> {
    t.looks
        .iter()
        .filter(|k| k.stack == stack)
        .map(|k| (k.shown, k.look))
        .collect()
}

const BLOODLUST_STEPS: [u8; 20] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];

/// An Ogre Mage (the tests' Faerie Dragon's look: its casting groups 17 to 19 of 12 frames at 40
/// ms) at 81 and 82, the Swordsmen it lusts at 83, the enemy Pikemen at 100.
fn ogres() -> Observation {
    snapshot(vec![
        at(0, Side::Attacker, FAERIE_DRAGON, 81, true),
        at(1, Side::Attacker, SWORDSMAN, 83, false),
        at(2, Side::Defender, PIKEMAN, 100, false),
    ])
}

#[test]
fn an_ogre_mage_s_bloodlust_glows_after_its_frames_and_its_line() {
    // Its 12 frames of group 17 (the Swordsmen beside it), outlined; the cast's start clears the
    // outline, the spell's sound starts, its effect and the line (by its 40, then the target's
    // name); 20 redraws of the glow at 100 ms, the last replaced in its millisecond; at the tail
    // it stands outlined; then the Pikemen's turn.
    let stack = Target::Stack { stack: 1 };
    let events = vec![
        stack_cast(0, BLOODLUST, stack),
        applied(1, BLOODLUST),
        Event::TurnStarted { stack: 2 },
    ];
    let t = play_blows(&ogres(), &[(Some(0), events.clone())], None, &area_looks());
    let cast: Vec<u32> = t
        .keys
        .iter()
        .filter(|k| k.stack == 0 && k.pose.group == 17)
        .map(|k| k.drawn)
        .collect();
    assert_eq!(cast.len(), 12);
    let last = cast[11];
    assert_eq!(t.outlined(last), Some(0));
    let line = Logged::CreatureCast {
        stack: 0,
        count: 40,
        spell: BLOODLUST,
        target: 1,
        target_count: 40,
    };
    let at = t.log.iter().find(|(_, l)| *l == line).map(|(at, _)| *at);
    let at = at.unwrap_or_else(|| panic!("{:?}", t.log));
    assert!(at > last);
    assert_eq!(t.outlined(at), None);
    assert!(cues(&t).contains(&(at, 0, Sound::Spell(BLOODLUST), false)));
    // The caster's last frame shown at its deadline (the cast's start, the line); the glow's
    // first redraw at the next, 40 ms on.
    let first = at + 40;
    let mut glow: Vec<(u32, Option<Effect>)> = (0..)
        .zip(BLOODLUST_STEPS)
        .map(|(i, s)| (first + 100 * i, Some(Effect::Bloodlust(s))))
        .collect();
    glow.push((first + 1_900, None));
    assert_eq!(looks_of(&t, 1), glow);
    // Standing and outlined at the tail, the next turn's stack after it.
    let stood = first_in(&t, 0, 2, last).unwrap();
    assert_eq!(stood, first + 1_900);
    assert!(t.outlines.contains(&(stood, Some(0))));
    assert_eq!(t.turns.last().copied(), Some((stood, 2)));
    // With sound the turn waits for its shot sound's end (as an area cast's, 0x4486E2).
    let long = Lengths {
        stacks: [((0, Sound::Shot), 5_000)].into_iter().collect(),
        ..Lengths::default()
    };
    let w = play_blows(&ogres(), &[(Some(0), events)], Some(&long), &area_looks());
    assert_eq!(looks_of(&w, 1), glow);
    let shot = cues(&w).iter().find(|c| c.2 == Sound::Shot).map(|c| c.0);
    assert_eq!(w.turns.last().copied(), shot.map(|s| (s + 5_000, 2)));
}

#[test]
fn a_creature_s_bloodlust_on_a_stack_not_followed_takes_effect_at_once() {
    // The target gone from the timeline: no frames, no line, no glow (the earlier path).
    let events = vec![
        stack_cast(0, BLOODLUST, Target::Stack { stack: 7 }),
        applied(7, BLOODLUST),
    ];
    let t = play_blows(&ogres(), &[(Some(0), events)], None, &area_looks());
    assert!(!t.keys.iter().any(|k| k.stack == 0 && k.pose.group == 17));
    assert!(
        !t.log
            .iter()
            .any(|(_, l)| matches!(l, Logged::CreatureCast { .. }))
    );
    assert!(t.looks.is_empty());
}
