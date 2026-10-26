//! E17 M4 T2c-1: what a walk meets in a siege (design §10.2): the drawbridge opening before the
//! step onto the gate's hexes and closing after the walk, a Land Mine's blast on the way, the
//! moat's damage at the walk's end and as a stack acts again in it; replayed from the recorded
//! sequences' events with their creatures' numbers (the asset tables' times, the DEFs' frame
//! counts, read from the user's archives: numbers only, ADR-0009).

use alloc::collections::BTreeMap;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::{Event, Observation, Side, Siege};
use h3_render::Bugs;

use super::siege::{
    CATAPULT, CHAMPION, GOBLIN, HOBGOBLIN, PIKEMAN, TITAN, TURRET, castle, machine_at, play_siege,
    recorded, siege_looks,
};
use super::{at, layout, snapshot, tables, until};
use crate::anim::{
    BridgeKey, Casts, Command, Logged, MINE_BLAST, MagicLook, ObjectLook, Sound, Speed, SpellLook,
    Stage, Timeline, timeline,
};
use crate::siege::{GATE_CLOSED, GATE_MOVING, GATE_OPEN};

const GATE: &str = include_str!("../../../../../situations/timelines/gate.txt");
const MOAT: &str = include_str!("../../../../../situations/timelines/moat.txt");
const MINE: &str = include_str!("../../../../../situations/timelines/mine.txt");

/// A town's walls, the gate closed: a Fort's without a moat (`fort` 1), or with its moat's hexes.
fn town(faction: u16, fort: u8, moat: u8, moat_hexes: Vec<u8>) -> Siege {
    Siege {
        faction,
        fort,
        moat,
        gate_open: false,
        walls: vec![12, 29, 45, 62, 78, 96, 112, 130, 147, 165, 182],
        moat_hexes,
        parts: vec![2; 8],
    }
}

/// `gate`'s siege (`seq-gate.toml`): a Castle town with a Fort, the attacker's Pikemen and
/// Catapult, the defender's Hobgoblins inside; the Catapult's turn opens it.
fn gate_siege(faction: u16) -> Observation {
    let mut catapult = machine_at(1, Side::Attacker, CATAPULT, (119, 19));
    catapult.two_hex = true;
    let mut o = snapshot(vec![
        catapult,
        at(0, Side::Attacker, PIKEMAN, 86, false),
        at(2, Side::Defender, HOBGOBLIN, 100, false),
    ]);
    o.siege = Some(town(faction, 1, 0, Vec::new()));
    o
}

/// The Hobgoblins' (2) walk along `path`, the gate opening as the engine reports it after the
/// step onto `gate` (none: no opening), then what `after` adds.
fn walk(path: &[u8], gate: Option<u8>, after: &[Event]) -> Vec<Event> {
    let mut events = Vec::new();
    for pair in path.windows(2) {
        events.push(Event::Moved {
            stack: 2,
            from: pair[0],
            to: pair[1],
        });
        if gate == Some(pair[1]) {
            events.push(Event::GateOpened {});
        }
    }
    events.extend_from_slice(after);
    events
}

/// The Catapult's (1) wall shot at `part` for 1, then the next turn.
fn shot(part: u8, next: u8) -> Vec<Event> {
    vec![
        Event::WallShot {
            stack: 1,
            aimed: part,
            part,
            damage: 1,
        },
        Event::TurnStarted { stack: next },
    ]
}

/// The gate's states drawn over time, in order.
fn gate_states(t: &Timeline) -> Vec<(u32, u32, u8)> {
    t.walls
        .iter()
        .map(|k| (k.drawn, k.shown, k.walls.gate))
        .collect()
}

