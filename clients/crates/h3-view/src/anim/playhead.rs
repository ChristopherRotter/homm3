//! The timeline read forward (E17 M5 T1a, R2-7): a frame drawn without rescanning the battle.
//! [`Moment::at`] reads every list of the timeline from the battle's start for each frame, and
//! runs the heroes' and the fidgets' idle steps from there; a frame late in a long battle costs
//! the battle's length. The playhead keeps what shows at the time it reached and moves on by what
//! showed since: each list's keys taken once in the order they show (by time, then by their place
//! in the list: at a time the last in the list wins, as the timeline's readers have it), the spans
//! (a projectile's flight, a ray, a shake, a drawbridge's redraws) open while they last, the idle
//! steps run on. A frame costs what changed since the frame before it and what it shows; going
//! back starts over from 0.
//!
//! Its moments are [`Moment::at`]'s at every time (the reference: the tests check every change of
//! every replay, `h3-view sequences` every recorded sequence's). It does not hold the timeline: a
//! playhead goes with the timeline it was made for, given again at each call (the window's
//! timeline changes as the mouse enters stacks: a new playhead then).

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec;
use alloc::vec::Vec;

use super::{Blast, Dissolve, Fidgets, Idle, Pose, Timeline, sees};
use crate::hero::Posing;
use crate::scene::Moment;

/// A timeline's moment at a time, moved on as the time goes forward (the module's text).
#[derive(Debug, Clone)]
pub struct Playhead {
    /// The time reached.
    at: u32,
    /// The end of the timeline as far as it was read (E17 M5 T1b: a grown one goes on from it).
    end: u32,
    /// The stacks' poses: each stack's last key shown by then.
    keys: Sweep,
    posed: BTreeMap<u8, usize>,
    /// The idle steps taken: the timeline's (up to its last command) and those after it.
    steps: usize,
    idle: Idle,
    pulse: usize,
    /// The viewer's fidgets after the last command, and the poses they gave.
    fidgets: Option<Fidgets>,
    fidgeted: BTreeMap<u8, Pose>,
    /// The heroes (none: none drawn) and the turn started by their last idle step.
    posing: Option<Posing>,
    turns: Sweep,
    turn: Option<usize>,
    /// The outline, the magic animations, the walls: the last key shown by then.
    outlines: Sweep,
    outlined: Option<usize>,
    magics: Sweep,
    magic: Option<usize>,
    spots: Sweep,
    spot: Option<usize>,
    fields: Sweep,
    field: Option<usize>,
    walls: Sweep,
    wall: Option<usize>,
    /// The objects by anchor, the counts and hues by stack, the turrets by record: the last key
    /// of each shown by then.
    objects: Sweep,
    placed: BTreeMap<u8, usize>,
    counts: Sweep,
    counted: BTreeMap<u8, usize>,
    tints: Sweep,
    tinted: BTreeMap<u8, usize>,
    /// The stacks' palette effects by their spells: the last key of each shown by then (E17 M5
    /// T5b1).
    looks: Sweep,
    looked: BTreeMap<u8, usize>,
    turrets: Sweep,
    turret: [Option<usize>; 3],
    /// The explosions: the last key shown by then, and each explosion's keys shown by then.
    blasts: Sweep,
    blast: Option<usize>,
    blasted: BTreeMap<usize, BTreeSet<usize>>,
    /// The projectiles' flights, the rays, the shakes and the drawbridge's redraws open then.
    flights: Spans,
    beams: Spans,
    shakes: Spans,
    bridges: Spans,
    /// The times the battle is kept at as the playhead passes them: just before each drawbridge's
    /// redraws ([`Timeline::bridge`]) and at each dissolve's start (E17 M5 T5a); the battles kept.
    befores: Befores,
    bridged: BTreeMap<usize, Box<Moment>>,
    /// The created stacks by id: the last key of each shown by then (E17 M5 T5a).
    created: Sweep,
    made: BTreeMap<u8, usize>,
    /// The dissolves blending then, and the battle as it stood at each one's start (E17 M5 T5a).
    dissolves: Spans,
    dissolved: BTreeMap<usize, Box<Moment>>,
}

