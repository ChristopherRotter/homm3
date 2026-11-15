//! The samples' tests (E27 M2 T6a).

use super::{HANDLES, Played, Samples};

#[test]
fn a_sample_takes_the_first_done_handle_else_the_next_in_turn() {
    let mut s = Samples::default();
    // `menu-sound`: New Game's press at 0 ms, Back's at 100 ms while the first plays (151 ms).
    let first = s.play(0, 151);
    let second = s.play(100, 151);
    assert_eq!(
        (first, second),
        (
            Played {
                time: 0,
                channel: 0,
                length: 151
            },
            Played {
                time: 100,
                channel: 1,
                length: 151
            }
        )
    );
    // Once the first is done its handle is the first free again: the harness's channel 0.
    assert_eq!(s.play(151, 10).channel, 0);
    // With every handle playing, the next in turn is cut: the first, then the second.
    let mut s = Samples::default();
    for k in 0..HANDLES {
        assert_eq!(s.play(0, 1000).channel, k);
    }
    assert_eq!(s.play(10, 1000).channel, 0);
    assert_eq!(s.play(20, 1000).channel, 1);
    assert_eq!(s.played().len(), HANDLES + 2);
    // The turn wraps round the range.
    let mut s = Samples::default();
    for _ in 0..=2 * HANDLES {
        let _ = s.play(0, 1000);
    }
    assert_eq!(s.play(0, 1000).channel, 1);
}