#[test]
fn the_gate_sequence_plays_as_recorded() {
    // `gate`: the Hobgoblins walk out from 100 to 93, the walk stopping at 97 before the step
    // onto the gate (96), which opens; they walk on and the gate closes after them. In round 2
    // they walk back in, the gate opening before their first step (onto 94).
    let o = gate_siege(0);
    let turn = |stack| Event::TurnStarted { stack };
    let out = walk(
        &[100, 99, 98, 97, 96, 95, 94, 93],
        Some(96),
        &[Event::GateClosed {}, turn(0)],
    );
    let back = walk(
        &[93, 94, 95, 96, 97, 98, 99, 100],
        Some(94),
        &[Event::GateClosed {}, turn(0)],
    );
    let commands = [
        (Some(0), shot(1, 2)),
        (Some(3775), out),
        (
            Some(9259),
            vec![
                Event::Defended { stack: 0 },
                Event::RoundStarted { round: 2 },
                turn(1),
            ],
        ),
        (Some(9259), shot(5, 2)),
        (Some(13134), back),
    ];
    let t = play_siege(&o, &commands, Bugs::Original, Some(&castle()));
    assert_eq!(until(&t.lines(), 19134), recorded(GATE, 19134));
    // The gate's states: the snapshot's, the shots' (closed), then the walk's opening (closed,
    // moving, open at 100 ms from the stop's whole redraw at 5 911), the closing after it, and
    // round 2's opening before the first step and closing after the turn back.
    let states = gate_states(&t);
    let moving: Vec<&(u32, u32, u8)> = states.iter().filter(|s| s.2 == GATE_MOVING).collect();
    assert_eq!(
        moving,
        [
            &(6011, 6111, 2),
            &(9059, 9159, 2),
            &(13234, 13334, 2),
            &(19518, 19618, 2)
        ]
    );
    assert!(states.contains(&(5911, 6011, GATE_CLOSED)));
    assert!(states.contains(&(6111, 6211, GATE_OPEN)));
    assert_eq!(states.last().map(|s| s.2), Some(GATE_CLOSED));
    // The screen shows each state at its deadline, the next state's drawing time.
    let shown = |ms| t.walls(ms).map(|w| w.gate);
    assert_eq!(
        [shown(6110), shown(6111), shown(6211)],
        [Some(GATE_CLOSED), Some(GATE_MOVING), Some(GATE_OPEN)]
    );
    // The drawbridge's sound with each move of the gate, on no stack; the stop's end sound and
    // the restart's start sound around the opening.
    let cues = |sound| -> Vec<u32> {
        t.cues
            .iter()
            .filter(|c| c.sound == sound && !c.stop)
            .map(|c| c.at)
            .collect()
    };
    assert_eq!(cues(Sound::Drawbridge), [5911, 8959, 13134, 19418]);
    assert!(
        t.cues
            .iter()
            .all(|c| c.sound != Sound::Drawbridge || c.stack == 0)
    );
    let starts = cues(Sound::Start);
    assert!(starts.contains(&3775) && starts.contains(&6211));
    assert!(cues(Sound::End).contains(&5743));
    // The Hobgoblins' box stays hidden through the stop and the openings (from the first state's
    // redraw before the first step).
    let boxless = |ms| t.at(ms).get(&2).is_some_and(|p| p.boxless);
    assert!(boxless(5911) && boxless(6150) && !boxless(13233) && boxless(13234));
    // With the slips fixed the drawbridge the same (it has none; the wall shots' states show at
    // their explosions' frame 5 instead, T2b).
    let fixed = play_siege(&o, &commands, Bugs::Fixed, Some(&castle()));
    let bridge = |states: Vec<(u32, u32, u8)>| -> Vec<(u32, u32, u8)> {
        states
            .into_iter()
            .filter(|k| ![3275, 12634].contains(&k.0))
            .collect()
    };
    assert_eq!(bridge(gate_states(&fixed)), bridge(states));
    // The drawbridge's redraws show its rectangle alone (CLT-Q23): round 2's from its first
    // state's showing to the turn's first frame, the screen beyond it the one before (the
    // Hobgoblins' box still drawn); with the slip fixed the whole battle.
    assert!(t.bridges.contains(&BridgeKey {
        from: 13234,
        until: 13534
    }));
    assert_eq!(t.bridges.len(), 4);
    assert_eq!(
        [
            t.bridge(13233),
            t.bridge(13234),
            t.bridge(13533),
            t.bridge(13534)
        ],
        [None, Some(13233), Some(13233), None]
    );
    assert!(fixed.bridges.is_empty());
    // A moment in the span carries the battle the screen shows beyond the rectangle.
    let moment = crate::scene::Moment::at(&t, 13300);
    let before = crate::scene::Moment::at(&t, 13233);
    assert_eq!(moment.bridge.as_deref(), Some(&before));
    assert!(before.bridge.is_none());
}

