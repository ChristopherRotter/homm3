//! E17 M5 T5a: the stacks a battle creates and the dissolves (design §7.25): Summon Elemental,
//! Clone and its slide, a Pit Lord's Demons, Teleport, the stacks that leave without a corpse and
//! the clones that vanish, the message and the line when no stack is made.

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::{Event, Observation, Side, Target};
use h3_render::Bugs;

use super::timeline;
use super::{PIKEMAN, SWORDSMAN, at, flags, layout, look, looks, magic, snapshot, struck, tables};
use crate::anim::{
    BATTLEFIELD, Body, CORPSE, CREATION_STEP, Caster, Casts, Command, DISSOLVE_STEPS, DissolveArea,
    DissolveKey, GONE, Logged, Look, Notice, STANDING, Speed, SpellLook, Stage, TELEPORT,
    TELEPORT_STEP, Timeline,
};
use crate::scene::Moment;

/// The created creatures of these tests (invented handles).
pub(crate) const FIRE_ELEMENTAL: u16 = 40;
const DEMON: u16 = 41;
const PIT_LORD: u16 = 42;
/// The spells' handles: the base pack's numbers (Teleport 63, Clone 65, Summon Fire Elemental
/// 66).
const TELEPORT_SPELL: u16 = 63;
const CLONE: u16 = 65;
pub(crate) const SUMMON: u16 = 66;
/// A death's dissolve step at speed 0 (trunc(150 F)).
const REMOVAL: u32 = 150;

fn created_looks() -> BTreeMap<u16, Look> {
    let body = || {
        look(
            [9000, 500, 500, 115],
            &[
                (0, 8),
                (2, 8),
                (3, 6),
                (4, 11),
                (5, 6),
                (17, 6),
                (18, 6),
                (19, 6),
            ],
            180,
        )
    };
    let mut l = looks();
    for c in [FIRE_ELEMENTAL, DEMON, PIT_LORD] {
        l.insert(c, body());
    }
    l
}

fn created_casts() -> Casts {
    let spell = |original| SpellLook {
        original,
        ..SpellLook::default()
    };
    Casts {
        spells: BTreeMap::from([
            (TELEPORT_SPELL, spell(TELEPORT)),
            (CLONE, spell(65)),
            (SUMMON, spell(66)),
            (38, spell(38)),
        ]),
        casters: [
            Some(Caster {
                point: [49, 48],
                before: 5,
                frames: 8,
            }),
            None,
        ],
        groups: [4, 0],
        bodies: BTreeMap::from([
            (
                FIRE_ELEMENTAL,
                Body {
                    two_hex: false,
                    flies: false,
                    shooter: false,
                },
            ),
            (
                DEMON,
                Body {
                    two_hex: false,
                    flies: false,
                    shooter: true,
                },
            ),
        ]),
        ..Casts::default()
    }
}

pub(crate) fn play_created(
    snapshot: &Observation,
    commands: &[(Option<u32>, Vec<Event>)],
) -> Timeline {
    play_with(snapshot, commands, &created_casts())
}

fn play_with(
    snapshot: &Observation,
    commands: &[(Option<u32>, Vec<Event>)],
    casts: &Casts,
) -> Timeline {
    let layout = layout();
    let magic = magic();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic: &magic,
        casts,
        bugs: Bugs::Original,
        siege: None,
        lengths: None,
    };
    let commands: Vec<Command<'_>> = commands
        .iter()
        .map(|(at, events)| Command { at: *at, events })
        .collect();
    timeline(
        snapshot,
        &commands,
        &created_looks(),
        stage,
        Speed::default(),
    )
    .unwrap()
}

pub(crate) fn spell(spell: u16, target: Target) -> Event {
    Event::SpellCast {
        side: Side::Attacker,
        spell,
        target,
        second: Target::None {},
    }
}

pub(crate) fn summoned(stack: u8, count: u32, hex: u8) -> Event {
    Event::StackCreated {
        stack,
        creature: FIRE_ELEMENTAL,
        count,
        hex,
        original: None,
    }
}

