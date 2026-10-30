//! E17 M5 T5b2: the battle waits for its samples when it plays its sounds (design §8.3,
//! `wait_for_sample` 0x59A4C0, `sample_wait` 0x599EC0), and each turn's acting side follows the
//! active stack's controller (R3-1: Hypnotize, a created stack).

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::{Event, Observation, Side};
use h3_render::Bugs;

use super::timeline;
use super::{
    GOBLIN, MARKSMAN, PIKEMAN, TENT, at, flags, healed, layout, looks, machine, magic, moved,
    roll_magic, snapshot, struck, tables,
};
use crate::anim::{Casts, Command, HYPNOTIZE, MagicLook, Sound, Speed, SpellLook, Stage, Timeline};
use crate::sound::{Lengths, SAMPLE_WAIT, length_ms};

/// Hypnotize's handle in these tests (the base pack's 59).
const HYPNOTIZE_SPELL: u16 = 59;

/// The timeline of `snapshot` and `commands` at speed 0 with the magic animations `magic`, the
/// casts `casts` and the samples' `lengths` (none: no sound).
fn waited(
    snapshot: &Observation,
    commands: &[(Option<u32>, Vec<Event>)],
    (magic, casts): (&BTreeMap<u8, MagicLook>, &Casts),
    lengths: Option<&Lengths>,
) -> Timeline {
    let layout = layout();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic,
        casts,
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

#[test]
fn a_samples_length_rounds_its_frames_up() {
    // `Regener`: 41 470 frames at 22 050 Hz, 1 880.7 ms.
    assert_eq!(length_ms(41_470, 22_050), 1_881);
    assert_eq!(
        [length_ms(22_050, 22_050), length_ms(1, 22_050)],
        [1_000, 1]
    );
    assert_eq!(length_ms(5, 0), 0);
    assert_eq!(length_ms(u64::MAX / 1000, 1), u32::MAX);
    // A cue's sample: the battle's own sound by its cue first, else the stack's.
    let lengths = Lengths {
        stacks: [((2, Sound::Shot), 700), ((2, Sound::Heal), 5)]
            .into_iter()
            .collect(),
        named: [(Sound::Heal, 900)].into_iter().collect(),
    };
    assert_eq!(lengths.of(2, Sound::Shot), Some(700));
    assert_eq!(lengths.of(2, Sound::Heal), Some(900));
    assert_eq!(
        [lengths.of(3, Sound::Shot), lengths.of(2, Sound::Move)],
        [None, None]
    );
}

#[test]
fn the_heal_waits_for_its_sound_when_it_outlasts_the_animation() {
    // `first_aid_tent_action` 0x478D28: the heal's frames (400 ms here), then the wait for
    // `Regener` from its start; the tent stays outlined through the wait (no redraw), the next
    // turn starts after it. A sound shorter than the animation is over before the wait.
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 86, false),
        machine(1, Side::Attacker, TENT, 153, 18),
        at(2, Side::Defender, GOBLIN, 100, false),
    ]);
    let events = vec![
        moved(2, 100, 99),
        Event::TurnStarted { stack: 1 },
        healed(1, 0),
        Event::TurnStarted { stack: 0 },
    ];
    let (magic, casts) = (magic(), Casts::default());
    let silent = waited(&o, &[(Some(0), events.clone())], (&magic, &casts), None);
    let turn = silent.turns[0].0;
    assert_eq!(silent.turns[1], (turn + 400, 0));
    let short = named(Sound::Heal, 399);
    let t = waited(
        &o,
        &[(Some(0), events.clone())],
        (&magic, &casts),
        Some(&short),
    );
    assert_eq!(t, silent);
    let long = named(Sound::Heal, 1_000);
    let t = waited(
        &o,
        &[(Some(0), events.clone())],
        (&magic, &casts),
        Some(&long),
    );
    assert_eq!(t.turns, [(turn, 1), (turn + 1_000, 0)]);
    assert_eq!(
        [turn + 400, turn + 999, turn + 1_000].map(|ms| t.outlined(ms)),
        [Some(1), Some(1), Some(0)]
    );
    // The frames as without sound: the wait draws nothing.
    assert_eq!(t.magics, silent.magics);
    // At most 4 s from the wait's call.
    let endless = named(Sound::Heal, 60_000);
    let t = waited(&o, &[(Some(0), events)], (&magic, &casts), Some(&endless));
    assert_eq!(t.turns[1], (turn + 400 + SAMPLE_WAIT, 0));
}

#[test]
fn good_morale_waits_for_its_sound_before_its_redraw() {
    // `good_morale_roll` 0x464B05: the animation, the line, the wait, a redraw shown at once
    // (`morale`: the sound at 4 776, the Marksmen's turn again at 6 876 without sound).
    let o = snapshot(vec![
        at(0, Side::Attacker, MARKSMAN, 35, false),
        at(1, Side::Attacker, PIKEMAN, 137, false),
        at(2, Side::Defender, GOBLIN, 49, false),
    ]);
    let shot = flags(true, false, false);
    let events = vec![
        struck(0, 2, 3, shot),
        Event::GoodMorale { stack: 0 },
        Event::TurnStarted { stack: 0 },
    ];
    let (magic, casts) = (roll_magic(), Casts::default());
    let silent = waited(&o, &[(None, events.clone())], (&magic, &casts), None);
    let cue = silent
        .cues
        .iter()
        .find(|c| c.sound == Sound::GoodMorale)
        .copied()
        .unwrap();
    let again = silent.turns.last().unwrap().0;
    let long = named(Sound::GoodMorale, again - cue.at + 500);
    let t = waited(&o, &[(None, events)], (&magic, &casts), Some(&long));
    assert_eq!(t.turns.last(), Some(&(again + 500, 0)));
}

