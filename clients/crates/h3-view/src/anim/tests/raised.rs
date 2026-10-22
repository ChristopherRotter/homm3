//! E17 M3 T7c-3: the raises, the creatures' own animations and Chain Lightning, replayed with the
//! engine's events of `raise`, `creatures` and `bolt` (their records, `h3-view sequences`) and
//! the paths no recording takes.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::{Event, Observation, Side, WireU64};

use super::{
    ARCHANGEL, MARKSMAN, PIKEMAN, SWORDSMAN, at, cast, cast_heroes, flags, layout, look, looks,
    moved, screenless, snapshot, spots_of, struck, tables, until,
};
use crate::anim::{
    BOLT, CHAIN_START, Caster, Casts, Command, Explosion, LIGHTNING_BOLT, Logged, Look, MagicLook,
    RAISE, SACRIFICED, SHIELD_RETURN, Sound, Speed, SpellLook, Stage, Timeline, timeline, unrolled,
};

const RAISE_LINES: &str = include_str!("../../../../../situations/timelines/raise.txt");
const CREATURES: &str = include_str!("../../../../../situations/timelines/creatures.txt");
const BOLT_LINES: &str = include_str!("../../../../../situations/timelines/bolt.txt");

const SKELETON: u16 = 30;
const GRAND_ELF: u16 = 31;
const EFREET_SULTAN: u16 = 32;
const POWER_LICH: u16 = 33;

/// The spells' handles in these tests: the base pack's numbers.
const FIRE_SHIELD: u16 = 29;
const RESURRECTION: u16 = 38;
const ANIMATE_DEAD: u16 = 39;
const SACRIFICE: u16 = 40;
const CHAIN_LIGHTNING: u16 = 19;
const DEATH_CLOUD: u16 = 76;
const CLOUD: u8 = 72;

/// The sequences' creatures (their DEFs' groups, `clients/assets/base.toml`'s numbers).
fn raise_looks() -> BTreeMap<u16, Look> {
    let full = |times, top| {
        look(
            times,
            &[
                (0, 8),
                (2, 8),
                (3, 6),
                (4, 11),
                (5, 6),
                (14, 12),
                (15, 12),
                (16, 12),
            ],
            top,
        )
    };
    let mut l = looks();
    l.insert(
        PIKEMAN,
        look(
            [9000, 500, 500, 115],
            &[
                (0, 6),
                (2, 7),
                (3, 6),
                (4, 13),
                (5, 5),
                (11, 10),
                (12, 10),
                (13, 11),
            ],
            171,
        ),
    );
    l.insert(SKELETON, full([9000, 500, 500, 115], 180));
    let mut elf = full([9000, 510, 500, 115], 180);
    elf.frames.insert(5, 5);
    elf.climax = 11;
    elf.missile_offsets = [[32, -77], [40, -61], [18, -55]];
    l.insert(GRAND_ELF, elf);
    l.insert(
        EFREET_SULTAN,
        look(
            [9000, 500, 500, 115],
            &[
                (0, 7),
                (2, 5),
                (3, 5),
                (4, 11),
                (5, 8),
                (11, 9),
                (12, 9),
                (13, 10),
            ],
            170,
        ),
    );
    let mut lich = full([9000, 500, 500, 115], 170);
    lich.frames.extend([(5, 7), (14, 11), (15, 11), (16, 11)]);
    lich.climax = 8;
    lich.missile_offsets = [[41, -92], [58, -79], [29, -14]];
    lich.explosion = Some(Explosion {
        spell: DEATH_CLOUD,
        animation: CLOUD,
        time: 100,
    });
    l.insert(POWER_LICH, lich);
    l
}

/// The magic animations these casts draw (their DEFs' sizes and frames).
fn raise_magic() -> BTreeMap<u8, MagicLook> {
    let m = |def: &str, frames, size, place, translucent| MagicLook {
        def: def.to_string(),
        frames,
        size,
        place,
        translucent,
    };
    BTreeMap::from([
        (BOLT, m("C11spA0.def", 1, [106, 494], 2, false)),
        (SHIELD_RETURN, m("C05spF0.def", 10, [75, 62], 1, false)),
        (CHAIN_START, m("C03spA0.def", 1, [106, 494], 2, false)),
        (38, m("C03spA1.def", 12, [114, 110], 1, false)),
        (49, m("C11spA1.def", 12, [114, 110], 0, false)),
        (RAISE, m("C12spE0.def", 20, [98, 95], 1, true)),
        (SACRIFICED, m("C12spF0.def", 13, [114, 108], 1, false)),
        (CLOUD, m("sp04_.def", 20, [142, 139], 15, true)),
    ])
}

