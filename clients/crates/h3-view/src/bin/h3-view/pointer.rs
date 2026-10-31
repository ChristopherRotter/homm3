//! The mouse over a record's battle in the adapters (E17 M3 T6c-2; `docs/epics/E17-design.md`
//! §9.10): the capture harness's actions as the mouse's inputs, each turn's battle from the
//! engine, and what the mouse shows through the battle (`h3_view::mouse::track`).

use std::collections::BTreeSet;

use h3_protocol::wire::{Command, Manifest, Observation, Side, Stack};
use h3_render::Bugs;
use h3_ui::text::Catalogue;
use h3_view::anim::Timeline;
use h3_view::auto::{Auto, turn_at};
use h3_view::layout::BattleLayout;
use h3_view::mouse::aim::Aim;
use h3_view::mouse::track::{self, Input, Lens, Shown, Start, Turn};
use h3_view::mouse::{self, CATAPULT, Facing, Field, Hint, Legal, Spot, WALL_EXEMPT};
use h3_view::record::Record;
use h3_view::tables::Tables;

use crate::scenario::Turned;

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

/// A point of `hex` in each of its twelve slots, outside the siege's rectangles where the slot
/// has one there (E17 M5 T2a's `clicks` test).
fn slot_points(layout: &BattleLayout, hex: u8) -> Vec<[i32; 2]> {
    let [left, top] = layout.hex_box(hex / 17, hex % 17);
    let [w, h] = layout.hex_step;
    let within =
        |[l, t, r, b]: [i32; 4], [x, y]: [i32; 2]| (l..=r).contains(&x) && (t..=b).contains(&y);
    let free = |p: [i32; 2]| !within(layout.mouse.keep, p) && !within(layout.mouse.tower, p);
    let inside: Vec<[i32; 2]> = (top - h..top + 2 * h)
        .flat_map(|y| (left..left + w).map(move |x| [x, y]))
        .filter(|&p| mouse::spot(layout, p, false) == Spot::Hex(hex))
        .collect();
    (0..12)
        .filter_map(|s| {
            let in_slot = |p: &&[i32; 2]| mouse::slot(layout, **p, hex) == s;
            let mut all = inside.iter().filter(in_slot);
            let first = all.clone().next().copied();
            all.find(|p| free(**p)).copied().or(first)
        })
        .collect()
}

/// Each hex's centre (the harness's point, [`hex_point`]) and a point in each of its slots: where
/// a press gives each of a turn's commands on the battlefield (E17 M5 T2a's `clicks` test; T2b's
/// unattended presses in the window).
pub fn press_points(layout: &BattleLayout) -> Vec<([i32; 2], Vec<[i32; 2]>)> {
    (0..187u8)
        .map(|hex| (hex_point(layout, hex), slot_points(layout, hex)))
        .collect()
}

