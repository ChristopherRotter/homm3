//! The persistent grid against a flat copy (E08 T5b, ADR-0016 point 1): run under callgrind to
//! count the instructions of one operation on a player's explored layer.
//!
//! ```text
//! cargo build --release -p h3-adventure --example grid_bench
//! valgrind --tool=callgrind --callgrind-out-file=out target/release/examples/grid_bench \
//!     <layout> <size> <operation> <count>
//! ```
//!
//! `layout`: `flat` (one bitset of the whole map, copied whole by a fork), `leaf8`, `leaf16`,
//! `leaf32`, `leaf64` ([`BitGrid`] with 8 × 8 to 64 × 64 tiles a leaf). `size`: `36` (36 × 36, one
//! level) or `144` (144 × 144, two levels). `operation`, done `count` times on a layer explored by
//! twenty scouting disks of radius 5:
//! `fork` (clone and drop), `fork-write` (clone, set one tile, drop), `read-all` (read every tile,
//! as an observation's plane does), `read-scattered` (read 1000 tiles at pseudo-random places),
//! `reveal` (set a disk of radius 5 at a pseudo-random place row by row, the layer owned),
//! `reveal-tiles` (the same tile by tile).
//! The cost of one operation is the difference of the counts of `count` and 0 operations,
//! divided by `count` (the setup is the same for both).

use std::hint::black_box;
use std::process::ExitCode;

use h3_adventure::{BitGrid, Pos};

/// What the measured layouts have in common.
trait Layer: Clone {
    fn new(width: u16, height: u16, levels: u8) -> Self;
    fn get(&self, pos: Pos) -> bool;
    fn set(&mut self, pos: Pos, on: bool);
    fn set_run(&mut self, start: Pos, len: u16, on: bool);
}

impl<const WORDS: usize> Layer for BitGrid<WORDS> {
    fn new(width: u16, height: u16, levels: u8) -> Self {
        BitGrid::new(width, height, levels)
    }
    fn get(&self, pos: Pos) -> bool {
        BitGrid::get(self, pos)
    }
    fn set(&mut self, pos: Pos, on: bool) {
        BitGrid::set(self, pos, on);
    }
    fn set_run(&mut self, start: Pos, len: u16, on: bool) {
        BitGrid::set_run(self, start, len, on);
    }
}

/// The baseline: one bitset of the whole map, which a fork copies whole.
#[derive(Clone)]
struct Flat {
    width: u16,
    height: u16,
    levels: u8,
    words: Vec<u64>,
}

impl Flat {
    fn index(&self, pos: Pos) -> Option<usize> {
        (pos.x < self.width && pos.y < self.height && pos.level < self.levels).then(|| {
            (usize::from(pos.level) * usize::from(self.height) + usize::from(pos.y))
                * usize::from(self.width)
                + usize::from(pos.x)
        })
    }
}

impl Layer for Flat {
    fn new(width: u16, height: u16, levels: u8) -> Self {
        let tiles = usize::from(width) * usize::from(height) * usize::from(levels);
        Self {
            width,
            height,
            levels,
            words: vec![0; tiles.div_ceil(64)],
        }
    }
    fn get(&self, pos: Pos) -> bool {
        self.index(pos)
            .is_some_and(|i| self.words[i / 64] >> (i % 64) & 1 == 1)
    }
    fn set(&mut self, pos: Pos, on: bool) {
        if let Some(i) = self.index(pos) {
            let bit = 1 << (i % 64);
            if on {
                self.words[i / 64] |= bit;
            } else {
                self.words[i / 64] &= !bit;
            }
        }
    }
    fn set_run(&mut self, start: Pos, len: u16, on: bool) {
        let end = start.x.saturating_add(len).min(self.width);
        let (Some(first), true) = (self.index(start), start.x < end) else {
            return;
        };
        let last = first + usize::from(end - start.x);
        let mut i = first;
        while i < last {
            let to = last.min((i / 64 + 1) * 64);
            let bits = (to - i) as u32;
            let mask = (u64::MAX >> (64 - bits)) << (i % 64);
            if on {
                self.words[i / 64] |= mask;
            } else {
                self.words[i / 64] &= !mask;
            }
            i = to;
        }
    }
}

