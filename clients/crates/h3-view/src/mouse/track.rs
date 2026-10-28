//! The mouse over time (E17 M3 T6c-2; `docs/epics/E17-design.md` §9.10): the battle's mouse
//! handler (`battle_buttons`' mouse move 0x4753A4) takes a move while a human's turn waits; while
//! an action plays the moves wait in the input queue, which every stack's turn start empties
//! (`battle_controls_state` 0x4782D0, `input_flush` 0x4EC570; E17 M5 T2a, design §9.11): a human's
//! turn forgets the mouse's hex and posts the mouse where it rests (0x4ECC00). The cursor's frame, the stack the mouse marks, the stacks it enters (their
//! group 1, `battle_mouse_stack` 0x477C20) and the status line's hints follow, the hints into the
//! battle's log where the battle writes them (`battle_log_add` with its second argument 0: an
//! attack's and a shot's forced over the log's new lines, the others not).

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use h3_protocol::wire::{Observation, Side};
use h3_render::Bugs;

use super::aim::Aim;
use super::aim::screen_hex;
use super::{Field, Hint, Kind, Spot, pointer_in, slot, slots_in, spot};
use crate::battle_log::Logged;
use crate::layout::{BattleLayout, Mouse};

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
    /// A spell's aim starts (E17 M3 T7b-1; read at M5 T3b, design §7.18): the hero's book closed
    /// on a spell that takes a target, or a Faerie Dragon's key F; the mouse aims it until a
    /// click on a hex it takes (an `Act`) or a `Cancel` (`enter_player_cast` 0x59EC50's loop).
    Aim {
        /// When.
        at: u32,
        /// The spell's handle.
        spell: u16,
        /// A creature's cast (the Faerie Dragon's: action 10), not the hero's.
        creature: bool,
    },
    /// The hero's spell book opened over the battle (E17 M3 T7c-1): its dialog takes the mouse,
    /// so the status line is empty (a spell without a target is cast from it at once, its line
    /// the next the log shows), and the battle takes no move until it closes (E17 M5 T3b).
    Book {
        /// When.
        at: u32,
    },
    /// The book closed without a spell, or the aim was given up (Escape, the right button:
    /// 0x5A317A, 0x5A3184; E17 M5 T3b): the battle takes the mouse again.
    Cancel {
        /// When.
        at: u32,
    },
}

impl Input {
    /// When it happens.
    #[must_use]
    pub const fn at(&self) -> u32 {
        match *self {
            Self::Move { at, .. }
            | Self::Act { at }
            | Self::Aim { at, .. }
            | Self::Book { at }
            | Self::Cancel { at } => at,
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
    /// The spell cursor from a time on (E17 M5 T3b): the time its frames start from (the
    /// aim's `SetCursor(spell + 1, 3)`), none while the combat cursor shows.
    pub spell: Vec<(u32, Option<u32>)>,
    /// The stacks an area spell's aim marks from a time on (their outline the mouse's, 0x43E268;
    /// E17 M5 T3b), in order.
    pub area: Vec<(u32, Vec<u8>)>,
    /// The hex an aim's cursor took from a time on (0x688324: a click casts there; E17 M5 T3b),
    /// none outside an aim or over a hex it does not take.
    pub aimed: Vec<(u32, Option<u8>)>,
}

/// A cursor's look: the combat cursor's frame, or the spell cursor's (E17 M5 T3b).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Look {
    /// `crcombat.DEF`'s frame (the mouse manager's type 2).
    Combat(u32),
    /// `crspell.DEF`'s frame (type 3).
    Spell(u32),
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

    /// The cursor's look at `t` (none before the mouse is anywhere): the spell cursor's frame by
    /// the poll's grid (`mouse_poll` 0x50D838: a frame on at each `step` of the clock after it
    /// was set, round its `frames`), else the combat cursor's.
    #[must_use]
    pub fn look(&self, t: u32, mouse: &Mouse) -> Option<Look> {
        match last(&self.spell, t).copied().flatten() {
            Some(since) => Some(Look::Spell(spell_frame(since, t, mouse))),
            None => self.frame(t).map(Look::Combat),
        }
    }

    /// The stacks an area spell's aim marks at `t`.
    #[must_use]
    pub fn area(&self, t: u32) -> &[u8] {
        last(&self.area, t).map_or(&[], Vec::as_slice)
    }

    /// The hex the aim's cursor took at `t`, where a click casts (none: no aim, or a hex it does
    /// not take).
    #[must_use]
    pub fn aimed(&self, t: u32) -> Option<u8> {
        last(&self.aimed, t).copied().flatten()
    }
}

/// The spell cursor's frame at `t` when set at `since`: one frame on at each point of the
/// poll's grid after `since` up to `t` (the grid's points the multiples of `step` from the
/// opening, as the harness aligns it: `mouse_poll` 0x50D827), round the DEF's frames.
#[must_use]
pub fn spell_frame(since: u32, t: u32, mouse: &Mouse) -> u32 {
    let step = mouse.spell_step.max(1);
    let steps = (t / step).saturating_sub(since / step);
    steps % mouse.spell_frames.max(1)
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
    /// An action plays: the moves wait in the input queue (and are dropped as the next turn
    /// starts).
    busy: bool,
    /// The spell being aimed (E17 M3 T7b-1; M5 T3b).
    aiming: Option<Aiming>,
    /// The hero's spell book is open: the battle takes no move (E17 M5 T3b).
    book: bool,
    /// The spell cursor's start (none: the combat cursor; E17 M5 T3b).
    spell_since: Option<u32>,
    /// The stacks an area spell's aim marks (+0x4F1; E17 M5 T3b).
    area: BTreeSet<u8>,
    /// The panel's item the mouse last entered (`hovered_item` 0x6AAC68; E17 M4 T3b): a move
    /// inside it writes no hint.
    hovered: Option<u16>,
    /// The panel's items take the battle's moves: not the move of the mouse resting where it was
    /// placed before the battle (E17 M4 T3b: the opening enters no item).
    items: bool,
    /// The profile: the client's forgets the hovered item over the battlefield (CLT-Q24).
    bugs: Bugs,
    shown: Shown,
}

/// A spell's aim under way (`enter_player_cast` 0x59EC50's loop, its handler 0x5A3070).
#[derive(Debug, Clone)]
struct Aiming {
    aim: Aim,
    /// The hex the cursor took at the last move (0x688324; none: −1): a click casts at it.
    last: Option<u8>,
}

impl Handler<'_> {
    /// The combat cursor's `frame` (`SetCursor(frame, 2)`): the spell cursor's type gives way to
    /// it, so its frame is drawn again (0x50CD03).
    fn set_frame(&mut self, now: u32, frame: u32) {
        if self.spell_since.take().is_some() {
            self.shown.spell.push((now, None));
            self.frame = None;
        }
        if self.frame != Some(frame) {
            self.frame = Some(frame);
            self.shown.frames.push((now, frame));
        }
    }