/// Orrin's casts of these sequences.
fn raise_casts() -> Casts {
    let spell = |original, animation| SpellLook {
        original,
        animation,
        ..SpellLook::default()
    };
    Casts {
        spells: BTreeMap::from([
            (LIGHTNING_BOLT, spell(LIGHTNING_BOLT, Some(49))),
            (CHAIN_LIGHTNING, spell(CHAIN_LIGHTNING, Some(38))),
            (FIRE_SHIELD, spell(FIRE_SHIELD, Some(SHIELD_RETURN))),
            (RESURRECTION, spell(RESURRECTION, Some(RAISE))),
            (ANIMATE_DEAD, spell(ANIMATE_DEAD, Some(4))),
            (SACRIFICE, spell(SACRIFICE, Some(52))),
            (DEATH_CLOUD, spell(DEATH_CLOUD, Some(CLOUD))),
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
        shielded: BTreeSet::from([EFREET_SULTAN]),
        ..Casts::default()
    }
}

/// The timeline of `snapshot` and `commands` with `casts`, the casts' heroes beside it.
fn play_raised(
    snapshot: &Observation,
    commands: &[(Option<u32>, Vec<Event>)],
    casts: &Casts,
) -> Timeline {
    let layout = layout();
    let magic = raise_magic();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic: &magic,
        casts,
    };
    let commands: Vec<Command<'_>> = commands
        .iter()
        .map(|(at, events)| Command { at: *at, events })
        .collect();
    timeline(snapshot, &commands, &raise_looks(), stage, Speed::default())
        .unwrap()
        .with_heroes(cast_heroes())
}

fn counted(mut stack: h3_protocol::wire::Stack, count: u32) -> h3_protocol::wire::Stack {
    stack.count = count;
    stack
}

fn flier(mut stack: h3_protocol::wire::Stack) -> h3_protocol::wire::Stack {
    stack.flies = true;
    stack
}

fn raised(target: u8, creatures: u32) -> Event {
    Event::SpellRaised {
        target,
        hit_points: WireU64(10),
        creatures,
        temporary: false,
    }
}

fn defend(stack: u8, next: u8) -> Vec<Event> {
    vec![
        Event::Defended { stack },
        Event::TurnStarted { stack: next },
    ]
}

/// The recorded `lines` without their screens and the heroes' rolled reactions, up to `ms`.
fn recorded(lines: &str, ms: u32) -> alloc::string::String {
    until(&unrolled(&screenless(lines), 0, [2, 3]), ms)
}

#[test]
fn the_raise_sequence_plays_as_recorded() {
    // `seq-raise.toml`: the Grand Elves shoot the lone Pikeman, the Marksmen the lone Skeleton;
    // Orrin resurrects the Pikeman (round 1) and animates the Skeleton (round 2).
    let o = snapshot(vec![
        counted(at(0, Side::Attacker, PIKEMAN, 35, false), 1),
        counted(at(1, Side::Attacker, SKELETON, 86, false), 1),
        at(2, Side::Attacker, SWORDSMAN, 137, false),
        at(3, Side::Defender, GRAND_ELF, 49, false),
        at(4, Side::Defender, MARKSMAN, 151, false),
    ]);
    let shot = flags(true, false, false);
    let commands = [
        (
            Some(0),
            vec![struck(3, 0, 1, shot), Event::TurnStarted { stack: 4 }],
        ),
        (
            Some(1112),
            vec![struck(4, 1, 1, shot), Event::TurnStarted { stack: 2 }],
        ),
        (Some(2633), vec![cast(RESURRECTION, 0), raised(0, 1)]),
        (Some(5533), defend(2, 0)),
        (
            Some(5533),
            vec![
                Event::Defended { stack: 0 },
                Event::RoundStarted { round: 2 },
                Event::TurnStarted { stack: 3 },
            ],
        ),
        (Some(5533), defend(3, 4)),
        (Some(5533), defend(4, 2)),
        (Some(5533), vec![cast(ANIMATE_DEAD, 1), raised(1, 1)]),
    ];
    let t = play_raised(&o, &commands, &raise_casts());
    assert_eq!(t.lines_with_heroes(8533), recorded(RAISE_LINES, 8533));
    // Each raise's line with its first frame, its sound on the stack raised; both stand with
    // their count again.
    let lines: Vec<u32> = t
        .log
        .iter()
        .filter(|(_, l)| matches!(l, Logged::Raised { count: 1, .. }))
        .map(|(at, _)| *at)
        .collect();
    let firsts: Vec<u32> = t
        .magics
        .iter()
        .filter(|k| k.animation == RAISE && k.frame == Some(0))
        .map(|k| k.drawn)
        .collect();
    assert_eq!(lines, firsts);
    assert!(
        t.cues
            .iter()
            .any(|c| c.sound == Sound::Spell(ANIMATE_DEAD) && c.stack == 1)
    );
    assert_eq!(t.counts(t.end).get(&0), Some(&1));
    assert!(t.at(t.end)[&1].holds_hex());
}

#[test]
fn the_creatures_sequence_plays_as_recorded() {
    // `seq-creatures.toml`: the Archangels strike the Efreet Sultans from 31 (their shield
    // strikes back before the retaliation), the Sultans defend, the Power Liches shoot the
    // Archangels (their cloud).
    let o = snapshot(vec![
        flier(at(0, Side::Attacker, ARCHANGEL, 86, true)),
        at(1, Side::Defender, EFREET_SULTAN, 49, true),
        at(2, Side::Defender, POWER_LICH, 151, false),
    ]);
    let melee = flags(false, false, false);
    let commands = [
        (
            Some(0),
            vec![
                moved(0, 86, 31),
                struck(0, 1, 2, melee),
                Event::Reflected {
                    stack: 0,
                    damage: WireU64(37),
                    kills: 0,
                },
                struck(1, 0, 0, flags(false, true, false)),
                Event::TurnStarted { stack: 1 },
            ],
        ),
        (Some(6130), defend(1, 2)),
        (
            Some(6130),
            vec![
                struck(2, 0, 0, flags(true, false, false)),
                Event::RoundStarted { round: 2 },
                Event::TurnStarted { stack: 0 },
            ],
        ),
    ];
    let t = play_raised(&o, &commands, &raise_casts());
    assert_eq!(t.lines_with_heroes(9470), recorded(CREATURES, 9470));
    // The shield's sound on the Archangels, its line named by Fire Shield after its animation.
    assert!(
        t.cues
            .iter()
            .any(|c| c.sound == Sound::SpellFile(FIRE_SHIELD) && c.stack == 0)
    );
    let line = Logged::SpellDamage {
        spell: FIRE_SHIELD,
        hit: (0, 37, 0),
    };
    let at = t.log.iter().find(|(_, l)| *l == line).unwrap().0;
    let shield = t.magics.iter().filter(|k| k.animation == SHIELD_RETURN);
    assert!(shield.clone().all(|k| k.drawn <= at));
    assert_eq!(shield.count(), 11);
    // The cloud: its 20 frames plain over the Archangels' centre, then gone; its sound.
    let cloud = spots_of(&t, CLOUD);
    assert_eq!(cloud.len(), 21);
    assert!(cloud.iter().all(|c| c.2 == cloud[0].2));
    assert!(t.spots.iter().all(|s| !s.translucent));
    assert!(t.cues.iter().any(|c| c.sound == Sound::Spell(DEATH_CLOUD)));
    // The Liches shoot at an area: their line as the reactions start, though one stack is struck.
    let lich = t
        .log
        .iter()
        .find(|(_, l)| matches!(l, Logged::Strike { attacker: 2, .. }))
        .unwrap();
    assert_eq!(lich.0, 8662);
    assert!(matches!(
        lich.1,
        Logged::Strike {
            spread: crate::battle_log::Spread::Around,
            ..
        }
    ));
}

#[test]
fn the_bolt_sequence_plays_as_recorded() {
    // `seq-bolt.toml`: Lightning Bolt (round 1) and Chain Lightning (round 2) at the Swordsmen at
    // row 4, then on to rows 6, 10 and 0.
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 86, false),
        at(1, Side::Defender, SWORDSMAN, 15, false),
        at(2, Side::Defender, SWORDSMAN, 83, false),
        at(3, Side::Defender, SWORDSMAN, 117, false),
        at(4, Side::Defender, SWORDSMAN, 185, false),
    ]);
    let hurt = |target, damage, kills| Event::SpellDamage {
        target,
        value: WireU64(damage),
        caster: WireU64(damage),
        damage: WireU64(damage),
        kills,
    };
    let mut commands: Vec<(Option<u32>, Vec<Event>)> = vec![
        (Some(0), defend(1, 2)),
        (Some(0), defend(2, 3)),
        (Some(0), defend(3, 4)),
        (Some(0), defend(4, 0)),
        (Some(0), vec![cast(LIGHTNING_BOLT, 2), hurt(2, 35, 1)]),
        (
            Some(2020),
            vec![
                Event::Defended { stack: 0 },
                Event::RoundStarted { round: 2 },
                Event::TurnStarted { stack: 1 },
            ],
        ),
    ];
    commands.extend([(1, 2), (2, 3), (3, 4), (4, 0)].map(|(s, n)| (Some(2020), defend(s, n))));
    commands.push((
        Some(2020),
        vec![
            cast(CHAIN_LIGHTNING, 2),
            hurt(2, 65, 1),
            hurt(3, 32, 0),
            hurt(4, 16, 0),
            hurt(1, 8, 0),
        ],
    ));
    let t = play_raised(&o, &commands, &raise_casts());
    assert_eq!(t.lines_with_heroes(4420), recorded(BOLT_LINES, 4420));
    // Three rays, each its ticks at once for 100 ms, the mouse hidden through the handler.
    let rays: Vec<(u32, u32)> = t.beams.iter().map(|b| (b.from, b.until)).collect();
    assert_eq!(rays, [(2620, 2720), (2720, 2820), (2820, 2920)]);
    assert!(t.ray(2620).is_some_and(|r| r.colours.len() == 6));
    assert!(t.ray(2919).is_some() && t.ray(2920).is_none() && t.ray(2619).is_none());
    assert!(t.cursor_hidden(2520) && t.cursor_hidden(3900) && !t.cursor_hidden(t.end));
    let line = Logged::AreaDamage {
        spell: CHAIN_LIGHTNING,
        damage: 65,
        kills: 1,
        target: None,
    };
    assert!(t.log.iter().any(|(_, l)| *l == line));
}

