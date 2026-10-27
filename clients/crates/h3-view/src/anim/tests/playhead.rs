//! E17 M5 T1a: the playhead against [`Moment::at`], the reference: at every change of every
//! replay of the tests (their `timeline` and `fidgeting` check it) and 1 ms after each, going
//! forward from 0 and back again; keys the lists hold out of the order they show. T1b: every
//! replay grown a command at a time, a playhead extended along. T2b: before each command given
//! at a time, the timeline waiting for it ([`Growing::waiting`]).

use alloc::collections::BTreeMap;
use alloc::vec;

use h3_protocol::wire::{Observation, Side};
use h3_render::Bugs;

use super::{PIKEMAN, at, layout, looks, magic, moved, play, snapshot, tables};
use crate::anim::{
    Casts, Command, Growing, Key, Look, Playhead, Pose, Speed, Stage, Timeline, TurretKey,
    TurretPose,
};
use crate::hero::Heroes;
use crate::scene::Moment;

/// How far past its end a timeline is checked: its idle steps, the fidgets and the heroes' idle
/// motion (more than 4 500 ms after their clocks) run on.
const PAST_THE_END: u32 = 6_000;

/// `timeline`'s playhead shows [`Moment::at`]'s moment at every change up to [`PAST_THE_END`]
/// after its end and 1 ms after each, going forward from 0; and again after going back.
pub(crate) fn agrees(timeline: &Timeline) {
    let times = timeline.changes(timeline.end + PAST_THE_END);
    let mut head = Playhead::new(timeline);
    let mut check = |t: u32| {
        head.seek(timeline, t);
        assert_eq!(head.at(), t);
        assert_eq!(head.moment(timeline), Moment::at(timeline, t), "at {t} ms");
    };
    for &t in &times {
        check(t);
        check(t + 1);
    }
    check(times[times.len() / 2]);
}

