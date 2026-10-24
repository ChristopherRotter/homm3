//! The objects' draw lists (E24 M1 T4a, *provisional*; `docs/epics/E24-design.md` §2.1): which
//! objects each tile draws, bottom to top, as the original builds the lists when it loads a map.
//!
//! A template's footprint and masks come from its picture's `.msk` (`msk_load` 0x514610: the
//! width and height in tiles, then the colour mask, the cells with body pixels, and the shadow
//! mask, the cells with shadow pixels; the file's bit `8 b + i` is the mask's bit, so a cell
//! (dx, dy) is bit `7 − dx` of byte `5 − dy`, as the `.h3m`'s masks: [`Shape`]). Every object in
//! the map's order puts an [`Entry`] on each tile of its footprint that lies on the map
//! (place_object_tiles 0x505B20), each cell at the layer the template's passability gives it
//! ([`layers`], object_tile_layers 0x505060), inserted by layer (tile_draw_insert 0x505230:
//! [`DrawLists`]). Heroes, boats, the Grail, random heroes, hero placeholders and events have no
//! entries.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use h3_assets::msk::Mask;

use h3_render::Bugs;

use crate::art::{CELLS, MapArt, TemplateArt, mask_bit};
use crate::live::Live;

/// What the passes need of a template's picture: its `.msk` and its DEF's frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shape {
    /// The footprint's width in tiles (0–8).
    pub width: u8,
    /// Its height (0–6).
    pub height: u8,
    /// The colour mask as stored: the cells pass 7 draws a body piece in.
    pub colour: [u8; 6],
    /// The shadow mask as stored: the cells pass 5 draws a shadow piece in.
    pub shadow: [u8; 6],
    /// The frames of the DEF's first group (its group 0 in every object DEF), at least one: an
    /// object's frame is its phase plus the moment's frame steps, modulo these (design §1.6).
    pub frames: u32,
}

/// Why a picture gives no shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeError {
    /// The `.msk` is wider than 8 or higher than 6 tiles (its masks have 8 × 6 cells).
    Size(u8, u8),
    /// The DEF has no frame in its first group (the original divides by the count).
    NoFrames,
}

impl fmt::Display for ShapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Size(w, h) => write!(f, "a {w} x {h} .msk is larger than 8 x 6"),
            Self::NoFrames => write!(f, "its DEF's first group has no frame"),
        }
    }
}

impl Shape {
    /// The shape of a picture whose `.msk` is `mask` and whose DEF's first group has `frames`.
    ///
    /// # Errors
    /// [`ShapeError`]: a mask larger than its cells, no frame.
    pub fn new(mask: &Mask, frames: u32) -> Result<Self, ShapeError> {
        if mask.width > CELLS.0 || mask.height > CELLS.1 {
            return Err(ShapeError::Size(mask.width, mask.height));
        }
        if frames == 0 {
            return Err(ShapeError::NoFrames);
        }
        Ok(Self {
            width: mask.width,
            height: mask.height,
            colour: mask.masks[0],
            shadow: mask.masks[1],
            frames,
        })
    }

    /// Whether the cell `dx` columns left of and `dy` rows above the anchor has body pixels.
    #[must_use]
    pub fn body(&self, dx: u8, dy: u8) -> bool {
        mask_bit(&self.colour, dx, dy).unwrap_or(false)
    }

    /// Whether the cell has shadow pixels.
    #[must_use]
    pub fn shadow(&self, dx: u8, dy: u8) -> bool {
        mask_bit(&self.shadow, dx, dy).unwrap_or(false)
    }
}

/// The shapes by DEF name, upper-cased (the archives look names up ignoring case), and the frames
/// of group 0 of the pictures drawn without a `.msk` (the heroes' and boats', E24 M1 T5a).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Shapes {
    shapes: BTreeMap<String, Shape>,
    frames: BTreeMap<String, u32>,
}

impl Shapes {
    /// Keeps `shape` as the shape of the DEF `name`.
    pub fn insert(&mut self, name: &str, shape: Shape) {
        self.shapes.insert(name.to_ascii_uppercase(), shape);
    }

    /// The shape of the DEF `name`.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Shape> {
        self.shapes.get(&name.to_ascii_uppercase())
    }

    /// Keeps `frames` as the frames of the DEF `name`'s group 0, for a picture without a shape.
    pub fn insert_frames(&mut self, name: &str, frames: u32) {
        self.frames.insert(name.to_ascii_uppercase(), frames);
    }

    /// The frames of the DEF `name`'s group 0: its shape's, or those kept without one.
    #[must_use]
    pub fn frames(&self, name: &str) -> Option<u32> {
        let upper = name.to_ascii_uppercase();
        self.shapes
            .get(&upper)
            .map(|s| s.frames)
            .or_else(|| self.frames.get(&upper).copied())
    }
}

