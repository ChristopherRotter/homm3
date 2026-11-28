//! The generator's objects (survey `docs/rules/surveys/E14-generator.md` §2.1, "Placing an
//! object", "The template's masks as read" and "The objects as ported (T4b)"; §2.2, "The placing
//! tests as read"): the prototypes (`rmg_objects_read` 0x536200), the placed objects and their
//! counts with the entrance's distance flood (`rmg_generator_add_object` 0x5402A0, the cells'
//! lists by `rmg_map_place_object` 0x531EA0, [`Cells`]), and the placing test (`rmg_map_can_place`
//! 0x531CF0: `rmg_map_footprint_blocked` 0x5318B0, `rmg_prototype_outline` 0x532C80,
//! `rmg_map_outline_clear` 0x531170). Steps 4 on test a place with [`Objects::can_place`] and
//! place there through [`Objects::add`] (the generator's slot 1), checking [`Objects::take_fault`]
//! once at their end.
//!
//! An object template's masks use [`ObjectTemplate`]'s bits: bit `k` is the cell `k % 8` columns
//! left of and `k / 8` rows above the anchor, the original's bit 47 − `k`.

use alloc::vec::Vec;

use crate::cells::{Cells, ENTRANCE, FREE, KEPT, OBSTACLES};
use crate::painter::{ROCK, WATER};
use crate::part::{OBJECT_TYPES, ObjectSettings, ObjectTemplate};
use crate::settings::MapFormat;

/// The masks' 48 bits.
const MASK48: u64 = (1 << 48) - 1;

/// The entrance offsets of a template without one (`object_template_no_entrance` 0x640278).
pub const NO_ENTRANCE: [i32; 2] = [8, 6];

/// The monsters' class (54), whose prototypes are sorted by subtype.
pub(crate) const MONSTERS: usize = 54;

/// `rmg_neighbours` 0x69CDC0 (`rmg_neighbours_init` 0x530DA0): E, SE, S, SW, W, NW, N, NE, y
/// down; the odd ones diagonal.
pub(crate) const NEIGHBOURS: [[i32; 2]; 8] = [
    [1, 0],
    [1, 1],
    [0, 1],
    [-1, 1],
    [-1, 0],
    [-1, -1],
    [0, -1],
    [1, -1],
];

/// What the object functions meet that the original does not survive: it divides by zero
/// (RMG-QUIRK-3), throws `std::bitset`'s range error (its code 1) or reads and writes outside
/// its arrays (RMG-QUIRK-4). None is reached with the shipped tables: the first is kept until
/// the step that met it ends ([`Objects::take_fault`]), the call falling back as documented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectFault {
    /// `rmg_map_outline_clear`'s ring index `i mod 0`: a prototype without an outline (no blocked
    /// cell on its anchor's row) tested by `rmg_map_can_place` (RMG-QUIRK-3); it is refused.
    EmptyOutline,
    /// A footprint cell past the masks' 48 bits (a footprint past 8 × 6): `std::bitset`'s range
    /// error 0x41B410, which the original throws; the placing stops there, the test refuses.
    MaskBit,
    /// A cell read or written without a test off the map: a footprint on a level the map has not,
    /// an entrance (or the cell below it) off the map (RMG-QUIRK-4); the flood and the count are
    /// skipped, the test refuses.
    Outside,
    /// An entrance whose zone byte is past the zone generators (RMG-QUIRK-4): not counted.
    Zone,
    /// The cell below an entrance marked as an object's (bit 22) with none on it: the original
    /// reads its list's first from nowhere (RMG-QUIRK-4); the test refuses.
    NoObject,
    /// A prototype taken by its index in a class group's vector past its end (steps 4 and 5's
    /// town of a type without its line: `rmg_town_place_central`'s +0x658 vector; step 6's
    /// one-way monolith exit with fewer exits than entrances: `rmg_connection_by_monoliths`'
    /// +0x2F8; RMG-QUIRK-4): nothing placed.
    Prototype,
    /// A town's owner read from the colours by a zone's position (Ownership less 1) outside -1 to
    /// 7: the original reads the generator's next fields (RMG-QUIRK-4); the town is neutral.
    Colour,
    /// A guard monster's creature (`rmg_guard_monster_create` 0x540B20) the original does not
    /// survive: a monster prototype of a subclass past 144 (it writes past its array), the walk
    /// past the creatures with a prototype (a counted creature without one: it reads the word
    /// below its array; RMG-QUIRK-4), or a creature of AI value 0 (its count divides by zero,
    /// RMG-QUIRK-3): no monster.
    Creature,
    /// Step 6's monoliths without any two-way or one-way monolith prototype: the original reads
    /// class 43's empty vector (`rmg_connection_by_monoliths` 0x542D62, RMG-QUIRK-4), then takes
    /// the index `mod 0` (`rmg_zone_connections_make` 0x543957, RMG-QUIRK-3); nothing placed, the
    /// index kept.
    NoMonolith,
    /// Step 6's shipyard prototype drawn `mod 0` (`rmg_connection_by_shipyard` 0x541B85): no
    /// prototype of class 87 (RMG-QUIRK-3); no shipyard, the connection not made that way.
    NoShipyard,
    /// Step 6's gate prototype drawn `mod 0` (`rmg_connection_by_gate` 0x5421C3): no prototype
    /// of class 103 (RMG-QUIRK-3); no gates, the connection not made that way.
    NoGate,
}

