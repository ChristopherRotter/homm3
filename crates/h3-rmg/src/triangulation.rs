//! Step 2's triangulation (RMG-ZON-009; survey `docs/rules/surveys/E14-generator.md` §2.2,
//! "Step 2's triangulation"): D. Lischinski's incremental Delaunay triangulation (Graphics Gems
//! IV, 1994) as `triangulation_init` 0x5FD010 to `triangulation_finish` 0x5FDB40 run it, in
//! integers and on half-edges instead of quad-edges.
//!
//! A record is a half-edge: its origin (a point and its data, the zone generator there), its
//! twin, the next and previous record counter-clockwise (y up) around its origin, a mark and the
//! circumcentre of the triangle on its left. The records keep the original's vector: every pair
//! in creation order (the edge before its twin), a deleted pair erased with the order kept, a
//! flip reusing its pair in place; the starting edge is the original's too, so the walks, the
//! circumcentres, the borders' first side and the fill's ties follow them.
//!
//! Three cases the original would not survive cannot arise in step 2, and each has a fallback
//! that keeps the functions total: a point on or outside the frame (step 2's points lie within
//! (−W, 2W) on a map of at most 144), a walk that does not end (none in a Delaunay
//! triangulation) and a flat triangle (the insertion leaves none).

use alloc::vec::Vec;

/// A record's index in the arena.
pub(crate) type Edge = usize;

/// A vertex: its point and the zone generator there (none for the frame's corners and level
/// 1's points).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Vertex {
    pub x: i32,
    pub y: i32,
    pub data: Option<usize>,
}

/// A record (0x24 bytes in the original).
#[derive(Debug, Clone, Copy)]
struct Record {
    /// +0 to +8: its origin.
    origin: Vertex,
    /// +0xC.
    twin: Edge,
    /// +0x10 and +0x14: the ring around its origin.
    onext: Edge,
    oprev: Edge,
    /// +0x18: its left triangle's circumcentre computed.
    mark: bool,
    /// +0x1C, +0x20: that circumcentre (-1 until the finish; a triangle without a zone corner
    /// keeps it).
    centre: [i32; 2],
}

/// The frame's corners (`triangulation_init`): (-200, -200) to (400, 400).
const FRAME: [i32; 2] = [-200, 400];

/// Twice the signed area of (a, b, c) in 32 bits (`triangulation_tri_area` 0x5FDAE0),
/// positive counter-clockwise with y up.
fn tri_area(a: [i32; 2], b: [i32; 2], c: [i32; 2]) -> i32 {
    b[0].wrapping_sub(a[0])
        .wrapping_mul(c[1].wrapping_sub(a[1]))
        .wrapping_sub(
            b[1].wrapping_sub(a[1])
                .wrapping_mul(c[0].wrapping_sub(a[0])),
        )
}

/// The squared distance in 32 bits (`triangulation_distance2` 0x5FDB10).
fn distance2(a: [i32; 2], b: [i32; 2]) -> i32 {
    let dx = a[0].wrapping_sub(b[0]);
    let dy = a[1].wrapping_sub(b[1]);
    dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy))
}

/// Lischinski's InCircle: d strictly inside the circle through a, b, c (counter-clockwise); the
/// squares and areas in 32 bits, each product and the sum in 64.
fn in_circle(a: [i32; 2], b: [i32; 2], c: [i32; 2], d: [i32; 2]) -> bool {
    let sq = |p: [i32; 2]| {
        i64::from(
            p[0].wrapping_mul(p[0])
                .wrapping_add(p[1].wrapping_mul(p[1])),
        )
    };
    let t = |p, q, r| i64::from(tri_area(p, q, r));
    sq(a) * t(b, c, d) - sq(b) * t(a, c, d) + sq(c) * t(a, b, d) - sq(d) * t(a, b, c) > 0
}

/// The triangulation of one level.
#[derive(Debug, Clone)]
pub(crate) struct Triangulation {
    /// Every record made; a deleted one stays here, out of `order`.
    records: Vec<Record>,
    /// The original's vector (+8 to +0x10): the records in creation order.
    order: Vec<Edge>,
    /// The starting edge (+0).
    start: Edge,
}

impl Triangulation {
    /// `triangulation_init` 0x5FD010: the frame's corners a (-200, -200), b (400, -200),
    /// c (400, 400), d (-200, 400) without data, the edges a→b, b→c, c→d, d→a spliced into a
    /// ring, then the diagonal a→c (`Connect(d→a, c→d)`); the starting edge a→b.
    // rule: RMG-ZON-009
    pub(crate) fn new() -> Self {
        let [lo, hi] = FRAME;
        let corner = |x, y| Vertex { x, y, data: None };
        let corners = [
            corner(lo, lo),
            corner(hi, lo),
            corner(hi, hi),
            corner(lo, hi),
        ];
        let mut t = Self {
            records: Vec::new(),
            order: Vec::new(),
            start: 0,
        };
        let edges: [Edge; 4] =
            core::array::from_fn(|k| t.make_edge(corners[k], corners[(k + 1) % 4]));
        for k in 0..4 {
            let sym = t.sym(edges[k]);
            t.splice(sym, edges[(k + 1) % 4]);
        }
        t.connect(edges[3], edges[2]);
        t.start = edges[0];
        t
    }