#[test]
fn rays_take_the_kept_draws_first() {
    // The same chain from the original's draws: they turn the rays (E17 M3 T7c-3).
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 86, false),
        at(1, Side::Defender, SWORDSMAN, 15, false),
        at(2, Side::Defender, SWORDSMAN, 83, false),
    ]);
    let hurt = |target| Event::SpellDamage {
        target,
        value: WireU64(10),
        caster: WireU64(10),
        damage: WireU64(10),
        kills: 0,
    };
    let commands = [(Some(0), vec![cast(CHAIN_LIGHTNING, 2), hurt(2), hurt(1)])];
    let own = play_raised(&o, &commands, &raise_casts());
    let kept = play_raised(
        &o,
        &commands,
        &Casts {
            rays: vec![-80; 8],
            ..raise_casts()
        },
    );
    assert_eq!(own.beams.len(), 1);
    assert_ne!(own.beams[0].ticks, kept.beams[0].ticks);
}

#[test]
fn raises_by_a_sacrifice_an_archangel_a_phoenix_and_on_the_living() {
    let o = snapshot(vec![
        counted(at(0, Side::Attacker, PIKEMAN, 35, false), 1),
        at(1, Side::Attacker, SWORDSMAN, 137, false),
        at(2, Side::Attacker, ARCHANGEL, 52, true),
        at(3, Side::Defender, MARKSMAN, 151, false),
        counted(at(4, Side::Defender, SWORDSMAN, 49, false), 3),
    ]);
    let commands = [
        // The Pikeman dies; Sacrifice of the Swordsmen raises it.
        (Some(0), vec![struck(3, 0, 1, flags(true, false, false))]),
        (
            None,
            vec![
                cast(SACRIFICE, 0),
                Event::Sacrificed {
                    victim: 1,
                    creatures: 10,
                },
                raised(0, 1),
            ],
        ),
        // The Archangels raise it again after a second death, a Phoenix-like rebirth of the
        // Swordsmen of the other side, Resurrection on a living stack, a raise with nothing to
        // raise, a teleport and a teleport of no stack.
        (None, vec![struck(3, 0, 1, flags(true, false, false))]),
        (
            None,
            vec![
                Event::CreatureRaise {
                    stack: 2,
                    target: 0,
                },
                raised(0, 1),
            ],
        ),
        (
            None,
            vec![
                struck(3, 4, 3, flags(true, false, false)),
                Event::Reborn {
                    stack: 4,
                    creatures: 2,
                },
            ],
        ),
        (None, vec![cast(RESURRECTION, 4), raised(4, 1)]),
        (None, vec![cast(RESURRECTION, 9), raised(9, 1)]),
        (
            None,
            vec![
                Event::CreatureRaise {
                    stack: 2,
                    target: 9,
                },
                Event::CreatureRaise {
                    stack: 9,
                    target: 0,
                },
                Event::Teleported {
                    stack: 3,
                    from: 151,
                    to: 140,
                },
                Event::Teleported {
                    stack: 9,
                    from: 1,
                    to: 2,
                },
            ],
        ),
    ];
    let t = play_raised(&o, &commands, &raise_casts());
    let sacrificed = t.magics.iter().filter(|k| k.animation == SACRIFICED);
    assert_eq!(sacrificed.count(), 14);
    let line = Logged::Cast {
        side: 0,
        spell: SACRIFICE,
        target: 1,
        count: 0,
    };
    assert!(t.log.iter().any(|(_, l)| *l == line));
    // Four raises (the Pikeman twice, the Swordsmen twice), and a fifth of none (an Archangel no
    // stack holds raising the Pikeman with no stack raised after it).
    let raises = t
        .log
        .iter()
        .filter(|(_, l)| matches!(l, Logged::Raised { .. }));
    assert_eq!(raises.count(), 5);
    let none = Logged::Raised { stack: 0, count: 0 };
    assert!(t.log.iter().any(|(_, l)| *l == none));
    assert_eq!(t.counts(t.end).get(&4), Some(&3));
    assert_eq!(t.counts(t.end).get(&0), Some(&1));
    // The Archangels cast toward the corpse and turn back; Resurrection's sound as a file.
    assert!(
        t.cues
            .iter()
            .any(|c| c.sound == Sound::SpellFile(RESURRECTION) && c.stack == 0)
    );
    assert!(
        t.cues
            .iter()
            .any(|c| c.sound == Sound::Spell(RESURRECTION) && c.stack == 4)
    );
    assert_eq!(t.at(t.end)[&3].hex, 140);
}

