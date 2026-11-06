//! E17 M6 T5d1: the effects of a turn's and a round's start (design §12.22): regeneration at the
//! pick (`stack_turn_start` 0x446F32), a Wraith's mana drain at its activation
//! (`stack_activation` 0x465626) and Poison's tenth at a round's start (`stack_round_start`
//! 0x4472AB), each with its sound, its line, its animation and its wait, before the turn starts or
//! the round's line.

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::{Event, Observation, Side};
use h3_render::Bugs;

use super::{
    GOBLIN, PIKEMAN, at, cues, layout, looks, moved, snapshot, strike_casts, strike_magic, tables,
    timeline,
};
use crate::anim::{
    Command, Logged, MANA_DRAIN, MagicLook, POISON, POISON_TENTH, REGENERATION, Sound, Speed,
    Stage, Timeline, drain_of,
};
use crate::sound::Lengths;

/// The strikes' animations with regeneration's (4 frames) and the mana drain's (6).
fn starts_magic() -> BTreeMap<u8, MagicLook> {
    let mut all = strike_magic();
    let entry = |def: &str, frames, place, translucent| MagicLook {
        def: def.into(),
        frames,
        size: [80, 90],
        place,
        translucent,
    };
    all.insert(REGENERATION, entry("sp12_.def", 4, 1, true));
    all.insert(MANA_DRAIN, entry("sp08_.def", 6, 0, false));
    all
}

/// The timeline of `snapshot` and `commands` at speed 0 with the samples' `lengths` (none: no
/// sound).
fn play_starts(
    snapshot: &Observation,
    commands: &[(Option<u32>, Vec<Event>)],
    lengths: Option<&Lengths>,
) -> Timeline {
    let layout = layout();
    let magic = starts_magic();
    let casts = strike_casts();
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
    timeline(snapshot, &commands, &looks(), stage, Speed::default()).unwrap()
}

fn named(sound: Sound, ms: u32) -> Lengths {
    Lengths {
        named: [(sound, ms)].into_iter().collect(),
        ..Lengths::default()
    }
}

/// The Pikemen (0, active) and the Goblins (1).
fn two() -> Observation {
    snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 86, false),
        at(1, Side::Defender, GOBLIN, 100, false),
    ])
}

/// The times `t` shows animation `number`'s frames and its erasing redraw (none).
fn shown(t: &Timeline, number: u8) -> Vec<(u32, Option<u32>)> {
    t.magics
        .iter()
        .filter(|k| k.animation == number)
        .map(|k| (k.shown, k.frame))
        .collect()
}

/// When `t` logs a line `is` picks.
fn logged(t: &Timeline, is: impl Fn(&Logged) -> bool) -> Option<u32> {
    t.log.iter().find(|(_, l)| is(l)).map(|(at, _)| *at)
}

#[test]
fn regeneration_plays_at_the_pick_before_the_turn_starts() {
    // The Pikemen walk; the Goblins' pick regenerates them: the heal's sound, the line (by their
    // 40), then 79 over them without their hit, a redraw each 100 ms and one that erases it; the
    // Pikemen stay outlined through it; then the Goblins' turn.
    let events = vec![
        moved(0, 86, 87),
        Event::TurnStarted { stack: 1 },
        Event::Regenerated {
            stack: 1,
            hit_points: 3,
        },
    ];
    let t = play_starts(&two(), &[(Some(0), events.clone())], None);
    let line = logged(&t, |l| {
        *l == Logged::Regenerated {
            stack: 1,
            count: 40,
        }
    });
    let line = line.unwrap_or_else(|| panic!("{:?}", t.log));
    let frames = shown(&t, REGENERATION);
    let first = frames[0].0;
    assert_eq!(
        frames,
        [
            (first, Some(0)),
            (first + 100, Some(1)),
            (first + 200, Some(2)),
            (first + 300, Some(3)),
            (first + 400, None)
        ]
    );
    assert!(line < first);
    assert!(cues(&t).contains(&(line, 1, Sound::Heal, false)));
    // Over the Goblins standing (no hit), the Pikemen outlined until the Goblins' turn.
    assert!((0..4).all(|f| t.at(first + 100 * f)[&1].group == 2));
    assert_eq!(t.outlined(first + 300), Some(0));
    let turn = t.turns.last().copied();
    assert_eq!(turn, Some((first + 400, 1)));
    let started = logged(&t, |l| *l == Logged::Turn(1));
    assert_eq!(started, Some(first + 400));
    // With sound the turn waits for `Regener`'s end, from its start.
    let long = named(Sound::Heal, 2_000);
    let w = play_starts(&two(), &[(Some(0), events)], Some(&long));
    assert_eq!(w.turns.last().copied(), Some((line + 2_000, 1)));
    assert_eq!(w.magics, t.magics);
}

