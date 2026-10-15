use super::*;
use alloc::vec;

/// The heads' pixels of a tick.
fn heads(tick: &[Stroke]) -> Vec<[i32; 2]> {
    tick.iter().map(|s| [s.x, s.y]).collect()
}

#[test]
fn a_ray_to_the_right_steps_one_pixel_at_a_time() {
    // d = 400: 41 steps a tick; the head enters a new pixel at every step (the f32 aim just
    // above π/2 leaves the row where it is).
    let ticks = trace([100, 300], [500, 300]);
    assert_eq!(ticks.len(), 10);
    let first = ticks.first().unwrap();
    assert_eq!(
        heads(first),
        (101..=141).map(|x| [x, 300]).collect::<Vec<_>>()
    );
    assert_eq!(
        first.first(),
        Some(&Stroke {
            x: 101,
            y: 300,
            lo: -2,
            hi: 2,
            column: true
        })
    );
    // Thinner once past halfway: aimed after tick 5 at x = 305 (205 of 400 gone).
    assert!(
        ticks
            .iter()
            .take(5)
            .flatten()
            .all(|s| (s.lo, s.hi) == (-2, 2))
    );
    assert!(
        ticks
            .iter()
            .skip(5)
            .flatten()
            .all(|s| (s.lo, s.hi) == (-2, 1))
    );
    // It ends 2 pixels short of the target.
    let last = ticks.last().unwrap();
    assert_eq!(heads(last).first(), Some(&[470, 300]));
    assert_eq!(heads(last).last(), Some(&[498, 300]));
}

#[test]
fn a_ray_to_the_left_and_one_down_the_screen() {
    let left = trace([500, 300], [100, 300]);
    assert_eq!(heads(left.first().unwrap()).first(), Some(&[499, 300]));
    assert_eq!(heads(left.last().unwrap()).last(), Some(&[102, 300]));
    // Straight down: the aim is 0, sideways cross-sections, 35 steps a tick (d = 300).
    let down = trace([300, 100], [300, 400]);
    let first = down.first().unwrap();
    assert_eq!(
        heads(first),
        (101..=135).map(|y| [300, y]).collect::<Vec<_>>()
    );
    assert!(!first.first().unwrap().column);
    assert_eq!(heads(down.last().unwrap()).last(), Some(&[300, 398]));
}

#[test]
fn a_slanted_ray_ends_by_its_target() {
    for (from, to) in [
        ([60, 400], [700, 150]),
        ([700, 150], [60, 400]),
        ([128, 222], [661, 223]),
        ([400, 50], [380, 520]),
    ] {
        let ticks = trace(from, to);
        let all: Vec<Stroke> = ticks.iter().flatten().copied().collect();
        // Every stroke is a pixel the head newly entered, one step from the last.
        for pair in all.windows(2) {
            let [a, b] = [pair[0], pair[1]];
            assert_ne!([a.x, a.y], [b.x, b.y]);
            assert!((a.x - b.x).abs() <= 1 && (a.y - b.y).abs() <= 1);
        }
        let end = all.last().unwrap();
        assert!(
            (end.x - to[0]).abs() + (end.y - to[1]).abs() <= 3,
            "{from:?} {to:?}"
        );
        assert!(ticks.iter().take(ticks.len() - 1).all(|t| !t.is_empty()));
    }
}

#[test]
fn points_off_the_battlefield_are_brought_onto_it() {
    let ticks = trace([-40, 700], [900, 700]);
    let first = ticks.first().unwrap();
    assert_eq!(heads(first).first(), Some(&[1, 555]));
    assert_eq!(heads(ticks.last().unwrap()).last(), Some(&[797, 555]));
}

#[test]
fn a_head_beyond_the_edges_stays_on_them() {
    // Aimed past the battlefield, as only an overshoot near its edge could be.
    for (at, to, inside) in [
        ([1, 300], [-60, 300], [0, 300]),
        ([798, 300], [860, 300], [RIGHT, 300]),
        ([300, 1], [300, -60], [300, 0]),
        ([300, 554], [300, 620], [300, BOTTOM]),
    ] {
        let mut head = Head::new(at, to);
        let (strokes, done) = head.advance(4);
        assert!(!done);
        assert_eq!(heads(&strokes), vec![inside]);
        assert_eq!(head.at, inside);
    }
}

#[test]
fn a_head_ends_when_it_moves_away() {
    // Within 15 of the target, then farther than the least distance plus one.
    let mut head = Head::new([300, 300], [306, 303]);
    head.step = [ONE, 0];
    let (strokes, done) = head.advance(20);
    assert!(done);
    assert_eq!(heads(&strokes).last(), Some(&[308, 300]));
}

#[test]
fn a_head_farther_than_the_start_has_gone_nowhere() {
    let mut head = Head::new([300, 300], [310, 300]);
    head.width = 1;
    head.at = [250, 300];
    head.aim();
    assert_eq!(head.width, START_WIDTH);
}

