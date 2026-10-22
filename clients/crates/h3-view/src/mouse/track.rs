//! The mouse over time (E17 M3 T6c-2; `docs/epics/E17-design.md` §9.10): the battle's mouse
//! handler (`battle_buttons`' mouse move 0x4753A4) takes a move while a human's turn waits; while
//! an action plays the moves wait in the input queue, which the battle takes when the next human
//! turn starts (`battle_controls_state` 0x4782D0: the mouse's hex forgotten, the mouse posted
//! again, 0x4ECC00). The cursor's frame, the stack the mouse marks, the stacks it enters (their
//! group 1, `battle_mouse_stack` 0x477C20) and the status line's hints follow, the hints into the
//! battle's log where the battle writes them (`battle_log_add` with its second argument 0: an
//! attack's and a shot's forced over the log's new lines, the others not).

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use h3_protocol::wire::Observation;

use super::{Field, Hint, Kind, Spot, pointer_in, slot, slots_in, spot};
use crate::battle_log::Logged;
use crate::layout::BattleLayout;

/// What the mouse does, in time order (the harness's actions, the window's events).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    /// The mouse moves to a point of the screen.
    Move {
        /// When (ms after the opening).
        at: u32,
        /// Where.
        to: [i32; 2],
    },
    /// A click or a key gives the command whose action starts then: the moves after it at that
    /// time come while the action plays.
    Act {
        /// When.
        at: u32,
    },
    /// The hero's spell book closed on a spell (E17 M3 T7b-1): the mouse aims it until the next
    /// `Act`, its hint the cast's ([`Hint::Cast`]).
    Aim {
        /// When.
        at: u32,
        /// The spell's handle.
        spell: u16,
    },
    /// The hero's spell book opened over the battle (E17 M3 T7c-1): its dialog takes the mouse,
    /// so the status line is empty (a spell without a target is cast from it at once, its line
    /// the next the log shows).
    Book {
        /// When.
        at: u32,
    },
}

impl Input {
    /// When it happens.
    #[must_use]
    pub const fn at(&self) -> u32 {
        match *self {
            Self::Move { at, .. } | Self::Act { at } | Self::Aim { at, .. } | Self::Book { at } => {
                at
            }
        }
    }
}

/// A stack's turn as the mouse sees it: the battle at its start and whether a human on this
/// computer plays it.
#[derive(Debug, Clone, Copy)]
pub struct Turn<'a> {
    /// The battle, the acting stack's legal commands, the stacks' facing.
    pub field: Field<'a>,
    /// A human on this computer plays it (else its start shows the pointer, 0x478873, and
    /// unmarks the mouse's stack, `ai_stack_turn` 0x422200).
    pub human: bool,
}

/// An action's start as the mouse sees it (`action_dispatcher` 0x478D80: every action shows the
/// pointer, 0x478E3A).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Start {
    /// It unmarks the mouse's stack (`battle_clear_highlight` 0x477E10: a walk, a strike, a shot,
    /// a cast).
    pub unmarks: bool,
}

/// What the mouse shows over time.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Shown {
    /// The combat cursor's frame from a time on, in order.
    pub frames: Vec<(u32, u32)>,
    /// Where the mouse is from a time on, in order.
    pub points: Vec<(u32, [i32; 2])>,
    /// The stack the mouse marks from a time on (its outline 0x70), in order.
    pub marked: Vec<(u32, Option<u8>)>,
    /// The stacks the mouse enters, in order: each starts its group 1, or stands
    /// (`battle_mouse_stack`; the timeline plays it, `Timeline::entered`).
    pub entered: Vec<(u32, u8)>,
    /// The battle's log with the status line's hints written where the battle writes them.
    pub log: Vec<(u32, Logged)>,
}

impl Shown {
    /// The cursor's frame at `t` (none before the mouse is anywhere).
    #[must_use]
    pub fn frame(&self, t: u32) -> Option<u32> {
        last(&self.frames, t).copied()
    }

    /// Where the mouse is at `t`.
    #[must_use]
    pub fn point(&self, t: u32) -> Option<[i32; 2]> {
        last(&self.points, t).copied()
    }

    /// The stack the mouse marks at `t`.
    #[must_use]
    pub fn marked(&self, t: u32) -> Option<u8> {
        last(&self.marked, t).copied().flatten()
    }
}

/// The value of the last change at or before `t`.
fn last<T>(changes: &[(u32, T)], t: u32) -> Option<&T> {
    let i = changes.partition_point(|(at, _)| *at <= t);
    changes.get(i.checked_sub(1)?).map(|(_, v)| v)
}