/// A rectangle's centre.
pub fn middle([l, t, r, b]: [i32; 4]) -> [i32; 2] {
    [(l + r) / 2, (t + b) / 2]
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
/// next draw of a roll and moves nothing (the engine's rolls are its seed's); `aim:<k>:<n>` (E17
/// M5 T3b) opens the book and clicks its place `k`, whose spell numbered `n` in the original
/// (`handle` gives its handle) takes a target: the aim starts with the mouse there; `key:Escape`
/// gives an aim up. E17 M5 T4c1 (design §7.22): Auto Combat's key (`key:a`, the original's
/// table) switches it on ([`Input::Auto`]); while it plays, a `key:`, `hex:` or `click:` stops it
/// ([`Input::Stop`]: the click moves the mouse and gives nothing); each in the turn started last
/// by its time ([`turn_at`]; the switch trusted to come on a human's waiting turn, as the
/// harness pressed it, T8b2 R5c-8). E17 M5 T4c2b: a battle that
/// opens with the tactics phase's message (`told`) takes its first `key:Return` as the box's OK,
/// which gives nothing (the first turn starts); the phase's `key:space` and `key:Return` are its
/// panel's Next Creature and Start Combat. *Not modelled:* the book over the battlefield, the
/// mouse moved over the phase's message.
pub fn inputs(
    actions: &[String],
    pre: u32,
    layout: &BattleLayout,
    (timeline, spells, handle): (&Timeline, &[u16], &dyn Fn(u16) -> Option<u16>),
    told: bool,
) -> Result<Vec<Input>, String> {
    let mut told = told;
    let mut spells = spells.iter().copied();
    let (mut cast_pending, mut auto) = (false, false);
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
                out.push(if core::mem::take(&mut auto) {
                    Input::Stop {
                        at: now,
                        turn: turn_at(&timeline.turns, now),
                    }
                } else {
                    Input::Act { at: now }
                });
                out.push(Input::Move { at: now, to: PARK });
            }
            // The tactics phase's message closed by its OK (E17 M5 T4c2b).
            "key" if arg == "Return" && core::mem::take(&mut told) => {}
            // A key while Auto Combat plays stops it (E17 M5 T4c1).
            "key" if core::mem::take(&mut auto) => out.push(Input::Stop {
                at: now,
                turn: turn_at(&timeline.turns, now),
            }),
            "key" if auto_combat(arg) => {
                auto = true;
                out.push(Input::Auto {
                    at: now,
                    turn: turn_at(&timeline.turns, now),
                });
            }
            "move" => out.push(Input::Move {
                at: now,
                to: point(arg)?,
            }),
            // Escape gives an aim up (E17 M5 T3b, 0x5A317A).
            "key" if arg == "Escape" => out.push(Input::Cancel { at: now }),
            "aim" => {
                let (place, original) = arg
                    .split_once(':')
                    .ok_or_else(|| format!("`{action}`: not aim:<place>:<spell>"))?;
                let original = original
                    .parse::<u16>()
                    .map_err(|e| format!("`{action}`: {e}"))?;
                let spell = handle(original)
                    .ok_or_else(|| format!("`{action}`: no spell numbered {original}"))?;
                let place = place_of(place).map_err(|e| format!("`{action}`: {e}"))?;
                // The aim goes on: no click yet, the mouse left where the place was.
                out.extend([
                    Input::Book { at: now },
                    Input::Move {
                        at: now,
                        to: book_place(place),
                    },
                    Input::Aim {
                        at: now,
                        spell,
                        creature: false,
                    },
                ]);
            }
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
                    .member_at(side, record)
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

/// Whether the harness's key `name` (xdotool's: `a`, `space`) is Auto Combat's in the original's
/// table (E17 M5 T4c1).
fn auto_combat(name: &str) -> bool {
    let mut chars = name.chars();
    let key: String = chars
        .next()
        .map(|c| c.to_ascii_uppercase())
        .into_iter()
        .chain(chars)
        .collect();
    h3_view::keys::Keys::original().command(h3_view::keys::Context::Battle, false, &key)
        == Some(h3_view::keys::Command::Button(h3_view::panel::AUTO_COMBAT))
}

/// The mouse's inputs of a hero's cast `<k>[:<hex>]` of `spell` at `now` (`inputs`): the book
/// opened, its place, the spell aimed (E17 M3 T7b-1) and the target; without one cast from the
/// book at once (E17 M3 T7c-1). The book takes the mouse's moves until it closes (E17 M5 T3b).
fn cast(arg: &str, (now, spell): (u32, u16), layout: &BattleLayout) -> Result<Vec<Input>, String> {
    let (place, target) = arg
        .split_once(':')
        .map_or((arg, None), |(p, t)| (p, Some(t)));
    let place = place_of(place)?;
    let mut out = vec![
        Input::Book { at: now },
        Input::Move {
            at: now,
            to: book_place(place),
        },
    ];
    if let Some(target) = target {
        let hex = target.parse::<u8>().map_err(|e| e.to_string())?;
        out.extend([
            Input::Aim {
                at: now,
                spell,
                creature: false,
            },
            Input::Move {
                at: now,
                to: hex_point(layout, hex),
            },
        ]);
    }
    out.extend([Input::Act { at: now }, Input::Move { at: now, to: PARK }]);
    Ok(out)
}

/// The spell book's place `arg` (0 to 11).
fn place_of(arg: &str) -> Result<u8, String> {
    arg.parse::<u8>()
        .ok()
        .filter(|&k| k < BOOK_PLACES)
        .ok_or_else(|| "the book has the places 0 to 11".to_owned())
}

/// The acting stack's legal commands `commands` in `observation` as the cursor asks them, with
/// what its own tests need of the stack (E17 M4 T2c-2): the Catapult, whether the walls halve its
/// shots (its creature's original id by the record's `manifest` and the asset tables `tables`, its
/// hero's bow by the wire's `no_distance_penalty`).
pub fn legal(
    manifest: &Manifest,
    tables: &Tables,
    observation: &Observation,
    (commands, aims): (&[Command], &[Aim]),
) -> Legal {
    let mut legal = Legal::new(commands);
    // The spells the side may aim, with the original's numbers (E17 M5 T3b).
    legal.aims = aims
        .iter()
        .map(|a| {
            let mut aim = a.clone();
            crate::aims::name(&mut aim, &manifest.keys.spells, tables);
            aim
        })
        .collect();
    let acting = observation
        .active
        .and_then(|id| observation.stacks.iter().find(|s| s.id == id));
    if let Some(s) = acting {
        let original = manifest
            .keys
            .creatures
            .get(usize::from(s.creature))
            .and_then(|key| tables.creatures.get(key))
            .map(|c| c.original);
        legal.catapult = original == Some(CATAPULT);
        legal.wall_exempt =
            s.no_distance_penalty || original.is_some_and(|o| WALL_EXEMPT.contains(&o));
    }
    legal
}

/// What the mouse shows over the record's battle as `timeline` plays it: `turned` the battle and
/// its legal commands before each command and after the last (`scenario::turned`; none for a
/// record without its scenario: the opening's battle then stands for every turn, with no legal
/// commands), `inputs` what the mouse does; its hints in `catalogue`'s words; the acting stacks'
/// creatures by `tables` ([`legal`]), the profile `bugs` (the siege's rectangles, CLT-Q21).
pub fn shown(
    record: &Record,
    timeline: &Timeline,
    turned: &[Turned],
    inputs: &[Input],
    (layout, catalogue): (&BattleLayout, &Catalogue),
    (tables, bugs): (&Tables, Bugs),
) -> Shown {
    let name =
        |hint: &Hint, observation: &Observation| hint_text(record, catalogue, hint, observation);
    followed(
        record,
        (timeline, turned),
        inputs,
        (layout, &name),
        (tables, bugs),
    )
}

/// A hint's words.
type Name<'a> = &'a dyn Fn(&Hint, &Observation) -> String;

/// [`shown`] with the hints in `name`'s words: the battle followed from its opening, every turn
/// read again (what [`Follower`] goes on from without).
fn followed(
    record: &Record,
    (timeline, turned): (&Timeline, &[Turned]),
    inputs: &[Input],
    (layout, name): (&BattleLayout, Name<'_>),
    (tables, bugs): (&Tables, Bugs),
) -> Shown {
    // The turns: the opening's, then each one the log starts, paired with the engine's battle
    // whose acting stack it is.
    let times = times(record, timeline);
    let mut battles: Vec<(Observation, Legal, Facing)> = Vec::new();
    let mut next = 0;
    for &(t, stack) in &times {
        let found = pair(turned, &mut next, stack);
        let Some((observation, legal)) = battle_of(record, tables, turned, (found, stack)) else {
            continue;
        };
        battles.push((observation, legal, facing(timeline, t)));
    }
    // A turn Auto Combat plays from its start is the computer's (E17 M5 T4c1).
    let starts: Vec<u32> = times.iter().map(|&(t, _)| t).collect();
    let auto = Auto::of(inputs, &starts);
    let turns: Vec<Turn<'_>> = battles
        .iter()
        .enumerate()
        .map(|(k, (observation, legal, facing))| Turn {
            field: Field {
                observation,
                legal,
                facing,
                bugs,
            },
            human: human(observation) && !auto.from_start(k),
        })
        .collect();
    track::follow(
        &timeline.log,
        &actions(timeline),
        &turns,
        inputs,
        layout,
        name,
    )
}

/// The turns the mouse reads in `timeline`: the opening's (its acting stack the record's
/// snapshot's), then each one the log starts, by its start and its acting stack.
fn times(record: &Record, timeline: &Timeline) -> Vec<(u32, Option<u8>)> {
    let snapshot = record.first_snapshot();
    let mut times = vec![(0, snapshot.and_then(|o| o.active))];
    times.extend(timeline.log.iter().filter_map(|(t, l)| match l {
        h3_view::battle_log::Logged::Turn(stack) => Some((*t, Some(*stack))),
        _ => None,
    }));
    times
}

/// The engine's battle (of `turned`) the turn whose acting stack is `stack` pairs with: the first
/// from `next` on with it (`next` then after it).
fn pair(turned: &[Turned], next: &mut usize, stack: Option<u8>) -> Option<usize> {
    let found = (*next..turned.len()).find(|&i| turned[i].observation.active == stack);
    if let Some(i) = found {
        *next = i + 1;
    }
    found
}

/// The battle and the legal commands of the turn whose acting stack is `stack`, paired with
/// `turned`'s `found`: that battle's, else the opening's with none (none without a snapshot).
fn battle_of(
    record: &Record,
    tables: &Tables,
    turned: &[Turned],
    (found, stack): (Option<usize>, Option<u8>),
) -> Option<(Observation, Legal)> {
    if let Some(turn) = found.and_then(|i| turned.get(i)) {
        let legal = legal(
            &record.header.manifest,
            tables,
            &turn.observation,
            (&turn.commands, &turn.aims),
        );
        return Some((turn.observation.clone(), legal));
    }
    let mut observation = record.first_snapshot()?.clone();
    observation.active = stack;
    Some((observation, Legal::default()))
}

/// The actions' starts as the mouse sees them.
fn actions(timeline: &Timeline) -> Vec<Start> {
    timeline
        .actions
        .iter()
        .map(|a| Start { unmarks: a.unmarks })
        .collect()
}

/// The mouse over a battle as it grows (E17 M5 T8a, R3-1, *provisional*; the window's): what
/// [`shown`] shows, each turn's battle, legal commands and facing read once as the turn comes
/// (again only when its engine's battle changed in place, or, its start not final yet, when the
/// timeline grew: a turn's facing at its start never changes after), and the mouse followed on
/// from its last mark ([`track::follow_on`]): a move pays for the inputs since the last turn's
/// start, not for the battle so far. Auto Combat comes from the battle's own switches and stops
/// (T8b2, R5c-3: a handful, kept apart from the mouse's inputs, so [`Auto::of`] runs over the
/// turns only).
#[derive(Debug, Default)]
pub struct Follower<'l> {
    /// Each turn read, and whether the timeline was final at its start then.
    turns: Vec<(TurnRead, bool)>,
    /// The turns' starts (Auto Combat's, [`Auto::of`]).
    starts: Vec<u32>,
    /// The actions' starts.
    actions: Vec<Start>,
    /// The lowest index of the engine's battles changed in place since the turns were read.
    touched: Option<usize>,
    mark: Option<track::Mark<'l>>,
    /// Auto Combat as the last call had it: a turn before the mark it plays otherwise now drops
    /// the mark.
    auto: Auto,
}

/// A turn as [`Follower`] read it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct TurnRead {
    /// Its start, its acting stack and the engine's battle paired with it (an index of
    /// `turned`; none: the opening's stands for it).
    key: (u32, Option<u8>, Option<usize>),
    observation: Observation,
    legal: Legal,
    facing: Facing,
    /// A human on this computer plays it (Auto Combat aside).
    human: bool,
}

