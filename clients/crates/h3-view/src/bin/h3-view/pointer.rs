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

/// The screen's centre of the spell book's place `k` (`h3drive.py`'s `book_place`; E17 M3
/// T7a-2a, design §7.16): twelve places of 78 × 65, row by row, the left page first, from the
/// dialog's (117, 89) and (333, 89) with the dialog at (90, 2). *Provisional:* constants until
/// the book has a layout of its own.
pub fn book_place(k: u8) -> [i32; 2] {
    let k = i32::from(k);
    [246 + 86 * (k % 2) + 216 * (k / 6), 123 + 96 * (k % 6 / 2)]
}

/// The spell book's places.
const BOOK_PLACES: u8 = 12;

/// The mouse's inputs for the harness's `actions` (`h3drive.py`'s `act`), the first acting `pre`
/// ms after the opening, the mouse parked before the battle: `hex:<n>` and `click:<x>,<y>` move
/// the mouse there, click (the command's action starts) and park it again; `move:<x>,<y>` moves
/// it; `key:<k>` gives a command (right after a cast, at the cast's end: E17 M3 T7b-1);
/// `cast:<k>[:<hex>]` (E17 M3 T7a-2a) moves it to the spell book's place `k` (the book opened by
/// its key), aims the next of the record's casts `spells` (T7b-1) and moves to the target hex, if
/// any, where the click starts the cast, and parks it; `at:<ms>` acts then,
/// `turn:<side>.<record>` when that stack's next turn starts (the timeline's, after the turn the
/// last `turn:` named; a first one naming the opening's active stack at once, T7b-1),
/// `step:<ms>` that much later; `roll:<name>[:fail]` (E17 M3 T7b-2b-1) forces the original's
/// next draw of a roll and moves nothing (the engine's rolls are its seed's). *Not modelled:* the
/// book over the battlefield.
pub fn inputs(
    actions: &[String],
    pre: u32,
    layout: &BattleLayout,
    (timeline, spells): (&Timeline, &[u16]),
) -> Result<Vec<Input>, String> {
    let mut spells = spells.iter().copied();
    let mut cast_pending = false;
    let mut out = vec![Input::Move { at: 0, to: PARK }];
    let (mut now, mut turns) = (pre, 0);
    // The stack whose turn the battle opens with (E17 M3 T7b-1): a first `turn:` naming it acts
    // at once, as the harness's does (the game names it the active stack already).
    let mut opening = timeline.outlines.first().and_then(|(_, stack)| *stack);
    let point = |arg: &str| -> Result<[i32; 2], String> {
        let (x, y) = arg
            .split_once(',')
            .ok_or_else(|| format!("`{arg}`: not x,y"))?;
        let parse = |v: &str| v.trim().parse::<i32>().map_err(|e| format!("`{arg}`: {e}"));
        Ok([parse(x)?, parse(y)?])
    };
    for action in actions {
        let (kind, arg) = action.split_once(':').unwrap_or((action, ""));
        let after_cast = core::mem::take(&mut cast_pending);
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
            "key" => {
                // A key pressed at the frozen clock right after a hero's cast acts once the cast
                // has played (the harness's runbook; E17 M3 T7b-1): at the end of the action the
                // cast started then.
                if after_cast {
                    let end = timeline.actions.iter().filter(|a| a.start == now);
                    now = end.map(|a| a.end).max().unwrap_or(now);
                }
                out.push(Input::Act { at: now });
            }
            "cast" => {
                cast_pending = true;
                let spell = spells.next().unwrap_or(u16::MAX);
                let aimed =
                    cast(arg, (now, spell), layout).map_err(|e| format!("`{action}`: {e}"))?;
                out.extend(aimed);
            }
            "at" => now = arg.parse().map_err(|e| format!("`{action}`: {e}"))?,
            "step" => {
                now += arg.parse::<u32>().map_err(|e| format!("`{action}`: {e}"))?;
            }
            // The harness's forced roll (E17 M3 T7b-2b-1): no input; a key after a cast keeps
            // acting at the cast's end.
            "roll" => cast_pending = after_cast,
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
                if opening.take() == Some(stack) && turns == 0 {
                    continue;
                }
                let i = (turns..timeline.turns.len())
                    .find(|&i| timeline.turns[i].1 == stack && timeline.turns[i].0 >= now)
                    .ok_or_else(|| format!("`{action}`: no such turn after {now} ms"))?;
                (now, turns) = (timeline.turns[i].0, i + 1);
            }
            _ => return Err(format!("`{action}`: an unknown action")),
        }
    }
    Ok(out)
}

/// The mouse's inputs of a hero's cast `<k>[:<hex>]` of `spell` at `now` (`inputs`): the book's
/// place, the spell aimed (E17 M3 T7b-1), the target; without one the book's empty status line
/// (E17 M3 T7c-1).
fn cast(arg: &str, (now, spell): (u32, u16), layout: &BattleLayout) -> Result<Vec<Input>, String> {
    let (place, target) = arg
        .split_once(':')
        .map_or((arg, None), |(p, t)| (p, Some(t)));
    let place = place
        .parse::<u8>()
        .ok()
        .filter(|&k| k < BOOK_PLACES)
        .ok_or("the book has the places 0 to 11")?;
    let mut out = vec![
        Input::Move {
            at: now,
            to: book_place(place),
        },
        Input::Aim { at: now, spell },
    ];
    if let Some(target) = target {
        let hex = target.parse::<u8>().map_err(|e| e.to_string())?;
        out.push(Input::Move {
            at: now,
            to: hex_point(layout, hex),
        });
    } else {
        // Cast from the book at once (E17 M3 T7c-1): the status line as the book left it.
        out.insert(1, Input::Book { at: now });
    }
    out.extend([Input::Act { at: now }, Input::Move { at: now, to: PARK }]);
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
    let spells = &record.header.manifest.keys.spells;
    let spell = |handle: u16| {
        let key = spells.get(usize::from(handle))?;
        Some(format!("spell.{key}"))
    };
    let name = |hint: &Hint, observation: &Observation| {
        hint.text(catalogue, observation, (&creature, &spell))
    };
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

#[cfg(test)]
mod tests {
    use super::*;
    use h3_view::layout::parse_battle;

    fn layout() -> BattleLayout {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../layouts/battle.toml");
        parse_battle(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn a_cast_moves_through_the_book_to_its_target() {
        // E17 M3 T7a-2a: the places' centres, the left page first (design §7.16).
        assert_eq!(book_place(0), [246, 123]);
        assert_eq!(book_place(5), [332, 315]);
        assert_eq!(book_place(6), [462, 123]);
        assert_eq!(book_place(11), [548, 315]);
        let layout = layout();
        assert_eq!(
            cast("3:49", (7, 54), &layout).unwrap(),
            vec![
                Input::Move {
                    at: 7,
                    to: [332, 219]
                },
                Input::Aim { at: 7, spell: 54 },
                Input::Move {
                    at: 7,
                    to: hex_point(&layout, 49)
                },
                Input::Act { at: 7 },
                Input::Move { at: 7, to: PARK },
            ]
        );
        // A spell without a target is cast at its place.
        assert_eq!(cast("0", (0, 54), &layout).unwrap().len(), 4);
        for bad in ["12", "x", "0:x", "-1:4"] {
            assert!(cast(bad, (0, 54), &layout).is_err(), "{bad}");
        }
    }
}
