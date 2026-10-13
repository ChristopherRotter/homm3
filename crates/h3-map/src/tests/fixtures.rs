//! The generated maps committed under `fixtures/maps/` (E08 T9a): `h3-sim`'s adventure
//! environment plays them in its tests, T9b's golden replays and benchmarks too. Each is laid
//! out by T8b2a's [`lay_out`] from a plan built here with a fixed generator (not proptest's, so
//! that the plan depends on this file alone), written as the game stores maps (gzip). The test
//! checks that the committed file reads to the same map and imports; it rewrites the file when
//! `H3_WRITE_FIXTURES` is set (a change of a committed map invalidates the replays that name it by
//! content hash, ADV-Q2).

use std::path::PathBuf;

use super::gzip_with;
use super::play::{BLOCKERS, FAITHFUL, KINDS, Plan, Spot, lay_out};
use crate::model::Version;
use crate::{Limits, import, read};

/// SplitMix64: a fixed generator for the plans.
struct Mix(u64);

impl Mix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A value below `n`.
    fn below(&mut self, n: u8) -> u8 {
        u8::try_from(self.next() % u64::from(n)).unwrap()
    }
}

/// A tile's terrain, road and river codes.
type Codes = (u8, u8, u8);

/// Terrain in patches of 6 × 6 tiles (all eight land terrains, some water and rock), a road
/// across each level (the middle row on the surface, the middle column underground), rivers on
/// some patches, and `count` blockers, for a two-level map of `side` tiles, all drawn from `mix`
/// in this order (`e08-duel-36.h3m`'s draws: changing the order changes the committed maps).
fn patchwork(side: u8, mix: &mut Mix, count: usize) -> (Vec<Codes>, Vec<(Spot, usize)>) {
    assert_eq!(side % 6, 0, "whole patches only");
    let per_row = usize::from(side / 6);
    let patches: Vec<(u8, u8)> = (0..2 * per_row * per_row)
        .map(|_| {
            let terrain = match mix.below(12) {
                10 => 8,
                11 => 9,
                land => land % 8,
            };
            let river = if mix.below(4) == 0 {
                1 + mix.below(4)
            } else {
                0
            };
            (terrain, river)
        })
        .collect();
    let middle = side / 2 - 1;
    let mut tiles = Vec::new();
    for z in 0..2u8 {
        for y in 0..side {
            for x in 0..side {
                let patch =
                    (usize::from(z) * per_row + usize::from(y / 6)) * per_row + usize::from(x / 6);
                let (terrain, river) = patches[patch];
                let road = if (z == 0 && y == middle) || (z == 1 && x == middle) {
                    1 + z
                } else {
                    0
                };
                let terrain = if road == 0 { terrain } else { 2 };
                tiles.push((terrain, road, river));
            }
        }
    }
    let blockers = (0..count)
        .map(|_| {
            let anchor = (mix.below(side), mix.below(side), mix.below(2));
            (anchor, usize::from(mix.below(3)))
        })
        .collect();
    (tiles, blockers)
}

/// The kind of [`KINDS`] with this class, subclass and resource body.
fn kind(class: u32, subclass: u32, amount: Option<u32>) -> usize {
    KINDS
        .iter()
        .position(|&k| k == (class, subclass, amount))
        .unwrap()
}

/// A pile of `resource` (0 wood, 1 mercury, 2 ore, 3 sulfur, 4 crystal, 5 gems, 6 gold) of a
/// drawn amount.
fn pile(resource: u32) -> usize {
    kind(79, resource, Some(0))
}

/// Where [`Placer::place`] draws an object's anchor.
#[derive(Clone, Copy)]
enum Area {
    /// Its visitable tile two to four tiles from this hero's, inside the hero's first reveal.
    Near(Spot),
    /// Its visitable tile on the level's road (shape 1: on the surface's row, both tiles on the
    /// road; underground the anchor east of the road's column).
    Road(u8),
    /// Anywhere on the level.
    Level(u8),
    /// Anywhere on the map.
    Anywhere,
}