#[test]
fn a_wraith_drains_mana_at_its_activation_after_its_regeneration() {
    // The Goblins' pick regenerates them, their activation drains the Pikemen's hero: its sound,
    // the line naming the drained side's hero, 77 over them, its wait; then their turn.
    let events = vec![
        moved(0, 86, 87),
        Event::TurnStarted { stack: 1 },
        Event::Regenerated {
            stack: 1,
            hit_points: 3,
        },
        Event::ManaDrained {
            stack: 1,
            side: Side::Attacker,
            mana: 2,
        },
    ];
    let t = play_starts(&two(), &[(Some(0), events.clone())], None);
    let drain = Logged::ManaDrained {
        stack: 1,
        count: 40,
        side: 0,
    };
    let line = logged(&t, |l| *l == drain).unwrap_or_else(|| panic!("{:?}", t.log));
    let healed = shown(&t, REGENERATION);
    let drained = shown(&t, MANA_DRAIN);
    assert_eq!(drained.len(), 7);
    let first = drained[0].0;
    // After regeneration's erasing redraw, standing on its hex (place 0).
    assert_eq!(line, healed[4].0);
    assert_eq!(first, line + 100);
    assert!(cues(&t).contains(&(line, 1, Sound::ManaDrain, false)));
    let over = t.magics.iter().find(|k| k.animation == MANA_DRAIN);
    assert_eq!(over.map(|k| k.over.len()), Some(1));
    assert_eq!(t.outlined(first + 500), Some(0));
    assert_eq!(t.turns.last().copied(), Some((first + 600, 1)));
    // The wait: from the drain's sound's start.
    let long = named(Sound::ManaDrain, 3_000);
    let w = play_starts(&two(), &[(Some(0), events)], Some(&long));
    assert_eq!(w.turns.last().copied(), Some((line + 3_000, 1)));
}

#[test]
fn a_drain_belongs_to_its_own_activation() {
    // None for a frozen or passed stack (the engine drains only an activated one), none past the
    // next turn's start, none of another stack.
    let drained = Event::ManaDrained {
        stack: 1,
        side: Side::Defender,
        mana: 2,
    };
    assert_eq!(
        drain_of(
            &[
                Event::EffectEnded {
                    target: 1,
                    spell: 3
                },
                drained.clone()
            ],
            1
        ),
        Some(Side::Defender)
    );
    for before in [
        Event::BadMorale { stack: 1 },
        Event::Frightened { stack: 1 },
        Event::TurnPassed { stack: 1 },
        Event::TurnStarted { stack: 0 },
        Event::RoundStarted { round: 2 },
    ] {
        assert_eq!(drain_of(&[before, drained.clone()], 1), None);
    }
    assert_eq!(drain_of(&[drained], 0), None);
}

#[test]
fn poison_takes_its_tenth_before_the_round_s_line() {
    // The round's start (`round_start` 0x47610D): Poison's sound as a file, its line, 67 over the
    // Goblins with their hit (`strike_animation`), its wait; then the round's line. The Pikemen
    // that acted last stay outlined (cleared only at the round start's end, 0x4761EE).
    let events = vec![
        moved(0, 86, 87),
        Event::RoundStarted { round: 2 },
        Event::Poisoned {
            stack: 1,
            hit_points: 4,
        },
        Event::TurnStarted { stack: 0 },
    ];
    let t = play_starts(&two(), &[(Some(0), events.clone())], None);
    let line = logged(&t, |l| {
        *l == Logged::Poisoned {
            stack: 1,
            count: 40,
        }
    });
    let line = line.unwrap_or_else(|| panic!("{:?}", t.log));
    let round = logged(&t, |l| *l == Logged::Round).unwrap_or_default();
    let frames = shown(&t, POISON_TENTH);
    let first = frames[0].0;
    assert!(line < first && first < round);
    assert_eq!(t.at(first)[&1].group, 3);
    assert!(cues(&t).contains(&(line, 1, Sound::SpellFile(POISON), false)));
    assert_eq!(t.outlined(first), Some(0));
    // With sound the round's line waits for `Poison`'s end.
    let long = named(Sound::SpellFile(POISON), 4_000);
    let w = play_starts(&two(), &[(Some(0), events)], Some(&long));
    assert_eq!(logged(&w, |l| *l == Logged::Round), Some(line + 4_000));
    // Without a poisoned stack nothing is drawn.
    let none = vec![
        moved(0, 86, 87),
        Event::RoundStarted { round: 2 },
        Event::TurnStarted { stack: 0 },
    ];
    let quiet = play_starts(&two(), &[(Some(0), none)], None);
    assert!(shown(&quiet, POISON_TENTH).is_empty());
}
