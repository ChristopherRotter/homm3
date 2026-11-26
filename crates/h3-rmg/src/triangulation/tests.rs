//! The triangulation's own tests (RMG-ZON-009): the frame, Delaunay's condition over seeded
//! points, the vertex and edge cases of the insertion, the circumcentres and the faults.

use alloc::vec::Vec;

use h3_kernel::rng::CrtRand;

use super::*;

fn vertex(x: i32, y: i32, data: Option<usize>) -> Vertex {
    Vertex { x, y, data }
}

/// Every record's rings and twins consistent, and each record of the vector distinct.
fn check_rings(t: &Triangulation) {
    for &e in &t.order {
        assert_eq!(t.sym(t.sym(e)), e);
        assert_ne!(t.sym(e), e);
        assert_eq!(t.oprev(t.onext(e)), e);
        assert_eq!(t.onext(t.oprev(e)), e);
        assert_eq!(t.org(t.onext(e)), t.org(e));
    }
    let mut sorted = t.order.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), t.order.len());
}

/// Each face on the left of a record, as three records, once (the frame's outside left out).
fn triangles(t: &Triangulation) -> Vec<[Edge; 3]> {
    let mut out = Vec::new();
    for &e in &t.order {
        let (b, c) = (t.lnext(e), t.lnext(t.lnext(e)));
        if t.lnext(c) == e && e < b && e < c && tri_area(t.org(e), t.org(b), t.org(c)) > 0 {
            out.push([e, b, c]);
        }
    }
    out
}

// covers: RMG-ZON-009
#[test]
fn the_frame_is_the_originals() {
    let t = Triangulation::new();
    check_rings(&t);
    // Four sides and the diagonal, each pair as made.
    assert_eq!(t.records(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(
        (t.org(t.start), t.dest(t.start)),
        ([-200, -200], [400, -200])
    );
    assert_eq!((t.org(8), t.dest(8)), ([-200, -200], [400, 400]));
    assert_eq!(triangles(&t).len(), 2);
    for &e in t.records() {
        assert_eq!((t.origin(e).data, t.centre(e)), (None, [-1, -1]));
    }
}

// covers: RMG-ZON-009
#[test]
fn seeded_points_keep_delaunays_condition() {
    for seed in 0..40 {
        let mut rand = CrtRand::new(seed);
        let mut t = Triangulation::new();
        let mut points = Vec::new();
        let count = 3 + seed as usize % 40;
        for k in 0..count {
            // On a map of 72 and around it, as step 2's points lie; small ranges make ties.
            let range = if seed % 3 == 0 { 9 } else { 200 };
            let x = (rand.rand() % range) as i32 - 50;
            let y = (rand.rand() % range) as i32 - 50;
            let new = t.insert(vertex(x, y, Some(k)));
            assert_eq!(new, !points.contains(&[x, y]), "seed {seed}: ({x}, {y})");
            if new {
                points.push([x, y]);
            }
        }
        check_rings(&t);
        let faces = triangles(&t);
        // Euler: a triangulated square with n points inside has 2n + 2 triangles.
        assert_eq!(faces.len(), 2 * points.len() + 2, "seed {seed}");
        for [a, b, c] in faces {
            let (a, b, c) = (t.org(a), t.org(b), t.org(c));
            for &p in &points {
                assert!(
                    !in_circle(a, b, c, p),
                    "seed {seed}: {p:?} in ({a:?}, {b:?}, {c:?})"
                );
            }
        }
        for &p in &points {
            let e = t.find(p);
            assert_eq!(t.org(e), p);
        }
    }
}

// covers: RMG-ZON-009
#[test]
fn a_point_at_a_vertex_is_dropped_and_one_on_an_edge_splits_it() {
    let mut t = Triangulation::new();
    // (0, 0) lies on the diagonal a→c: that edge is deleted, four edges made.
    assert!(t.insert(vertex(0, 0, Some(0))));
    check_rings(&t);
    assert!(!t.records().contains(&8) && !t.records().contains(&9));
    assert_eq!(triangles(&t).len(), 4);
    // Found at a destination: the twin, whose origin it is.
    assert_eq!(t.org(t.find([-200, -200])), [-200, -200]);
    let before = t.records().len();
    assert!(!t.insert(vertex(0, 0, Some(1))));
    assert_eq!(t.records().len(), before);
    let e = t.find([0, 0]);
    assert_eq!(t.origin(e).data, Some(0));
}

// covers: RMG-ZON-009
#[test]
fn the_finish_gives_each_zone_triangle_its_circumcentre_from_its_first_corner() {
    let mut t = Triangulation::new();
    for (k, (x, y)) in [(0, 0), (100, 0), (0, 100), (101, 103)]
        .into_iter()
        .enumerate()
    {
        assert!(t.insert(vertex(x, y, Some(k))));
    }
    t.finish();
    let e = t.find([0, 0]);
    let mut ring = Vec::new();
    let mut r = e;
    loop {
        ring.push(r);
        r = t.onext(r);
        if r == e {
            break;
        }
    }
    // The triangle (0, 0), (100, 0), (0, 100): its circumcentre (50, 50) on each of its records.
    let inner = ring
        .iter()
        .copied()
        .find(|&r| t.dest(r) == [100, 0] && t.dest(t.onext(r)) == [0, 100])
        .expect("the triangle");
    assert_eq!(t.centre(inner), [50, 50]);
    assert_eq!(t.centre(t.lnext(inner)), [50, 50]);
    assert!(t.records[inner].mark);
    // Every record with data has a centre; none without a zone corner.
    for &r in t.records() {
        let corners = [r, t.lnext(r), t.lnext(t.lnext(r))];
        let zone = corners.iter().any(|&c| t.origin(c).data.is_some());
        assert_eq!(t.centre(r) != [-1, -1], zone, "record {r}");
    }
    // A triangle's three records share the centre of its first corner in the vector's order
    // (the truncations differ by corner).
    for [a, b, c] in triangles(&t) {
        assert_eq!((t.centre(a), t.centre(b)), (t.centre(c), t.centre(c)));
    }
}

// covers: RMG-ZON-009
#[test]
fn the_cases_the_original_would_not_survive_fall_back() {
    let mut t = Triangulation::new();
    let before = t.records().len();
    for (x, y) in [(400, 0), (0, -200), (-201, 0), (0, 401)] {
        assert!(!t.insert(vertex(x, y, None)), "({x}, {y})");
    }
    assert_eq!(t.records().len(), before);
    // A ring broken so that the walk turns round for ever: it stops after as many steps as
    // records.
    let mut broken = Triangulation::new();
    let start = broken.start;
    broken.records[start].twin = start;
    broken.records[start].origin = vertex(0, 0, None);
    assert!(broken.find([5, -5]) < broken.records.len());
    // A flat triangle (three corners on a line) has no circumcentre: it keeps -1.
    let mut flat = Triangulation::new();
    for r in &mut flat.records {
        r.origin = vertex(r.origin.x, r.origin.x, Some(0));
    }
    flat.finish();
    assert!(flat.records().iter().all(|&r| flat.centre(r) == [-1, -1]));
}