/// An object template as the generator's copy (`rmg_object_template`, 0x4C bytes) holds what the
/// placing reads, from its `objects.txt` line and its picture's `.msk` (`object_template_text_read`
/// 0x514B80, `object_template_entrances_set` 0x514A60; RMG-OBJ-009).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shape {
    /// +0x4, the passable cells: the passability text OR NOT the colour mask (a cell without
    /// body pixels is passable).
    pub passable: u64,
    /// +0xC, the entrances: the actions text AND NOT +0x4.
    pub entrances: u64,
    /// +0x29: +0xC has a cell.
    pub has_entrance: bool,
    /// +0x2C and +0x30: the first entrance's columns left of and rows above the anchor, rows from
    /// the anchor's upward and each from the anchor's column leftward ([`NO_ENTRANCE`] without
    /// one).
    pub entrance: [i32; 2],
    /// +0x34 and +0x38: the footprint's columns and rows.
    pub width: i32,
    /// The footprint's rows.
    pub height: i32,
    /// +0x24 (the group) is 0 and +0x18 (the fourth column's terrains) holds water: the
    /// footprint's blocked cells go on water, never on land (the sea objects); clear, on land
    /// only.
    pub sea: bool,
    /// +0x1C, the class (the object type).
    pub class: u32,
    /// +0x20, the subclass (the subtype).
    pub subclass: u32,
}

impl Shape {
    /// The shape of `template` (`object_template_text_read` 0x514B80 and
    /// `object_template_entrances_set` 0x514A60).
    // rule: RMG-OBJ-009
    pub fn of(template: &ObjectTemplate) -> Self {
        let passable = (template.passability | !template.colour) & MASK48;
        let entrances = template.actions & !passable & MASK48;
        let has_entrance = entrances != 0;
        // The rows from the anchor's upward, each from its column leftward: the lowest bit.
        let entrance = if has_entrance {
            let k = entrances.trailing_zeros() as i32;
            [k % 8, k / 8]
        } else {
            NO_ENTRANCE
        };
        Self {
            passable,
            entrances,
            has_entrance,
            entrance,
            width: i32::from(template.width),
            height: i32::from(template.height),
            // The fourth column's character 0 (the original's bit 8) is water.
            sea: template.group == 0 && template.terrains & 1 != 0,
            class: template.class,
            subclass: template.subclass,
        }
    }

    /// The footprint cell `c` columns left of and `r` rows above the anchor: whether it is
    /// passable (+0x4) and an entrance (+0xC), the masks' bit 8r + c (the original's
    /// 47 − 8r − c); `None` past bit 47, `std::bitset`'s range error 0x41B410.
    pub fn cell(&self, c: i32, r: i32) -> Option<(bool, bool)> {
        let k = r.wrapping_mul(8).wrapping_add(c);
        (0..48)
            .contains(&k)
            .then(|| (self.passable >> k & 1 != 0, self.entrances >> k & 1 != 0))
    }
}