/// The battle's mouse fields (the manager's +0x132CC to +0x132E0, the acting stacks' targets).
struct Handler<'l> {
    layout: &'l BattleLayout,
    /// Where the mouse is.
    point: Option<[i32; 2]>,
    /// The mouse's hex (+0x132D4; none: -1).
    hex: Option<Spot>,
    /// The cursor's kind there (+0x132E0) and the hint written for it.
    kind: Kind,
    hint: Hint,
    /// The kind the status line was last written for (+0x132DC; none: -99).
    written: Option<Kind>,
    /// The cursor's frame.
    frame: Option<u32>,
    /// The stack marked (+0x132CC, +0x132D0).
    marked: Option<u8>,
    /// Each acting stack's target (its +0x10, +0x14).
    aims: BTreeMap<u8, Option<u8>>,
    /// The moves waiting while an action plays.
    queued: Vec<[i32; 2]>,
    busy: bool,
    /// The hero's spell being aimed (E17 M3 T7b-1).
    aiming: Option<u16>,
    shown: Shown,
}

impl Handler<'_> {
    fn set_frame(&mut self, now: u32, frame: u32) {
        if self.frame != Some(frame) {
            self.frame = Some(frame);
            self.shown.frames.push((now, frame));
        }
    }

    fn set_marked(&mut self, now: u32, marked: Option<u8>) {
        if self.marked != marked {
            self.marked = marked;
            self.shown.marked.push((now, marked));
        }
    }

    fn write(
        &mut self,
        now: u32,
        hint: &Hint,
        field: Option<&Field<'_>>,
        name: &dyn Fn(&Hint, &Observation) -> String,
    ) {
        let text = field.map_or_else(String::new, |f| name(hint, f.observation));
        let forced = hint.forced();
        self.shown.log.push((now, Logged::Status { text, forced }));
    }

    /// The battle takes a move to `point` (0x4753A4), the turn's battle `field` if one waits.
    fn take(
        &mut self,
        now: u32,
        point: [i32; 2],
        field: Option<&Field<'_>>,
        name: &dyn Fn(&Hint, &Observation) -> String,
    ) {
        let [width, height] = self.layout.mouse.field;
        if !(0..width).contains(&point[0]) || !(0..height).contains(&point[1]) {
            // Outside the battlefield: unmarked, the items' hints, the pointer (0x475402).
            self.set_marked(now, None);
            let hint = self
                .layout
                .panel
                .item_at(point)
                .and_then(crate::ui::Item::hint)
                .map_or(Hint::Empty, |key| Hint::Control(key.into()));
            self.write(now, &hint, field, name);
            self.set_frame(now, self.layout.mouse.panel_frame);
            self.hex = None;
            self.written = None;
            return;
        }
        let Some(field) = field else {
            return;
        };
        let at = match spot(self.layout, point) {
            Spot::Nothing => None,
            other => Some(other),
        };
        if let Some(spell) = self.aiming {
            // Aiming a hero's spell (E17 M3 T7b-1, *provisional*): its hint over the battlefield,
            // the cursor and the marked stack as they were (not read).
            let hint = match at {
                Some(Spot::Hex(hex)) => Hint::Cast(spell, field.at(hex).map(|s| s.id)),
                _ => Hint::Empty,
            };
            self.write(now, &hint, Some(field), name);
            self.hex = None;
            self.written = None;
            return;
        }
        if at.is_some() && at == self.hex {
            // The same hex: only an attack's slot again (0x475553).
            let mut changed = false;
            if let (Kind::Attack, Some(Spot::Hex(hex))) = (self.kind, at) {
                let frame = slots_in(field, hex)[slot(self.layout, point, hex)];
                changed = self.frame != Some(frame);
                self.set_frame(now, frame);
            }
            if self.written != Some(self.kind) || (self.kind == Kind::Attack && changed) {
                let hint = self.hint.clone();
                self.write(now, &hint, Some(field), name);
                self.written = Some(self.kind);
            }
            return;
        }
        let acting = field.observation.active.unwrap_or(u8::MAX);
        let mut aim = self.aims.get(&acting).copied().flatten();
        let pointer = pointer_in(field, self.layout, point, &mut aim);
        self.aims.insert(acting, aim);
        if at != self.hex {
            // `battle_mouse_stack`: nothing over the stack already marked.
            let stack = match at {
                Some(Spot::Hex(_)) => pointer.marked,
                _ => None,
            };
            if stack.is_none() || stack != self.marked {
                self.set_marked(now, stack);
                if let Some(id) = stack {
                    self.shown.entered.push((now, id));
                }
            }
        }
        self.hex = at;
        self.kind = pointer.kind;
        self.hint = pointer.hint;
        self.set_frame(now, pointer.frame);
        let hint = self.hint.clone();
        self.write(now, &hint, Some(field), name);
        self.written = Some(self.kind);
    }
}