impl Playhead {
    /// The playhead of `timeline` at 0.
    #[must_use]
    pub fn new(timeline: &Timeline) -> Self {
        let mut head = Self {
            at: 0,
            end: timeline.end,
            keys: Sweep::new(timeline.keys.iter().map(|k| k.shown)),
            posed: BTreeMap::new(),
            steps: 0,
            idle: timeline.idle,
            pulse: 0,
            fidgets: timeline.fidgets.clone(),
            fidgeted: BTreeMap::new(),
            posing: timeline.heroes.as_ref().map(crate::hero::Heroes::posing),
            turns: Sweep::new(timeline.turns.iter().map(|(at, _)| *at)),
            turn: None,
            outlines: Sweep::new(timeline.outlines.iter().map(|(at, _)| *at)),
            outlined: None,
            magics: Sweep::new(timeline.magics.iter().map(|k| k.shown)),
            magic: None,
            spots: Sweep::new(timeline.spots.iter().map(|k| k.shown)),
            spot: None,
            fields: Sweep::new(timeline.fields.iter().map(|k| k.shown)),
            field: None,
            walls: Sweep::new(timeline.walls.iter().map(|k| k.shown)),
            wall: None,
            objects: Sweep::new(timeline.objects.iter().map(|k| k.shown)),
            placed: BTreeMap::new(),
            counts: Sweep::new(timeline.counts.iter().map(|k| k.shown)),
            counted: BTreeMap::new(),
            tints: Sweep::new(timeline.tints.iter().map(|k| k.shown)),
            tinted: BTreeMap::new(),
            looks: Sweep::new(timeline.looks.iter().map(|k| k.shown)),
            looked: BTreeMap::new(),
            turrets: Sweep::new(timeline.turrets.iter().map(|k| k.shown)),
            turret: [None; 3],
            blasts: Sweep::new(timeline.blasts.iter().map(|k| k.shown)),
            blast: None,
            blasted: BTreeMap::new(),
            flights: Spans::new(timeline.flights.iter().map(|f| (f.from, f.until()))),
            beams: Spans::new(timeline.beams.iter().map(|b| (b.from, u64::from(b.until)))),
            shakes: Spans::new(timeline.quakes.iter().map(|q| (q.from, q.shaken()))),
            bridges: Spans::new(
                timeline
                    .bridges
                    .iter()
                    .map(|b| (b.from, u64::from(b.until))),
            ),
            befores: Befores::of(timeline),
            bridged: BTreeMap::new(),
            created: Sweep::new(timeline.created.iter().map(|k| k.shown)),
            made: BTreeMap::new(),
            dissolves: Spans::new(timeline.dissolves.iter().map(blending)),
            dissolved: BTreeMap::new(),
        };
        head.reach(timeline, 0);
        head
    }

    /// The time reached.
    #[must_use]
    pub fn at(&self) -> u32 {
        self.at
    }

    /// Moves to `t` of `timeline` (the one it was made for): forward by what showed since, back
    /// by starting over.
    pub fn seek(&mut self, timeline: &Timeline, t: u32) {
        if t < self.at {
            *self = Self::new(timeline);
        }
        self.reach(timeline, t);
    }