    /// The spell cursor (`SetCursor(spell + 1, 3)`, E17 M5 T3b): its frame 0 drawn now unless
    /// it shows already at the frame `asked` (0x50CD37: type 3 draws its frame 0 whatever is
    /// asked, and keeps it as the frame shown, which the poll then steps).
    fn set_spell(&mut self, now: u32, asked: u32) {
        let shown = self
            .spell_since
            .map(|since| spell_frame(since, now, &self.layout.mouse));
        if shown != Some(asked) {
            self.spell_since = Some(now);
            self.shown.spell.push((now, Some(now)));
        }
    }

    /// The stacks an area spell's aim marks (`set_area_mark` 0x43EFE0 for each living stack):
    /// a stack newly marked starts its group 1 as the mouse's does.
    fn set_area(&mut self, now: u32, marked: BTreeSet<u8>) {
        if marked == self.area {
            return;
        }
        for &id in marked.difference(&self.area) {
            self.shown.entered.push((now, id));
        }
        self.shown
            .area
            .push((now, marked.iter().copied().collect()));
        self.area = marked;
    }

    /// The aim's cursor over the screen hex `hex` (`spell_target_cursor` 0x59F700): the hex it
    /// takes, else none.
    fn aim_at(
        &mut self,
        now: u32,
        (aim, hex): (&Aim, i32),
        field: &Field<'_>,
        name: &dyn Fn(&Hint, &Observation) -> String,
    ) -> Option<u8> {
        let taken = aim.takes(field, hex);
        if let Some(h) = taken {
            self.set_spell(now, u32::from(aim.original) + 1);
            self.write(now, &aim.line(field, h), Some(field), name);
            if aim.area.is_some() {
                self.set_area(now, aim.marks(field, hex));
            } else {
                // `battle_mouse_stack`: nothing over the stack already marked.
                let stack = field.at(h).map(|s| s.id);
                if stack.is_none() || stack != self.marked {
                    self.set_marked(now, stack);
                    if let Some(id) = stack {
                        self.shown.entered.push((now, id));
                    }
                }
            }
        } else {
            self.set_frame(now, 0);
            // The side to act (+0x132C0): the acting stack's controller.
            let side = field.acting().map_or(Side::Attacker, |s| s.controller);
            self.write(now, &aim.refusal(field, hex, side), Some(field), name);
            // `battle_clear_highlight` 0x477E10.
            self.set_marked(now, None);
            if aim.area.is_some() {
                self.set_area(now, aim.marks(field, hex));
            }
        }
        taken
    }