/// A small deterministic generator (no randomness crate in an example of a logic crate).
struct Lcg(u64);

impl Lcg {
    fn next(&mut self, below: u16) -> u16 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) % u64::from(below)) as u16
    }
}

/// Sets the tiles within `radius` of `centre` (`dx² + dy² ≤ r² + r`, VCMI's disk), row by row
/// (`by_runs`) or tile by tile.
fn reveal<L: Layer>(layer: &mut L, centre: Pos, radius: i32, by_runs: bool) {
    for dy in -radius..=radius {
        let half = ((radius * radius + radius - dy * dy) as u32).isqrt() as i32;
        let (x0, y) = (i32::from(centre.x) - half, i32::from(centre.y) + dy);
        let Ok(y) = u16::try_from(y) else { continue };
        let (x0, len) = (x0.max(0), (2 * half + 1 + x0.min(0)) as u16);
        let x0 = x0 as u16;
        if by_runs {
            layer.set_run(Pos::new(x0, y, centre.level), len, true);
        } else {
            for x in x0..x0.saturating_add(len) {
                layer.set(Pos::new(x, y, centre.level), true);
            }
        }
    }
}

fn run<L: Layer>(size: u16, levels: u8, operation: &str, count: u32) -> Result<(), String> {
    let mut rng = Lcg(7);
    let mut layer = L::new(size, size, levels);
    for _ in 0..20 {
        let centre = Pos::new(
            rng.next(size),
            rng.next(size),
            rng.next(u16::from(levels)) as u8,
        );
        reveal(&mut layer, centre, 5, true);
    }
    let mut seen = 0u32;
    match operation {
        "fork" => {
            for _ in 0..count {
                black_box(black_box(&layer).clone());
            }
        }
        "fork-write" => {
            for _ in 0..count {
                let mut fork = black_box(&layer).clone();
                fork.set(
                    Pos::new(size / 2, size / 2, 0),
                    !layer.get(Pos::new(size / 2, size / 2, 0)),
                );
                black_box(fork);
            }
        }
        "read-all" => {
            for _ in 0..count {
                for level in 0..levels {
                    for y in 0..size {
                        for x in 0..size {
                            seen += u32::from(black_box(&layer).get(Pos::new(x, y, level)));
                        }
                    }
                }
            }
        }
        "read-scattered" => {
            for _ in 0..count {
                for _ in 0..1000 {
                    let pos = Pos::new(
                        rng.next(size),
                        rng.next(size),
                        rng.next(u16::from(levels)) as u8,
                    );
                    seen += u32::from(black_box(&layer).get(pos));
                }
            }
        }
        "reveal" | "reveal-tiles" => {
            for _ in 0..count {
                let centre = Pos::new(
                    rng.next(size),
                    rng.next(size),
                    rng.next(u16::from(levels)) as u8,
                );
                reveal(black_box(&mut layer), centre, 5, operation == "reveal");
            }
        }
        other => return Err(format!("unknown operation {other}")),
    }
    black_box(seen);
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [layout, size, operation, count] = args.as_slice() else {
        eprintln!(
            "usage: grid_bench <flat|leaf8|leaf16|leaf32|leaf64> <36|144> <operation> <count>"
        );
        return ExitCode::FAILURE;
    };
    let (size, levels) = match size.as_str() {
        "36" => (36, 1),
        "144" => (144, 2),
        other => {
            eprintln!("unknown size {other}");
            return ExitCode::FAILURE;
        }
    };
    let Ok(count) = count.parse::<u32>() else {
        eprintln!("count must be a number");
        return ExitCode::FAILURE;
    };
    let result = match layout.as_str() {
        "flat" => run::<Flat>(size, levels, operation, count),
        "leaf8" => run::<BitGrid<1>>(size, levels, operation, count),
        "leaf16" => run::<BitGrid<4>>(size, levels, operation, count),
        "leaf32" => run::<BitGrid<16>>(size, levels, operation, count),
        "leaf64" => run::<BitGrid<64>>(size, levels, operation, count),
        other => Err(format!("unknown layout {other}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