/// A prototype (`rmg_prototype`, 0xE8 bytes): an object template the generator may place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prototype {
    /// +0x0: its template, by its index in the part's object table.
    pub template: usize,
    /// What the placing reads of the template.
    pub shape: Shape,
    /// +0x8: the objects created from it ([`Objects::created`]).
    pub used: i32,
    /// +0x14: the ring of (dx, dy) cells around its footprint's blocked cells, computed when first
    /// tested (`rmg_prototype_outline` 0x532C80); empty before, and after for a template without
    /// a blocked cell on its anchor's row.
    pub outline: Vec<[i32; 2]>,
}

/// A placed object (`rmg_object`), one of the generator's +0xEC4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Object {
    /// +0x4: its prototype, by its index among [`Objects::prototypes`].
    pub prototype: usize,
    /// Its prototype's template, by its index in the part's object table.
    pub template: usize,
    /// The template's class.
    pub class: u32,
    /// The template's subclass.
    pub subclass: u32,
    /// +0x8 to +0x10: x, y, z.
    pub position: [i32; 3],
}

/// What `rmg_map_can_place` reads of a zone generator: its zone's id (+0x0's +0x0, the zone's
/// index once its template is kept) and whether its terrain (+0xC) is water.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaceZone {
    /// The zone's id, which a cell's zone byte must equal.
    pub id: i32,
    /// The zone generator's terrain is water (8).
    pub water: bool,
}

/// The generator's objects: its prototypes (+0x34, by class group), its placed objects (+0xEC4,
/// in placing order), the counts by type (+0x1110) and each zone generator's (+0x44).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Objects {
    prototypes: Vec<Prototype>,
    groups: Vec<Vec<usize>>,
    settings: Vec<ObjectSettings>,
    placed: Vec<Object>,
    count_by_type: Vec<i32>,
    zone_counts: Vec<Vec<i32>>,
    serial: i32,
    fault: Option<ObjectFault>,
}

impl Objects {
    /// The prototypes as `rmg_objects_read` 0x536200 files them for `format`: per line of the
    /// part's object table `templates`, in order, whose class the format allows (RoE below 165,
    /// AB below 222, SoD below 232; before SoD classes 43 to 45 only subclasses 0 to 2), a
    /// prototype appended to its class group's vector (the class's object settings +8); then the
    /// monsters' vector (54) sorted by subclass with the original's exchange sort. Nothing placed,
    /// every count 0, `zones` zone generators.
    // rule: RMG-OBJ-009
    pub fn new(
        templates: &[ObjectTemplate],
        settings: &[ObjectSettings],
        format: MapFormat,
        zones: usize,
    ) -> Self {
        let version = format.version();
        let mut prototypes = Vec::new();
        let mut groups = alloc::vec![Vec::new(); OBJECT_TYPES];
        for (index, t) in templates.iter().enumerate() {
            // `c_atoi`'s numbers, compared signed.
            let (class, subclass) = (t.class.cast_signed(), t.subclass.cast_signed());
            let allowed = (version >= 2 || class < 222)
                && (version >= 1 || class < 165)
                && (version >= 2 || !(43..=45).contains(&class) || subclass < 3)
                && (0..OBJECT_TYPES as i32).contains(&class);
            if !allowed {
                continue;
            }
            let group = settings
                .get(t.class as usize)
                .map_or(t.class as usize, |s| usize::from(s.class_group));
            if let Some(vector) = groups.get_mut(group) {
                vector.push(prototypes.len());
                prototypes.push(Prototype {
                    template: index,
                    shape: Shape::of(t),
                    used: 0,
                    outline: Vec::new(),
                });
            }
        }
        let mut objects = Self {
            prototypes,
            groups,
            settings: settings.to_vec(),
            placed: Vec::new(),
            count_by_type: alloc::vec![0; OBJECT_TYPES],
            zone_counts: alloc::vec![alloc::vec![0; OBJECT_TYPES]; zones],
            serial: 1,
            fault: None,
        };
        let mut monsters = core::mem::take(&mut objects.groups[MONSTERS]);
        exchange_sort(&mut monsters, |p| {
            objects.prototypes[p].shape.subclass.cast_signed()
        });
        objects.groups[MONSTERS] = monsters;
        objects
    }

    /// Every prototype, in the order filed.
    pub fn prototypes(&self) -> &[Prototype] {
        &self.prototypes
    }

