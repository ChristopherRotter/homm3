//! The live part of a map (E24 M1 T2, *provisional*: T4 and T5 settle what the drawing needs):
//! what changes in play and what the map file leaves to the game's start. Which objects stand,
//! what a random object became (the original draws them at the game's start, and the harness saw
//! the draws differ from run to run: E17 design §6.3), the owners, the heroes and the boats
//! (they have no draw lists, E24 design §2.3), the viewer and the tiles it has explored (§1.5),
//! and the moment (§1.6). In M1 a comparison fills it from the original's memory (`h3mem.py`);
//! [`Live::at_start`] gives what a map's start holds without a draw; from M4 on the engine fills
//! it.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use h3_map::RawMap;
use h3_map::model::{Body, Buildings, Pos};

use crate::art::{CELLS, MapArt, TemplateArt};
use crate::tables::{self, TownArt};

/// The live part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Live {
    /// One per map object, in the map's order ([`MapArt::objects`]).
    pub objects: Vec<LiveObject>,
    /// The heroes on the map.
    pub heroes: Vec<LiveHero>,
    /// The boats without a hero.
    pub boats: Vec<LiveBoat>,
    /// The viewing player's colour (0–7): the explored tiles' bit, the corner gems' frame, the
    /// frame picture's colours (E24 design §1.1, §1.5, §1.9).
    pub viewer: u8,
    /// The tiles the viewer has explored.
    pub explored: Explored,
    /// The moment drawn.
    pub moment: Moment,
}

/// One object's live state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveObject {
    /// Still on the map (a picked-up pile or a beaten monster is not).
    pub standing: bool,
    /// What a random object became (`None`: not random, or not drawn yet).
    pub drawn: Option<Drawn>,
    /// The owner's colour (0–7), `None` for no owner.
    pub owner: Option<u8>,
    /// The object's animation phase (the 12-byte object record's +0xA, random at load; §1.6).
    pub phase: u8,
}

/// What a random object became (E24 M1 T4b; design §2.7). A new game copies the object's
/// template and gives the copy this DEF, class and subclass (0x4C9990), keeping the template's
/// size, masks, passability and priority; the object keeps the entries it was placed with. The
/// original draws `def`'s frames with the random template's masks, and names an artifact's DEF by
/// its id, which from 128 on is the picture of the artifact before it (MCL-Q3): the client draws
/// `own` with its own masks, the comparisons the original's (`h3_render::Bugs::Original`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drawn {
    /// The DEF the copy names.
    pub def: String,
    /// The DEF of the game's template for the class and subclass it became (`Objects.txt`'s last
    /// such, as 0x505EA0 finds a monster's), if the game has one.
    pub own: Option<String>,
    /// The class it became (an artifact 5, a monster 54, a resource pile 79, a town 98, …).
    pub class: u32,
    /// Its subclass.
    pub subclass: u32,
}

/// A hero on the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiveHero {
    /// The tile it stands on (its template's visitable cell, not the anchor).
    pub at: Pos,
    /// Its class (0–17: its map DEF `ah00_` … `ah17_`; the hero record's +0x30, design §2.3).
    pub class: u8,
    /// Its owner's colour (0–7): its flag.
    pub owner: u8,
    /// Where it faces.
    pub facing: Facing,
    /// The boat it sails, if it is in one.
    pub boat: Option<Boat>,
}

/// A boat's pictures (design §2.3: the waves, the flag, the boat by its type).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Boat {
    /// Its type (0–2: `ab01_` … `ab03_`).
    pub kind: u8,
    /// Its flag's colour (0–7).
    pub colour: u8,
}

/// A boat without a hero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiveBoat {
    /// The tile it lies on.
    pub at: Pos,
    /// Its type (0–2).
    pub kind: u8,
    /// Where it faces.
    pub facing: Facing,
}

/// Where a hero or boat faces, in the original's numbering (the hero record's +0x47, the boat
/// record's +0x1B; design §2.3, §2.8): each draws its group of the tables' `groups` (0x4D9110,
/// 0x4D9160: up 0, up-right 1, right 2, down-right 3, down 4, the left ones those of the right
/// ones), mirrored above 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Facing {
    /// 0.
    Up,
    /// 1.
    UpRight,
    /// 2.
    Right,
    /// 3.
    DownRight,
    /// 4.
    Down,
    /// 5.
    DownLeft,
    /// 6.
    Left,
    /// 7.
    UpLeft,
}