/// The battle `whole` is ([`super::timeline`]'s at once) grown a command at a time (E17 M5 T1b),
/// each the computer's when the next has no time, as at once: each command's timeline begins
/// with the one before it (its lists only appended to); before its end, a playhead extended
/// along shows the whole battle's moments at every change; finished, it is `whole`.
pub(crate) fn grows(
    (snapshot, commands, looks): (&Observation, &[Command<'_>], &BTreeMap<u16, Look>),
    stage: Stage<'_>,
    (speed, seed): (Speed, Option<u32>),
    whole: &Timeline,
) {
    let mut growing = Growing::new(snapshot, looks, stage, speed, seed).unwrap();
    let mut before = growing.timeline().clone();
    let mut head = Playhead::new(&before);
    for (i, command) in commands.iter().enumerate() {
        // A command given at a time (a player's) waited for at the end (E17 M5 T2b): before it,
        // the timeline waiting shows the whole battle's moments, a playhead's and at once.
        if let Some(given) = command.at {
            let tail = growing.waiting();
            assert!(begins(&tail, &before), "command {i}, waiting");
            let mut head = Playhead::new(&tail);
            let times = tail.changes(given);
            for t in times
                .into_iter()
                .filter(|t| (before.end..given).contains(t))
            {
                head.seek(&tail, t);
                let moment = Moment::at(whole, t);
                assert_eq!(head.moment(&tail), moment, "at {t} ms, waiting for {i}");
                assert_eq!(Moment::at(&tail, t), moment, "at {t} ms, waiting for {i}");
            }
        }
        let computer = commands.get(i + 1).is_some_and(|c| c.at.is_none());
        growing.push(command, computer);
        let now = growing.timeline();
        assert_eq!(growing.commands(), i + 1);
        assert!(begins(now, &before), "command {i}");
        head.extend(now);
        let times = now.changes(now.end);
        for t in times
            .into_iter()
            .filter(|t| (before.end..now.end).contains(t))
        {
            head.seek(now, t);
            assert_eq!(
                head.moment(now),
                Moment::at(whole, t),
                "at {t} ms, command {i}"
            );
        }
        before = now.clone();
    }
    assert_eq!(&growing.finish(), whole);
}

/// Whether `now`'s lists begin with `before`'s.
fn begins(now: &Timeline, before: &Timeline) -> bool {
    fn starts<T: PartialEq>(now: &[T], before: &[T]) -> bool {
        now.get(..before.len()) == Some(before)
    }
    now.members == before.members
        && starts(&now.keys, &before.keys)
        && starts(&now.turns, &before.turns)
        && starts(&now.flights, &before.flights)
        && starts(&now.beams, &before.beams)
        && starts(&now.magics, &before.magics)
        && starts(&now.spots, &before.spots)
        && starts(&now.objects, &before.objects)
        && starts(&now.fields, &before.fields)
        && starts(&now.hero_keys, &before.hero_keys)
        && starts(&now.counts, &before.counts)
        && starts(&now.tints, &before.tints)
        && starts(&now.outlines, &before.outlines)
        && starts(&now.steps, &before.steps)
        && starts(&now.cues, &before.cues)
        && starts(&now.actions, &before.actions)
        && starts(&now.log, &before.log)
        && starts(&now.turrets, &before.turrets)
        && starts(&now.walls, &before.walls)
        && starts(&now.blasts, &before.blasts)
        // A drawbridge span open before (its end not known) has its end now.
        && now.bridges.len() >= before.bridges.len()
        && now.bridges.iter().zip(&before.bridges).all(|(n, b)| {
            n.from == b.from && (n.until == b.until || b.until == u32::MAX)
        })
        && starts(&now.quakes, &before.quakes)
        && now.end >= before.end
}

#[test]
fn a_growing_timeline_carries_its_heroes() {
    let (layout, tables, magic, casts) = (layout(), tables(), magic(), Casts::default());
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables.battle.groups,
        magic: &magic,
        casts: &casts,
        bugs: Bugs::Original,
        siege: None,
    };
    let o = snapshot(vec![at(0, Side::Attacker, PIKEMAN, 50, false)]);
    let heroes = Heroes::new([None, None], [true, true], tables.battle.hero_groups, None);
    let growing = Growing::new(&o, &looks(), stage, Speed::default(), None)
        .unwrap()
        .with_heroes(heroes.clone());
    assert_eq!(growing.commands(), 0);
    assert_eq!(growing.timeline().heroes, Some(heroes.clone()));
    assert_eq!(growing.finish().heroes, Some(heroes));
}

#[test]
fn a_playhead_past_the_old_end_starts_over_as_the_timeline_grows() {
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 50, false),
        at(1, Side::Defender, PIKEMAN, 60, false),
    ]);
    let whole = play(&o, &[(None, vec![moved(0, 50, 51)])]);
    // A playhead past the end of an earlier timeline (the idle steps after it ran), and one
    // that reached a key the grown timeline adds before it: both start over.
    let mut early = whole.clone();
    early.keys.truncate(1);
    early.end = 1;
    let mut head = Playhead::new(&early);
    head.seek(&early, whole.end);
    head.extend(&whole);
    assert_eq!(head.at(), whole.end);
    assert_eq!(head.moment(&whole), Moment::at(&whole, whole.end));
    let mut head = Playhead::new(&early);
    let shown = whole.keys[1].shown;
    early.end = whole.end;
    head.seek(&early, shown);
    head.extend(&whole);
    assert_eq!(head.moment(&whole), Moment::at(&whole, shown));
}

#[test]
fn keys_held_out_of_the_order_they_show_keep_the_last_in_the_list() {
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 50, false),
        at(1, Side::Defender, PIKEMAN, 60, false),
    ]);
    let mut timeline = play(&o, &[(None, vec![moved(0, 50, 51)])]);
    let walked = timeline.keys.len();
    assert!(walked > 2);
    // A pose the list holds last but that shows at 1 ms: from then on it shows over every
    // earlier one in the list, whenever those show.
    let pose = Pose {
        frame: 7,
        ..timeline.keys[0].pose
    };
    timeline.keys.push(Key {
        stack: 0,
        drawn: 1,
        shown: 1,
        pose,
    });
    // An outline the list holds last, shown at 1 ms; a turret record no turret has.
    timeline.outlines.push((1, Some(1)));
    timeline.turrets.push(TurretKey {
        record: 3,
        drawn: 1,
        shown: 1,
        pose: TurretPose {
            group: 0,
            frame: 0,
            right: false,
        },
    });
    let end = Moment::at(&timeline, timeline.end);
    assert_eq!(end.poses[&0].frame, 7);
    assert_eq!(end.active, Some(1));
    assert_eq!(end.turrets, [None; 3]);
    agrees(&timeline);
}