#[test]
fn a_flyer_landing_on_the_gate_opens_it_before_its_flight() {
    // A defender's flyer whose destination is the gate's hex stands in a whole redraw shown at
    // once and the gate opens before its flight (0x4B4A95; *provisional*, not recorded).
    let mut o = gate_siege(0);
    o.stacks[2].flies = true;
    let commands = [(
        Some(0),
        vec![
            Event::Moved {
                stack: 2,
                from: 100,
                to: 96,
            },
            Event::GateOpened {},
            Event::TurnStarted { stack: 0 },
        ],
    )];
    let t = play_siege(&o, &commands, Bugs::Original, Some(&castle()));
    let states = gate_states(&t);
    assert_eq!(states[1..], [(0, 100, 3), (100, 200, 2), (200, 300, 1)]);
    // Its flight starts after the opening: group 20 drawn as the gate shows open.
    let first = t.keys.iter().find(|k| k.stack == 2 && k.pose.group == 20);
    assert_eq!(first.map(|k| k.drawn), Some(300));
    // It stood in the redraw before (shown at once, at 0).
    assert!(
        t.keys
            .iter()
            .any(|k| k.stack == 2 && k.shown == 0 && k.pose.group == 2)
    );
}

#[test]
fn the_gate_opened_or_closed_outside_a_move_plays_at_once() {
    // A gate opened with no move before it (a summoned elemental placed on it, 0x5A71F1) and
    // closed after an action: each the drawbridge's three states from the command's time.
    let o = gate_siege(0);
    let commands = [(Some(0), vec![Event::GateOpened {}, Event::GateClosed {}])];
    let t = play_siege(&o, &commands, Bugs::Original, Some(&castle()));
    let gates: Vec<u8> = gate_states(&t).iter().map(|s| s.2).collect();
    assert_eq!(gates, [3, 3, 2, 1, 1, 2, 3]);
    assert_eq!(gate_states(&t).last(), Some(&(500, 600, GATE_CLOSED)));
    // A destroyed gate (the gate's target at 0) never moves, nor a battle without a fort.
    let mut broken = gate_siege(0);
    if let Some(s) = broken.siege.as_mut() {
        s.parts[3] = 0;
    }
    let t = play_siege(&broken, &commands, Bugs::Original, Some(&castle()));
    assert_eq!(t.walls.len(), 1);
    assert!(t.cues.iter().all(|c| c.sound != Sound::Drawbridge));
    let mut field = gate_siege(0);
    field.siege = None;
    let t = play_siege(&field, &commands, Bugs::Original, None);
    assert!(t.walls.is_empty());
}

/// A Castle town's Citadel with its moat, the attacker's Pikemen (0) at 74 or a flyer there.
fn moat_siege(flies: bool) -> Observation {
    let mut pikemen = at(0, Side::Attacker, PIKEMAN, 74, false);
    pikemen.flies = flies;
    let mut o = snapshot(vec![pikemen, at(2, Side::Defender, HOBGOBLIN, 100, false)]);
    o.siege = Some(town(
        0,
        2,
        2,
        vec![11, 28, 44, 61, 77, 95, 111, 129, 146, 164, 181],
    ));
    o
}

#[test]
fn the_moat_that_kills_a_walker_ends_its_walk() {
    // The Pikemen (40) walk into the moat at 77 and die of its damage: the walk's end, the line,
    // their death without a striker, no turn back; the move sound not started again.
    let o = moat_siege(false);
    let moat = Event::MoatDamage {
        stack: 0,
        hex: 77,
        damage: h3_protocol::wire::WireU64(500),
        kills: 40,
    };
    let walked = vec![
        Event::Moved {
            stack: 0,
            from: 74,
            to: 75,
        },
        Event::Moved {
            stack: 0,
            from: 75,
            to: 76,
        },
        Event::Moved {
            stack: 0,
            from: 76,
            to: 77,
        },
        moat.clone(),
        Event::TurnStarted { stack: 2 },
    ];
    let t = play_siege(&o, &[(Some(0), walked)], Bugs::Original, Some(&castle()));
    let last = t.keys.iter().rfind(|k| k.stack == 0);
    assert_eq!(last.map(|k| (k.pose.group, k.pose.hex)), Some((5, 77)));
    assert!(
        t.log
            .iter()
            .any(|(_, l)| *l == Logged::Moat { hit: (0, 500, 40) })
    );
    let moves = t.cues.iter().filter(|c| c.sound == Sound::Move && !c.stop);
    assert_eq!(moves.count(), 1);
    // A flyer landing there dies alike (its landing's re-entry, 0x4B4A1E; *provisional*).
    let o = moat_siege(true);
    let flown = vec![
        Event::Moved {
            stack: 0,
            from: 74,
            to: 77,
        },
        moat,
        Event::TurnStarted { stack: 2 },
    ];
    let t = play_siege(
        &o,
        &[(Some(0), flown.clone())],
        Bugs::Original,
        Some(&castle()),
    );
    let last = t.keys.iter().rfind(|k| k.stack == 0);
    assert_eq!(last.map(|k| (k.pose.group, k.pose.hex)), Some((5, 77)));
    // One that lives reacts and stands there.
    let mut lived = flown;
    lived[1] = Event::MoatDamage {
        stack: 0,
        hex: 77,
        damage: h3_protocol::wire::WireU64(70),
        kills: 1,
    };
    let t = play_siege(&o, &[(Some(0), lived)], Bugs::Original, Some(&castle()));
    let last = t.keys.iter().rfind(|k| k.stack == 0);
    assert_eq!(last.map(|k| (k.pose.group, k.pose.hex)), Some((2, 77)));
    assert!(t.keys.iter().any(|k| k.stack == 0 && k.pose.group == 3));
}