impl Facing {
    /// Every facing, in the original's order.
    pub const ALL: [Self; 8] = [
        Self::Up,
        Self::UpRight,
        Self::Right,
        Self::DownRight,
        Self::Down,
        Self::DownLeft,
        Self::Left,
        Self::UpLeft,
    ];

    /// The original's number.
    #[must_use]
    pub fn number(self) -> u8 {
        self as u8
    }

    /// The facing of the original's number, if it is one.
    #[must_use]
    pub fn from_number(number: u8) -> Option<Self> {
        Self::ALL.get(usize::from(number)).copied()
    }

    /// Drawn mirrored (the number above 4).
    #[must_use]
    pub fn mirrored(self) -> bool {
        self.number() > 4
    }
}

/// The tiles the viewer has explored (its bit of each tile's vision word, design §1.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explored {
    size: usize,
    levels: usize,
    tiles: Vec<bool>,
}

impl Explored {
    /// Every tile of a map of `size` and `levels` explored, or none.
    #[must_use]
    pub fn all(size: usize, levels: usize, explored: bool) -> Self {
        Self {
            size,
            levels,
            tiles: vec![explored; size.saturating_mul(size).saturating_mul(levels)],
        }
    }

    fn index(&self, x: i32, y: i32, z: u8) -> Option<usize> {
        let (x, y) = (usize::try_from(x).ok()?, usize::try_from(y).ok()?);
        let z = usize::from(z);
        (x < self.size && y < self.size && z < self.levels)
            .then(|| (z * self.size + y) * self.size + x)
    }

    /// Whether the viewer has explored (x, y, z); a tile off the map has not been (the
    /// unexplored tiles' mask counts it so, design §1.5).
    #[must_use]
    pub fn explored(&self, x: i32, y: i32, z: u8) -> bool {
        self.index(x, y, z)
            .and_then(|i| self.tiles.get(i).copied())
            .unwrap_or(false)
    }

    /// Marks (x, y, z) explored or not; a tile off the map is left alone.
    pub fn set(&mut self, x: i32, y: i32, z: u8, explored: bool) {
        if let Some(tile) = self.index(x, y, z).and_then(|i| self.tiles.get_mut(i)) {
            *tile = explored;
        }
    }
}

/// The moment a still shows (design §1.6).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Moment {
    /// The palette cycles' steps since the DEFs were loaded (k: one step rotates every animated
    /// terrain's and river's ranges by one).
    pub cycles: u32,
    /// The objects' frame steps at the draw (the manager's +0x100): an object's frame is its
    /// phase plus this, modulo its group's frames; a hero's this alone.
    pub frames: u32,
}

impl Moment {
    /// The moment `ms` after the DEFs were loaded on a clock that never runs late: a cycle step
    /// every `cycle_ms`, a frame step every `frame_ms` (the tables' `[map]`; a step of 0 never
    /// comes). The original polls both and catches no late step up (design §1.6), so its moment
    /// at a time is at most this; a comparison reads the original's from memory (M1 T3b2).
    #[must_use]
    pub fn at(ms: u32, cycle_ms: u32, frame_ms: u32) -> Self {
        Self {
            cycles: ms.checked_div(cycle_ms).unwrap_or(0),
            frames: ms.checked_div(frame_ms).unwrap_or(0),
        }
    }
}

/// The hero objects' class (34; design §2.1).
pub const HERO_CLASS: u32 = 34;
/// The boats' class (8).
pub const BOAT_CLASS: u32 = 8;
/// The towns' class (98; a random town is 77).
pub const TOWN_CLASS: u32 = 98;