#[test]
fn a_one_target_shot_waits_for_the_shooters_shot_sound() {
    // `stack_shot` 0x43FDF9: after the target's reaction and the line, `sample_wait` on the
    // shooter's own shot sound (the stack's, by its handle); the next turn starts after it.
    let o = snapshot(vec![
        at(0, Side::Attacker, MARKSMAN, 35, false),
        at(2, Side::Defender, GOBLIN, 49, false),
    ]);
    let shot = flags(true, false, false);
    let events = vec![struck(0, 2, 3, shot), Event::TurnStarted { stack: 2 }];
    let (magic, casts) = (magic(), Casts::default());
    let silent = waited(&o, &[(None, events.clone())], (&magic, &casts), None);
    let cue = silent
        .cues
        .iter()
        .find(|c| c.sound == Sound::Shot && c.stack == 0)
        .copied()
        .unwrap();
    let next = silent.turns.last().unwrap().0;
    let lengths = Lengths {
        stacks: [((0, Sound::Shot), next - cue.at + 250)]
            .into_iter()
            .collect(),
        ..Lengths::default()
    };
    let t = waited(&o, &[(None, events)], (&magic, &casts), Some(&lengths));
    assert_eq!(t.turns.last(), Some(&(next + 250, 2)));
}

fn hypnotize_casts() -> Casts {
    let mut casts = Casts::default();
    casts.spells.insert(
        HYPNOTIZE_SPELL,
        SpellLook {
            original: HYPNOTIZE,
            ..SpellLook::default()
        },
    );
    casts
}

#[test]
fn a_hypnotized_stacks_turn_is_its_controllers() {
    // `stack_activation` 0x465330 sets the acting side +0x132C0 to the stack's controller: the
    // Goblins' turns are the attacker's while Hypnotize holds them, their own again once it
    // ends; the screen shows the acting side's hidden objects (a human plays both here).
    let mut o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 86, false),
        at(2, Side::Defender, GOBLIN, 100, false),
    ]);
    for side in &mut o.sides {
        side.human = true;
    }
    let hypnotized = Event::EffectApplied {
        target: 2,
        spell: HYPNOTIZE_SPELL,
        rounds: Some(1),
    };
    let commands = [
        (Some(0), vec![hypnotized, Event::TurnStarted { stack: 2 }]),
        (
            Some(1_000),
            vec![
                Event::EffectEnded {
                    target: 2,
                    spell: HYPNOTIZE_SPELL,
                },
                Event::TurnStarted { stack: 2 },
            ],
        ),
        // A stack the timeline does not follow counts as the attacker's.
        (Some(2_000), vec![Event::TurnStarted { stack: 9 }]),
    ];
    let (magic, casts) = (magic(), hypnotize_casts());
    let t = waited(&o, &commands, (&magic, &casts), None);
    assert_eq!(
        t.controllers,
        [Side::Attacker, Side::Defender, Side::Attacker]
    );
    let [first, second] = [t.turns[0].0, t.turns[1].0];
    assert_eq!(
        [t.screen(first), t.screen(second)],
        [Some(Side::Attacker), Some(Side::Defender)]
    );
    // Without Hypnotize's look (a record that never casts it) the stack's own side.
    let none = Casts::default();
    let t = waited(&o, &commands[..1], (&magic, &none), None);
    assert_eq!(t.controllers, [Side::Defender]);
}

#[test]
fn a_command_after_a_wait_starts_the_frames_grid_at_the_waits_end() {
    // E17 M5 T5b2 (`gate-sound`): no tick keeps the deadline on its grid while the battle waits
    // for a sample, so a command at the wait's end (or later) has its frames from there: the
    // Goblins' first step 100 ms after the wait's end, not on the grid the heal left behind.
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 86, false),
        machine(1, Side::Attacker, TENT, 153, 18),
        at(2, Side::Defender, GOBLIN, 100, false),
    ]);
    let heal = vec![
        Event::TurnStarted { stack: 1 },
        healed(1, 0),
        Event::TurnStarted { stack: 2 },
    ];
    let (magic, casts) = (magic(), Casts::default());
    let silent = waited(&o, &[(Some(0), heal.clone())], (&magic, &casts), None);
    let turn = silent.turns[0].0;
    // The heal's frames end at turn + 400; the sound 302 ms later.
    let lengths = named(Sound::Heal, 702);
    let end = turn + 702;
    let walk = |at| (Some(at), vec![moved(2, 100, 99), Event::TurnStarted { stack: 0 }]);
    let t = waited(&o, &[(Some(0), heal.clone()), walk(end)], (&magic, &casts), Some(&lengths));
    assert_eq!(t.turns[1], (end, 2));
    let first = |t: &Timeline, after: u32| {
        t.keys
            .iter()
            .filter(|k| k.stack == 2 && k.shown > after)
            .map(|k| k.shown)
            .min()
    };
    assert_eq!(first(&t, end), Some(end + 100));
    // A wait that ends within a grid step of the deadline leaves the grid as it was.
    let near = named(Sound::Heal, 450);
    let t = waited(&o, &[(Some(0), heal), walk(turn + 450)], (&magic, &casts), Some(&near));
    assert_eq!(t.turns[1], (turn + 450, 2));
    assert_eq!(first(&t, turn + 450).map(|s| (s - turn) % 100), Some(0));
}