    /// The aim ends (a click's cast or a cancel: 0x5A3109, 0x5A31A3): no stack keeps an area's
    /// mark.
    fn end_aim(&mut self, now: u32) {
        if self.aiming.take().is_some() {
            self.shown.aimed.push((now, None));
        }
        self.set_area(now, BTreeSet::new());
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
        if let (Some(aiming), Some(field)) = (&self.aiming, field) {
            // The aim takes every move, the panel's points too (0x5A309D): the cursor again only
            // over another hex than the one it last took (0x5A30C4).
            let hex = screen_hex(spot(self.layout, point, field.rects()));
            if aiming.last.map_or(-1, i32::from) != hex {
                let aim = aiming.aim.clone();
                let last = self.aim_at(now, (&aim, hex), field, name);
                self.aiming = self.aiming.take().map(|a| Aiming { last, ..a });
                self.shown.aimed.push((now, last));
            }
            return;
        }
        let [width, height] = self.layout.mouse.field;
        if !(0..width).contains(&point[0]) || !(0..height).contains(&point[1]) {
            // Outside the battlefield: unmarked, the items' hints, the pointer (0x475402). The
            // panel's items take the move (0x475512 → 0x602320 → 0x5FEDE0): the item under the
            // mouse writes its hint, an empty line without one (0x472B80), only as the mouse
            // enters it, another being the hovered item (0x5FE73E; E17 M4 T3b, read per frame:
            // the panel's picture 2000 entered at the first move into the panel after the
            // opening, so `walk`'s next human turn empties the line and `hit`'s keeps the
            // Catapult's hint; `aim`'s mouse rests over the field until its shot). The
            // battlefield's last hint so stays over the panel's picture: the original's, kept in
            // its profile (CLT-Q24, answered (a)).
            self.set_marked(now, None);
            let item = if self.items {
                self.layout.panel.item_at(point)
            } else {
                None
            };
            if let Some(item) = item
                && self.hovered != Some(item.id())
            {
                self.hovered = Some(item.id());
                let hint = item
                    .hint()
                    .map_or(Hint::Empty, |key| Hint::Control(key.into()));
                self.write(now, &hint, field, name);
            }
            self.set_frame(now, self.layout.mouse.panel_frame);
            self.hex = None;
            self.written = None;
            return;
        }
        // The client's profile forgets the hovered item as the battle takes a move over the
        // battlefield, so the panel's picture writes its empty line again (CLT-Q24, E17 M5 T2a).
        if self.bugs == Bugs::Fixed {
            self.hovered = None;
        }
        let Some(field) = field else {
            return;
        };
        let at = match spot(self.layout, point, field.rects()) {
            Spot::Nothing => None,
            other => Some(other),
        };
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
/// start; the first the mouse resting where it was placed before the battle). `name` words a hint
/// in the battle a turn shows.
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
        busy: false,
        aiming: None,
        book: false,
        spell_since: None,
        area: BTreeSet::new(),
        hovered: None,
        items: false,
        bugs: turns.first().map_or(Bugs::Fixed, |t| t.field.bugs),
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
            Input::Aim {
                at: now,
                spell,
                creature,
            } => {
                h.book = false;
                let turn = turns.get(at.turn);
                let aim = turn.and_then(|t| {
                    t.field
                        .legal
                        .aims
                        .iter()
                        .find(|a| a.spell == spell && a.creature == creature)
                });
                if let (Some(turn), Some(aim)) = (turn, aim) {
                    h.aiming = Some(Aiming {
                        aim: aim.clone(),
                        last: None,
                    });
                    // The cursor where the mouse is now (0x59EE2E), the hex it takes not kept
                    // (0x59EE56): a click before the next move casts nothing.
                    let hex = h
                        .point
                        .map_or(-1, |p| screen_hex(spot(layout, p, turn.field.rects())));
                    h.aim_at(now, (aim, hex), &turn.field, name);
                }
            }
            Input::Book { at: now } => {
                h.book = true;
                h.write(now, &Hint::Empty, None, name);
                h.hex = None;
                h.written = None;
                // The book's items take the mouse and are destroyed as it closes, the hovered
                // one forgotten with them (0x5FE442; E17 M4 T3b, *provisional*: not read per
                // frame).
                h.hovered = None;
            }
            Input::Cancel { at: now } => {
                let aimed = h.aiming.as_ref().map(|a| a.aim.creature);
                h.end_aim(now);
                h.book = false;
                if aimed.is_some() {
                    // `enter_player_cast`'s end (0x59F65C).
                    h.set_frame(now, layout.mouse.panel_frame);
                }
                if aimed != Some(true) {
                    // The book's button's end (0x47519A): the mouse's hex forgotten and the
                    // mouse posted where it rests (0x4ECC00); the key F's posts nothing.
                    h.hex = None;
                    if let (Some(point), Some(turn)) = (h.point, turns.get(at.turn)) {
                        h.take(now, point, Some(&turn.field), name);
                    }
                }
            }
            Input::Act { at: now } => {
                // In the aim a click casts only at the hex the last move took (0x5A30E9).
                if h.aiming.as_ref().is_some_and(|a| a.last.is_none()) {
                    continue;
                }
                h.end_aim(now);
                h.book = false;
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
                // Else it waits in the input queue, to be dropped as the next turn starts; the
                // spell book open takes it (E17 M5 T3b).
                match turns.get(at.turn) {
                    _ if h.book => {}
                    Some(turn) if turn.human && !h.busy => h.take(now, to, Some(&turn.field), name),
                    None if !h.busy => h.take(now, to, None, name),
                    _ => {}
                }
                h.items = true;
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
                    // The waiting moves dropped (0x478301; E17 M5 T2a), the mouse's hex forgotten
                    // (0x47883E) and the mouse posted where it rests (0x478857).
                    self.hex = None;
                    if let Some(point) = self.point {
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