#[test]
fn creature_animations_without_their_looks_or_stacks() {
    // No Fire Shield among the spells, no magic for the return or the cloud, a shield of no
    // stack, a shooter's explosion at nothing: what can be drawn is.
    let o = snapshot(vec![
        at(0, Side::Attacker, ARCHANGEL, 86, true),
        at(1, Side::Defender, EFREET_SULTAN, 49, true),
        at(2, Side::Defender, POWER_LICH, 151, false),
    ]);
    let commands = [
        (
            Some(0),
            vec![
                moved(0, 86, 31),
                struck(0, 1, 0, flags(false, false, false)),
                Event::Reflected {
                    stack: 0,
                    damage: WireU64(5),
                    kills: 0,
                },
                Event::Reflected {
                    stack: 7,
                    damage: WireU64(5),
                    kills: 0,
                },
            ],
        ),
        (None, vec![struck(2, 0, 0, flags(true, false, false))]),
        (None, vec![cast(CHAIN_LIGHTNING, 0)]),
    ];
    let layout = layout();
    let magic = BTreeMap::new();
    let casts = Casts::default();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic: &magic,
        casts: &casts,
    };
    let commands: Vec<Command<'_>> = commands
        .iter()
        .map(|(at, events)| Command { at: *at, events })
        .collect();
    let t = timeline(&o, &commands, &raise_looks(), stage, Speed::default()).unwrap();
    assert!(t.magics.is_empty() && t.spots.is_empty() && t.beams.is_empty());
    assert!(
        !t.log
            .iter()
            .any(|(_, l)| matches!(l, Logged::SpellDamage { .. }))
    );
}