/// Increment 2's objects laid out on a map's terrain around its blockers and heroes (E08 T16a),
/// each drawn from the plan's generator after the terrain and the blockers, so that both stay as
/// they were: an anchor drawn until every tile of the object is a free land tile off the roads
/// (but a guard's or a gate's on the road), and its visitable tile keeps two free land neighbours
/// it is entered or visited from not from above (ADV-OBJ-004); those neighbours stay free of later
/// objects. So no object covers a hero, another object, or shares an anchor (setup's refusals).
struct Placer<'a> {
    side: u8,
    tiles: &'a [Codes],
    /// Tiles covered by a blocker, a hero or a placed object.
    taken: Vec<bool>,
    /// Tiles kept free beside a placed object's visitable tile.
    kept: Vec<bool>,
    placed: Vec<(Spot, usize)>,
}

impl<'a> Placer<'a> {
    fn new(side: u8, tiles: &'a [Codes], blockers: &[(Spot, usize)], heroes: &[Spot]) -> Self {
        let count = 2 * usize::from(side) * usize::from(side);
        let mut placer = Placer {
            side,
            tiles,
            taken: vec![false; count],
            kept: vec![false; count],
            placed: Vec::new(),
        };
        for &((x, y, z), object) in blockers {
            let anchor = (x % side, y % side, z % 2);
            for spot in placer.covered(anchor, object).into_iter().flatten() {
                let i = placer.index(spot);
                placer.taken[i] = true;
            }
        }
        for &(x, y, z) in heroes {
            let i = placer.index((x % side, y % side, z % 2));
            placer.taken[i] = true;
        }
        placer
    }

    fn index(&self, (x, y, z): Spot) -> usize {
        (usize::from(z) * usize::from(self.side) + usize::from(y)) * usize::from(self.side)
            + usize::from(x)
    }

    /// The tile `(column, row)` left of and above `anchor`, if it is on the map.
    fn at(&self, (x, y, z): Spot, (column, row): (u8, u8)) -> Option<Spot> {
        Some((x.checked_sub(column)?, y.checked_sub(row)?, z))
    }

    /// Every tile of `object` at `anchor` (`None` for one off the map).
    fn covered(&self, anchor: Spot, object: usize) -> Vec<Option<Spot>> {
        let (blocked, visitable) = BLOCKERS[object % BLOCKERS.len()];
        blocked
            .iter()
            .chain(visitable)
            .map(|&tile| self.at(anchor, tile))
            .collect()
    }

    fn road(&self, (x, y, z): Spot) -> bool {
        let middle = self.side / 2 - 1;
        (z == 0 && y == middle) || (z == 1 && x == middle)
    }

    fn free(&self, spot: Spot) -> bool {
        let i = self.index(spot);
        self.tiles[i].0 < 8 && !self.taken[i] && !self.kept[i]
    }

    /// The free land neighbours of `visit` a hero steps from onto it not from above.
    fn entries(&self, visit: Spot, own: &[Spot]) -> Vec<Spot> {
        let (vx, vy, vz) = visit;
        let mut entries = Vec::new();
        for (dx, dy) in [(-1i16, 0i16), (1, 0), (-1, 1), (0, 1), (1, 1)] {
            let (x, y) = (i16::from(vx) + dx, i16::from(vy) + dy);
            let on = |v: i16| u8::try_from(v).ok().filter(|&v| v < self.side);
            if let (Some(x), Some(y)) = (on(x), on(y)) {
                let spot = (x, y, vz);
                if self.free(spot) && !own.contains(&spot) {
                    entries.push(spot);
                }
            }
        }
        entries
    }

