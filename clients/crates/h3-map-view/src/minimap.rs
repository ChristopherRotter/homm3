//! The minimap (E24 M3 T3a, *provisional*; `docs/epics/E24-design.md` §4.4): what
//! `minimap_draw` 0x412C40 draws into the dialog's area 1, as two of a scene's draws, its tiles
//! and the view's box. A tile is black where the viewer has not explored it, else in its
//! terrain's open colour (its DEF's palette entry 8) but a hero's tile in its owner's colour, an
//! owned object's blocked or entrance tile in its owner's, a listed decoration's blocked tile in
//! the terrain's blocked colour (entry 9); it takes 4 × 4, 2 × 2, 2-1-1 by 1-1-2 or 1 × 1 pixels
//! by the map's size. The box is `radar.def`'s frame by the map's size at the view's corner times
//! the size's scale, cut to the area. [`Base`] is a level's colours from its terrain, objects
//! and explored tiles, worked out once with the level's index ([`crate::scene::TileIndex`]);
//! [`Base::tiles`] lays the heroes over them and [`Base::radar`] places the box, each scene, as
//! the minimap was last drawn ([`Shown`], E24 M3 T3b: the original draws it again at a scroll, a
//! hero's selection and each step's end, not through a step's frames; a still's as the view and
//! the heroes stand).

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_map::model::Pos;
use h3_render::Bugs;
use serde::Deserialize;

use crate::art::MapArt;
use crate::layout::AdventureLayout;
use crate::live::Live;
use crate::objects::{self, DrawLists};
use crate::panels;
use crate::scene::{Draw, ViewAt};
use crate::tables::Tables;

/// The asset tables' `[map.minimap]` (E24 M3 T3a; design §4.4).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinimapArt {
    /// The view's box (`radar.def`, the manager's +0xC8).
    pub radar: String,
    /// A terrain DEF's palette entries of its open and blocked colours (8 and 9: +0x2C, +0x2E of
    /// its 16-bit palette, whose colours start at +0x1C).
    pub terrain: [u8; 2],
    /// The classes (the game's) whose blocked or entrance tiles take their owner's colour (the
    /// switch 0x412FF0's cases 0 to 4).
    pub owned: Vec<u32>,
    /// The classes whose blocked tiles take the terrain's blocked colour (its case 5).
    pub blocking: Vec<u32>,
    /// The tiles' pixels and the box by the map's size (its height: 0x413198, 0x4132C5).
    pub sizes: Vec<SizeArt>,
    /// The box on a map of any other size, which draws no tile.
    pub other: RadarArt,
}

/// A map size's pixels and box.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SizeArt {
    /// The map's side in tiles.
    pub size: usize,
    /// The pixels each tile's column takes, over and over from the left.
    pub columns: Vec<u8>,
    /// The pixels each tile's row takes, over and over from the top.
    pub rows: Vec<u8>,
    /// The box.
    pub radar: RadarArt,
}

/// The box's frame and scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadarArt {
    /// The frame of `radar.def`'s group 0.
    pub frame: u32,
    /// What the view's corner is multiplied by for the box's place: the f32 the executable holds,
    /// by its bits (0x3FAA3D71: 1.33). A positive normal number.
    pub scale: u32,
}

impl MinimapArt {
    /// The frames of the box's DEF the minimap draws: one past the highest.
    #[must_use]
    pub fn frames(&self) -> usize {
        let highest = self
            .sizes
            .iter()
            .map(|s| s.radar.frame)
            .chain([self.other.frame])
            .max()
            .unwrap_or(0);
        usize::try_from(highest).map_or(usize::MAX, |f| f.saturating_add(1))
    }

    /// The pixels and the box of a map of `side` tiles: its size's, else none and `other`.
    fn of(&self, side: usize) -> (Option<Pattern>, RadarArt) {
        self.sizes.iter().find(|s| s.size == side).map_or_else(
            || (None, self.other),
            |s| {
                (
                    Some(Pattern {
                        columns: s.columns.clone(),
                        rows: s.rows.clone(),
                    }),
                    s.radar,
                )
            },
        )
    }