    /// Follows `timeline` grown from the one it was read from ([`super::Growing`], E17 M5 T1b):
    /// its lists only appended to, what was added showing from the old end on, which joins what
    /// is still to take. A playhead that reached the old end (the idle steps after it may have
    /// run) or the time something added shows starts over, at the time it reached.
    pub fn extend(&mut self, timeline: &Timeline) {
        let at = self.at;
        let grown = [
            at < self.end,
            self.keys.extend(timeline.keys.iter().map(|k| k.shown), at),
            self.turns
                .extend(timeline.turns.iter().map(|(t, _)| *t), at),
            self.outlines
                .extend(timeline.outlines.iter().map(|(t, _)| *t), at),
            self.magics
                .extend(timeline.magics.iter().map(|k| k.shown), at),
            self.spots
                .extend(timeline.spots.iter().map(|k| k.shown), at),
            self.fields
                .extend(timeline.fields.iter().map(|k| k.shown), at),
            self.walls
                .extend(timeline.walls.iter().map(|k| k.shown), at),
            self.objects
                .extend(timeline.objects.iter().map(|k| k.shown), at),
            self.counts
                .extend(timeline.counts.iter().map(|k| k.shown), at),
            self.tints
                .extend(timeline.tints.iter().map(|k| k.shown), at),
            self.looks
                .extend(timeline.looks.iter().map(|k| k.shown), at),
            self.turrets
                .extend(timeline.turrets.iter().map(|k| k.shown), at),
            self.blasts
                .extend(timeline.blasts.iter().map(|k| k.shown), at),
            self.flights
                .extend(timeline.flights.iter().map(|f| (f.from, f.until())), at),
            self.beams.extend(
                timeline.beams.iter().map(|b| (b.from, u64::from(b.until))),
                at,
            ),
            self.shakes
                .extend(timeline.quakes.iter().map(|q| (q.from, q.shaken())), at),
            self.bridges.extend(
                timeline
                    .bridges
                    .iter()
                    .map(|b| (b.from, u64::from(b.until))),
                at,
            ),
            // The battle before a redraw added is kept as the playhead passes it, even now.
            self.befores.extend(timeline, at),
            self.created
                .extend(timeline.created.iter().map(|k| k.shown), at),
            self.dissolves
                .extend(timeline.dissolves.iter().map(blending), at),
        ];
        if grown.into_iter().all(|g| g) {
            // No idle step after the old end has run: they go on from the new one.
            self.idle = timeline.idle;
            self.fidgets.clone_from(&timeline.fidgets);
            self.end = timeline.end;
        } else {
            *self = Self::new(timeline);
            self.reach(timeline, at);
        }
    }

    /// Moves forward to `t`, keeping the battle as it stands before each drawbridge's redraws and
    /// at each dissolve's start on the way, in their order.
    fn reach(&mut self, timeline: &Timeline, t: u32) {
        while let Some(&(before, kept)) = self.befores.next_by(t) {
            self.befores.next += 1;
            self.advance(timeline, before);
            let moment = Box::new(self.moment(timeline));
            match kept {
                Kept::Bridge(i) => self.bridged.insert(i, moment),
                Kept::Dissolve(i) => self.dissolved.insert(i, moment),
            };
        }
        self.advance(timeline, t);
    }