    /// The prototypes of class group `class` (+0x34's vector of that index), as indices among
    /// [`Objects::prototypes`] in the vector's order; none past [`OBJECT_TYPES`].
    pub fn group(&self, class: usize) -> &[usize] {
        self.groups.get(class).map_or(&[], Vec::as_slice)
    }

    /// The prototypes of class group `class` whose template's subclass is `subclass`, in the
    /// vector's order.
    pub fn with_subclass(&self, class: usize, subclass: u32) -> impl Iterator<Item = usize> + '_ {
        self.group(class)
            .iter()
            .copied()
            .filter(move |&p| self.prototypes[p].shape.subclass == subclass)
    }

    /// The placed objects (+0xEC4) in placing order.
    pub fn placed(&self) -> &[Object] {
        &self.placed
    }

    /// The object settings (`object_settings_pointer` 0x660428) of the class of the first object
    /// on the cell (x, y, z) (its template's +0x1C); none without one, past the placed objects
    /// or past the table.
    pub(crate) fn first_settings(
        &self,
        cells: &Cells,
        [x, y, z]: [i32; 3],
    ) -> Option<&ObjectSettings> {
        cells
            .objects_at(x, y, z)
            .first()
            .and_then(|&i| self.placed.get(i as usize))
            .and_then(|o| self.settings.get(o.class as usize))
    }

    /// The placed objects by type (+0x1110).
    pub fn count_by_type(&self) -> &[i32] {
        &self.count_by_type
    }

    /// Zone generator `zone`'s objects by type whose entrance lies in its zone (+0x44); none past
    /// the zone generators.
    pub fn zone_count(&self, zone: usize) -> &[i32] {
        self.zone_counts.get(zone).map_or(&[], Vec::as_slice)
    }

    /// The first fault met since the last call, cleared: a step ends with it as its error.
    pub fn take_fault(&mut self) -> Option<ObjectFault> {
        self.fault.take()
    }

    /// Records `fault` unless one is kept already.
    pub(crate) fn fail(&mut self, fault: ObjectFault) {
        self.fault = self.fault.or(Some(fault));
    }

    /// The generator's +0xF44, the serial the next town, prison or guard monster takes (1 from
    /// the constructor).
    pub fn serial(&self) -> i32 {
        self.serial
    }

    /// The serial taken by a town, prison or guard monster created (+0xF44, raised by 1).
    pub(crate) fn take_serial(&mut self) -> i32 {
        let serial = self.serial;
        self.serial = serial.wrapping_add(1);
        serial
    }

    /// An object created from `prototype` (the object generators' create methods): its use
    /// count (+0x8) raised.
    pub fn created(&mut self, prototype: usize) {
        self.prototypes[prototype].used += 1;
    }

    /// The generator's slot 1, `rmg_generator_add_object` 0x5402A0, how every step places an
    /// object: an object of `prototype` at (x, y, z) put on the map (`rmg_map_place_object`,
    /// [`Cells`]), appended to the placed objects, its type counted; with an entrance, at
    /// (x − +0x2C, y − +0x30, z), the type counted in the zone generator its cell's zone byte
    /// names (when not negative), then the entrance's distance flood (RMG-OBJ-010). Returns the
    /// object's index among the placed objects.
    // rule: RMG-OBJ-009
    pub fn add(&mut self, cells: &mut Cells, prototype: usize, at: [i32; 3]) -> usize {
        let p = &self.prototypes[prototype];
        let (shape, template) = (p.shape, p.template);
        let index = self.placed.len();
        if let Err(fault) = cells.place(index as u32, &shape, at) {
            self.fail(fault);
        }
        self.placed.push(Object {
            prototype,
            template,
            class: shape.class,
            subclass: shape.subclass,
            position: at,
        });
        // A prototype's class is below the types (`Objects::new`).
        let class = shape.class as usize;
        self.count_by_type[class] += 1;
        if shape.has_entrance {
            let [x, y, z] = at;
            let e = [
                x.wrapping_sub(shape.entrance[0]),
                y.wrapping_sub(shape.entrance[1]),
            ];
            match cells.get_mut(e[0], e[1], z) {
                None => self.fail(ObjectFault::Outside),
                Some(c) => {
                    let zone = c.zone_index();
                    // The flood's start: the entrance's distance 0.
                    c.zone &= 0xFFFF_0000;
                    if let Ok(zone) = usize::try_from(zone) {
                        match self.zone_counts.get_mut(zone) {
                            Some(counts) => counts[class] += 1,
                            None => self.fail(ObjectFault::Zone),
                        }
                    }
                    flood(cells, e, z);
                }
            }
        }
        index
    }

    /// `rmg_map_can_place` 0x531CF0: whether `prototype` fits at (x, y, z) in `zone`, in order:
    /// its footprint not blocked (`rmg_map_footprint_blocked`, flag clear); its outline (computed
    /// once, `rmg_prototype_outline`) clear (`rmg_map_outline_clear`, its object flag the class's
    /// object settings +1 and +2 both set, kept flag clear); without an entrance, yes; else the
    /// cell below it, (x − +0x2C, y − +0x30 + 1), on the map's rows (its column untested), free,
    /// not rock, of the zone (its zone byte not negative and the id), without an entrance unless
    /// the first object on it is of a class with settings +2, and water exactly when the zone is
    /// (RMG-OBJ-011). A fault refuses ([`ObjectFault`]).
    // rule: RMG-OBJ-011
    pub fn can_place(
        &mut self,
        cells: &Cells,
        prototype: usize,
        at: [i32; 3],
        zone: PlaceZone,
    ) -> bool {
        match self.test(cells, prototype, at, zone) {
            Ok(fits) => fits,
            Err(fault) => {
                self.fail(fault);
                false
            }
        }
    }

    fn test(
        &mut self,
        cells: &Cells,
        prototype: usize,
        [x, y, z]: [i32; 3],
        zone: PlaceZone,
    ) -> Result<bool, ObjectFault> {
        let shape = self.prototypes[prototype].shape;
        if footprint_blocked(cells, &shape, [x, y, z], zone.id, false)? {
            return Ok(false);
        }
        let p = &mut self.prototypes[prototype];
        if p.outline.is_empty() {
            // The footprint test read every footprint cell: the outline's are within the bits.
            p.outline = outline(&shape).unwrap_or_default();
        }
        let object = self
            .settings
            .get(shape.class as usize)
            .is_some_and(|s| s.from_above && s.removable);
        if !outline_clear(cells, &p.outline, [x, y, z], object, zone, false)? {
            return Ok(false);
        }
        if !shape.has_entrance {
            return Ok(true);
        }
        let (u, v) = (
            x.wrapping_sub(shape.entrance[0]),
            y.wrapping_sub(shape.entrance[1]).wrapping_add(1),
        );
        if v >= cells.size().1 {
            return Ok(false);
        }
        let cell = cells.get(u, v, z).ok_or(ObjectFault::Outside)?;
        if cell.tile_data & FREE == 0 || cell.land() == u32::from(ROCK) {
            return Ok(false);
        }
        let index = cell.zone_index();
        if index < 0 || index != zone.id {
            return Ok(false);
        }
        if cell.tile_data & ENTRANCE != 0 {
            let first = cells.objects_at(u, v, z).first();
            let object = first
                .and_then(|&i| self.placed.get(i as usize))
                .ok_or(ObjectFault::NoObject)?;
            let removable = self
                .settings
                .get(object.class as usize)
                .is_some_and(|s| s.removable);
            if !removable {
                return Ok(false);
            }
        }
        Ok(zone.water == (cell.land() == u32::from(WATER)))
    }
}