    /// Places an object of `kind` in `area`: the anchor drawn from `mix` until it fits.
    fn place(&mut self, mix: &mut Mix, kind: usize, area: Area) {
        let side = self.side;
        let middle = side / 2 - 1;
        for _ in 0..100_000 {
            let shape = match area {
                Area::Road(_) => 1,
                _ => 1 + usize::from(mix.below(2)),
            };
            let anchor = match area {
                Area::Near((hx, hy, hz)) => {
                    let near = |h: u8, d: u8| u16::from(h) + u16::from(side) + u16::from(d) - 4;
                    let x = near(hx, mix.below(9)) % u16::from(side);
                    let y = near(hy, mix.below(9)) % u16::from(side);
                    (u8::try_from(x).unwrap(), u8::try_from(y).unwrap(), hz)
                }
                Area::Road(0) => (1 + mix.below(side - 1), middle, 0),
                Area::Road(_) => (middle + 1, mix.below(side), 1),
                Area::Level(z) => (mix.below(side), mix.below(side), z),
                Area::Anywhere => (mix.below(side), mix.below(side), mix.below(2)),
            };
            let object = kind * BLOCKERS.len() + shape;
            let covered = self.covered(anchor, object);
            let Some(own) = covered.iter().copied().collect::<Option<Vec<Spot>>>() else {
                continue;
            };
            let (_, visitable) = BLOCKERS[shape];
            let visit = self.at(anchor, visitable[0]).unwrap();
            let roads = own.iter().filter(|&&spot| self.road(spot)).count();
            let fits = match area {
                Area::Near((hx, hy, _)) => {
                    let (dx, dy) = (visit.0.abs_diff(hx), visit.1.abs_diff(hy));
                    let near = u16::from(dx).pow(2) + u16::from(dy).pow(2) <= 20;
                    roads == 0 && dx.max(dy) >= 2 && near
                }
                Area::Road(_) => self.road(visit),
                Area::Level(_) | Area::Anywhere => roads == 0,
            };
            if !fits {
                continue;
            }
            let entries = self.entries(visit, &own);
            if own.iter().all(|&spot| self.free(spot)) && entries.len() >= 2 {
                for spot in own {
                    let i = self.index(spot);
                    self.taken[i] = true;
                }
                for spot in entries {
                    let i = self.index(spot);
                    self.kept[i] = true;
                }
                self.placed.push((anchor, object));
                return;
            }
        }
        panic!("no tile for an object of kind {kind}");
    }
}

/// Increment 2's objects on a map with these heroes (E08 T16a): `scale` times the duel's,
/// [`Placer`] laying them out. Beside each hero (its first reveal, so random play visits them
/// early) a pile, a random resource, a Keymaster's tent (green on the surface, red underground)
/// and a land Cartographer (surface) or a campfire (underground); `roads` border guards and as
/// many gates of the level's colour on each level's road; the rest anywhere on its level or the
/// map. The duel's counts: 26 piles of the seven resources (drawn amounts, some gold piles of a
/// fixed 300), 24 random resources, 4 campfires, 4 tents, 2 guards, 2 gates, 2 Cartographers, a
/// Redwood Observatory and a Pillar of Fire: 66 objects, about 3 % of its land tiles, as the
/// shipped maps (2.5 to 3.7 % by size; the T16a log).
fn objects(
    side: u8,
    tiles: &[Codes],
    blockers: &[(Spot, usize)],
    heroes: &[Spot],
    mix: &mut Mix,
    scale: usize,
    roads: usize,
) -> Vec<(Spot, usize)> {
    let mut placer = Placer::new(side, tiles, blockers, heroes);
    // A tent's, a guard's or a gate's kind on a level: green on the surface, red underground.
    let keyed = |class: u32, z: u8| kind(class, 2 - u32::from(z == 0), None);
    let (chart, campfire, random) = (kind(13, 1, None), kind(12, 0, None), kind(76, 0, Some(0)));
    let mut piles = (0u32..).map(|i| match (i % 7, i / 7 % 2) {
        (6, 1) => kind(79, 6, Some(3)),
        (resource, _) => pile(resource),
    });
    let mut counts = [26, 24, 4, 4, 2, 2, 2, 1, 1].map(|n| n * scale);
    let [
        piles_left,
        randoms,
        campfires,
        tents,
        guards,
        gates,
        charts,
        redwoods,
        pillars,
    ] = &mut counts;
    for &hero in heroes {
        let near = Area::Near(hero);
        placer.place(mix, piles.next().unwrap(), near);
        placer.place(mix, random, near);
        placer.place(mix, keyed(10, hero.2), near);
        if hero.2 == 0 {
            placer.place(mix, chart, near);
            *charts -= 1;
        } else {
            placer.place(mix, campfire, near);
            *campfires -= 1;
        }
        *piles_left -= 1;
        *randoms -= 1;
        *tents -= 1;
    }
    for z in 0..2 {
        for _ in 0..roads {
            placer.place(mix, keyed(9, z), Area::Road(z));
            placer.place(mix, keyed(212, z), Area::Road(z));
        }
    }
    *guards -= 2 * roads;
    *gates -= 2 * roads;
    for _ in 0..*piles_left {
        placer.place(mix, piles.next().unwrap(), Area::Anywhere);
    }
    for _ in 0..*randoms {
        placer.place(mix, random, Area::Anywhere);
    }
    for _ in 0..*campfires {
        placer.place(mix, campfire, Area::Anywhere);
    }
    for z in 0..2 {
        for (class, count) in [(10, *tents), (9, *guards), (212, *gates)] {
            for _ in 0..count / 2 {
                placer.place(mix, keyed(class, z), Area::Level(z));
            }
        }
    }
    for _ in 0..*charts {
        placer.place(mix, chart, Area::Level(0));
    }
    for _ in 0..*redwoods {
        placer.place(mix, kind(58, 0, None), Area::Level(0));
    }
    for _ in 0..*pillars {
        placer.place(mix, kind(60, 0, None), Area::Level(1));
    }
    placer.placed
}