/// Every DEF whose shape the objects of `art` and `live` need (the map's templates', for their
/// size and masks, and the DEFs random objects were drawn as and their own, §2.7), as named,
/// without repeats (ignoring case).
#[must_use]
pub fn pictures(art: &MapArt, live: &Live) -> Vec<String> {
    let mut seen = BTreeMap::new();
    let drawn = live
        .objects
        .iter()
        .filter_map(|o| o.drawn.as_ref())
        .flat_map(|d| core::iter::once(&d.def).chain(&d.own));
    for def in art.templates.iter().map(|t| &t.def).chain(drawn) {
        seen.entry(def.to_ascii_uppercase())
            .or_insert_with(|| def.clone());
    }
    seen.into_values().collect()
}

/// The layer of each cell `[dx][dy]` of a template drawn with `shape` (object_tile_layers
/// 0x505060; design §2.1): a flat template's are all 0. Otherwise per column, walking up from the
/// anchor's row with L = 1: a blocked cell on a passable one starts again at 1; a passable cell
/// beside a blocked one on its right takes that one's layer; the cell gets L and L grows by one.
#[must_use]
pub fn layers(template: &TemplateArt, shape: &Shape) -> [[u8; 6]; 8] {
    let mut out = [[0; 6]; 8];
    if template.flat() {
        return out;
    }
    for dx in 0..shape.width.min(CELLS.0) {
        let mut layer = 1u8;
        for dy in 0..shape.height.min(CELLS.1) {
            let passable = template.passable(dx, dy);
            if dy > 0 && !passable && template.passable(dx, dy - 1) {
                layer = 1;
            }
            if dx > 0 && passable && !template.passable(dx - 1, dy) {
                layer = cell(&out, dx - 1, dy);
            }
            out[usize::from(dx)][usize::from(dy)] = layer;
            layer = layer.saturating_add(1);
        }
    }
    out
}

/// A cell of a layer grid: `dx` below 8, `dy` below 6.
fn cell(layers: &[[u8; 6]; 8], dx: u8, dy: u8) -> u8 {
    layers[usize::from(dx)][usize::from(dy)]
}

/// One draw-list entry (map_item +0xE's 4-byte entries).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    /// The object's index in the map's order.
    pub object: usize,
    /// The cell: columns left of the anchor.
    pub dx: u8,
    /// Rows above the anchor.
    pub dy: u8,
    /// The cell's layer.
    pub layer: u8,
}

/// The classes that get no entries (place_object_tiles 0x505B20): the boat (8), the hero (34),
/// the Grail (36), the random hero (70), the hero placeholder (214); an event (26) only marks its
/// tiles.
pub const NO_ENTRIES: [u32; 6] = [8, 26, 34, 36, 70, 214];

/// Why a level has no draw lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectsError {
    /// An object's DEF has no shape: the object and the DEF's name.
    Shape(usize, String),
}

impl fmt::Display for ObjectsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shape(object, def) => write!(f, "object {object}: no .msk for {def}"),
        }
    }
}

/// A placed object: what the insertion compares.
#[derive(Debug, Clone, Copy)]
struct Placed {
    /// The anchor.
    x: i32,
    y: i32,
    /// The footprint's size.
    width: i32,
    height: i32,
    /// The cells' layers.
    layers: [[u8; 6]; 8],
}

impl Placed {
    /// The layer of its cell on tile (x, y) of its footprint (0 outside its cells).
    fn layer_at(&self, x: i32, y: i32) -> u8 {
        let (dx, dy) = (usize::try_from(self.x - x), usize::try_from(self.y - y));
        dx.ok()
            .zip(dy.ok())
            .and_then(|(dx, dy)| self.layers.get(dx).and_then(|column| column.get(dy)))
            .copied()
            .unwrap_or(0)
    }
}

/// The draw lists of one level: per tile, its entries bottom to top.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawLists {
    size: usize,
    lists: Vec<Vec<Entry>>,
}