    /// Whether a rule is broken: the box's name, a size twice, a pattern empty or with a 0, a
    /// scale not a positive normal f32.
    ///
    /// # Errors
    /// The rule broken.
    pub fn check(&self) -> Result<(), String> {
        let upper = self.radar.to_ascii_uppercase();
        if upper.len() <= 4 || !upper.ends_with(".DEF") {
            return Err(alloc::format!("`{}` is not a .DEF name", self.radar));
        }
        for (i, size) in self.sizes.iter().enumerate() {
            if self.sizes[..i].iter().any(|s| s.size == size.size) {
                return Err(alloc::format!("the size {} twice", size.size));
            }
            for pattern in [&size.columns, &size.rows] {
                if pattern.is_empty() || pattern.contains(&0) {
                    return Err(alloc::format!(
                        "the size {}: a tile's pixels empty or 0",
                        size.size
                    ));
                }
            }
        }
        for radar in self.sizes.iter().map(|s| s.radar).chain([self.other]) {
            let exponent = (radar.scale >> 23) & 0xFF;
            if radar.scale >> 31 != 0 || exponent == 0 || exponent == 0xFF {
                return Err(alloc::format!(
                    "the scale {:#x} is not a positive normal f32",
                    radar.scale
                ));
            }
        }
        Ok(())
    }
}

/// The pixels each tile's columns and rows take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    /// The columns', over and over from the left.
    pub columns: Vec<u8>,
    /// The rows', over and over from the top.
    pub rows: Vec<u8>,
}

/// A tile's colour on the minimap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Colour {
    /// Black: not explored.
    Black,
    /// The terrain's open colour (its code).
    Open(u8),
    /// The terrain's blocked colour (its code).
    Blocked(u8),
    /// An owner's: the flag palette's entry ([`crate::tables::OwnerArt::colour`]).
    Owner(u8),
}

/// What draws a map's minimap but its tiles' colours: the area, the map's side, the pixels by its
/// size, the terrains' DEFs and the entries of their open and blocked colours.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Look {
    /// The area's corner on the screen.
    pub at: [i32; 2],
    /// The area's width and height.
    pub size: [i32; 2],
    /// The map's side in tiles.
    pub side: usize,
    /// The pixels each tile takes; none on a map of another size (no tile drawn).
    pub pattern: Option<Pattern>,
    /// The terrains' DEFs by code (an empty name for a code the tables lack).
    pub terrains: Vec<String>,
    /// The palette entries of a terrain's open and blocked colours.
    pub entries: [u8; 2],
}

/// The minimap as last drawn (E24 M3 T3b, design §4.4): the view's corner its box shows and the
/// selected hero (its index among the live part's heroes) on the tile it stood on then. A scene
/// draws the minimap so; without it, at the scene's view and with the heroes where they stand (a
/// still).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shown {
    /// The view's corner.
    pub view: ViewAt,
    /// The selected hero and its tile.
    pub hero: Option<(usize, Pos)>,
}

/// The minimap's tiles as a scene draws them: the level's colours, the heroes' over them (E24 M3
/// T3b: the level's made once with its index, compared by their allocation first).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tiles {
    /// How they are drawn.
    pub look: Arc<Look>,
    /// The level's tiles' colours, row by row, the unexplored ones black.
    pub colours: Arc<Vec<Colour>>,
    /// The heroes' tiles over them: each one's place in `colours` and its owner's colour.
    pub heroes: Vec<(usize, Colour)>,
}

impl Tiles {
    /// Each tile column's pixels on the screen from the left, `[left, right)`; none on a map of
    /// another size.
    #[must_use]
    pub fn columns(&self) -> Vec<[i32; 2]> {
        let look = &self.look;
        look.spans(
            look.pattern.as_ref().map(|p| p.columns.as_slice()),
            look.at[0],
        )
    }

    /// Each tile row's pixels on the screen from the top, `[top, bottom)`; none on a map of
    /// another size.
    #[must_use]
    pub fn rows(&self) -> Vec<[i32; 2]> {
        let look = &self.look;
        look.spans(look.pattern.as_ref().map(|p| p.rows.as_slice()), look.at[1])
    }
}

impl Tiles {
    /// Tiles of `colours` without a hero over them.
    #[must_use]
    pub fn new(look: Arc<Look>, colours: Vec<Colour>) -> Self {
        Self {
            look,
            colours: Arc::new(colours),
            heroes: Vec::new(),
        }
    }