    /// Moves forward to `t`.
    fn advance(&mut self, timeline: &Timeline, t: u32) {
        for i in self.keys.to(t) {
            latest_of(&mut self.posed, timeline.keys[i].stack, i);
        }
        // The idle steps: the timeline's up to its last command, then those after it (with the
        // viewer's fidgets); each a hero's step and the outline's pulse.
        while let Some(&now) = timeline.steps.get(self.steps).filter(|s| **s <= t) {
            self.steps += 1;
            self.step(timeline, now, false);
        }
        let mut idle = self.idle;
        idle.run(t, |now| self.step(timeline, now, true));
        self.idle = idle;
        if let Some(posing) = self.posing.as_mut() {
            // The heroes' frames since the last idle step.
            posing.keys_to(timeline, t, false);
        }
        for i in self.outlines.to(t) {
            latest(&mut self.outlined, i);
        }
        for i in self.magics.to(t) {
            latest(&mut self.magic, i);
        }
        for i in self.spots.to(t) {
            latest(&mut self.spot, i);
        }
        for i in self.fields.to(t) {
            latest(&mut self.field, i);
        }
        for i in self.walls.to(t) {
            latest(&mut self.wall, i);
        }
        for i in self.objects.to(t) {
            latest_of(&mut self.placed, timeline.objects[i].anchor, i);
        }
        for i in self.counts.to(t) {
            latest_of(&mut self.counted, timeline.counts[i].stack, i);
        }
        for i in self.tints.to(t) {
            latest_of(&mut self.tinted, timeline.tints[i].stack, i);
        }
        for i in self.looks.to(t) {
            latest_of(&mut self.looked, timeline.looks[i].stack, i);
        }
        for i in self.turrets.to(t) {
            if let Some(slot) = self.turret.get_mut(usize::from(timeline.turrets[i].record)) {
                latest(slot, i);
            }
        }
        for i in self.created.to(t) {
            latest_of(&mut self.made, timeline.created[i].member.stack, i);
        }
        for i in self.blasts.to(t) {
            latest(&mut self.blast, i);
            self.blasted
                .entry(timeline.blasts[i].blast)
                .or_default()
                .insert(i);
        }
        self.flights.to(t);
        self.beams.to(t);
        self.shakes.to(t);
        self.bridges.to(t);
        self.dissolves.to(t);
        self.at = t;
    }

    /// An idle step at `now` (`after`: one after the last command, which plays the fidgets).
    fn step(&mut self, timeline: &Timeline, now: u32, after: bool) {
        self.pulse += 1;
        if let Some(fidgets) = self.fidgets.as_mut().filter(|_| after) {
            self.fidgeted.extend(fidgets.step(now));
        }
        if let Some((heroes, posing)) = timeline.heroes.as_ref().zip(self.posing.as_mut()) {
            for i in self.turns.to(now) {
                latest(&mut self.turn, i);
            }
            let active = self.turn.map(|i| timeline.turns[i].1);
            let human = heroes.human(timeline, active);
            heroes.step_at(posing, timeline, now, (false, human));
        }
    }

    /// The battle at the time reached ([`Moment::at`]'s at that time).
    #[must_use]
    pub fn moment(&self, timeline: &Timeline) -> Moment {
        let t = self.at;
        let screen = timeline.screen(t);
        let mut poses: BTreeMap<u8, Pose> = timeline
            .members
            .iter()
            .map(|m| (m.stack, m.start))
            .collect();
        poses.extend(self.posed.iter().map(|(&s, &i)| (s, timeline.keys[i].pose)));
        poses.extend(&self.fidgeted);
        Moment {
            poses,
            active: self.outlined.and_then(|i| timeline.outlines[i].1),
            pulse: u32::try_from(self.pulse).unwrap_or(u32::MAX),
            projectile: self.flights.open().find_map(|i| timeline.flights[i].at(t)),
            ray: self.beams.open().find_map(|i| timeline.beams[i].at(t)),
            heroes: timeline
                .heroes
                .as_ref()
                .zip(self.posing.as_ref())
                .map_or([None; 2], |(h, p)| h.poses(p)),
            magic: self.magic.and_then(|i| timeline.magics[i].shows()),
            spot: self.spot.and_then(|i| timeline.spots[i].shows()),
            objects: self
                .placed
                .iter()
                .filter_map(|(&a, &i)| timeline.objects[i].object.clone().map(|o| (a, o)))
                .filter(|(_, o)| sees(screen, o))
                .collect(),
            field: self.field.and_then(|i| timeline.fields[i].shows()),
            counts: self
                .counted
                .iter()
                .map(|(&s, &i)| (s, timeline.counts[i].count))
                .collect(),
            hues: self
                .tinted
                .iter()
                .filter_map(|(&s, &i)| timeline.tints[i].hue.map(|h| (s, h)))
                .collect(),
            looks: self
                .looked
                .iter()
                .filter_map(|(&s, &i)| timeline.looks[i].look.map(|l| (s, l)))
                .collect(),
            marked: None,
            area: Vec::new(),
            walls: self.wall.map(|i| timeline.walls[i].walls),
            turrets: self.turret.map(|k| k.map(|i| timeline.turrets[i].pose)),
            blast: self.blast(timeline),
            bridge: self
                .bridges
                .open()
                .next()
                .and_then(|i| self.bridged.get(&i).cloned()),
            shake: self.shakes.open().find_map(|i| timeline.quakes[i].steps(t)),
            shade: BTreeSet::new(),
            created: self
                .made
                .iter()
                .map(|(&s, &i)| (s, timeline.created[i].stack.clone()))
                .collect(),
            dissolve: self.dissolves.open().next().and_then(|i| {
                let d = &timeline.dissolves[i];
                let kept = self.dissolved.get(&i).zip(d.step_at(t));
                kept.map(|(before, step)| Dissolve {
                    before: before.clone(),
                    step,
                    area: d.area.clone(),
                })
            }),
        }
    }