#[test]
fn the_moat_sequence_plays_as_recorded() {
    // `moat`: a Castle town's Citadel; the Champions walk until their front enters the moat at
    // 77: the walk's end, then the damage, its line and their reaction (group 3); in round 2
    // they wait there: the re-entry's damage after the wait's line.
    let mut catapult = machine_at(1, Side::Attacker, CATAPULT, (119, 19));
    catapult.two_hex = true;
    let mut champions = at(0, Side::Attacker, CHAMPION, 86, true);
    champions.count = 5;
    let mut o = snapshot(vec![
        machine_at(3, Side::Defender, TURRET, (254, 20)),
        champions,
        catapult,
        at(2, Side::Defender, GOBLIN, 100, false),
    ]);
    o.siege = Some(town(
        0,
        2,
        2,
        vec![11, 28, 44, 61, 77, 95, 111, 129, 146, 164, 181],
    ));
    let turn = |stack| Event::TurnStarted { stack };
    let moat = |kills| Event::MoatDamage {
        stack: 0,
        hex: 77,
        damage: h3_protocol::wire::WireU64(70),
        kills,
    };
    let mut walked: Vec<Event> = [86, 69, 70, 71, 72, 73, 74, 75, 76]
        .windows(2)
        .map(|p| Event::Moved {
            stack: 0,
            from: p[0],
            to: p[1],
        })
        .collect();
    walked.extend([moat(0), turn(2)]);
    let commands = [
        (Some(0), vec![Event::Defended { stack: 3 }, turn(1)]),
        (Some(0), shot(1, 0)),
        (Some(3775), walked),
        (
            Some(9059),
            vec![
                Event::Defended { stack: 2 },
                Event::RoundStarted { round: 2 },
                turn(3),
            ],
        ),
        (Some(9059), vec![Event::Defended { stack: 3 }, turn(1)]),
        (Some(9059), shot(5, 0)),
        (
            Some(12934),
            vec![Event::Waited { stack: 0 }, moat(1), turn(2)],
        ),
    ];
    let t = play_siege(&o, &commands, Bugs::Original, Some(&castle()));
    assert_eq!(until(&t.lines(), 15934), recorded(MOAT, 15934));
    // The lines: the walk's damage after its end (8 159, the reaction's first frame), the
    // re-entry's after the wait's line.
    let moats: Vec<(u32, &Logged)> = t
        .log
        .iter()
        .filter(|(_, l)| matches!(l, Logged::Moat { .. }))
        .map(|(at, l)| (*at, l))
        .collect();
    assert_eq!(
        moats,
        [
            (8159, &Logged::Moat { hit: (0, 70, 0) }),
            (12934, &Logged::Moat { hit: (0, 70, 1) })
        ]
    );
    let waited = t
        .log
        .iter()
        .position(|(_, l)| matches!(l, Logged::Wait { .. }));
    let reentered = t
        .log
        .iter()
        .position(|(_, l)| *l == Logged::Moat { hit: (0, 70, 1) });
    assert!(waited < reentered);
}

/// Land Mine (spell 11 here as in the record): its object and animations (the asset tables').
fn mine_casts() -> Casts {
    let mut casts = Casts::default();
    casts.spells.insert(
        11,
        SpellLook {
            original: 11,
            animation: Some(47),
            objects: vec![ObjectLook {
                def: "C09spF1.def".to_string(),
                frames: 4,
                hexes: 1,
                rows: 1,
                before_stacks: true,
                placed: 47,
                vanish: 59,
            }],
            object_sound: true,
            ..SpellLook::default()
        },
    );
    casts
}