/// What the mouse shows through a battle: `log` the battle's log (the timeline's, in time
/// order), `starts` its actions' starts (one per `Logged::Action`), `turns` its turns (the
/// opening's first, then one per `Logged::Turn`), `inputs` what the mouse does (in time order:
/// a move comes after the log's entries at its time up to an action's start, an `Act` takes that
/// start). `name` words a hint in the battle a turn shows.
#[must_use]
pub fn follow(
    log: &[(u32, Logged)],
    starts: &[Start],
    turns: &[Turn<'_>],
    inputs: &[Input],
    layout: &BattleLayout,
    name: &dyn Fn(&Hint, &Observation) -> String,
) -> Shown {
    let mut h = Handler {
        layout,
        point: None,
        hex: None,
        kind: Kind::Nothing,
        hint: Hint::Empty,
        written: None,
        frame: None,
        marked: None,
        aims: BTreeMap::new(),
        queued: Vec::new(),
        busy: false,
        aiming: None,
        shown: Shown::default(),
    };
    let mut at = Cursor::default();
    if turns.first().is_some_and(|t| !t.human) {
        h.set_frame(0, layout.mouse.panel_frame);
    }
    let (mut i, mut j) = (0, 0);
    loop {
        let input = inputs.get(j);
        if let Some((t, entry)) = log.get(i) {
            let first = input.is_none_or(|input| {
                *t < input.at() || (*t == input.at() && !matches!(entry, Logged::Action))
            });
            if first {
                i += 1;
                h.entry(*t, entry, starts, turns, &mut at, name);
                continue;
            }
        }
        let Some(input) = input else { break };
        j += 1;
        match *input {
            Input::Aim { spell, .. } => h.aiming = Some(spell),
            Input::Book { at: now } => {
                h.write(now, &Hint::Empty, None, name);
                h.hex = None;
                h.written = None;
            }
            Input::Act { at: now } => {
                h.aiming = None;
                // The action it gives starts now (none: a key that gives nothing).
                if let Some((t, entry @ Logged::Action)) = log.get(i)
                    && *t == now
                {
                    i += 1;
                    h.entry(now, entry, starts, turns, &mut at, name);
                }
            }
            Input::Move { at: now, to } => {
                if h.point != Some(to) {
                    h.shown.points.push((now, to));
                }
                h.point = Some(to);
                match turns.get(at.turn) {
                    Some(turn) if turn.human && !h.busy => h.take(now, to, Some(&turn.field), name),
                    None if !h.busy => h.take(now, to, None, name),
                    _ => h.queued.push(to),
                }
            }
        }
    }
    h.shown
}

/// Where [`follow`] is in the battle's actions and turns.
#[derive(Debug, Clone, Copy, Default)]
struct Cursor {
    /// The actions started.
    started: usize,
    /// The turn now (0 the opening's).
    turn: usize,
}

impl Handler<'_> {
    /// An entry of the battle's log at `now`.
    fn entry(
        &mut self,
        now: u32,
        entry: &Logged,
        starts: &[Start],
        turns: &[Turn<'_>],
        at: &mut Cursor,
        name: &dyn Fn(&Hint, &Observation) -> String,
    ) {
        self.shown.log.push((now, entry.clone()));
        match entry {
            Logged::Action => {
                let start = starts.get(at.started).copied().unwrap_or_default();
                at.started += 1;
                self.busy = true;
                self.set_frame(now, self.layout.mouse.panel_frame);
                if start.unmarks {
                    self.set_marked(now, None);
                }
            }
            Logged::Turn(_) => {
                at.turn += 1;
                self.busy = false;
                let Some(turn) = turns.get(at.turn) else {
                    return;
                };
                if turn.human {
                    // The mouse's hex forgotten (0x47883E), the waiting moves taken, then the
                    // mouse posted where it rests (0x478857).
                    self.hex = None;
                    let queued = core::mem::take(&mut self.queued);
                    for point in queued.into_iter().chain(self.point) {
                        self.take(now, point, Some(&turn.field), name);
                    }
                } else {
                    self.set_frame(now, self.layout.mouse.panel_frame);
                    self.set_marked(now, None);
                }
            }
            // `round_start` unmarks the mouse's stack (0x476081), and so does a cast as it starts
            // (0x5A0055; E17 M3 T7c-1).
            Logged::Round | Logged::Unmark => self.set_marked(now, None),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