impl DrawLists {
    /// The lists of level `z`: every object of the level in the map's order put on the tiles of
    /// its footprint on the map, by its template in the map (a random object too: the original
    /// places it before the draw, which keeps the template's size and masks, §2.7); then the
    /// fallen objects' entries taken out, the others keeping their order, as the original takes
    /// an object off the map (remove_map_object 0x4AABB0: its entry deleted from each tile's
    /// list, the later ones moved down, 0x4AACE2; E24 M1 T6b, the reviewer's R1-1), so that the
    /// standing objects are ordered as they were among the fallen ones.
    ///
    /// # Errors
    /// [`ObjectsError::Shape`]: an object of the level with entries whose DEF has no shape.
    pub fn build(art: &MapArt, live: &Live, shapes: &Shapes, z: u8) -> Result<Self, ObjectsError> {
        let size = art.size;
        let mut lists = vec![Vec::new(); size.saturating_mul(size)];
        let mut placed: Vec<Option<Placed>> = Vec::with_capacity(art.objects.len());
        for (index, object) in art.objects.iter().enumerate() {
            let template = template(art, index);
            let drawn = template.filter(|t| object.anchor.z == z && !NO_ENTRIES.contains(&t.class));
            let Some(template) = drawn else {
                placed.push(None);
                continue;
            };
            let shape = shapes
                .get(&template.def)
                .ok_or_else(|| ObjectsError::Shape(index, template.def.clone()))?;
            let new = Placed {
                x: i32::from(object.anchor.x),
                y: i32::from(object.anchor.y),
                width: i32::from(shape.width),
                height: i32::from(shape.height),
                layers: layers(template, shape),
            };
            // A shape is at most the template's 8 × 6 cells (`Shape::new`; its fields are public).
            for dx in 0..shape.width.min(CELLS.0) {
                for dy in 0..shape.height.min(CELLS.1) {
                    let (x, y) = (new.x - i32::from(dx), new.y - i32::from(dy));
                    let Some(list) = tile_index(size, x, y).and_then(|t| lists.get_mut(t)) else {
                        continue;
                    };
                    let entry = Entry {
                        object: index,
                        dx,
                        dy,
                        layer: cell(&new.layers, dx, dy),
                    };
                    insert(list, entry, &new, &placed, size);
                }
            }
            placed.push(Some(new));
        }
        let fallen = |e: &Entry| live.objects.get(e.object).is_some_and(|o| !o.standing);
        for list in &mut lists {
            list.retain(|e| !fallen(e));
        }
        Ok(Self { size, lists })
    }

    /// The entries of tile (x, y), bottom to top; none off the map.
    #[must_use]
    pub fn at(&self, x: i32, y: i32) -> &[Entry] {
        tile_index(self.size, x, y)
            .and_then(|i| self.lists.get(i))
            .map_or(&[], Vec::as_slice)
    }
}

/// The template of object `index` in the map: its size, masks and priority, a random object's
/// too (§2.7).
#[must_use]
pub fn template(art: &MapArt, index: usize) -> Option<&TemplateArt> {
    art.objects
        .get(index)
        .and_then(|o| art.templates.get(o.template))
}

/// The class object `index` is drawn as: what a random object became, else its template's (the
/// view passes switch on the drawn template's class, +0x38: design §2.9).
#[must_use]
pub fn class(art: &MapArt, live: &Live, index: usize) -> Option<u32> {
    let drawn = live.objects.get(index).and_then(|o| o.drawn.as_ref());
    drawn
        .map(|d| d.class)
        .or_else(|| template(art, index).map(|t| t.class))
}

/// The tile of object `index`'s entrance as the original finds it (object_entrance_tile
/// 0x4FEC30; design §2.9): its template's first visitable cell on the map, by row from the
/// anchor's upwards, each row from the anchor's column leftwards. The original looks only within
/// its picture's `.msk` (width +0x10, height +0x11), this in all 8 × 6 cells: the same, since no
/// template of `Objects.txt` or the shipped maps has a visitable cell outside it (the gate checks
/// it; E24 M1 T6b, the reviewer's R1-5).
#[must_use]
pub fn entrance(art: &MapArt, index: usize) -> Option<(i32, i32)> {
    let (object, template) = art.objects.get(index).zip(template(art, index))?;
    let side = i32::try_from(art.size).unwrap_or(i32::MAX);
    let on_map = |v: i32| (0..side).contains(&v);
    (0..CELLS.1)
        .flat_map(|dy| (0..CELLS.0).map(move |dx| (dx, dy)))
        .map(|(dx, dy)| {
            let at = (
                i32::from(object.anchor.x) - i32::from(dx),
                i32::from(object.anchor.y) - i32::from(dy),
            );
            (at, template.visitable(dx, dy))
        })
        .find(|&((x, y), visitable)| visitable && on_map(x) && on_map(y))
        .map(|(at, _)| at)
}