impl<'l> Follower<'l> {
    /// The engine's battle `index` changed in place (E17 M5 T3b: an aim the book added; T6b1: a
    /// turn served again): the turn paired with it is read again.
    pub fn touch(&mut self, index: usize) {
        self.touched = Some(self.touched.map_or(index, |t| t.min(index)));
    }

    /// Reads `timeline`'s turns paired with the engine's battles `turned` (the assets `tables`):
    /// those new, those whose battle was touched and, when the timeline `grew` (final before
    /// `settled`), those whose start was not final. The mark goes when a turn before its own
    /// changed.
    pub fn read(
        &mut self,
        record: &Record,
        (timeline, turned): (&Timeline, &[Turned]),
        tables: &Tables,
        (settled, grew): (u32, bool),
    ) {
        if !grew && self.touched.is_none() {
            return;
        }
        let touched = self.touched.take().unwrap_or(usize::MAX);
        let times = times(record, timeline);
        let (mut next, mut k, mut changed) = (0, 0, usize::MAX);
        for &(t, stack) in &times {
            let found = pair(turned, &mut next, stack);
            let key = (t, stack, found);
            let kept = self.turns.get(k).is_some_and(|(r, fixed)| {
                r.key == key && found.is_none_or(|i| i < touched) && (*fixed || !grew)
            });
            if !kept {
                let Some((observation, legal)) = battle_of(record, tables, turned, (found, stack))
                else {
                    continue;
                };
                let read = TurnRead {
                    key,
                    human: human(&observation),
                    observation,
                    legal,
                    facing: facing(timeline, t),
                };
                if self.turns.get(k).is_none_or(|(r, _)| *r != read) {
                    changed = changed.min(k);
                }
                let entry = (read, t < settled);
                match self.turns.get_mut(k) {
                    Some(slot) => *slot = entry,
                    None => self.turns.push(entry),
                }
            }
            k += 1;
        }
        if k < self.turns.len() {
            self.turns.truncate(k);
            changed = changed.min(k);
        }
        self.starts = times.iter().map(|&(t, _)| t).collect();
        if grew {
            self.actions = actions(timeline);
        }
        if self.mark.as_ref().is_some_and(|m| changed < m.turn()) {
            self.mark = None;
        }
    }