#[test]
fn the_heroes_rolled_reactions_are_left_out() {
    let lines = "0 hero 0 g0 f0\n100 hero 0 g2 f1\n100 0.0 g2 f0 h1\n200 hero 0 g0 f0\n\
                 300 hero 1 g3 f0\n400 hero 1 g4 f0\n500 hero 0 g1 f0\n";
    assert_eq!(
        unrolled(lines, 0, [2, 3]),
        "0 hero 0 g0 f0\n100 0.0 g2 f0 h1\n400 hero 1 g4 f0\n500 hero 0 g1 f0\n"
    );
}

/// The timeline of `snapshot` and `commands` with `casts` and `magic`, no heroes.
fn play_bare(
    snapshot: &Observation,
    commands: &[(Option<u32>, Vec<Event>)],
    casts: &Casts,
    magic: &BTreeMap<u8, MagicLook>,
) -> Timeline {
    let layout = layout();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic,
        casts,
    };
    let commands: Vec<Command<'_>> = commands
        .iter()
        .map(|(at, events)| Command { at: *at, events })
        .collect();
    timeline(snapshot, &commands, &raise_looks(), stage, Speed::default()).unwrap()
}

#[test]
fn chains_shields_and_raises_without_their_stacks_spells_or_looks() {
    let mut shielded = at(1, Side::Defender, SWORDSMAN, 49, false);
    shielded.effects.push(h3_protocol::wire::Effect {
        spell: FIRE_SHIELD,
        rounds: Some(1),
        mastery: 1,
        value: 20,
        attack: 0,
        defense: 0,
        speed: 0,
    });
    let o = snapshot(vec![
        counted(at(0, Side::Attacker, PIKEMAN, 35, false), 1),
        shielded,
        at(2, Side::Attacker, ARCHANGEL, 52, true),
        at(3, Side::Defender, SWORDSMAN, 117, false),
        at(4, Side::Defender, MARKSMAN, 151, false),
    ]);
    let hurt = |target| Event::SpellDamage {
        target,
        value: WireU64(10),
        caster: WireU64(10),
        damage: WireU64(10),
        kills: 0,
    };
    let commands = [
        // A chain past a stack no one holds, with a resistance among its effects; one striking
        // nothing.
        (
            Some(0),
            vec![
                cast(CHAIN_LIGHTNING, 1),
                Event::SpellResisted { target: 3 },
                hurt(1),
                hurt(9),
                hurt(3),
            ],
        ),
        (None, vec![cast(CHAIN_LIGHTNING, 1)]),
        // A shield cast on the Swordsmen, none on the Marksmen: both struck by the Archangels.
        (
            None,
            vec![
                Event::Struck {
                    attacker: 2,
                    target: 1,
                    damage: WireU64(10),
                    kills: 0,
                    flags: flags(false, false, false),
                },
                Event::Struck {
                    attacker: 2,
                    target: 4,
                    damage: WireU64(10),
                    kills: 0,
                    flags: flags(false, false, true),
                },
                Event::Reflected {
                    stack: 2,
                    damage: WireU64(4),
                    kills: 0,
                },
            ],
        ),
        // The Pikeman dies; a Sacrifice of no stack raises it, then an Archangel and a rebirth
        // with no Resurrection among the spells.
        (None, vec![struck(4, 0, 1, flags(true, false, false))]),
        (
            None,
            vec![
                cast(SACRIFICE, 0),
                Event::Sacrificed {
                    victim: 9,
                    creatures: 1,
                },
                raised(0, 1),
            ],
        ),
        (
            None,
            vec![
                Event::CreatureRaise {
                    stack: 2,
                    target: 0,
                },
                raised(0, 1),
                Event::Reborn {
                    stack: 3,
                    creatures: 1,
                },
            ],
        ),
    ];
    let mut bare = raise_casts();
    bare.spells.remove(&RESURRECTION);
    let t = play_bare(&o, &commands, &bare, &raise_magic());
    // One ray (to the third stack), 38 over the two it found; the Fire Shield over the
    // Swordsmen only.
    assert_eq!(t.beams.len(), 1);
    let chained = t.magics.iter().find(|k| k.animation == 38).unwrap();
    assert_eq!(chained.over.len(), 2);
    let shield = t
        .magics
        .iter()
        .find(|k| k.animation == SHIELD_RETURN)
        .unwrap();
    assert_eq!(shield.over.iter().map(|o| o.0).collect::<Vec<_>>(), [1]);
    assert!(!t.log.iter().any(|(_, l)| matches!(l, Logged::Cast { .. })));
    let raises = t
        .log
        .iter()
        .filter(|(_, l)| matches!(l, Logged::Raised { .. }));
    assert_eq!(raises.count(), 3);
    assert!(!t.cues.iter().any(|c| matches!(
        c.sound,
        Sound::Spell(RESURRECTION) | Sound::SpellFile(RESURRECTION)
    )));
    // Without Chain Lightning's animation its rays still play; nothing rides the stacks.
    let t = play_bare(&o, &commands[..1], &bare, &BTreeMap::new());
    assert_eq!(t.beams.len(), 1);
    assert!(t.magics.is_empty());
}