    fn sym(&self, e: Edge) -> Edge {
        self.records[e].twin
    }

    pub(crate) fn onext(&self, e: Edge) -> Edge {
        self.records[e].onext
    }

    pub(crate) fn oprev(&self, e: Edge) -> Edge {
        self.records[e].oprev
    }

    /// The next record counter-clockwise around the left face: `twin.Oprev`.
    fn lnext(&self, e: Edge) -> Edge {
        self.oprev(self.sym(e))
    }

    /// The previous record around the left face: `Onext.twin`.
    fn lprev(&self, e: Edge) -> Edge {
        self.sym(self.onext(e))
    }

    /// `twin.Oprev.twin`.
    fn dprev(&self, e: Edge) -> Edge {
        self.sym(self.lnext(e))
    }

    /// Its origin.
    pub(crate) fn origin(&self, e: Edge) -> Vertex {
        self.records[e].origin
    }

    /// Its destination (its twin's origin).
    pub(crate) fn destination(&self, e: Edge) -> Vertex {
        self.origin(self.sym(e))
    }

    fn org(&self, e: Edge) -> [i32; 2] {
        let v = self.origin(e);
        [v.x, v.y]
    }

    fn dest(&self, e: Edge) -> [i32; 2] {
        self.org(self.sym(e))
    }

    /// The circumcentre of its left triangle (-1, -1 for a triangle without a zone corner).
    pub(crate) fn centre(&self, e: Edge) -> [i32; 2] {
        self.records[e].centre
    }

    /// The records in the original's vector order.
    #[cfg(test)]
    pub(crate) fn records(&self) -> &[Edge] {
        &self.order
    }

    fn right_of(&self, p: [i32; 2], e: Edge) -> bool {
        tri_area(p, self.dest(e), self.org(e)) > 0
    }

    /// `triangulation_make_edge` 0x5FD390: a new pair from `a` to `b`, each its own ring,
    /// appended to the vector (the edge, then its twin).
    fn make_edge(&mut self, a: Vertex, b: Vertex) -> Edge {
        let e = self.records.len();
        let record = |origin, me, twin| Record {
            origin,
            twin,
            onext: me,
            oprev: me,
            mark: false,
            centre: [-1, -1],
        };
        self.records.push(record(a, e, e + 1));
        self.records.push(record(b, e + 1, e));
        self.order.push(e);
        self.order.push(e + 1);
        e
    }

    /// Splice(a, b): their Onext swapped, the Oprev of the two swapped kept inverse.
    fn splice(&mut self, a: Edge, b: Edge) {
        let an = self.onext(a);
        let bn = self.onext(b);
        self.records[a].onext = bn;
        self.records[b].onext = an;
        self.records[an].oprev = b;
        self.records[bn].oprev = a;
    }

    /// `triangulation_delete_edge` 0x5FD5B0: the pair spliced out of its rings and erased from
    /// the vector (the order kept).
    fn delete_edge(&mut self, e: Edge) {
        let sym = self.sym(e);
        self.splice(e, self.oprev(e));
        self.splice(sym, self.oprev(sym));
        self.order.retain(|&r| r != e && r != sym);
    }

    /// Connect(a, b): a new pair from a's destination to b's origin, spliced to a.Lnext and b.
    fn connect(&mut self, a: Edge, b: Edge) -> Edge {
        let e = self.make_edge(self.destination(a), self.origin(b));
        self.splice(e, self.lnext(a));
        let sym = self.sym(e);
        self.splice(sym, b);
        e
    }

    /// Swap(e) (0x5FCFA0 detaches the pair): its ends become the two far corners, the pair
    /// reused in place.
    fn swap(&mut self, e: Edge) {
        let a = self.oprev(e);
        let sym = self.sym(e);
        let b = self.oprev(sym);
        self.splice(e, a);
        self.splice(sym, b);
        self.splice(e, self.lnext(a));
        self.splice(sym, self.lnext(b));
        self.records[e].origin = self.destination(a);
        self.records[sym].origin = self.destination(b);
    }

