//! E17 M4 T2c-1: what a walk meets in a siege (design §10.2): the drawbridge opening before the
//! step onto the gate's hexes and closing after the walk, a Land Mine's blast on the way, the
//! moat's damage at the walk's end and as a stack acts again in it; replayed from the recorded
//! sequences' events with their creatures' numbers (the asset tables' times, the DEFs' frame
//! counts, read from the user's archives: numbers only, ADR-0009).

use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::{Event, Observation, Side, Siege};
use h3_render::Bugs;

use super::siege::{CATAPULT, HOBGOBLIN, PIKEMAN, castle, machine_at, play_siege, recorded};
use super::{at, snapshot, until};
use crate::anim::{Sound, Timeline};
use crate::siege::{GATE_CLOSED, GATE_MOVING, GATE_OPEN};

const GATE: &str = include_str!("../../../../../situations/timelines/gate.txt");

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
}