    /// What the mouse shows through `timeline` by `inputs` and Auto Combat's `switches` (the
    /// battle's [`Input::Auto`] and [`Input::Stop`]; the inputs' own do nothing here): as
    /// [`shown`]'s by the inputs with the switches among them (the hints in `name`'s words, the
    /// turns as last read): where in what the last call showed (cut there, [`Shown::splice`])
    /// the lists returned go.
    pub fn shown(
        &mut self,
        timeline: &Timeline,
        (inputs, switches): (&[Input], &[Input]),
        (layout, name, bugs): (&'l BattleLayout, Name<'_>, Bugs),
    ) -> (Lens, Shown) {
        let auto = Auto::of(switches, &self.starts);
        let before = self.mark.as_ref().map_or(0, track::Mark::turn);
        if (0..before).any(|k| auto.from_start(k) != self.auto.from_start(k)) {
            self.mark = None;
        }
        let turns: Vec<Turn<'_>> = (self.turns.iter().enumerate())
            .map(|(k, (r, _))| Turn {
                field: Field {
                    observation: &r.observation,
                    legal: &r.legal,
                    facing: &r.facing,
                    bugs,
                },
                human: r.human && !auto.from_start(k),
            })
            .collect();
        let log = (&timeline.log[..], &self.actions[..], &turns[..]);
        let (from, shown, mark) = track::follow_on(self.mark.take(), log, inputs, layout, name);
        self.mark = mark;
        self.auto = auto;
        (from, shown)
    }
}