#[test]
fn the_thickness_follows_the_way_gone() {
    assert_eq!(width(5, 4, 0), 5);
    assert_eq!(width(5, 4, ONE / 2 - 1), 5);
    assert_eq!(width(5, 4, ONE / 2), 4);
    assert_eq!(width(5, 4, ONE), 3);
    assert_eq!(width(3, 3, ONE), 3);
    assert_eq!(width(2, 5, ONE / 2), 4);
    assert_eq!(width(1, 0, ONE), 1);
}

#[test]
fn cross_sections_colour_from_the_edges_inward() {
    let stroke = |lo, hi, column| Stroke {
        x: 10,
        y: 20,
        lo,
        hi,
        column,
    };
    let entries = |s: Stroke| s.pixels().map(|(_, e)| e).collect::<Vec<_>>();
    assert_eq!(entries(stroke(-2, 2, true)), [0, 1, 2, 1, 0]);
    assert_eq!(entries(stroke(-2, 1, true)), [0, 1, 1, 0]);
    assert_eq!(entries(stroke(-1, 1, false)), [0, 1, 0]);
    let points: Vec<_> = stroke(-1, 1, false).pixels().map(|(p, _)| p).collect();
    assert_eq!(points, [[9, 20], [10, 20], [11, 20]]);
    // Clipped to the battlefield.
    let corner = Stroke {
        x: 0,
        y: BOTTOM,
        lo: -2,
        hi: 2,
        column: true,
    };
    assert_eq!(corner.pixels().count(), 3);
    let side = Stroke {
        column: false,
        ..corner
    };
    assert_eq!(
        side.pixels().map(|(p, _)| p).collect::<Vec<_>>(),
        [[0, BOTTOM], [1, BOTTOM], [2, BOTTOM]]
    );
}

#[test]
fn roundings_are_the_x87s() {
    assert_eq!(round(0, 24), 0);
    assert_eq!(round(5, 24), 5);
    // 2^25 + 1 to 24 bits: the tie goes to even (down), 2^25 + 3 up.
    assert_eq!(round((1 << 25) + 1, 24), 1 << 25);
    assert_eq!(round((1 << 25) + 3, 24), (1 << 25) + 4);
    assert_eq!(round(-((1 << 25) + 3), 24), -((1 << 25) + 4));
    assert_eq!(round((1 << 26) + 3, 24), 1 << 26);
    assert_eq!(round((1 << 26) + 5, 24), (1 << 26) + 8);
    assert_eq!(f32_ratio(0, 7), 0);
    // Ties at the 24th bit go to even: 1 + 2^-24 down, 1 + 3 × 2^-24 up.
    assert_eq!(f32_ratio((1 << 24) + 1, 1 << 24), ONE);
    assert_eq!(f32_ratio((1 << 24) + 3, 1 << 24), ONE + (ONE >> 22));
    assert_eq!(f32_ratio(1, 2), ONE / 2);
    assert_eq!(f32_ratio(7, 7), ONE);
    // 1/3 as an f32: 0xAAAAAB × 2^-25.
    assert_eq!(f32_ratio(1, 3), 0xAA_AAAB << (FRAC - 25));
    // 2/3: 0xAAAAAB × 2^-24 (the remainder above half rounds up).
    assert_eq!(f32_ratio(2, 3), 0xAA_AAAB << (FRAC - 24));
    assert_eq!(trunc(-(ONE + 1)), -1);
    assert_eq!(trunc(ONE - 1), 0);
}

#[test]
fn the_series_are_exact_enough() {
    let close = |a: i128, b: i128| (a - b).abs() < 64;
    // atan2 in every direction, against the angles known (π/4 = 905_502_432_259_640_355.3).
    let quarter = 905_502_432_259_640_355;
    assert_eq!(atan2(0, 0), 0);
    assert_eq!(atan2(0, 5), 0);
    assert!(close(atan2(5, 5), quarter));
    assert!(close(atan2(5, -5), 3 * quarter));
    assert!(close(atan2(-5, 5), -quarter));
    assert!(close(atan2(-5, -5), -3 * quarter));
    assert!(close(atan2(7, 0), 2 * quarter));
    assert!(close(atan2(0, -7), PI));
    // atan(1/2) (× 2^60: 534_549_298_976_576_474).
    assert!(close(atan2(1, 2), 534_549_298_976_576_474));
    assert!(close(atan2(2, 1), 2 * quarter - 534_549_298_976_576_474));
    // sin and cos of π/6 (cos: √3/2 × 2^60 = 998_459_311_558_907_193.8).
    let [sin, cos] = sin_cos(PI / 6);
    assert!(close(sin, ONE / 2));
    assert!(close(cos, 998_459_311_558_907_194));
    assert_eq!(sin_cos(0), [0, ONE]);
    let [sin, cos] = sin_cos(-PI);
    assert!(sin.abs() < 64 && close(cos, -ONE));
}