    /// Each tile's colour as drawn, row by row: the heroes' over the level's.
    #[must_use]
    pub fn shown(&self) -> Vec<Colour> {
        let mut colours = self.colours.to_vec();
        for &(i, colour) in &self.heroes {
            if let Some(c) = colours.get_mut(i) {
                *c = colour;
            }
        }
        colours
    }

    /// The terrains' DEFs its tiles take a colour of, once each by code, and whether a tile takes
    /// an owner's (the flag palette's).
    #[must_use]
    pub fn drawn(&self) -> (Vec<&str>, bool) {
        let mut codes = [false; 256];
        let mut owner = false;
        for colour in &self.shown() {
            match colour {
                Colour::Open(code) | Colour::Blocked(code) => codes[usize::from(*code)] = true,
                Colour::Owner(_) => owner = true,
                Colour::Black => {}
            }
        }
        let defs = self
            .look
            .terrains
            .iter()
            .zip(codes)
            .filter(|&(_, drawn)| drawn)
            .map(|(def, _)| def.as_str())
            .collect();
        (defs, owner)
    }
}

impl Look {
    /// The map's tiles' pixels along one side from `from`, each taking the next of `list` over
    /// and over (the two counters of 0x412E31 and 0x413221); none without a list.
    fn spans(&self, list: Option<&[u8]>, from: i32) -> Vec<[i32; 2]> {
        let Some(list) = list else {
            return Vec::new();
        };
        let mut at = from;
        (0..self.side)
            .map(|i| {
                let width = i32::from(list.get(i % list.len().max(1)).copied().unwrap_or(0));
                let span = [at, at.saturating_add(width)];
                at = span[1];
                span
            })
            .collect()
    }
}

/// A level's minimap but the heroes (E24 M3 T3a): each tile's colour by its terrain and the
/// object it records ([`crate::record`]), black where the viewer has not explored it, worked out
/// once with the level's index (T3b); and the box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Base {
    look: Arc<Look>,
    radar: (String, RadarArt),
    z: u8,
    colours: Arc<Vec<Colour>>,
}

impl Base {
    /// Level `z`'s minimap for `live`'s objects drawn in `lists` and its explored tiles, into the
    /// layout's area (the dialog's item [`panels::id::MINIMAP`]); none without that area. It
    /// holds for the live part it was built with, as the index does (E24 M3 T3b: whatever changes
    /// the explored tiles builds it again). An owned object without an
    /// entrance on the map takes its owner's colour (MCL-Q4's answer for its flags), its
    /// terrain's under the original's bugs (`tile_object_entrance` 0x4FCAA0 finds none).
    #[must_use]
    pub fn build(
        art: &MapArt,
        live: &Live,
        tables: &Tables,
        layout: &AdventureLayout,
        lists: &DrawLists,
        z: u8,
        bugs: Bugs,
    ) -> Option<Self> {
        let (at, size) = layout
            .dialog
            .item(panels::id::MINIMAP)
            .and_then(h3_ui::ui::Item::area)?;
        let place = layout.dialog.place;
        let minimap = &tables.map.minimap;
        let side = art.size;
        let (pattern, radar) = minimap.of(side);
        let highest = tables
            .terrains
            .keys()
            .max()
            .map_or(0, |&c| usize::from(c) + 1);
        let terrains = (0..highest)
            .map(|code| {
                u8::try_from(code)
                    .ok()
                    .and_then(|c| tables.terrains.get(&c))
                    .map(|t| t.def.clone())
                    .unwrap_or_default()
            })
            .collect();
        let look = Look {
            at: [
                at[0].saturating_add(place[0]),
                at[1].saturating_add(place[1]),
            ],
            size,
            side,
            pattern,
            terrains,
            entries: minimap.terrain,
        };
        let owners = &tables.map.owners;
        let records = &tables.records;
        let mut colours = Vec::with_capacity(side.saturating_mul(side));
        for y in 0..side {
            for x in 0..side {
                let (x, y) = (to_i32(x), to_i32(y));
                let Some(tile) = art
                    .tile(x, y, z)
                    .filter(|_| live.explored.explored(x, y, z))
                else {
                    colours.push(Colour::Black);
                    continue;
                };
                let code = tile.terrain.code;
                let record = records.record(art, live, tile, lists.at(x, y));
                let owner = record.object.filter(|&object| {
                    minimap.owned.contains(&record.class)
                        && (record.blocked || record.entrance)
                        && (record.entrance
                            || bugs == Bugs::Fixed
                            || objects::entrance(art, object).is_some())
                });
                colours.push(if let Some(object) = owner {
                    Colour::Owner(owners.colour(live.objects.get(object).and_then(|o| o.owner)))
                } else if record.blocked && minimap.blocking.contains(&record.class) {
                    Colour::Blocked(code)
                } else {
                    Colour::Open(code)
                });
            }
        }
        Some(Self {
            look: Arc::new(look),
            radar: (minimap.radar.clone(), radar),
            z,
            colours: Arc::new(colours),
        })
    }