/// The timeline of `snapshot` and `commands` at a Tower with Land Mine's casts and the blast
/// (animation 57, `C09spF3.def`'s 17 frames).
fn play_mine(snapshot: &Observation, commands: &[(Option<u32>, Vec<Event>)]) -> Timeline {
    let layout = layout();
    let magic = BTreeMap::from([(
        MINE_BLAST,
        MagicLook {
            def: "C09spF3.def".to_string(),
            frames: 17,
            size: [140, 153],
            place: 0,
            translucent: false,
        },
    )]);
    let casts = mine_casts();
    let castle = castle();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic: &magic,
        casts: &casts,
        bugs: Bugs::Original,
        siege: Some(&castle),
    };
    let commands: Vec<Command<'_>> = commands
        .iter()
        .map(|(at, events)| Command { at: *at, events })
        .collect();
    timeline(snapshot, &commands, &siege_looks(), stage, Speed::default()).unwrap()
}

/// `mine`'s siege (`seq-mine.toml`): a Tower town's Citadel (its mines hidden from the
/// spectator), the attacker's Titans, Pikemen and Catapult, the defender's Goblins and keep.
fn mine_siege() -> Observation {
    let mut catapult = machine_at(2, Side::Attacker, CATAPULT, (119, 19));
    catapult.two_hex = true;
    let mut titans = at(0, Side::Attacker, TITAN, 35, false);
    titans.count = 3;
    let mut o = snapshot(vec![
        machine_at(4, Side::Defender, TURRET, (254, 20)),
        titans,
        at(1, Side::Attacker, PIKEMAN, 137, false),
        catapult,
        at(3, Side::Defender, GOBLIN, 100, false),
    ]);
    o.siege = Some(town(2, 2, 2, Vec::new()));
    o
}

/// The Titans' walk from 35 into the town through the mid-upper wall's breach, over the mine at
/// 61: its blast's events after the step onto it.
fn over_the_mine() -> Vec<Event> {
    let step = |from, to| Event::Moved { stack: 0, from, to };
    let mut events: Vec<Event> = [35, 36, 37, 38, 39, 40, 41, 42, 43, 61]
        .windows(2)
        .map(|p| step(p[0], p[1]))
        .collect();
    events.extend([
        Event::ObstacleRevealed {
            side: Side::Defender,
            spell: 11,
            hexes: vec![61],
        },
        Event::ObstacleTriggered {
            stack: 0,
            spell: 11,
            hex: 61,
        },
        Event::SpellDamage {
            target: 0,
            value: h3_protocol::wire::WireU64(150),
            caster: h3_protocol::wire::WireU64(150),
            damage: h3_protocol::wire::WireU64(150),
            kills: 0,
        },
        Event::ObstacleRemoved {
            spell: 11,
            hexes: vec![61],
        },
        step(61, 78),
        step(78, 79),
        Event::TurnStarted { stack: 3 },
    ]);
    events
}

#[test]
fn the_mine_sequence_plays_as_recorded() {
    // `mine`: the keep defends, the Catapult breaks the mid-upper wall; the Titans walk over the
    // mine at 61: their reaction under animation 57 (held at its last frame until the iterations
    // left are no more than its frames), the walk going on with the moving group.
    let o = mine_siege();
    let turn = |stack| Event::TurnStarted { stack };
    let commands = [
        (Some(0), vec![Event::Defended { stack: 4 }, turn(2)]),
        (
            Some(0),
            vec![
                Event::WallShot {
                    stack: 2,
                    aimed: 2,
                    part: 2,
                    damage: 2,
                },
                Event::WallDestroyed { part: 2 },
                turn(0),
            ],
        ),
        (Some(3375), over_the_mine()),
    ];
    let t = play_mine(&o, &commands);
    assert_eq!(until(&t.lines(), 12875), recorded(MINE, 12875));
    // The line named by Land Mine before the blast; its sound, the move sound stopped first and
    // started again after it.
    assert!(t.log.iter().any(|(at, l)| *at == 9655
        && *l
            == Logged::SpellDamage {
                spell: 11,
                hit: (0, 150, 0)
            }));
    let at = |sound: Sound, stop: bool| -> Vec<u32> {
        t.cues
            .iter()
            .filter(|c| c.stack == 0 && c.sound == sound && c.stop == stop)
            .map(|c| c.at)
            .collect()
    };
    assert_eq!(at(Sound::Mine, false), [9655]);
    assert!(at(Sound::Move, true).contains(&9655));
    assert!(at(Sound::Move, false).contains(&11331));
}