/// The dissolves of `t` as (start, step, area).
fn dissolves(t: &Timeline) -> Vec<(u32, u32, DissolveArea)> {
    t.dissolves
        .iter()
        .map(|d| (d.start, d.step, d.area.clone()))
        .collect()
}

#[test]
fn teleport_dissolves_the_battlefield_in_33_ms_steps() {
    let o = snapshot(vec![
        at(0, Side::Attacker, SWORDSMAN, 86, false),
        at(1, Side::Defender, PIKEMAN, 50, false),
    ]);
    let commands = [(
        Some(0),
        vec![
            spell(TELEPORT_SPELL, Target::Stack { stack: 0 }),
            Event::Teleported {
                stack: 0,
                from: 86,
                to: 90,
            },
        ],
    )];
    let t = play_created(&o, &commands);
    let [(start, step, ref area)] = dissolves(&t)[..] else {
        panic!("{:?}", t.dissolves);
    };
    assert_eq!((step, area), (TELEPORT_STEP, &DissolveArea::Battlefield));
    // After the hero's five casting frames (0x5A01CF), whatever the speed.
    assert_eq!(start, 500);
    assert_eq!((t.at(start)[&0].hex, t.at(start + 1)[&0].hex), (86, 90));
    // The saved picture at its start, the steps blending, the new picture at its end.
    assert_eq!(Moment::at(&t, start).dissolve, None);
    let blending = Moment::at(&t, start + 1).dissolve.unwrap();
    assert_eq!((blending.step, blending.before.poses[&0].hex), (0, 86));
    assert_eq!(Moment::at(&t, start + 7 * step).dissolve.unwrap().step, 7);
    let end = start + DISSOLVE_STEPS * step;
    assert_eq!(Moment::at(&t, end).dissolve, None);
    // The hero's frames 5 to 7 after it.
    assert!(t.hero_keys.iter().any(|k| k.drawn >= end));
    assert_eq!(BATTLEFIELD, [0, 0, 800, 556]);
    // A Teleport that moved no stack, or one the timeline does not know, dissolves nothing.
    let commands = [
        (
            Some(0),
            vec![
                spell(TELEPORT_SPELL, Target::Stack { stack: 0 }),
                Event::SpellResisted { target: 0 },
            ],
        ),
        (
            None,
            vec![
                spell(TELEPORT_SPELL, Target::Stack { stack: 9 }),
                Event::Teleported {
                    stack: 9,
                    from: 40,
                    to: 41,
                },
            ],
        ),
    ];
    assert!(play_created(&o, &commands).dissolves.is_empty());
}

