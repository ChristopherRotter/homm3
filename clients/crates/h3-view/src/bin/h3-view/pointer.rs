//! The mouse over a record's battle in the adapters (E17 M3 T6c-2; `docs/epics/E17-design.md`
//! §9.10): the capture harness's actions as the mouse's inputs, each turn's battle from the
//! engine, and what the mouse shows through the battle (`h3_view::mouse::track`).

use std::collections::BTreeSet;

use h3_protocol::wire::{Command, Observation, Side, Stack};
use h3_view::anim::Timeline;
use h3_view::layout::BattleLayout;
use h3_view::mouse::track::{self, Input, Shown, Start, Turn};
use h3_view::mouse::{Facing, Field, Hint, Legal};
use h3_view::record::Record;
use h3_view::text::Catalogue;

/// Where the harness parks the mouse (`h3drive.py`'s `PARK`): in the panel beside the log's item,
/// over nothing that has a hint.
pub const PARK: [i32; 2] = [618, 593];

/// The point of `hex` the harness clicks (`h3drive.py`'s `hex_point`): 22 right of its box's left
/// edge and 26 below its top.
pub fn hex_point(layout: &BattleLayout, hex: u8) -> [i32; 2] {
    let (row, column) = (i32::from(hex / 17), i32::from(hex % 17));
    let shift = if row % 2 == 0 {
        layout.even_row_shift
    } else {
        0
    };
    [
        layout.hex_origin[0] + layout.hex_step[0] * column + shift + 22,
        layout.hex_origin[1] + layout.hex_step[1] * row + 26,
    ]
}

/// The mouse's inputs for the harness's `actions` (`h3drive.py`'s `act`), the first acting `pre`
/// ms after the opening, the mouse parked before the battle: `hex:<n>` and `click:<x>,<y>` move
/// the mouse there, click (the command's action starts) and park it again; `move:<x>,<y>` moves
/// it; `key:<k>` gives a command; `at:<ms>` acts then, `turn:<side>.<record>` when that stack's
/// next turn starts (the timeline's), `step:<ms>` that much later.
pub fn inputs(
    actions: &[String],
    pre: u32,
    layout: &BattleLayout,
    timeline: &Timeline,
) -> Result<Vec<Input>, String> {
    let mut out = vec![Input::Move { at: 0, to: PARK }];
    let mut now = pre;
    let point = |arg: &str| -> Result<[i32; 2], String> {
        let (x, y) = arg
            .split_once(',')
            .ok_or_else(|| format!("`{arg}`: not x,y"))?;
        let parse = |v: &str| v.trim().parse::<i32>().map_err(|e| format!("`{arg}`: {e}"));
        Ok([parse(x)?, parse(y)?])
    };
    for action in actions {
        let (kind, arg) = action.split_once(':').unwrap_or((action, ""));
        match kind {
            "hex" | "click" => {
                let to = if kind == "hex" {
                    let hex = arg.parse::<u8>().map_err(|e| format!("`{action}`: {e}"))?;
                    hex_point(layout, hex)
                } else {
                    point(arg)?
                };
                out.push(Input::Move { at: now, to });
                out.push(Input::Act { at: now });
                out.push(Input::Move { at: now, to: PARK });
            }
            "move" => out.push(Input::Move {
                at: now,
                to: point(arg)?,
            }),
            "key" => out.push(Input::Act { at: now }),
            "at" => now = arg.parse().map_err(|e| format!("`{action}`: {e}"))?,
            "step" => {
                now += arg.parse::<u32>().map_err(|e| format!("`{action}`: {e}"))?;
            }
            "turn" => {
                let (side, record) = arg
                    .split_once('.')
                    .ok_or_else(|| format!("`{action}`: not <side>.<record>"))?;
                let side = if side == "0" {
                    Side::Attacker
                } else {
                    Side::Defender
                };
                let record: u8 = record.parse().map_err(|e| format!("`{action}`: {e}"))?;
                let stack = timeline
                    .members
                    .iter()
                    .find(|m| m.side == side && m.record == record)
                    .map(|m| m.stack)
                    .ok_or_else(|| format!("`{action}`: no such stack"))?;
                now = timeline
                    .turns
                    .iter()
                    .find(|&&(t, s)| s == stack && t >= now)
                    .map(|&(t, _)| t)
                    .ok_or_else(|| format!("`{action}`: no such turn after {now} ms"))?;
            }
            _ => return Err(format!("`{action}`: an unknown action")),
        }
    }
    Ok(out)
}