    /// The wall shot's explosion showing ([`Timeline::blast`]).
    fn blast(&self, timeline: &Timeline) -> Option<Blast> {
        let last = &timeline.blasts[self.blast?];
        let frame = last.frame?;
        let frames = if last.restored {
            vec![(frame, last.walls)]
        } else {
            self.blasted
                .get(&last.blast)
                .into_iter()
                .flatten()
                .filter_map(|&i| {
                    timeline.blasts[i]
                        .frame
                        .map(|f| (f, timeline.blasts[i].walls))
                })
                .collect()
        };
        Some(Blast {
            def: last.def.clone(),
            landings: last.landings.clone(),
            frames,
        })
    }
}

/// One list's keys in the order they show: each one's time and place in the list, sorted (the
/// place breaks a tie); the next to take.
#[derive(Debug, Clone, Default)]
struct Sweep {
    order: Vec<(u32, usize)>,
    next: usize,
}

impl Sweep {
    /// The keys showing at `times`, in the list's order.
    fn new(times: impl Iterator<Item = u32>) -> Self {
        let mut order: Vec<(u32, usize)> = times.zip(0..).collect();
        order.sort_unstable();
        Self { order, next: 0 }
    }

    /// Adds the keys of the list `times` past those it holds (its old length on), to be taken
    /// in the order they show; whether each shows after `at` (else nothing is added).
    fn extend(&mut self, times: impl Iterator<Item = u32>, at: u32) -> bool {
        let added: Vec<(u32, usize)> = times.zip(0..).skip(self.order.len()).collect();
        let after = added.iter().all(|&(t, _)| t > at);
        if after {
            self.order.extend(added);
            self.order[self.next..].sort_unstable();
        }
        after
    }

    /// The places of the keys shown by `t` not taken yet, in the order they show; taken.
    fn to(&mut self, t: u32) -> impl Iterator<Item = usize> + '_ {
        let from = self.next;
        let rest = self.order.get(from..).unwrap_or_default();
        self.next = from + rest.partition_point(|(at, _)| *at <= t);
        self.order[from..self.next].iter().map(|&(_, i)| i)
    }
}

/// A battle the playhead keeps as it passes its time: before a drawbridge's redraws, at a
/// dissolve's start (E17 M5 T5a), each by its place in its list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kept {
    Bridge(usize),
    Dissolve(usize),
}

/// The battles kept ([`Kept`]) in the order of their times (a drawbridge's first at a time), the
/// next to take, and how many of each list's are in.
#[derive(Debug, Clone, Default)]
struct Befores {
    order: Vec<(u32, Kept)>,
    next: usize,
    bridges: usize,
    dissolves: usize,
}

impl Befores {
    /// The battles `timeline` keeps.
    fn of(timeline: &Timeline) -> Self {
        let mut order: Vec<(u32, Kept)> = Self::added(timeline, 0, 0).collect();
        order.sort_unstable();
        Self {
            order,
            next: 0,
            bridges: timeline.bridges.len(),
            dissolves: timeline.dissolves.len(),
        }
    }