#[test]
fn a_summoned_stack_dissolves_in_and_away_and_its_record_goes_to_the_next() {
    let o = snapshot(vec![
        at(0, Side::Attacker, SWORDSMAN, 86, false),
        at(1, Side::Defender, PIKEMAN, 50, false),
    ]);
    let commands = [
        (
            Some(0),
            vec![spell(SUMMON, Target::None {}), summoned(2, 6, 18)],
        ),
        // The Pikemen kill the elementals: they leave no corpse.
        (None, vec![struck(1, 2, 6, flags(false, false, false))]),
        // Orrin clones his Swordsmen: the clone takes the dead elementals' id and record.
        (
            None,
            vec![
                spell(CLONE, Target::Stack { stack: 0 }),
                Event::StackCreated {
                    stack: 2,
                    creature: SWORDSMAN,
                    count: 40,
                    hex: 70,
                    original: Some(0),
                },
            ],
        ),
        // Its rounds run out at a round's start.
        (
            None,
            vec![
                Event::RoundStarted { round: 2 },
                Event::Vanished {
                    stack: 2,
                    creatures: 40,
                },
            ],
        ),
    ];
    let t = play_created(&o, &commands);
    let ds = dissolves(&t);
    assert_eq!(ds.len(), 3, "{ds:?}");
    let (start, step, _) = ds[0];
    assert_eq!(
        (start, step, &ds[0].2),
        (500, CREATION_STEP, &DissolveArea::Stacks(vec![2]))
    );
    // Its record is the attacker's next, set up as the dissolve starts, drawn from 1 ms on.
    let made = &t.created[0];
    assert_eq!(
        (made.member.record, made.drawn, made.shown),
        (1, start, start + 1)
    );
    assert!(
        t.lines()
            .contains(&alloc::format!("{start} 0.1 g2 f0 h18\n"))
    );
    assert!(!Moment::at(&t, start).created.contains_key(&2));
    assert_eq!(Moment::at(&t, start + 1).created[&2].count, 6);
    assert_eq!(t.counts(start + 1).get(&2), Some(&6));
    let line = Logged::Summoned {
        side: 0,
        creature: FIRE_ELEMENTAL,
        count: 6,
    };
    assert!(t.log.iter().any(|(_, l)| *l == line));
    // Killed: gone from the field in a dissolve of trunc(150 F) steps.
    let (gone, step, ref area) = ds[1];
    assert_eq!((step, area), (REMOVAL, &DissolveArea::Stacks(vec![2])));
    assert_eq!(t.at(gone + 1)[&2].layer, GONE);
    // The clone slides from its original's place in 16 redraws, then stands: no dissolve; its
    // record is the elementals'.
    let clone = &t.created[1];
    assert_eq!(clone.member.record, 1);
    let slid: Vec<[i32; 2]> = t
        .keys
        .iter()
        .filter(|k| k.stack == 2 && k.drawn > gone + DISSOLVE_STEPS * REMOVAL)
        .filter(|k| k.pose.layer != GONE)
        .map(|k| k.pose.offset)
        .collect();
    assert_eq!(slid.len(), 17, "{slid:?}");
    assert_eq!(slid[16], [0, 0]);
    let first = slid[0];
    assert_ne!(first, [0, 0]);
    assert_eq!(slid[8], first.map(|v| v * 8 / 16));
    assert!(t.stacks_created_at_end(&[(2, SWORDSMAN)]));
    // Vanished at the round's start: one more dissolve.
    let (vanished, _, ref area) = ds[2];
    assert_eq!(area, &DissolveArea::Stacks(vec![2]));
    assert_eq!(t.at(vanished + 1)[&2].layer, GONE);
}

#[test]
fn a_defenders_summon_takes_its_sides_next_record() {
    let o = snapshot(vec![
        at(0, Side::Attacker, SWORDSMAN, 86, false),
        at(1, Side::Defender, PIKEMAN, 50, false),
    ]);
    let cast = |spell, target| Event::SpellCast {
        side: Side::Defender,
        spell,
        target,
        second: Target::None {},
    };
    let commands = [
        (
            Some(0),
            vec![cast(SUMMON, Target::None {}), summoned(2, 6, 32)],
        ),
        // A clone of a creature without a look: no stack the timeline draws.
        (
            None,
            vec![
                cast(CLONE, Target::Stack { stack: 1 }),
                Event::StackCreated {
                    stack: 3,
                    creature: 999,
                    count: 40,
                    hex: 40,
                    original: Some(1),
                },
            ],
        ),
    ];
    let t = play_created(&o, &commands);
    let made = t.member_at(Side::Defender, 1).unwrap();
    assert_eq!((made.stack, made.side), (2, Side::Defender));
    assert_eq!(t.member(2).map(|m| m.record), Some(1));
    assert!(t.member_at(Side::Attacker, 1).is_none());
    assert_eq!(t.created.len(), 1);
    assert!(t.member(3).is_none());
}