/// The owner whose colour object `index`'s flag pixels take (passes 4 and 7: tile_object_owner
/// 0x40FD20 of the tile object_entrance_tile 0x4FEC30 gives; design §2.9): its own. The original
/// takes it from the object on its entrance tile, and from tile (0, 0) of the surface for an
/// object without an entrance on the map (MCL-Q4): with its bugs, that tile's object's owner (the
/// last object whose entrance it is), none without one. *Provisional*: the original's tile object
/// is the topmost entry of the tile's draw list whose cell is visitable (tile_refresh_object
/// 0x505610), a hero there standing for it; no shipped map has an owned object without an
/// entrance (the gate counts them), so this approximation is not compared (the reviewer's R1-5).
#[must_use]
pub fn flag_owner(art: &MapArt, live: &Live, index: usize, bugs: Bugs) -> Option<u8> {
    let own = |i: usize| live.objects.get(i).and_then(|o| o.owner);
    if bugs == Bugs::Fixed || entrance(art, index).is_some() {
        return own(index);
    }
    (0..art.objects.len())
        .rev()
        .find(|&i| {
            art.objects.get(i).is_some_and(|o| o.anchor.z == 0) && entrance(art, i) == Some((0, 0))
        })
        .and_then(own)
}

/// The DEF object `index` is drawn from: a random object's own (the client's, MCL-Q3) or, with the
/// original's bugs or without one, the one the original drew it as; else its template's.
#[must_use]
pub fn def<'a>(art: &'a MapArt, live: &'a Live, index: usize, bugs: Bugs) -> Option<&'a str> {
    let drawn = live.objects.get(index).and_then(|o| o.drawn.as_ref());
    drawn
        .map(|d| match (bugs, &d.own) {
            (Bugs::Fixed, Some(own)) => own.as_str(),
            _ => d.def.as_str(),
        })
        .or_else(|| template(art, index).map(|t| t.def.as_str()))
}

/// The shape object `index` is drawn with: its template's size and masks (a random object's are
/// its random template's, which the original's copy keeps: 0x4C9990, §2.7) and the frames of the
/// DEF it is drawn from ([`def`]); a random object's own picture's whole shape when the client
/// draws its own (MCL-Q3).
#[must_use]
pub fn shape(
    art: &MapArt,
    live: &Live,
    shapes: &Shapes,
    index: usize,
    bugs: Bugs,
) -> Option<Shape> {
    let drawn = def(art, live, index, bugs).and_then(|d| shapes.get(d))?;
    let own = live
        .objects
        .get(index)
        .and_then(|o| o.drawn.as_ref())
        .is_some_and(|d| bugs == Bugs::Fixed && d.own.is_some());
    if own {
        return Some(*drawn);
    }
    let template = template(art, index).and_then(|t| shapes.get(&t.def))?;
    Some(Shape {
        frames: drawn.frames,
        ..*template
    })
}

/// The index of tile (x, y) of a level of `size`, if it is on the map.
fn tile_index(size: usize, x: i32, y: i32) -> Option<usize> {
    let (x, y) = (usize::try_from(x).ok()?, usize::try_from(y).ok()?);
    (x < size && y < size).then(|| y * size + x)
}

/// Inserts `entry` of the object `new` into a tile's `list` (tile_draw_insert 0x505230): walking
/// back from the end, past every entry of a higher layer, and past an entry of the same layer
/// whose object is higher than `new` on some tile both footprints cover on the map; it stops above
/// an entry of a lower layer, above one of the same layer not higher anywhere, or at the start.
fn insert(
    list: &mut Vec<Entry>,
    entry: Entry,
    new: &Placed,
    placed: &[Option<Placed>],
    size: usize,
) {
    let mut at = list.len();
    while let Some(below) = at.checked_sub(1).and_then(|i| list.get(i)) {
        let passes = match entry.layer.cmp(&below.layer) {
            core::cmp::Ordering::Greater => false,
            core::cmp::Ordering::Less => true,
            core::cmp::Ordering::Equal => placed
                .get(below.object)
                .copied()
                .flatten()
                .is_some_and(|other| higher_somewhere(&other, new, size)),
        };
        if !passes {
            break;
        }
        at -= 1;
    }
    list.insert(at, entry);
}

/// Whether `other` has a higher layer than `new` on some tile of their footprints'
/// intersection on the map (IntersectRect, clipped to the map: 0x505342 to 0x505384).
fn higher_somewhere(other: &Placed, new: &Placed, size: usize) -> bool {
    let side = i32::try_from(size).unwrap_or(i32::MAX);
    let left = (new.x - new.width + 1)
        .max(other.x - other.width + 1)
        .max(0);
    let top = (new.y - new.height + 1)
        .max(other.y - other.height + 1)
        .max(0);
    let right = (new.x + 1).min(other.x + 1).min(side);
    let bottom = (new.y + 1).min(other.y + 1).min(side);
    (left..right).any(|x| (top..bottom).any(|y| other.layer_at(x, y) > new.layer_at(x, y)))
}

#[cfg(test)]
mod tests;