impl Live {
    /// The live part at a map's start, as far as it needs no draw: every object standing and
    /// undrawn, phase 0 (the original's are random: a comparison reads them), the owners the map
    /// gives (a colour above 7, `0xFF` among them, is no owner); the heroes placed with an owner
    /// and a class (a random hero, class 70, needs a draw and is left out), facing right, on land
    /// (as the original's new game has them, a hero on water too: E24 M1 T5a, design §2.8); the
    /// boats of the map, facing right; every town (not a random one) with its picture by its
    /// buildings at the start (`towns`; E24 M1 T5b1, design §2.9: a new game redraws every town
    /// so, as a copy of its template); every tile explored (*provisional*: what a start reveals
    /// is the engine's, a comparison reads it); the moment 0.
    #[must_use]
    pub fn at_start(raw: &RawMap, art: &MapArt, viewer: u8, towns: &TownArt) -> Self {
        let mut heroes = Vec::new();
        let mut boats = Vec::new();
        let mut objects = Vec::with_capacity(art.objects.len());
        for (object, raw_object) in art.objects.iter().zip(&raw.objects) {
            let template = art.templates.get(object.template);
            let at = template.and_then(|t| entrance(object.anchor, t));
            let class = template.map(|t| (t.class, t.subclass));
            match (class, at, &raw_object.body) {
                (Some((HERO_CLASS, subclass)), Some(at), Body::Hero(hero)) => {
                    if let (Ok(class), Some(owner)) = (u8::try_from(subclass), colour(hero.owner)) {
                        heroes.push(LiveHero {
                            at,
                            class,
                            owner,
                            facing: Facing::Right,
                            boat: None,
                        });
                    }
                }
                (Some((BOAT_CLASS, subclass)), Some(at), _) => {
                    if let Ok(kind) = u8::try_from(subclass) {
                        boats.push(LiveBoat {
                            at,
                            kind,
                            facing: Facing::Right,
                        });
                    }
                }
                _ => {}
            }
            let drawn = match (template, &raw_object.body) {
                (Some(t), Body::Town(town)) if t.class == TOWN_CLASS => towns
                    .picture(t.subclass, built_at_start(&town.buildings))
                    .filter(|def| !def.eq_ignore_ascii_case(&t.def))
                    .map(|def| Drawn {
                        def: def.into(),
                        own: None,
                        class: TOWN_CLASS,
                        subclass: t.subclass,
                    }),
                _ => None,
            };
            objects.push(LiveObject {
                standing: true,
                drawn,
                owner: owner(&raw_object.body),
                phase: 0,
            });
        }
        Self {
            objects,
            heroes,
            boats,
            viewer,
            explored: Explored::all(art.size, art.levels, true),
            moment: Moment::default(),
        }
    }
}

/// The tile of a template's first visitable cell (cells by row from the anchor's, each row from
/// the anchor's column leftwards), if it has one on the map's coordinates.
fn entrance(anchor: Pos, template: &TemplateArt) -> Option<Pos> {
    (0..CELLS.1)
        .flat_map(|dy| (0..CELLS.0).map(move |dx| (dx, dy)))
        .find(|&(dx, dy)| template.visitable(dx, dy))
        .and_then(|(dx, dy)| {
            Some(Pos {
                x: anchor.x.checked_sub(dx)?,
                y: anchor.y.checked_sub(dy)?,
                z: anchor.z,
            })
        })
}

/// The buildings of a town's picture it has at a new game, by the game's ids (bit `i` building
/// `i`; design §2.9): the standard set's Fort if it has one (0x5C0C0F); the chosen ones' Capitol,
/// Fort, Citadel and Castle, the map's buildings 2 to 5 (the map-to-game table 0x6888C0, the same
/// for every faction).
fn built_at_start(buildings: &Buildings) -> u64 {
    let bit = |id: u32| 1u64 << id;
    match buildings {
        Buildings::Standard { fort } => u64::from(*fort) * bit(tables::FORT),
        Buildings::Custom { built, .. } => [
            (2, tables::CAPITOL),
            (3, tables::FORT),
            (4, tables::CITADEL),
            (5, tables::CASTLE),
        ]
        .into_iter()
        .filter(|&(map, _)| built[0] >> map & 1 == 1)
        .fold(0, |set, (_, id)| set | bit(id)),
    }
}

/// An owner as the map stores it: 0–7 a colour, anything else none.
fn colour(owner: impl Into<u32>) -> Option<u8> {
    u8::try_from(owner.into()).ok().filter(|&c| c < 8)
}

/// The owner a map object's body gives.
fn owner(body: &Body) -> Option<u8> {
    match body {
        Body::Town(town) => colour(town.owner),
        Body::Hero(hero) => colour(hero.owner),
        Body::Mine { owner } | Body::Owned { owner } | Body::Garrison { owner, .. } => {
            colour(*owner)
        }
        Body::RandomDwelling(dwelling) => colour(dwelling.owner),
        Body::HeroPlaceholder { owner, .. } => colour(*owner),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
