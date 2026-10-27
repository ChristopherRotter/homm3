use alloc::vec;

use h3_render::Bugs;

use super::*;
use crate::scene::ViewAt;
use crate::timeline::tests::{recorded, rules};
use crate::timeline::{Direction, play};

/// The changes of what the screen shows in a headless play's states: each state whose draw or
/// cursor differs from the one before (the first always).
fn headless(id: &str) -> Vec<Change> {
    let (start, actions, ms) = recorded(id);
    let states = play(rules(Bugs::Original), &start, &actions, ms).unwrap();
    let mut changes: Vec<Change> = Vec::new();
    for s in states {
        if changes
            .last()
            .is_none_or(|c| c.drawn != s.drawn || c.cursor != s.cursor)
        {
            changes.push(Change {
                t: s.t,
                drawn: s.drawn,
                cursor: s.cursor,
            });
        }
    }
    changes
}

/// The changes a replay gives up to `until`, each asked for after the one before.
fn replayed(replay: &mut Replay, until: u32) -> Vec<Change> {
    let mut changes = vec![replay.at(0).unwrap()];
    while let Some(next) = replay.after(changes.last().unwrap().t, 1_000).unwrap() {
        if next.t > until {
            break;
        }
        changes.push(next);
    }
    changes
}

#[test]
fn its_changes_are_the_headless_plays() {
    for id in ["scroll-limits", "walk-1", "walk-sail"] {
        let (start, actions, ms) = recorded(id);
        let mut replay = Replay::new(rules(Bugs::Original), &start, actions).unwrap();
        let want = headless(id);
        assert!(want.len() >= 20, "{id}");
        assert_eq!(replayed(&mut replay, ms), want, "{id}");
        // Again from the start.
        replay.restart();
        assert_eq!(replay.ran(), 0);
        assert_eq!(replayed(&mut replay, ms), want, "{id}");
    }
}

/// A 36-tile map standing still, the pointer parked.
fn idle() -> Replay {
    let start = Start {
        view: ViewAt { x: 9, y: 10, z: 0 },
        size: 36,
        scroll_speed: 1,
        hero_speed: 2,
        pointer: [700, 300],
        hero: None,
        paths: Vec::new(),
    };
    let actions = BTreeMap::from([
        (0, vec![Action::Scroll(Direction::East)]),
        (500, vec![Action::Scroll(Direction::East)]),
    ]);
    Replay::new(rules(Bugs::Fixed), &start, actions).unwrap()
}

#[test]
fn the_change_shown_at_a_time_and_the_next() {
    let mut r = idle();
    let first = r.at(0).unwrap();
    // The action at 0 is not played.
    assert_eq!((first.t, first.drawn.view.x, first.drawn.frames), (0, 9, 0));
    // The redraws every 180 ms; the scroll at 500 drawn at once, in the redraw's place.
    assert_eq!(r.at(100).unwrap(), first);
    assert_eq!(r.ran(), 100);
    assert_eq!(r.after(0, 10).unwrap(), None);
    assert_eq!(r.ran(), 100);
    let at_180 = r.after(0, 1_000).unwrap().unwrap();
    assert_eq!((at_180.t, at_180.drawn.frames), (180, 1));
    assert_eq!(r.ran(), 180);
    let scrolled = r.at(500).unwrap();
    assert_eq!((scrolled.t, scrolled.drawn.view.x), (500, 11));
    // Run ahead, an earlier time is still kept.
    assert_eq!(r.at(400).unwrap().t, 360);
    assert_eq!(r.at(200).unwrap(), at_180);
    // What comes after one shown forgets the ones before it.
    assert_eq!(r.after(400, 1_000).unwrap().unwrap().t, 500);
    assert_eq!(r.at(200).unwrap().t, 360);
    assert_eq!(r.after(500, 1_000).unwrap().unwrap().t, 720);
    assert_eq!(r.at(0).unwrap().t, 500);
    r.restart();
    assert_eq!(r.at(0).unwrap(), first);
}

#[test]
fn what_it_cannot_play() {
    let mut start = recorded("scroll-0").0;
    start.pointer = [300, 300];
    assert_eq!(
        Replay::new(rules(Bugs::Original), &start, BTreeMap::new()).unwrap_err(),
        Unplayable::OverView([300, 300])
    );
    start.pointer = [700, 300];
    let actions = BTreeMap::from([(5, vec![Action::Pointer([300, 300])])]);
    let mut r = Replay::new(rules(Bugs::Original), &start, actions).unwrap();
    assert_eq!(r.at(10), Err((5, Unplayable::OverView([300, 300]))));
    assert_eq!(r.ran(), 4);
}
