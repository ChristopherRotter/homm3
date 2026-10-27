//! E17 M5 T1a: the playhead against [`Moment::at`], the reference: at every change of every
//! replay of the tests (their `timeline` and `fidgeting` check it) and 1 ms after each, going
//! forward from 0 and back again; keys the lists hold out of the order they show.

use alloc::vec;

use h3_protocol::wire::Side;

use super::{PIKEMAN, at, moved, play, snapshot};
use crate::anim::{Key, Playhead, Pose, Timeline, TurretKey, TurretPose};
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