    /// The tiles as `live` shows them: a hero's explored tile on the level in its owner's colour
    /// (the screen's player's selected hero's and every other standing hero's alike, 0x412F79 and
    /// 0x412FA5), the selected hero `hero` (its index) on the tile given with it (where the
    /// minimap last drew it, [`Shown`]).
    #[must_use]
    pub fn tiles(&self, live: &Live, tables: &Tables, hero: Option<(usize, Pos)>) -> Tiles {
        let side = self.look.side;
        let heroes = live
            .heroes
            .iter()
            .enumerate()
            .filter_map(|(i, h)| {
                let at = hero.filter(|&(j, _)| j == i).map_or(h.at, |(_, at)| at);
                let (x, y) = (usize::from(at.x), usize::from(at.y));
                let shown = at.z == self.z
                    && x < side
                    && y < side
                    && live.explored.explored(to_i32(x), to_i32(y), self.z);
                shown.then(|| {
                    let colour = Colour::Owner(tables.map.owners.colour(Some(h.owner)));
                    (y * side + x, colour)
                })
            })
            .collect();
        Tiles {
            look: Arc::clone(&self.look),
            colours: Arc::clone(&self.colours),
            heroes,
        }
    }

    /// The box at the view's corner `at`: the canvas placed at the area's corner plus the
    /// corner times the scale (each coordinate of a corner below 0 at the area's, its canvas cut
    /// by as much), drawn inside the area.
    #[must_use]
    pub fn radar(&self, at: ViewAt) -> Draw {
        let (def, radar) = &self.radar;
        let look = &self.look;
        let place = |v: i32, from: i32| {
            let scaled = scaled(v.unsigned_abs(), radar.scale);
            if v >= 0 {
                (from.saturating_add(scaled), 0)
            } else {
                (from, scaled)
            }
        };
        let (x, cut_x) = place(at.x, look.at[0]);
        let (y, cut_y) = place(at.y, look.at[1]);
        Draw::Radar {
            def: def.clone(),
            frame: radar.frame,
            at: [x, y],
            cut: [cut_x, cut_y],
            area: [
                look.at[0],
                look.at[1],
                look.at[0].saturating_add(look.size[0]),
                look.at[1].saturating_add(look.size[1]),
            ],
        }
    }
}

/// trunc(`v` × the f32 of `bits`), exactly (its mantissa and exponent in integers; a positive
/// normal f32, [`MinimapArt::check`]). The executable's x87 roundings of the product (and of its
/// sum with the area's corner) give the same for every corner the view reaches (design §4.4).
#[must_use]
pub fn scaled(v: u32, bits: u32) -> i32 {
    let mantissa = u64::from(bits & 0x7F_FFFF | 0x80_0000);
    let exponent = i64::from((bits >> 23) & 0xFF) - 127 - 23;
    let product = u64::from(v).saturating_mul(mantissa);
    let shift = u32::try_from(exponent.unsigned_abs().min(63)).unwrap_or(63);
    let value = if exponent < 0 {
        product >> shift
    } else {
        product
            .checked_shl(shift)
            .filter(|v| v >> shift == product)
            .unwrap_or(u64::MAX)
    };
    i32::try_from(value).unwrap_or(i32::MAX)
}

/// A tile's coordinate (a map's side fits).
fn to_i32(v: usize) -> i32 {
    i32::try_from(v).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests;