/// The original's exchange sort (`rmg_objects_read` 0x536321 to 0x536386): for each place from
/// the first, each later place whose key is less is exchanged into it (`jle`: equal keys stay),
/// which need not keep equal keys in their order.
fn exchange_sort(v: &mut [usize], key: impl Fn(usize) -> i32) {
    for i in 0..v.len().saturating_sub(1) {
        for j in i + 1..v.len() {
            if key(v[i]) > key(v[j]) {
                v.swap(i, j);
            }
        }
    }
}

/// The entrance's distance flood (`rmg_generator_add_object` 0x540321 to 0x540594; RMG-OBJ-010):
/// from the entrance (its distance, +0x20 bits 0-15, made 0 by the caller), a queue of cells by
/// distance, the nearest last;
/// the last cell taken, its distance d (as the cell holds it), each neighbour of
/// [`NEIGHBOURS`] on the map (on level `z`, whatever it holds) whose distance is above d + 2
/// (orthogonal) or d + 3 (diagonal) gets that distance and goes into the queue before the first
/// cell of a distance at most its own (a binary search: the equal ones taken first in, first
/// out), until the queue is empty.
// rule: RMG-OBJ-010
fn flood(cells: &mut Cells, [x, y]: [i32; 2], z: i32) {
    // The cells and their distances, falling.
    let mut queue: Vec<([i32; 2], u32)> = alloc::vec![([x, y], 0)];
    while let Some(([x, y], _)) = queue.pop() {
        let d = cells.get(x, y, z).map_or(0, |c| c.zone & 0xFFFF) + 2;
        for (i, [dx, dy]) in NEIGHBOURS.into_iter().enumerate() {
            let near = d + (i as u32 & 1);
            let (u, v) = (x.wrapping_add(dx), y.wrapping_add(dy));
            let Some(c) = cells.get_mut(u, v, z) else {
                continue;
            };
            if near >= c.zone & 0xFFFF {
                continue;
            }
            c.zone = (c.zone & 0xFFFF_0000) | (near & 0xFFFF);
            enqueue(&mut queue, [u, v], near);
        }
    }
}