    /// `triangulation_find` 0x5FD6B0 (Locate) from the starting edge: p at e's origin gives e, at
    /// its destination e's twin; RightOf(p, e) the twin; else not RightOf(p, Onext) Onext; else
    /// not RightOf(p, Dprev) Dprev; else e. A walk in a Delaunay triangulation visits no triangle
    /// twice: one longer than the records stops where it is.
    // rule: RMG-ZON-009
    pub(crate) fn find(&self, p: [i32; 2]) -> Edge {
        let mut e = self.start;
        for _ in 0..=self.records.len() {
            if p == self.org(e) {
                return e;
            }
            if p == self.dest(e) {
                return self.sym(e);
            }
            e = if self.right_of(p, e) {
                self.sym(e)
            } else if !self.right_of(p, self.onext(e)) {
                self.onext(e)
            } else if !self.right_of(p, self.dprev(e)) {
                self.dprev(e)
            } else {
                return e;
            };
        }
        e
    }

    /// `triangulation_insert` 0x5FD790 (InsertSite): a point already a vertex is not inserted
    /// (false; its data is lost); a point on an edge (area 0, within both ends' squared
    /// distances of the edge's length) deletes that edge; the point is connected to the corners
    /// around it, the new base edge the starting edge; then the flips while the strict RightOf
    /// and InCircle tests hold (four cocircular points keep their diagonal). A point on or outside
    /// the frame is not inserted either.
    // rule: RMG-ZON-009
    pub(crate) fn insert(&mut self, v: Vertex) -> bool {
        let p = [v.x, v.y];
        let [lo, hi] = FRAME;
        let inside = lo < v.x && v.x < hi && lo < v.y && v.y < hi;
        let mut e = self.find(p);
        if !inside || p == self.org(e) || p == self.dest(e) {
            return false;
        }
        let (o, d) = (self.org(e), self.dest(e));
        let length = distance2(o, d);
        if tri_area(o, d, p) == 0 && distance2(p, o) <= length && distance2(p, d) <= length {
            e = self.oprev(e);
            self.delete_edge(self.onext(e));
        }
        let mut base = self.make_edge(self.origin(e), v);
        self.splice(base, e);
        self.start = base;
        loop {
            base = self.connect(e, self.sym(base));
            e = self.oprev(base);
            if self.lnext(e) == self.start {
                break;
            }
        }
        // The predicates are exact inside the frame (the areas below 2^21, the products below
        // 2^40), so the flips end as Lischinski's do.
        loop {
            let t = self.oprev(e);
            let far = self.dest(t);
            if self.right_of(far, e) && in_circle(self.org(e), far, self.dest(e), p) {
                self.swap(e);
                e = self.oprev(e);
            } else if self.onext(e) == self.start {
                return true;
            } else {
                e = self.lprev(self.onext(e));
            }
        }
    }

    /// `triangulation_finish` 0x5FDB40: per record of the vector in order with data and no mark,
    /// its left triangle's circumcentre from that corner A (B its destination, C its Onext's):
    /// u = B − A, v = C − B, w = A − C, r = (−u.y, u.x) × dot(v, w) / cross(u, w),
    /// A + (u + r) / 2 (32-bit products, both divisions toward zero), written to the triangle's
    /// three records (the record, its Onext's twin, that one's Onext's twin) and marked. A flat
    /// triangle (cross 0, which the insertion never leaves) keeps -1.
    // rule: RMG-ZON-009
    pub(crate) fn finish(&mut self) {
        for k in 0..self.order.len() {
            let e = self.order[k];
            if self.records[e].origin.data.is_none() || self.records[e].mark {
                continue;
            }
            let a = self.org(e);
            let b = self.dest(e);
            let c = self.dest(self.onext(e));
            let sub = |p: [i32; 2], q: [i32; 2]| [p[0].wrapping_sub(q[0]), p[1].wrapping_sub(q[1])];
            let (u, v, w) = (sub(b, a), sub(c, b), sub(a, c));
            let dot = v[1]
                .wrapping_mul(w[1])
                .wrapping_add(v[0].wrapping_mul(w[0]));
            let cross = u[0]
                .wrapping_mul(w[1])
                .wrapping_add(u[1].wrapping_neg().wrapping_mul(w[0]));
            // `checked_div`: none for a flat triangle (and for `idiv`'s overflow).
            let r = [u[1].wrapping_neg(), u[0]].map(|n| n.wrapping_mul(dot).checked_div(cross));
            let [Some(rx), Some(ry)] = r else {
                continue;
            };
            let half = |n: i32| n / 2;
            let centre = [
                a[0].wrapping_add(half(u[0].wrapping_add(rx))),
                a[1].wrapping_add(half(u[1].wrapping_add(ry))),
            ];
            let second = self.sym(self.onext(e));
            let third = self.sym(self.onext(second));
            for t in [e, second, third] {
                self.records[t].centre = centre;
                self.records[t].mark = true;
            }
        }
    }
}

#[cfg(test)]
mod tests;
