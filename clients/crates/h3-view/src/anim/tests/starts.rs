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
    BAD_MORALE, Command, Logged, MANA_DRAIN, MagicLook, POISON, POISON_TENTH, REGENERATION, Sound,
    Speed, Stage, Timeline, drain_of, passes,
};
use crate::sound::Lengths;

/// The strikes' animations with regeneration's (4 frames), the mana drain's (6) and bad
/// morale's (3).
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
    all.insert(BAD_MORALE, entry("cbadmor.def", 3, 1, false));
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

/// The Pikemen (0, active), the Goblins (1) and more Goblins (2).
fn three() -> Observation {
    snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 86, false),
        at(1, Side::Defender, GOBLIN, 100, false),
        at(2, Side::Defender, GOBLIN, 134, false),
    ])
}

#[test]
fn a_round_s_first_pick_outlines_nothing_through_its_effects() {
    // The round's start cleared the outline (0x4761EE): the Goblins' regeneration and drain at
    // its first pick show none; the round's end clears it even with no turn after it (a battle
    // that ends there).
    let events = vec![
        moved(0, 86, 87),
        Event::RoundStarted { round: 2 },
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
    let t = play_starts(&two(), &[(Some(0), events)], None);
    let healed = shown(&t, REGENERATION);
    let drained = shown(&t, MANA_DRAIN);
    assert_eq!(t.outlined(healed[0].0), None);
    assert_eq!(t.outlined(drained[3].0), None);
    assert_eq!(t.turns.last().copied(), Some((drained[6].0, 1)));
    let ended = vec![
        moved(0, 86, 87),
        Event::RoundStarted { round: 2 },
        Event::Poisoned {
            stack: 1,
            hit_points: 4,
        },
    ];
    let e = play_starts(&two(), &[(Some(0), ended)], None);
    let tenth = shown(&e, POISON_TENTH);
    let last = tenth.last().map(|f| f.0).unwrap_or_default();
    assert_eq!(e.outlined(tenth[0].0), Some(0));
    assert_eq!(e.outlined(last + 1_000), None);
}

#[test]
fn a_poison_tenth_that_opens_the_command_keeps_the_last_outline() {
    // The Pikemen defend (nothing drawn): the tenth is the command's first frame, which clears
    // the outline, yet the round start's redraws show the Pikemen's until its end.
    let events = vec![
        Event::Defended { stack: 0 },
        Event::RoundStarted { round: 2 },
        Event::Poisoned {
            stack: 1,
            hit_points: 4,
        },
        Event::TurnStarted { stack: 1 },
    ];
    let t = play_starts(&two(), &[(Some(0), events)], None);
    let tenth = shown(&t, POISON_TENTH);
    assert_eq!(t.outlined(tenth[0].0), Some(0));
    assert_eq!(t.outlined(tenth[1].0), Some(0));
}

#[test]
fn several_tenths_play_one_after_another() {
    // Two poisoned stacks, in the events' order, each its sound, line and wait before the next.
    let events = vec![
        moved(0, 86, 87),
        Event::RoundStarted { round: 2 },
        Event::Poisoned {
            stack: 2,
            hit_points: 4,
        },
        Event::Poisoned {
            stack: 1,
            hit_points: 4,
        },
        Event::TurnStarted { stack: 0 },
    ];
    let long = named(Sound::SpellFile(POISON), 4_000);
    let t = play_starts(&three(), &[(Some(0), events)], Some(&long));
    let lines: Vec<(u32, u8)> = t
        .log
        .iter()
        .filter_map(|(at, l)| match *l {
            Logged::Poisoned { stack, .. } => Some((*at, stack)),
            _ => None,
        })
        .collect();
    assert_eq!(lines.len(), 2);
    assert_eq!((lines[0].1, lines[1].1), (2, 1));
    assert_eq!(lines[1].0, lines[0].0 + 4_000);
    let round = t.log.iter().find(|(_, l)| *l == Logged::Round);
    assert_eq!(round.map(|(at, _)| *at), Some(lines[1].0 + 4_000));
    let over: Vec<u8> = t
        .magics
        .iter()
        .filter(|k| k.animation == POISON_TENTH && k.frame == Some(0))
        .flat_map(|k| k.over.iter().map(|o| o.0))
        .collect();
    assert_eq!(over, [2, 1]);
}

#[test]
fn a_frozen_stack_regenerates_before_its_freeze() {
    // Regeneration at the pick, then bad morale's check freezes the Goblins: 79, then 30; no
    // drain and no turn of theirs.
    let events = vec![
        moved(0, 86, 87),
        Event::TurnStarted { stack: 1 },
        Event::Regenerated {
            stack: 1,
            hit_points: 3,
        },
        Event::BadMorale { stack: 1 },
    ];
    let t = play_starts(&two(), &[(Some(0), events)], None);
    let healed = shown(&t, REGENERATION);
    let frozen = shown(&t, BAD_MORALE);
    assert!(!frozen.is_empty());
    assert!(healed[4].0 <= frozen[0].0);
    assert!(shown(&t, MANA_DRAIN).is_empty());
    assert!(t.turns.iter().all(|&(_, s)| s != 1));
    assert_eq!(t.outlined(frozen[0].0), Some(0));
}

#[test]
fn a_passed_stack_leaves_the_outline_to_the_last() {
    // The Goblins (1) are passed over (blind): the second Goblins' regeneration shows the
    // Pikemen outlined, as the original's pick never made the first active.
    let events = vec![
        moved(0, 86, 87),
        Event::TurnStarted { stack: 1 },
        Event::TurnPassed { stack: 1 },
        Event::TurnStarted { stack: 2 },
        Event::Regenerated {
            stack: 2,
            hit_points: 3,
        },
    ];
    let t = play_starts(&three(), &[(Some(0), events)], None);
    let healed = shown(&t, REGENERATION);
    assert_eq!(t.outlined(healed[2].0), Some(0));
    assert!(passes(&[Event::TurnPassed { stack: 1 }], 1));
    assert!(!passes(&[Event::TurnPassed { stack: 1 }], 2));
    assert!(!passes(&[Event::BadMorale { stack: 1 }], 1));
}

#[test]
fn a_stack_the_timeline_does_not_know_draws_nothing() {
    // Stack 9 is not on the battlefield: no line, no animation, no cue.
    let events = vec![
        moved(0, 86, 87),
        Event::TurnStarted { stack: 9 },
        Event::Regenerated {
            stack: 9,
            hit_points: 3,
        },
        Event::ManaDrained {
            stack: 9,
            side: Side::Attacker,
            mana: 2,
        },
        Event::RoundStarted { round: 2 },
        Event::Poisoned {
            stack: 9,
            hit_points: 4,
        },
    ];
    let t = play_starts(&two(), &[(Some(0), events)], None);
    for number in [REGENERATION, MANA_DRAIN, POISON_TENTH] {
        assert!(shown(&t, number).is_empty());
    }
    assert!(cues(&t).iter().all(|c| c.1 != 9));
}