/// The flood's insertion (0x54052D to 0x54056E): `cell` at distance `near` goes into `queue`
/// (distances falling) before the first entry of a distance at most `near`, found by a binary
/// search; taken from the end, equal distances go first in, first out.
fn enqueue(queue: &mut Vec<([i32; 2], u32)>, cell: [i32; 2], near: u32) {
    let (mut lo, mut hi) = (0, queue.len());
    while lo < hi {
        let mid = (lo + hi) / 2;
        if near >= queue[mid].1 {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    queue.insert(lo, (cell, near));
}

/// `rmg_map_footprint_blocked` 0x5318B0 (`flag` its sixth argument, clear from
/// `rmg_map_can_place`): blocked when x < width − 1, x ≥ the map's width, y < height − 1 or
/// y ≥ its height; else per footprint cell (x − c, y − r), rows r from 0, columns c from 0: an
/// entrance unless free (bit 25), not rock, without an entrance (bit 22), of the zone (its zone
/// byte `zone`) and, with `flag`, without bit 26; then any cell not passable (the entrances too)
/// unless free, not rock, without an entrance, of the zone and on water exactly when the shape
/// is a sea object's. Passable cells are not tested (RMG-OBJ-011).
// rule: RMG-OBJ-011
pub(crate) fn footprint_blocked(
    cells: &Cells,
    shape: &Shape,
    [x, y, z]: [i32; 3],
    zone: i32,
    flag: bool,
) -> Result<bool, ObjectFault> {
    let (width, height, _) = cells.size();
    if x < shape.width.wrapping_sub(1)
        || x >= width
        || y < shape.height.wrapping_sub(1)
        || y >= height
    {
        return Ok(true);
    }
    for r in 0..shape.height {
        for c in 0..shape.width {
            let (passable, entrance) = shape.cell(c, r).ok_or(ObjectFault::MaskBit)?;
            let cell = cells
                .get(x.wrapping_sub(c), y.wrapping_sub(r), z)
                .ok_or(ObjectFault::Outside)?;
            let free = cell.tile_data & FREE != 0
                && cell.land() != u32::from(ROCK)
                && cell.tile_data & ENTRANCE == 0
                && cell.zone_index() == zone;
            if entrance && !(free && !(flag && cell.tile_data & OBSTACLES != 0)) {
                return Ok(true);
            }
            if !passable && !(free && (cell.land() == u32::from(WATER)) == shape.sea) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// `rmg_prototype_outline` 0x532C80: on the anchor's row, from the anchor leftward, the first
/// cell not passable (or an entrance); none within the width: no outline. Else from the cell below
/// it, as if come from above, a walk of orthogonal steps ([`NEIGHBOURS`]' even entries): from the
/// way back it turns a quarter clockwise and tries that way (a right turn), straight on, a left
/// turn, back, entering a cell outside the footprint or passable and no entrance in it (after
/// four refusals it steps the last way tried), each cell recorded (dx, dy) before its step, until
/// it is back at the first: a clockwise ring on the screen around the blocked group the start
/// touches (RMG-OBJ-011). The walk stays within one cell past the footprint's box on each side
/// (outside the footprint every first try is taken), so a walk that has not returned after as
/// many steps as it has states (four ways in those cells) never would: it stops there (no
/// footprint's walk reaches that, the exhaustive check up to 18 cells).
///
/// # Errors
///
/// [`ObjectFault::MaskBit`] for a cell past the masks' bits.
// rule: RMG-OBJ-011
pub(crate) fn outline(shape: &Shape) -> Result<Vec<[i32; 2]>, ObjectFault> {
    let (w, h) = (shape.width, shape.height);
    let mut dx = 0i32;
    while dx.wrapping_neg() < w {
        let (passable, entrance) = shape.cell(-dx, 0).ok_or(ObjectFault::MaskBit)?;
        if !passable || entrance {
            break;
        }
        dx -= 1;
    }
    let mut ring = Vec::new();
    if dx == w.wrapping_neg() {
        return Ok(ring);
    }
    let start = [dx, 1];
    let mut at = start;
    let mut way = 6;
    let states = w
        .saturating_add(4)
        .saturating_mul(h.saturating_add(4))
        .saturating_mul(4);
    for _ in 0..states {
        ring.push(at);
        let mut refused = 0;
        loop {
            way = (way + 6) & 7;
            let [nx, ny] = [at[0] + NEIGHBOURS[way][0], at[1] + NEIGHBOURS[way][1]];
            if nx > 0 || -nx >= w || ny > 0 || -ny >= h {
                break;
            }
            let (passable, entrance) = shape.cell(-nx, -ny).ok_or(ObjectFault::MaskBit)?;
            refused += 1;
            if (passable && !entrance) || refused == 4 {
                break;
            }
        }
        at = [at[0] + NEIGHBOURS[way][0], at[1] + NEIGHBOURS[way][1]];
        way = (way + 4) & 7;
        if at == start {
            break;
        }
    }
    Ok(ring)
}

/// `rmg_map_outline_clear` 0x531170 over `outline` around (x, y, z) (`object` and `kept` its
/// flags): a cell is open when on the map, free, not rock, without an entrance, with `kept` also
/// with bit 27, water exactly when `zone` is, and of the zone; without `object` an entrance on the
/// ring refuses at once. The ring is walked cyclically from its first cell to it again (n + 1
/// cells), the state before the first counted closed: a second change from open to closed
/// refuses; at the end it is clear when the last cell (the first) is open or after one change,
/// not when all are closed. So the open cells form one unbroken arc or the whole ring
/// (RMG-OBJ-011).
///
/// # Errors
///
/// [`ObjectFault::EmptyOutline`] (the ring's index divides by its length, 0) and
/// [`ObjectFault::Outside`] on a level the map has not.
// rule: RMG-OBJ-011
pub(crate) fn outline_clear(
    cells: &Cells,
    outline: &[[i32; 2]],
    [x, y, z]: [i32; 3],
    object: bool,
    zone: PlaceZone,
    kept: bool,
) -> Result<bool, ObjectFault> {
    let n = outline.len();
    if n == 0 {
        return Err(ObjectFault::EmptyOutline);
    }
    let (width, height, _) = cells.size();
    let (mut closed, mut changed) = (true, false);
    for i in 0..=n {
        let was = closed;
        let [dx, dy] = outline[i % n];
        let (u, v) = (x.wrapping_add(dx), y.wrapping_add(dy));
        closed = true;
        if (0..width).contains(&u) && (0..height).contains(&v) {
            let c = cells.get(u, v, z).ok_or(ObjectFault::Outside)?;
            if !object && c.tile_data & ENTRANCE != 0 {
                return Ok(false);
            }
            closed = c.tile_data & FREE == 0
                || c.land() == u32::from(ROCK)
                || c.tile_data & ENTRANCE != 0
                || (kept && c.tile_data & KEPT == 0)
                || (c.land() == u32::from(WATER)) != zone.water
                || c.zone_index() != zone.id;
        }
        if closed && !was {
            if changed {
                return Ok(false);
            }
            changed = true;
        }
    }
    Ok(!closed || changed)
}

#[cfg(test)]
pub(crate) mod tests;