/// `e08-duel-36.h3m`: decision 11's random-play map. SoD, 36 × 36 with an underground, red and
/// blue with two heroes each (one on each level, red's in the west, blue's in the east; their
/// types' starting armies), [`patchwork`] terrain, 72 blockers, then (E08 T16a) increment 2's 66
/// typed objects ([`objects`], a guard and a gate on each road).
fn duel() -> Plan {
    let mut mix = Mix(36);
    let (tiles, mut blockers) = patchwork(36, &mut mix, 72);
    let heroes = [(4, 5, 0), (31, 30, 0), (5, 30, 1), (30, 5, 1)];
    let typed = objects(36, &tiles, &blockers, &heroes, &mut mix, 1, 1);
    blockers.extend(typed);
    Plan {
        version: Version::Sod,
        side: 36,
        two_levels: true,
        tiles,
        players: vec![(0, true), (1, true)],
        // Owners go round the players: red, blue, red, blue.
        heroes: heroes.iter().map(|&spot| (spot, None)).collect(),
        hero_types: 3,
        blockers,
        difficulty: 1,
    }
}

/// `e08-xl-144.h3m`: decision 11's pathfinding and fork map (E08 T9b). SoD, 144 × 144 with an
/// underground (the largest map, 41 472 tiles), red and blue with four heroes each (two on each
/// level, red's in the west, blue's in the east), [`patchwork`] terrain, 1 152 blockers (the
/// duel's density), then (E08 T16a) 1 056 typed objects (the duel's density: [`objects`] 16 times
/// the duel's, four guards and four gates on each road).
fn xl() -> Plan {
    let mut mix = Mix(144);
    let (tiles, mut blockers) = patchwork(144, &mut mix, 72 * 16);
    // Owners go round the players: red, blue, red, blue, …
    let heroes = [
        (10, 10, 0),
        (133, 133, 0),
        (10, 130, 0),
        (133, 13, 0),
        (12, 72, 1),
        (131, 72, 1),
        (70, 20, 1),
        (73, 123, 1),
    ];
    let typed = objects(144, &tiles, &blockers, &heroes, &mut mix, 16, 4);
    blockers.extend(typed);
    Plan {
        version: Version::Sod,
        side: 144,
        two_levels: true,
        tiles,
        players: vec![(0, true), (1, true)],
        heroes: heroes.iter().map(|&spot| (spot, None)).collect(),
        hero_types: 5,
        blockers,
        difficulty: 1,
    }
}

/// A committed generated map: its file name and plan.
type Fixture = (&'static str, fn() -> Plan);

/// The committed generated maps.
const FIXTURES: [Fixture; 2] = [("e08-duel-36.h3m", duel), ("e08-xl-144.h3m", xl)];

#[test]
fn generated_maps_are_committed_as_generated() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/maps");
    for (name, plan) in FIXTURES {
        let m = lay_out(&plan());
        let path = dir.join(name);
        if std::env::var_os("H3_WRITE_FIXTURES").is_some() {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(&path, gzip_with(&super::encode::encode(&m), 0, &[])).unwrap();
        }
        let file = std::fs::read(&path).unwrap();
        let read = read(&file, &Limits::default()).unwrap();
        assert!(
            read == m,
            "{name} is not its plan's map: H3_WRITE_FIXTURES=1 rewrites it"
        );
        let setup = import(&read, &FAITHFUL).unwrap();
        assert_eq!(setup.heroes.len(), plan().heroes.len(), "{name}");
    }
}