    /// The battles `timeline` keeps past its first `bridges` drawbridge spans and `dissolves`
    /// dissolves.
    fn added(
        timeline: &Timeline,
        bridges: usize,
        dissolves: usize,
    ) -> impl Iterator<Item = (u32, Kept)> + '_ {
        let b = timeline.bridges.iter().enumerate().skip(bridges);
        let d = timeline.dissolves.iter().enumerate().skip(dissolves);
        b.map(|(i, b)| (b.from.saturating_sub(1), Kept::Bridge(i)))
            .chain(d.map(|(i, d)| (d.start, Kept::Dissolve(i))))
    }

    /// Adds the times of `timeline`'s drawbridge spans and dissolves past those it holds; whether
    /// each comes after `at` (a drawbridge's at it may: the battle just before its first redraw
    /// is kept even now); else nothing is added.
    fn extend(&mut self, timeline: &Timeline, at: u32) -> bool {
        let added: Vec<(u32, Kept)> = Self::added(timeline, self.bridges, self.dissolves).collect();
        let after = added.iter().all(|&(t, kept)| match kept {
            Kept::Bridge(_) => t >= at.saturating_sub(1),
            Kept::Dissolve(_) => t > at,
        });
        if after {
            self.order.extend(added);
            self.order[self.next..].sort_unstable();
            self.bridges = timeline.bridges.len();
            self.dissolves = timeline.dissolves.len();
        }
        after
    }

    /// The next battle to keep, if its time comes by `t`.
    fn next_by(&self, t: u32) -> Option<&(u32, Kept)> {
        self.order.get(self.next).filter(|(at, _)| *at <= t)
    }
}

/// One list's spans, each `[from, until)` by its place in the list: in the order they begin, and
/// those open at the time reached.
#[derive(Debug, Clone, Default)]
struct Spans {
    starts: Sweep,
    ends: Vec<u64>,
    open: BTreeSet<usize>,
}

impl Spans {
    /// The spans of a list, in its order.
    fn new(spans: impl Iterator<Item = (u32, u64)>) -> Self {
        let (starts, ends): (Vec<u32>, Vec<u64>) = spans.unzip();
        Self {
            starts: Sweep::new(starts.into_iter()),
            ends,
            open: BTreeSet::new(),
        }
    }

    /// Adds the spans of the list `spans` past those it holds (the ends of all read again: an
    /// open one's may be known now); whether each added begins after `at` (else the playhead
    /// starts over).
    fn extend(&mut self, spans: impl Iterator<Item = (u32, u64)>, at: u32) -> bool {
        let (starts, ends): (Vec<u32>, Vec<u64>) = spans.unzip();
        self.ends = ends;
        self.starts.extend(starts.into_iter(), at)
    }

    /// Moves to `t`: the spans begun by then open, those ended by then close.
    fn to(&mut self, t: u32) {
        self.open.extend(self.starts.to(t));
        let ends = &self.ends;
        self.open.retain(|&i| ends[i] > u64::from(t));
    }

    /// The places of the spans open, in the list's order.
    fn open(&self) -> impl Iterator<Item = usize> + '_ {
        self.open.iter().copied()
    }
}

/// A dissolve's blending span (E17 M5 T5a, [`super::DissolveKey::step_at`]).
fn blending(d: &super::DissolveKey) -> (u32, u64) {
    (d.start + 1, u64::from(d.end()))
}

/// `slot` holds the later of its key and the key at `i` in the list.
fn latest(slot: &mut Option<usize>, i: usize) {
    if slot.is_none_or(|s| s < i) {
        *slot = Some(i);
    }
}

/// `of`'s slot for `key` holds the later of its key and the key at `i` in the list.
fn latest_of(of: &mut BTreeMap<u8, usize>, key: u8, i: usize) {
    let slot = of.entry(key).or_insert(i);
    *slot = (*slot).max(i);
}