#[test]
fn a_clone_gone_with_its_original_dissolves_with_the_deaths() {
    let o = snapshot(vec![
        at(0, Side::Attacker, SWORDSMAN, 86, false),
        at(1, Side::Defender, PIKEMAN, 50, false),
    ]);
    let commands = [
        (
            Some(0),
            vec![
                spell(CLONE, Target::Stack { stack: 0 }),
                Event::StackCreated {
                    stack: 2,
                    creature: SWORDSMAN,
                    count: 40,
                    hex: 70,
                    original: Some(0),
                },
            ],
        ),
        // The Pikemen kill the Swordsmen: their clone vanishes with them.
        (
            None,
            vec![
                struck(1, 0, 40, flags(false, false, false)),
                Event::Vanished {
                    stack: 2,
                    creatures: 40,
                },
            ],
        ),
        // A clone gone where no death plays (its original's corpse raised away, say).
        (
            None,
            vec![
                spell(CLONE, Target::Stack { stack: 1 }),
                Event::StackCreated {
                    stack: 3,
                    creature: PIKEMAN,
                    count: 40,
                    hex: 40,
                    original: Some(1),
                },
                Event::Vanished {
                    stack: 3,
                    creatures: 40,
                },
            ],
        ),
    ];
    let t = play_created(&o, &commands);
    let ds = dissolves(&t);
    assert_eq!(
        ds.iter().map(|d| d.2.clone()).collect::<Vec<_>>(),
        [DissolveArea::Stacks(vec![2]), DissolveArea::Stacks(vec![3])]
    );
    // The original lies as a corpse.
    assert_ne!(t.at(t.end)[&0].layer, GONE);
}

#[test]
fn a_pit_lord_raises_demons_where_a_corpse_lay() {
    let o = snapshot(vec![
        at(0, Side::Attacker, PIT_LORD, 86, false),
        at(1, Side::Attacker, PIKEMAN, 52, false),
        at(2, Side::Defender, SWORDSMAN, 53, false),
    ]);
    let commands = [
        (Some(0), vec![struck(2, 1, 40, flags(false, false, false))]),
        (
            None,
            vec![
                Event::CreatureRaise {
                    stack: 0,
                    target: 1,
                },
                Event::RoseFromCorpse {
                    stack: 3,
                    corpse: 1,
                    creature: DEMON,
                    count: 10,
                    hex: 52,
                },
            ],
        ),
        // A raise of no corpse the timeline knows draws nothing.
        (
            None,
            vec![
                Event::CreatureRaise {
                    stack: 0,
                    target: 9,
                },
                Event::RoseFromCorpse {
                    stack: 4,
                    corpse: 9,
                    creature: DEMON,
                    count: 10,
                    hex: 60,
                },
            ],
        ),
    ];
    let t = play_created(&o, &commands);
    let [(start, step, ref area)] = dissolves(&t)[..] else {
        panic!("{:?}", t.dissolves);
    };
    assert_eq!(
        (step, area),
        (CREATION_STEP, &DissolveArea::Stacks(vec![3]))
    );
    // The corpse goes and the Demons stand there, both with the dissolve's new picture; the
    // attacker's records 0 and 1 are taken: theirs is 2.
    assert_eq!(t.at(start + 1)[&1].layer, GONE);
    // It goes in the pose it lay in, raised a command after its death (memory keeps the pose).
    let (lay, gone) = (t.at(start)[&1], t.at(start + 1)[&1]);
    assert_eq!((gone.group, gone.frame), (lay.group, lay.frame));
    assert_eq!(lay.layer, CORPSE);
    assert_eq!(t.at(start + 1)[&3].hex, 52);
    assert_eq!(t.created[0].member.record, 2);
    assert!(t.created[0].stack.raised && t.created[0].stack.shooter);
    let line = Logged::Raised {
        stack: 3,
        count: 10,
    };
    assert!(t.log.iter().any(|(_, l)| *l == line));
    assert_eq!(t.created.len(), 1);
}