/// What the mouse shows over the record's battle as `timeline` plays it: `turned` the battle and
/// its legal commands before each command and after the last (`scenario::turned`; none for a
/// record without its scenario: the opening's battle then stands for every turn, with no legal
/// commands), `inputs` what the mouse does; its hints in `catalogue`'s words.
pub fn shown(
    record: &Record,
    timeline: &Timeline,
    turned: &[(Observation, Vec<Command>)],
    inputs: &[Input],
    layout: &BattleLayout,
    catalogue: &Catalogue,
) -> Shown {
    // The turns: the opening's, then each one the log starts, paired with the engine's battle
    // whose acting stack it is.
    let snapshot = record.first_snapshot();
    let mut times = vec![(0, snapshot.and_then(|o| o.active))];
    times.extend(timeline.log.iter().filter_map(|(t, l)| match l {
        h3_view::battle_log::Logged::Turn(stack) => Some((*t, Some(*stack))),
        _ => None,
    }));
    let mut battles: Vec<(Observation, Legal, Facing)> = Vec::new();
    let mut next = 0;
    for &(t, stack) in &times {
        let found = (next..turned.len()).find(|&i| turned[i].0.active == stack);
        let (observation, legal) = match found {
            Some(i) => {
                next = i + 1;
                (turned[i].0.clone(), Legal::new(&turned[i].1))
            }
            None => {
                let Some(mut observation) = snapshot.cloned() else {
                    continue;
                };
                observation.active = stack;
                (observation, Legal::default())
            }
        };
        let left: BTreeSet<u8> = timeline
            .at(t)
            .into_iter()
            .filter(|(_, p)| p.mirrored)
            .map(|(id, _)| id)
            .collect();
        battles.push((observation, legal, Facing::Left(left)));
    }
    let turns: Vec<Turn<'_>> = battles
        .iter()
        .map(|(observation, legal, facing)| Turn {
            field: Field {
                observation,
                legal,
                facing,
            },
            human: human(observation),
        })
        .collect();
    let starts: Vec<Start> = timeline
        .actions
        .iter()
        .map(|a| Start { unmarks: a.unmarks })
        .collect();
    let keys = &record.header.manifest.keys.creatures;
    let creature = |s: &Stack| {
        let key = keys.get(usize::from(s.creature))?;
        Some(format!("creature.{key}"))
    };
    let name =
        |hint: &Hint, observation: &Observation| hint.text(catalogue, observation, &creature);
    track::follow(&timeline.log, &starts, &turns, inputs, layout, &name)
}

/// The timeline with the stacks the mouse entered playing their group 1 ([`Timeline::enter`]): each
/// stack's frames in it by its creature's look.
pub fn entered(
    timeline: &mut Timeline,
    shown: &Shown,
    record: &Record,
    looks: &std::collections::BTreeMap<u16, h3_view::anim::Look>,
    group: u8,
) {
    let Some(snapshot) = record.first_snapshot() else {
        return;
    };
    let frames = |id: u8| {
        snapshot
            .stacks
            .iter()
            .find(|s| s.id == id)
            .and_then(|s| looks.get(&s.creature))
            .and_then(|l| l.frames.get(&u32::from(group)).copied())
            .unwrap_or(0)
    };
    timeline.enter(&shown.entered, u32::from(group), &frames);
}

/// Whether a human on this computer plays the acting stack's turn: its controller's side's
/// `human`.
fn human(observation: &Observation) -> bool {
    let Some(stack) = observation
        .active
        .and_then(|id| observation.stacks.iter().find(|s| s.id == id))
    else {
        return false;
    };
    let side = usize::from(stack.controller == Side::Defender);
    observation.sides.get(side).is_some_and(|s| s.human)
}