/// When Auto Combat plays `timeline`'s turns (the opening's, then each one started) by the mouse's
/// and the keys' `inputs` (E17 M5 T4c1).
pub fn auto(inputs: &[Input], timeline: &Timeline) -> Auto {
    let starts: Vec<u32> = core::iter::once(0)
        .chain(timeline.turns.iter().map(|&(t, _)| t))
        .collect();
    Auto::of(inputs, &starts)
}

/// Which way the stacks face at `t` of `timeline`: those its poses then mirror face left.
pub fn facing(timeline: &Timeline, t: u32) -> Facing {
    let left: BTreeSet<u8> = timeline
        .at(t)
        .into_iter()
        .filter(|(_, p)| p.mirrored)
        .map(|(id, _)| id)
        .collect();
    Facing::Left(left)
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

/// A hint's text with the record's names, the window's and a still's alike (E17 M4 T3a, R2-3): a
/// stack's creature (`creature.<key>`), a spell (`spell.<key>`), a town's wall part
/// (`wall.<faction>.<part>`, E17 M4 T2c-2).
pub fn hint_text(
    record: &Record,
    catalogue: &Catalogue,
    hint: &Hint,
    observation: &Observation,
) -> String {
    let keys = &record.header.manifest.keys;
    let creature = |s: &Stack| {
        let key = keys.creatures.get(usize::from(s.creature))?;
        Some(format!("creature.{key}"))
    };
    let spell = |handle: u16| {
        let key = keys.spells.get(usize::from(handle))?;
        Some(format!("spell.{key}"))
    };
    let part = |faction: u16, part: usize| {
        let key = keys.factions.get(usize::from(faction))?;
        Some(format!("wall.{key}.{part}"))
    };
    hint.text(catalogue, observation, (&creature, &spell, &part))
}

#[cfg(test)]
mod tests {
    use super::*;
    use h3_view::layout::parse_battle;

    fn layout() -> BattleLayout {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../layouts/battle.toml");
        parse_battle(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    /// The window's mouse step by step (E17 M5 T8a, R3-1).
    struct Steps<'l> {
        record: &'l Record,
        tables: &'l Tables,
        layout: &'l BattleLayout,
        follower: Follower<'l>,
        shown: Shown,
        checked: usize,
    }

    impl Steps<'_> {
        /// The follower's lists after `inputs` over `timeline` (final before `settled`; `grew`
        /// since the last step) and the engine's battles `turned`, put in place of the last ones
        /// from where they go, are the whole follow's; they changed as the whole follow's did.
        fn check(
            &mut self,
            (timeline, settled, grew): (&Timeline, u32, bool),
            turned: &[Turned],
            (inputs, switches): (&[Input], &[Input]),
        ) {
            let name = |hint: &Hint, _: &Observation| format!("{hint:?}");
            let read = (timeline, turned);
            (self.follower).read(self.record, read, self.tables, (settled, grew));
            let words = (self.layout, &name as _, Bugs::Fixed);
            let (from, tail) = self.follower.shown(timeline, (inputs, switches), words);
            let words = (self.layout, &name as _);
            // The whole follow's inputs: the switches among the mouse's by time.
            let mut all = inputs.to_vec();
            for &switch in switches {
                let i = all.partition_point(|i| i.at() <= switch.at());
                all.insert(i, switch);
            }
            let whole = followed(self.record, read, &all, words, (self.tables, Bugs::Fixed));
            let same = self.shown.ends_with(from, &tail);
            assert_eq!(same, self.shown == whole, "step {}", self.checked);
            self.shown.splice(from, tail);
            assert_eq!(self.shown, whole, "step {}", self.checked);
            self.checked += 1;
        }
    }

    #[test]
    fn the_window_s_mouse_goes_on_as_the_whole_follow_shows() {
        // E17 M5 T8a (R3-1): `scenarios/battle.toml` in hot seat, grown a command at a time as the
        // window grows it (`Battle::command`: every side's events, the turn after each, the
        // timeline waiting for the next); the mouse over hexes, the panel and its log while each
        // turn waits, while each action plays and at each turn's start, the spell book opened and
        // closed, a seek back's inputs gone, a turn's legal commands changed in place: at every
        // step the follower's lists are the whole follow's (every turn read again; 70 turns).
        use h3_view::anim::{Casts, Command as Given, Growing, Look, Speed};
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let tables = crate::assets::load(&dir.join("assets"), crate::assets::ALL).unwrap();
        let layout = layout();
        let (scenario, packs) = (dir.join("scenarios/battle.toml"), dir.join("../packs"));
        let (ruleset, mut setup) = crate::scenario::battle(&scenario, &packs, 1).unwrap();
        setup.human = [true, true];
        let mut live = crate::live::Live::new(&ruleset, setup, 1, [true, true]).unwrap();
        let record = h3_view::record::parse(&live.record().unwrap()).unwrap();
        let snapshot = record.first_snapshot().unwrap();
        // Every creature's DEF groups, four frames each (the looks' numbers do not matter here).
        let look = Look {
            frames: (0..24).map(|g| (g, 4)).collect(),
            times: [3000, 500, 500, 100],
            ..Look::default()
        };
        let looks = (snapshot.stacks.iter())
            .map(|s| (s.creature, look.clone()))
            .collect();
        let (magic, casts) = (std::collections::BTreeMap::new(), Casts::default());
        let stage = crate::motion::stage(&record, &tables, &layout, (&magic, &casts, None));
        let mut growing = Growing::new(snapshot, &looks, stage, Speed::new(2), Some(1)).unwrap();
        let mut turned: Vec<Turned> = live.turn().into_iter().collect();
        let mut timeline = growing.waiting();
        let mut steps = Steps {
            record: &record,
            tables: &tables,
            layout: &layout,
            follower: Follower::default(),
            shown: Shown::default(),
            checked: 0,
        };
        steps.check(
            (&timeline, growing.timeline().end, true),
            &turned,
            (&[], &[]),
        );
        let hexes = [40, 57, 75, 90, 110, 128, 150].map(|h| hex_point(&layout, h));
        let points: Vec<[i32; 2]> = hexes
            .into_iter()
            .chain([PARK, [700, 570], [400, 580]])
            .collect();
        let mut point = points.iter().copied().cycle();
        let mut inputs: Vec<Input> = Vec::new();
        // Auto Combat's switches and stops, the battle's own (T8b2, R5c-3).
        let mut switches: Vec<Input> = Vec::new();
        let mut press = 0;
        for c in 0.. {
            let start = growing.timeline().turns.last().map_or(0, |&(t, _)| t);
            let end = growing.timeline().end;
            let waits = (&timeline, end, false);
            // While the action plays, at the turn's start, while the turn waits.
            let mut times: Vec<u32> = (1..=3)
                .map(|m| press + 7 * m)
                .filter(|&t| t < start)
                .collect();
            times.push(start);
            times.extend((0..4).map(|m| end + 33 * m));
            for at in times {
                let to = point.next().unwrap();
                inputs.push(Input::Move { at, to });
                steps.check(waits, &turned, (&inputs, &switches));
            }
            let mut now = end + 33 * 4;
            if c % 3 == 1 {
                let book = [
                    Input::Book { at: now },
                    Input::Move {
                        at: now + 5,
                        to: book_place(2),
                    },
                    Input::Cancel { at: now + 20 },
                ];
                for input in book {
                    inputs.push(input);
                    steps.check(waits, &turned, (&inputs, &switches));
                }
                now += 30;
            }
            // Auto Combat switched on in the turn waiting, stopped in a later one (R5c-3); once a
            // switch naming the opening's turn: every turn before the mark plays otherwise.
            let turn = growing.timeline().turns.len();
            let switch = match c % 6 {
                1 => Some(Input::Auto { at: now, turn }),
                4 => Some(Input::Stop { at: now, turn }),
                _ => None,
            };
            if let Some(switch) = switch {
                switches.push(switch);
                steps.check(waits, &turned, (&inputs, &switches));
            }
            if c == 30 {
                switches.insert(0, Input::Auto { at: 0, turn: 0 });
                steps.check(waits, &turned, (&inputs, &switches));
            }
            if c % 5 == 2 {
                // A seek back to the turn's start, then a move from there.
                inputs.retain(|i| i.at() <= start);
                steps.check(waits, &turned, (&inputs, &switches));
                let to = point.next().unwrap();
                inputs.push(Input::Move { at: start, to });
                steps.check(waits, &turned, (&inputs, &switches));
            }
            if c % 4 == 3
                && let Some(turn) = turned.last_mut()
            {
                // The turn's battle changed in place (as the book's aim changes it).
                turn.commands.pop();
                steps.follower.touch(turned.len() - 1);
                steps.check(waits, &turned, (&inputs, &switches));
            }
            press = now;
            inputs.push(Input::Act { at: press });
            // The first commands defend: a longer battle, its rounds' starts among its turns.
            let choice = live.choice().unwrap();
            let choice = if c < 48 { Command::Defend {} } else { choice };
            let played = live.play(choice).unwrap().unwrap();
            let (events, seen): (Vec<_>, Vec<_>) = played.events.into_iter().unzip();
            let given = Given {
                at: Some(press.max(end)),
                events: &events,
            };
            growing.push_seen(&given, false, &seen);
            turned.extend(live.turn());
            if live.over() {
                let end = growing.finish();
                steps.check((&end, u32::MAX, true), &turned, (&inputs, &switches));
                break;
            }
            timeline = growing.waiting();
            steps.check(
                (&timeline, growing.timeline().end, true),
                &turned,
                (&inputs, &switches),
            );
        }
        assert!(steps.checked > 600, "{}", steps.checked);
    }

    #[test]
    fn auto_combat_s_key_switches_it_on_and_a_press_then_stops_it() {
        // E17 M5 T4c1: A switches it on; while it plays a key or a click stops it, the click
        // giving nothing; then A switches it on again and a key gives a command once it is off.
        let layout = layout();
        let timeline = Timeline::default();
        let actions: Vec<String> = ["key:a", "at:10", "hex:40", "key:a", "key:Escape", "key:d"]
            .map(String::from)
            .into();
        let got = inputs(&actions, 0, &layout, (&timeline, &[], &|_| None), false).unwrap();
        assert_eq!(
            got,
            vec![
                Input::Move { at: 0, to: PARK },
                Input::Auto { at: 0, turn: 0 },
                Input::Move {
                    at: 10,
                    to: hex_point(&layout, 40)
                },
                Input::Stop { at: 10, turn: 0 },
                Input::Move { at: 10, to: PARK },
                Input::Auto { at: 10, turn: 0 },
                Input::Stop { at: 10, turn: 0 },
                Input::Act { at: 10 },
            ]
        );
        assert!(auto_combat("a") && auto_combat("A") && !auto_combat("d") && !auto_combat(""));
        // When it plays the timeline's turns: the opening's from 0.
        let auto = auto(&got, &timeline);
        assert!(auto.plays(0, 0) && !auto.from_start(0));
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
                Input::Book { at: 7 },
                Input::Move {
                    at: 7,
                    to: [332, 219]
                },
                Input::Aim {
                    at: 7,
                    spell: 54,
                    creature: false
                },
                Input::Move {
                    at: 7,
                    to: hex_point(&layout, 49)
                },
                Input::Act { at: 7 },
                Input::Move { at: 7, to: PARK },
            ]
        );
        // A spell without a target is cast at its place, the book's status line empty (E17 M3
        // T7c-1).
        let untargeted = cast("0", (0, 54), &layout).unwrap();
        assert_eq!(untargeted.len(), 4);
        assert_eq!(untargeted[0], Input::Book { at: 0 });
        for bad in ["12", "x", "0:x", "-1:4"] {
            assert!(cast(bad, (0, 54), &layout).is_err(), "{bad}");
        }
    }

    #[test]
    fn the_acting_stack_s_own_tests_follow_its_creature() {
        // E17 M4 T3a, R2-6: the Catapult by its original id (145), the shooters the walls never
        // halve (34, 35, 136, 137, 149, the asset tables' ids) and a hero's bow
        // (`no_distance_penalty`); a handle the manifest lacks neither.
        use h3_protocol::wire::Seat;
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let tables = crate::assets::load(&dir.join("assets"), crate::assets::ALL).unwrap();
        let text = crate::scenario::record_of(
            &dir.join("situations/seq-keep.toml"),
            &dir.join("../packs"),
            108,
            Seat::Spectator,
        )
        .unwrap();
        let record = h3_view::record::parse(&text).unwrap();
        let mut o = record.first_snapshot().unwrap().clone();
        let at = o.stacks.iter().position(|s| s.hex < 187).unwrap();
        o.active = Some(o.stacks[at].id);
        let keys = &record.header.manifest.keys.creatures;
        let handle = |key: &str| {
            let i = keys.iter().position(|k| k == key).unwrap();
            u16::try_from(i).unwrap()
        };
        let tested = |creature: u16, bow: bool| {
            let mut o = o.clone();
            o.stacks[at].creature = creature;
            o.stacks[at].no_distance_penalty = bow;
            let l = legal(&record.header.manifest, &tables, &o, (&[], &[]));
            (l.catapult, l.wall_exempt)
        };
        assert_eq!(tested(handle("base:catapult"), false), (true, false));
        // (The record's packs are the base game's: the AB's 136 and 137 by the same list.)
        for exempt in ["base:mage", "base:arch_mage"] {
            assert_eq!(tested(handle(exempt), false), (false, true), "{exempt}");
        }
        assert_eq!(tested(handle("base:archer"), false), (false, false));
        assert_eq!(tested(handle("base:archer"), true), (false, true));
        assert_eq!(tested(u16::MAX, false), (false, false));
    }

    #[test]
    fn the_cursor_s_shot_line_steps_as_the_engine_s_x87() {
        // E17 M4 T2c-2: the library's integer axis (`h3_view::mouse::line`) against the x87's
        // steps as the engine computes them (`h3_kernel::arith::X87`, its own test of the same):
        // every start, difference and longer difference of the field.
        use h3_kernel::arith::{DOUBLE_BITS, SINGLE_BITS, X87};
        use h3_view::mouse::line::Axis;
        for long in 1..=16u32 {
            let reach = i32::try_from(long).unwrap();
            for d in -reach..=reach {
                let unit = X87::ratio(d.unsigned_abs(), long, DOUBLE_BITS)
                    .unwrap()
                    .round(SINGLE_BITS);
                let by = unit.over(X87::int(17)).unwrap().round(SINGLE_BITS);
                for from in 0..=16i32 {
                    let mut x87 = X87::int(u64::from(from.unsigned_abs()));
                    let mut axis = Axis::new(from, d, long);
                    for step in 0..17 * long {
                        x87 = if d < 0 { x87.minus(by) } else { x87.plus(by) }.round(SINGLE_BITS);
                        let column = axis.advance();
                        assert_eq!(
                            (axis.position(), i64::from(column)),
                            (x87.to_fixed(), i64::try_from(x87.trunc()).unwrap()),
                            "from {from}, difference {d} of {long}, step {step}"
                        );
                    }
                }
            }
        }
    }
}