#[test]
fn demons_in_the_corpses_id_or_of_an_unknown_creature() {
    // Without Resurrection's sound in the ruleset; the Demons taking the corpse's own id (no
    // corpse key: theirs replaces it); an unknown creature's Demons: no stack, no dissolve.
    let o = snapshot(vec![
        at(0, Side::Attacker, PIT_LORD, 86, false),
        at(1, Side::Attacker, PIKEMAN, 52, false),
        at(2, Side::Attacker, PIKEMAN, 54, false),
        at(3, Side::Defender, SWORDSMAN, 53, false),
    ]);
    let raise = |target, (new, creature, hex)| {
        vec![
            Event::CreatureRaise { stack: 0, target },
            Event::RoseFromCorpse {
                stack: new,
                corpse: target,
                creature,
                count: 10,
                hex,
            },
        ]
    };
    let commands = [
        (Some(0), vec![struck(3, 1, 40, flags(false, false, false))]),
        (None, vec![struck(3, 2, 40, flags(false, false, false))]),
        (None, raise(1, (1, DEMON, 52))),
        (None, raise(2, (4, 999, 54))),
    ];
    let mut casts = created_casts();
    casts.spells.remove(&38);
    let t = play_with(&o, &commands, &casts);
    let [(start, ..)] = dissolves(&t)[..] else {
        panic!("{:?}", t.dissolves);
    };
    assert_eq!(t.at(start + 1)[&1].layer, STANDING);
    assert!(t.member(4).is_none());
}

#[test]
fn no_place_opens_clones_message_and_writes_the_summons_line() {
    let o = snapshot(vec![
        at(0, Side::Attacker, SWORDSMAN, 86, false),
        at(1, Side::Defender, PIKEMAN, 50, false),
    ]);
    let commands = [
        (
            Some(0),
            vec![
                spell(CLONE, Target::Stack { stack: 0 }),
                Event::NotCreated {
                    side: Side::Attacker,
                    creature: SWORDSMAN,
                    count: 40,
                    original: Some(0),
                },
            ],
        ),
        (
            None,
            vec![
                spell(SUMMON, Target::None {}),
                Event::NotCreated {
                    side: Side::Defender,
                    creature: FIRE_ELEMENTAL,
                    count: 6,
                    original: None,
                },
            ],
        ),
        // A clone of a stack the timeline does not know, a summon of a creature without a
        // look, an original turned into its own clone (quirk BAT-QUIRK-7): 16 redraws, nothing
        // made.
        (
            None,
            vec![
                spell(CLONE, Target::Stack { stack: 7 }),
                Event::StackCreated {
                    stack: 8,
                    creature: SWORDSMAN,
                    count: 1,
                    hex: 40,
                    original: Some(7),
                },
            ],
        ),
        (
            None,
            vec![
                spell(SUMMON, Target::None {}),
                Event::StackCreated {
                    stack: 9,
                    creature: 99,
                    count: 1,
                    hex: 40,
                    original: None,
                },
            ],
        ),
        (
            None,
            vec![
                spell(CLONE, Target::Stack { stack: 0 }),
                Event::TurnedSummoned {
                    stack: 0,
                    rounds: 2,
                },
            ],
        ),
    ];
    let t = play_created(&o, &commands);
    assert_eq!(t.notices, [(500, Notice::NoPlace)]);
    let line = Logged::Summoned {
        side: 1,
        creature: FIRE_ELEMENTAL,
        count: 6,
    };
    assert!(t.log.iter().any(|(_, l)| *l == line));
    assert!(t.dissolves.is_empty() && t.created.is_empty());
    // The turned original's 16 redraws: its stack stays as it stood.
    assert_eq!(t.at(t.end)[&0].hex, 86);
}

#[test]
fn a_dissolve_steps_from_the_saved_picture_to_the_new() {
    let d = DissolveKey {
        start: 100,
        step: 75,
        area: DissolveArea::Battlefield,
    };
    assert_eq!(d.end(), 700);
    assert_eq!(
        [99, 100, 101, 174, 175, 699, 700].map(|t| d.step_at(t)),
        [None, None, Some(0), Some(0), Some(1), Some(7), None]
    );
}

impl Timeline {
    /// Whether the created stacks at the end are `stacks` (id, creature).
    fn stacks_created_at_end(&self, stacks: &[(u8, u16)]) -> bool {
        let at = self.created_at(self.end);
        at.len() == stacks.len()
            && stacks
                .iter()
                .all(|&(id, c)| at.get(&id).is_some_and(|s| s.creature == c))
    }
}