#[test]
fn a_mine_the_screen_shows_goes_without_its_vanish_animation() {
    // The defender's view shows its mines: the blast removes the one at 61 from the next frame
    // on, without its vanish animation (59).
    let o = mine_siege();
    let commands = [
        (
            Some(0),
            vec![Event::ObstaclePlaced {
                side: Side::Defender,
                spell: 11,
                hexes: vec![61],
            }],
        ),
        (Some(100), over_the_mine()),
    ];
    let t = play_mine(&o, &commands);
    assert!(t.objects(200).contains_key(&61));
    assert!(!t.objects(10_000).contains_key(&61));
    assert!(t.spots.iter().all(|s| s.animation != 59));
}

#[test]
fn a_mine_at_the_walk_s_end_blasts_after_it() {
    // The Titans' walk ending on the mine at 61 (no breach): the walk's end first, then the
    // blast, the move sound neither stopped again nor started.
    let o = mine_siege();
    let mut walked = over_the_mine();
    walked.retain(|e| !matches!(e, Event::Moved { from: 61 | 78, .. }));
    let t = play_mine(&o, &[(Some(0), walked)]);
    let blast = t.cues.iter().find(|c| c.sound == Sound::Mine).map(|c| c.at);
    let moves: Vec<(u32, bool)> = t
        .cues
        .iter()
        .filter(|c| c.stack == 0 && c.sound == Sound::Move)
        .map(|c| (c.at, c.stop))
        .collect();
    assert_eq!(moves.len(), 2);
    assert!(moves.iter().all(|&(at, _)| Some(at) < blast));
    assert!(t.magics.iter().any(|m| m.animation == MINE_BLAST));
    let last = t.keys.iter().rfind(|k| k.stack == 0);
    assert_eq!(last.map(|k| (k.pose.group, k.pose.hex)), Some((2, 61)));
}

#[test]
fn every_event_a_hex_entered_brings_is_played() {
    // E17 M4 T3a, R1-1: a mine on a moat hex blasts and then the moat damages (the engine's
    // `enter_hexes`), and a two-hex walker's two mines both blast: each entry of the hex is
    // played, in order.
    let mut o = mine_siege();
    o.siege = Some(town(
        0,
        2,
        2,
        vec![11, 28, 44, 61, 77, 95, 111, 129, 146, 164, 181],
    ));
    let mut walked = over_the_mine();
    walked.retain(|e| !matches!(e, Event::Moved { from: 61 | 78, .. }));
    let turn = walked.pop();
    walked.push(Event::MoatDamage {
        stack: 0,
        hex: 61,
        damage: h3_protocol::wire::WireU64(70),
        kills: 0,
    });
    walked.extend(turn);
    let t = play_mine(&o, &[(Some(0), walked)]);
    let lines: Vec<&Logged> = t
        .log
        .iter()
        .filter(|(_, l)| matches!(l, Logged::Moat { .. } | Logged::SpellDamage { .. }))
        .map(|(_, l)| l)
        .collect();
    assert_eq!(
        lines,
        [
            &Logged::SpellDamage {
                spell: 11,
                hit: (0, 150, 0)
            },
            &Logged::Moat { hit: (0, 70, 0) }
        ]
    );
    // Two mines under the walk's end.
    let mut walked = over_the_mine();
    walked.retain(|e| !matches!(e, Event::Moved { from: 61 | 78, .. }));
    let turn = walked.pop();
    let blast: Vec<Event> = walked[walked.len() - 4..]
        .iter()
        .map(|e| match e.clone() {
            Event::ObstacleRevealed { side, spell, .. } => Event::ObstacleRevealed {
                side,
                spell,
                hexes: vec![60],
            },
            Event::ObstacleTriggered { stack, spell, .. } => Event::ObstacleTriggered {
                stack,
                spell,
                hex: 60,
            },
            Event::ObstacleRemoved { spell, .. } => Event::ObstacleRemoved {
                spell,
                hexes: vec![60],
            },
            e => e,
        })
        .collect();
    walked.extend(blast);
    walked.extend(turn);
    let t = play_mine(&mine_siege(), &[(Some(0), walked)]);
    let blasts = t.cues.iter().filter(|c| c.sound == Sound::Mine).count();
    assert_eq!(blasts, 2);
    let lines = t
        .log
        .iter()
        .filter(|(_, l)| matches!(l, Logged::SpellDamage { spell: 11, .. }))
        .count();
    assert_eq!(lines, 2);
}
