//! Writer for the subset of the SoD map format (`.h3m`, version `0x1C`) that staged experiments
//! need (E04a T2, OPEN-QUESTIONS Q28): a base terrain with rectangles of other terrains and
//! straight east-west roads (E08 T10a), an underground level below it (E08 T22a1), players
//! without a main town, heroes with custom armies, skills, artifacts, spells and primary skills
//! (or their hero type's own), wandering monsters, resource piles (E08 T16b; guarded since E08
//! T22a1), special grounds (E04c T3) and towns of a fixed faction (E04c T5b).
//!
//! Layout notes are in `docs/formats/h3m.md`. The field order follows VCMI 1.7.5's map reader
//! (`lib/mapping/MapFormatH3M.cpp`, `MapReaderH3M.cpp`, `ObjectTemplate::readMap`; GPL-2.0-or-later,
//! the order and sizes were ported, no code), and was checked by parsing SoD maps shipped with the
//! game. The output is a whole map file: [`write`] produces the uncompressed bytes, [`gzip`]
//! wraps them the way the game stores maps.
//!
//! Every value is validated before anything is written; [`write`] never panics. The writer moved
//! here from `h3-formats` with the content split (R01a, ADR-0015 point 3).

use std::fmt;

use crate::gzip::crc32;

/// Why a map could not be written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteError {
    /// A value is outside the range the format or the map allows.
    OutOfRange {
        /// What was being written.
        what: &'static str,
        /// The offending value.
        value: u64,
    },
    /// An `Objects.txt` template line is malformed.
    Template(&'static str),
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange { what, value } => write!(f, "{what} out of range: {value}"),
            Self::Template(message) => write!(f, "Objects.txt template: {message}"),
        }
    }
}

impl std::error::Error for WriteError {}

/// Format version of The Shadow of Death maps.
pub const SOD_FORMAT: u32 = 0x1C;
/// Number of heroes in a SoD map's tables.
pub const HERO_COUNT: u8 = 156;
/// Number of hero classes.
pub const HERO_CLASS_COUNT: u8 = 18;
/// Number of artifacts in a SoD map's tables.
pub const ARTIFACT_COUNT: u16 = 144;
/// Number of creatures a SoD map may name.
pub const CREATURE_COUNT: u16 = 145;
/// Number of secondary skills.
pub const SKILL_COUNT: u8 = 28;
/// Artifact slots of a SoD hero (`ArtSlots.txt`: head … war machines, spell book, misc 5).
pub const ARTIFACT_SLOTS: usize = 19;
/// Army slots of a hero.
pub const ARMY_SLOTS: usize = 7;
/// Players of a map.
pub const PLAYERS: usize = 8;

/// Object class of a hero on the map.
const OBJECT_HERO: u32 = 34;
/// Object classes whose body is a wandering monster (monster, random monsters).
const MONSTER_OBJECTS: [u32; 5] = [54, 71, 162, 163, 164];
/// Object class of a resource pile (subclass = resource, `RESTYPES.TXT` order: wood, mercury,
/// ore, sulfur, crystal, gems, gold).
const PILE_OBJECT: u32 = 79;
/// Resources a pile may hold.
const RESOURCE_COUNT: u32 = 7;
/// Object classes of the special grounds, whose objects have no body (E04c T3): Cursed Ground
/// (21, SoD 223), Magic Plains (46, SoD 230), Clover Field (222), Evil Fog (224), Favorable Winds
/// (225), Fiery Fields (226), Holy Ground (227), Lucid Pools (228), Magic Clouds (229),
/// Rocklands (231); `Objects.txt` lists their pictures.
pub const GROUND_OBJECTS: [u32; 12] = [21, 46, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231];
/// Object classes of the objects without a body the writer places, their template alone: the
/// Cartographers (13; subclass 0 water, 1 land, 2 subterranean), entered and not visited from
/// above (`packs/base/adventure.toml`; E08 T32a: batch E4's attack on a hero standing on one);
/// since E08 T36a the trees, obstacles that only block (Oak Trees 135, Pine Trees 137, Trees
/// 155), and the Border Gates (212; subclass the key's colour, 0 light blue … 7 black), visited
/// from the tile before (batch E5's flights over a tree line and a gate).
pub const PLAIN_OBJECTS: [u32; 5] = [13, 135, 137, 155, 212];
/// Object class of a town of a fixed faction (subclass = faction, `TOWNTYPE.TXT` order: Castle 0
/// … Conflux 8; E04c T5b).
pub const TOWN_OBJECT: u32 = 98;
/// Factions a town may have.
pub const FACTION_COUNT: u32 = 9;
/// Buildings a map's town lists (bits 0–40 of its six bytes; `docs/formats/h3m.md`, "Town
/// buildings").
pub const MAP_BUILDINGS: u8 = 41;
/// The largest fixed amount a pile may hold: the original keeps it in 19 bits, and setup refuses
/// more (`h3_adventure::MAX_PICKUP_AMOUNT`, mirrored here: the writer does not depend on the
/// adventure crate). Gold counts in hundreds.
const MAX_PILE_AMOUNT: u32 = 0x7_FFFF;
/// Longest string we write (the game reads a `u32` length; this keeps files sane).
const MAX_STRING: usize = 30_000;
/// Hero types banned from the random pool by default in SoD maps (the RoE Lord Haart, 4, and the
/// campaign heroes 145–155), as in the editor's default (observed in the game's own SoD maps).
const DEFAULT_ALLOWED_HEROES: [u8; 20] = [
    0xef, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0x01, 0x00,
];
/// Artifacts banned from random generation by default (127–143: Vial of Dragon Blood,
/// Armageddon's Blade, the combination artifacts and the three unused ones), as in the game's
/// own SoD maps.
const DEFAULT_BANNED_ARTIFACTS: [u8; 18] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80, 0xff, 0xff,
];

/// Terrain types in map files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terrain {
    /// Dirt.
    Dirt = 0,
    /// Sand.
    Sand = 1,
    /// Grass.
    Grass = 2,
    /// Snow.
    Snow = 3,
    /// Swamp.
    Swamp = 4,
    /// Rough.
    Rough = 5,
    /// Subterranean.
    Subterranean = 6,
    /// Lava.
    Lava = 7,
    /// Water.
    Water = 8,
    /// Rock (impassable).
    Rock = 9,
}

impl Terrain {
    /// The first of eight interior tile pictures ("view" in map files) of this terrain: a tile
    /// whose eight neighbours have its terrain shows one of them. Observed in the installation's
    /// maps (E08 T10a, every interior tile of 164 counted, four of them the user's own): dirt
    /// 21–44, sand 0–23, the six other land terrains 49–72, water 21–32, rock 0–7; the first eight
    /// of each range are by far the most common. Between two terrains the writer puts interior
    /// pictures too, which the game draws with a hard edge.
    const fn interior_view(self) -> u8 {
        match self {
            Self::Dirt | Self::Water => 21,
            Self::Sand | Self::Rock => 0,
            Self::Grass
            | Self::Snow
            | Self::Swamp
            | Self::Rough
            | Self::Subterranean
            | Self::Lava => 49,
        }
    }
}

/// An object template ("def"): picture, footprint and object class, as stored in a map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectTemplate {
    /// Picture file name.
    pub def: String,
    /// Passability bits (1 = passable), 6 rows of 8, map-file bit order.
    pub passable: [u8; 6],
    /// Visitability bits (1 = visitable), same order.
    pub visitable: [u8; 6],
    /// Terrains the object may stand on (bit per terrain).
    pub terrains: u16,
    /// Editor menu terrains (bit per terrain).
    pub menu: u16,
    /// Object class (34 hero, 54 monster, …).
    pub class: u32,
    /// Object subclass (hero class, creature id, …).
    pub subclass: u32,
    /// Editor group.
    pub group: u8,
    /// Drawing priority.
    pub priority: u8,
}

impl ObjectTemplate {
    /// Parses one template line of the game's `Objects.txt` (`def`, two 48-character masks, two
    /// 9-character terrain masks, class, subclass, group, priority, separated by spaces).
    ///
    /// # Errors
    /// [`WriteError::Template`] if the line is malformed.
    pub fn from_objects_txt(line: &str) -> Result<Self, WriteError> {
        let bad = WriteError::Template;
        let fields: Vec<&str> = line.split_whitespace().collect();
        let [
            def,
            block,
            visit,
            land,
            menu,
            class,
            subclass,
            group,
            priority,
        ] = fields[..]
        else {
            return Err(bad("an Objects.txt template needs 9 fields"));
        };
        let number = |s: &str| s.parse::<u32>().map_err(|_| bad("bad number in template"));
        let small = |s: &str| s.parse::<u8>().map_err(|_| bad("bad number in template"));
        Ok(Self {
            def: def.to_owned(),
            passable: mask_from_text(block).ok_or(bad("bad passability mask"))?,
            visitable: mask_from_text(visit).ok_or(bad("bad visitability mask"))?,
            terrains: terrain_mask_from_text(land).ok_or(bad("bad terrain mask"))?,
            menu: terrain_mask_from_text(menu).ok_or(bad("bad terrain mask"))?,
            class: number(class)?,
            subclass: number(subclass)?,
            group: small(group)?,
            priority: small(priority)?,
        })
    }

    /// The map editor's template for a hero of `class` (as in the game's own SoD maps): picture
    /// `ahNN_e.def`, one blocked and visitable tile one column left of the object's position.
    pub fn hero(class: u8) -> Self {
        Self {
            def: format!("ah{class:02}_e.def"),
            passable: [0xff, 0xff, 0xff, 0xff, 0xff, 0xbf],
            visitable: [0, 0, 0, 0, 0, 0x40],
            terrains: 0xff,
            menu: 0xff,
            class: OBJECT_HERO,
            subclass: u32::from(class),
            group: 3,
            priority: 0,
        }
    }

    fn write(&self, out: &mut Out) -> Result<(), WriteError> {
        out.string(&self.def)?;
        out.bytes(&self.passable);
        out.bytes(&self.visitable);
        out.u16(self.terrains);
        out.u16(self.menu);
        out.u32(self.class);
        out.u32(self.subclass);
        out.u8(self.group);
        out.u8(self.priority);
        out.bytes(&[0; 16]);
        Ok(())
    }
}

/// `Objects.txt` masks are 48 characters, the anchor first: character `8r + c` is the tile `c`
/// columns left of and `r` rows above the bottom right corner (`docs/formats/h3m.md`, "Template
/// masks"); map files store them as 6 bytes whose bit `j` of byte `i` is character `47 − (8i + j)`
/// (VCMI `ObjectTemplate::readTxt` and `readMap` agree on this mapping).
fn mask_from_text(text: &str) -> Option<[u8; 6]> {
    let chars = text.as_bytes();
    if chars.len() != 48 || !chars.iter().all(|c| matches!(c, b'0' | b'1')) {
        return None;
    }
    let mut mask = [0u8; 6];
    for (k, byte) in mask.iter_mut().enumerate() {
        for bit in 0..8 {
            if chars[47 - (8 * k + bit)] == b'1' {
                *byte |= 1 << bit;
            }
        }
    }
    Some(mask)
}

/// A 9-character terrain mask, terrain 0 last (binary, most significant terrain first).
fn terrain_mask_from_text(text: &str) -> Option<u16> {
    if text.len() != 9 {
        return None;
    }
    u16::from_str_radix(text, 2).ok()
}

/// A player's slot in the map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    /// A human may play it.
    pub human: bool,
    /// The computer may play it.
    pub computer: bool,
    /// Allowed towns (bit per town, Castle = bit 0, Conflux = bit 8).
    pub towns: u16,
}

/// A hero's artifacts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroArtifacts {
    /// Worn artifacts by slot (`ArtSlots.txt` order: 13–16 are the war machines Ballista, Ammo
    /// Cart, First Aid Tent, Catapult; 17 is the spell book; 18 is misc 5).
    pub worn: [Option<u16>; ARTIFACT_SLOTS],
    /// Backpack.
    pub backpack: Vec<u16>,
}

/// A hero placed on the map. Every `Option` is "use the hero type's default".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hero {
    /// Tile the hero stands on.
    pub tile: (u8, u8),
    /// Owning player (0 = red … 7 = pink).
    pub owner: u8,
    /// Hero type (0–155, `HOTRAITS.TXT` order).
    pub hero_type: u8,
    /// The hero type's class (0–17); decides the map picture.
    pub class: u8,
    /// Custom name.
    pub name: Option<String>,
    /// Experience.
    pub experience: Option<u32>,
    /// Secondary skills (skill id, level 1–3), at most 8.
    pub skills: Option<Vec<(u8, u8)>>,
    /// Army (creature id, count) per slot.
    pub army: Option<[Option<(u16, u16)>; ARMY_SLOTS]>,
    /// Tight formation.
    pub tight_formation: bool,
    /// Artifacts.
    pub artifacts: Option<HeroArtifacts>,
    /// Spells in the spell book (bit per spell id, 70 spells).
    pub spells: Option<[u8; 9]>,
    /// Attack, defense, power, knowledge.
    pub primary: Option<[u8; 4]>,
}

/// A wandering monster placed on the map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Monster {
    /// Tile the monster stands on.
    pub tile: (u8, u8),
    /// Template (class 54 and the creature id as subclass for a fixed creature).
    pub template: ObjectTemplate,
    /// Number of creatures (0 = random).
    pub count: u16,
    /// Disposition: 0 compliant, 1 friendly, 2 aggressive, 3 hostile, 4 savage.
    pub disposition: u8,
    /// Never flees.
    pub never_flees: bool,
    /// Does not grow.
    pub fixed_count: bool,
}

/// A resource pile placed on the map (E08 T16b), with or without guards (E08 T22a1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pile {
    /// Tile the pile stands on.
    pub tile: (u8, u8),
    /// Template (class 79 and the resource as subclass, from `Objects.txt`).
    pub template: ObjectTemplate,
    /// Amount as the map stores it (0 = drawn when the game starts; `docs/formats/h3m.md`).
    pub amount: u32,
    /// The guards; `None`: no message and no guards.
    pub guards: Option<Guards>,
}

/// A pile's guards (E08 T22a1; ADV-PIK-004): the format keeps them in the pile's message
/// (`docs/formats/h3m.md`, "Shared parts of the bodies").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Guards {
    /// The stacks (creature id, count) by slot; at least one.
    pub army: [Option<(u16, u16)>; ARMY_SLOTS],
    /// The message's text: the game shows it and asks before the battle (the engine's offer to
    /// fight); empty, the guards fight at once.
    pub question: String,
}

/// A map's underground level (E08 T22a1): a base terrain, rectangles of other terrains and the
/// objects on it, as the surface's; no roads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Underground {
    /// Terrain of every tile no patch covers.
    pub terrain: Terrain,
    /// Rectangles of other terrains; a later patch covers an earlier one.
    pub patches: Vec<Patch>,
    /// Objects on the level, written after the surface's.
    pub objects: Vec<MapObject>,
}

/// A special ground placed on the map (E04c T3): an object without a body, which heroes and
/// monsters stand on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ground {
    /// The object's position: the bottom-right tile of its picture (the picture covers the tiles
    /// to the left and above, as its `.msk` says).
    pub tile: (u8, u8),
    /// Template (a class of [`GROUND_OBJECTS`], from `Objects.txt`).
    pub template: ObjectTemplate,
}

/// An object without a body placed on the map (E08 T32a, T36a): its template alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plain {
    /// The object's position: the bottom-right tile of its picture (its masks say which tiles it
    /// blocks and where it is visited).
    pub tile: (u8, u8),
    /// Template (a class of [`PLAIN_OBJECTS`], from `Objects.txt`).
    pub template: ObjectTemplate,
}

/// A town of a fixed faction placed on the map (E04c T5b: siege experiments), without a name,
/// events or required and banned spells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Town {
    /// The object's position: the bottom-right tile of its picture. Its entrance, the one visitable
    /// tile, is two columns left of it on its row (the towns' templates in `Objects.txt`); a hero
    /// on the entrance is the town's visiting hero.
    pub tile: (u8, u8),
    /// Template (class [`TOWN_OBJECT`] and the faction as subclass, from `Objects.txt`).
    pub template: ObjectTemplate,
    /// Owning player (0 = red … 7 = pink), `None` for a neutral town.
    pub owner: Option<u8>,
    /// Garrison (creature id, count) per slot; `None` = the editor's default.
    pub garrison: Option<[Option<(u16, u16)>; ARMY_SLOTS]>,
    /// The garrison's tight formation.
    pub tight_formation: bool,
    /// Buildings.
    pub buildings: TownBuildings,
}

/// A town's buildings as a map stores them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TownBuildings {
    /// The editor's standard set, with or without a Fort.
    Standard {
        /// A Fort is built.
        fort: bool,
    },
    /// Chosen buildings: a bit per building in the map's order (bit `j` of byte `i` is building
    /// `8i + j`, below [`MAP_BUILDINGS`]; `docs/formats/h3m.md`, "Town buildings").
    Custom {
        /// Built.
        built: [u8; 6],
        /// Forbidden.
        forbidden: [u8; 6],
    },
}

/// An object placed on the map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapObject {
    /// A hero.
    Hero(Hero),
    /// A wandering monster.
    Monster(Monster),
    /// A resource pile.
    Pile(Pile),
    /// A special ground.
    Ground(Ground),
    /// An object without a body (E08 T32a): a Cartographer, a tree, a Border Gate.
    Plain(Plain),
    /// A town.
    Town(Town),
}

/// A rectangle of tiles with a terrain of its own (E08 T10a: movement experiments).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Patch {
    /// Terrain of the rectangle's tiles.
    pub terrain: Terrain,
    /// Top left tile (column, row).
    pub from: (u8, u8),
    /// Bottom right tile (column, row), inclusive.
    pub to: (u8, u8),
}

impl Patch {
    fn covers(&self, x: u8, y: u8) -> bool {
        (self.from.0..=self.to.0).contains(&x) && (self.from.1..=self.to.1).contains(&y)
    }
}

/// Road kinds in map files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoadKind {
    /// Dirt road.
    Dirt = 1,
    /// Gravel road.
    Gravel = 2,
    /// Cobblestone road.
    Cobblestone = 3,
}

/// A straight road from west to east on one row, at least two tiles long (E08 T10a: movement
/// experiments). Its tile pictures are the ones the game's own maps give such a road (counted
/// over 164 maps, four of them the user's own): 15 at the west end, 12 and 13 between, 15
/// mirrored at the east end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Road {
    /// Kind of road.
    pub kind: RoadKind,
    /// Row.
    pub row: u8,
    /// West end's column.
    pub from: u8,
    /// East end's column (inclusive), east of `from`.
    pub to: u8,
}

impl Road {
    /// The road's picture and mirror flags on the column `x` of its row, if it covers it.
    fn picture(&self, x: u8) -> Option<(u8, u8)> {
        match x {
            _ if x == self.from => Some((15, 0)),
            _ if x == self.to => Some((15, ROAD_MIRRORED)),
            _ if (self.from..self.to).contains(&x) => Some((12 + (x - self.from + 1) % 2, 0)),
            _ => None,
        }
    }
}

/// A tile's flag that mirrors its road picture left to right.
const ROAD_MIRRORED: u8 = 0x10;
/// A land tile's flag: water lies on one of its eight neighbours (E04c T3: every land tile next
/// to water in the game's own maps has it and no other tile does; the game reads the coast, the
/// shore battlefield's tiles, from it, batch 3b).
const COASTAL: u8 = 0x40;

/// A map: a base terrain, rectangles of other terrains, roads and objects on the surface, and
/// an underground level if it has one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map {
    /// Name shown in the scenario list.
    pub name: String,
    /// Description shown in the scenario list.
    pub description: String,
    /// Width and height (36, 72, 108 or 144).
    pub size: u8,
    /// Terrain of every tile no patch covers.
    pub terrain: Terrain,
    /// Rectangles of other terrains; a later patch covers an earlier one.
    pub patches: Vec<Patch>,
    /// Roads; no two cover the same tile.
    pub roads: Vec<Road>,
    /// Players (index = colour); `None` = nobody plays this colour.
    pub players: [Option<Player>; PLAYERS],
    /// Objects on the surface.
    pub objects: Vec<MapObject>,
    /// The underground level (E08 T22a1); `None`: the map has one level.
    pub underground: Option<Underground>,
    /// The scenario's level limit (E08 T32a; 0: none): no hero gains a level past it (HER-EXP-003;
    /// the engine counts a limit above 74 as none, A-8).
    pub level_limit: u8,
}

impl Map {
    /// Every object with its level (0 the surface, 1 the underground), the surface's first: the
    /// order they are written in.
    fn placed(&self) -> impl Iterator<Item = (u8, &MapObject)> {
        let below = self.underground.iter().flat_map(|u| &u.objects);
        self.objects
            .iter()
            .map(|o| (0, o))
            .chain(below.map(|o| (1, o)))
    }
}

/// Writes `map` as an uncompressed SoD map.
///
/// # Errors
/// [`WriteError::OutOfRange`] if a value does not fit the format or the map (tile outside the
/// map, unknown hero, too many skills, an owner without a playable player, …).
pub fn write(map: &Map) -> Result<Vec<u8>, WriteError> {
    validate(map)?;
    let mut out = Out::default();
    out.u32(SOD_FORMAT);
    out.bool(map.players.iter().any(Option::is_some));
    out.u32(u32::from(map.size));
    out.bool(map.underground.is_some());
    out.string(&map.name)?;
    out.string(&map.description)?;
    out.u8(1); // difficulty: normal
    out.u8(map.level_limit);
    for (colour, player) in map.players.iter().enumerate() {
        write_player(&mut out, map, colour, player.as_ref())?;
    }
    out.u8(0xff); // victory: standard
    out.u8(0xff); // loss: standard
    out.u8(0); // no teams
    out.bytes(&DEFAULT_ALLOWED_HEROES);
    out.u32(0); // no campaign hero placeholders
    out.u8(0); // no disposed heroes
    out.bytes(&[0; 31]); // map options (unused in SoD)
    out.bytes(&DEFAULT_BANNED_ARTIFACTS);
    out.bytes(&[0; 9]); // no banned spells
    out.bytes(&[0; 4]); // no banned skills
    out.u32(0); // no rumors
    out.bytes(&[0; HERO_COUNT as usize]); // no customised hero types
    write_terrain(&mut out, map.size, map.terrain, &map.patches, &map.roads);
    if let Some(underground) = &map.underground {
        write_terrain(
            &mut out,
            map.size,
            underground.terrain,
            &underground.patches,
            &[],
        );
    }
    let templates = templates(map);
    out.u32(len32(templates.len()));
    for template in &templates {
        template.write(&mut out)?;
    }
    out.u32(len32(map.placed().count()));
    for (index, (level, object)) in map.placed().enumerate() {
        let template = object_template(object);
        let template_index = templates.iter().position(|t| *t == template).unwrap_or(0);
        let (x, y) = position(object);
        out.bytes(&[x, y, level]);
        out.u32(len32(template_index));
        out.bytes(&[0; 5]);
        let identifier = len32(index).saturating_add(1);
        match object {
            MapObject::Hero(hero) => write_hero(&mut out, hero, identifier)?,
            MapObject::Monster(monster) => write_monster(&mut out, monster, identifier),
            MapObject::Pile(pile) => write_pile(&mut out, pile)?,
            MapObject::Ground(_) | MapObject::Plain(_) => {} // no body
            MapObject::Town(town) => write_town(&mut out, town, identifier),
        }
    }
    out.u32(0); // no timed events
    Ok(out.0)
}

/// Wraps `data` in a gzip stream (RFC 1952), the way the game stores maps.
pub fn gzip(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 0xff];
    out.extend_from_slice(&miniz_oxide::deflate::compress_to_vec(data, 9));
    out.extend_from_slice(&crc32(data).to_le_bytes());
    // ISIZE is the length modulo 2^32 (RFC 1952 §2.3.1).
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out
}

fn out_of_range(what: &'static str, value: impl TryInto<u64>) -> WriteError {
    WriteError::OutOfRange {
        what,
        value: value.try_into().unwrap_or(u64::MAX),
    }
}

fn validate(map: &Map) -> Result<(), WriteError> {
    if ![36, 72, 108, 144].contains(&map.size) {
        return Err(out_of_range("map size", map.size));
    }
    let below = map.underground.iter().flat_map(|u| &u.patches);
    for patch in map.patches.iter().chain(below) {
        if patch.from.0 > patch.to.0 || patch.from.1 > patch.to.1 {
            return Err(out_of_range("patch corners (from after to)", patch.from.0));
        }
        if patch.to.0 >= map.size || patch.to.1 >= map.size {
            return Err(out_of_range(
                "patch tile",
                u64::from(patch.to.0) << 8 | u64::from(patch.to.1),
            ));
        }
    }
    for (index, road) in map.roads.iter().enumerate() {
        if road.from >= road.to || road.to >= map.size || road.row >= map.size {
            return Err(out_of_range("road (west end, east end, row)", road.to));
        }
        let overlaps =
            |other: &Road| other.row == road.row && other.from <= road.to && road.from <= other.to;
        if map.roads[..index].iter().any(overlaps) {
            return Err(out_of_range("road over another road (row)", road.row));
        }
    }
    for (_, object) in map.placed() {
        let (x, y) = match object {
            MapObject::Hero(hero) => hero.tile,
            MapObject::Monster(monster) => monster.tile,
            MapObject::Pile(pile) => pile.tile,
            MapObject::Ground(ground) => ground.tile,
            MapObject::Plain(plain) => plain.tile,
            MapObject::Town(town) => town.tile,
        };
        if x >= map.size || y >= map.size {
            return Err(out_of_range(
                "object tile",
                u64::from(x) << 8 | u64::from(y),
            ));
        }
        match object {
            MapObject::Hero(hero) => validate_hero(map, hero)?,
            MapObject::Monster(monster) => validate_monster(monster)?,
            MapObject::Pile(pile) => validate_pile(pile)?,
            MapObject::Ground(ground) => validate_ground(ground)?,
            MapObject::Plain(plain) => validate_plain(plain)?,
            MapObject::Town(town) => validate_town(map, town)?,
        }
    }
    Ok(())
}

fn validate_hero(map: &Map, hero: &Hero) -> Result<(), WriteError> {
    if map
        .players
        .get(usize::from(hero.owner))
        .and_then(Option::as_ref)
        .is_none()
    {
        return Err(out_of_range("hero owner (no playable player)", hero.owner));
    }
    if hero.hero_type >= HERO_COUNT {
        return Err(out_of_range("hero type", hero.hero_type));
    }
    if hero.class >= HERO_CLASS_COUNT {
        return Err(out_of_range("hero class", hero.class));
    }
    // The hero's object position is one column right of its tile.
    if hero.tile.0.checked_add(1).is_none_or(|x| x >= map.size) {
        return Err(out_of_range("hero tile (last column)", hero.tile.0));
    }
    if let Some(skills) = &hero.skills {
        if skills.len() > 8 {
            return Err(out_of_range("secondary skill count", skills.len()));
        }
        for &(skill, level) in skills {
            if skill >= SKILL_COUNT {
                return Err(out_of_range("secondary skill", skill));
            }
            if !(1..=3).contains(&level) {
                return Err(out_of_range("secondary skill level", level));
            }
        }
    }
    if let Some(army) = &hero.army {
        validate_army(army)?;
    }
    if let Some(artifacts) = &hero.artifacts {
        let worn = artifacts.worn.iter().flatten();
        for &artifact in worn.chain(&artifacts.backpack) {
            if artifact >= ARTIFACT_COUNT {
                return Err(out_of_range("artifact", artifact));
            }
        }
        if artifacts.backpack.len() > 64 {
            return Err(out_of_range("backpack size", artifacts.backpack.len()));
        }
    }
    Ok(())
}

fn validate_monster(monster: &Monster) -> Result<(), WriteError> {
    if !MONSTER_OBJECTS.contains(&monster.template.class) {
        return Err(out_of_range("monster object class", monster.template.class));
    }
    if monster.disposition > 4 {
        return Err(out_of_range("monster disposition", monster.disposition));
    }
    Ok(())
}

fn validate_pile(pile: &Pile) -> Result<(), WriteError> {
    if pile.template.class != PILE_OBJECT {
        return Err(out_of_range("pile object class", pile.template.class));
    }
    if pile.template.subclass >= RESOURCE_COUNT {
        return Err(out_of_range("pile resource", pile.template.subclass));
    }
    if pile.amount > MAX_PILE_AMOUNT {
        return Err(out_of_range("pile amount", pile.amount));
    }
    if let Some(guards) = &pile.guards {
        if guards.army.iter().all(Option::is_none) {
            return Err(out_of_range("pile guards (no stack)", 0u8));
        }
        validate_army(&guards.army)?;
    }
    Ok(())
}

/// Every stack of an army names a creature the map may name and holds one or more.
fn validate_army(army: &[Option<(u16, u16)>; ARMY_SLOTS]) -> Result<(), WriteError> {
    for &(creature, count) in army.iter().flatten() {
        if creature >= CREATURE_COUNT {
            return Err(out_of_range("creature", creature));
        }
        if count == 0 {
            return Err(out_of_range("stack size", count));
        }
    }
    Ok(())
}

fn validate_ground(ground: &Ground) -> Result<(), WriteError> {
    if GROUND_OBJECTS.contains(&ground.template.class) {
        Ok(())
    } else {
        Err(out_of_range("ground object class", ground.template.class))
    }
}

fn validate_plain(plain: &Plain) -> Result<(), WriteError> {
    if PLAIN_OBJECTS.contains(&plain.template.class) {
        Ok(())
    } else {
        Err(out_of_range("plain object class", plain.template.class))
    }
}

fn validate_town(map: &Map, town: &Town) -> Result<(), WriteError> {
    if town.template.class != TOWN_OBJECT {
        return Err(out_of_range("town object class", town.template.class));
    }
    if town.template.subclass >= FACTION_COUNT {
        return Err(out_of_range("town faction", town.template.subclass));
    }
    if let Some(owner) = town.owner
        && map
            .players
            .get(usize::from(owner))
            .and_then(Option::as_ref)
            .is_none()
    {
        return Err(out_of_range("town owner (no playable player)", owner));
    }
    if let Some(garrison) = &town.garrison {
        validate_army(garrison)?;
    }
    if let TownBuildings::Custom { built, forbidden } = &town.buildings {
        let last = MAP_BUILDINGS % 8;
        if built[5] >> last != 0 || forbidden[5] >> last != 0 {
            return Err(out_of_range("town building (bit)", MAP_BUILDINGS));
        }
    }
    Ok(())
}

fn write_player(
    out: &mut Out,
    map: &Map,
    colour: usize,
    player: Option<&Player>,
) -> Result<(), WriteError> {
    let Some(player) = player else {
        // Nobody plays this colour: the game skips 13 bytes; the editor writes the fields of an
        // empty player there (no town, no main hero).
        out.bytes(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0, 0, 0, 0, 0]);
        return Ok(());
    };
    out.bool(player.human);
    out.bool(player.computer);
    out.u8(0); // AI tactic: random
    out.u8(0); // town choice not offered
    out.u16(player.towns);
    out.bool(false); // town not random
    out.bool(false); // no main town
    let heroes: Vec<u8> = map
        .placed()
        .filter_map(|(_, object)| match object {
            MapObject::Hero(hero) if usize::from(hero.owner) == colour => Some(hero.hero_type),
            _ => None,
        })
        .collect();
    out.bool(false); // no random hero
    match heroes.first() {
        Some(&main) => {
            out.u8(main);
            out.u8(0xff); // portrait of the hero type
            out.string("")?; // name of the hero type
        }
        None => out.u8(0xff),
    }
    out.u8(0); // unknown, always 0 in the game's maps
    out.u32(len32(heroes.len()));
    for hero in heroes {
        out.u8(hero);
        out.string("")?;
    }
    Ok(())
}

/// One level's tiles, row by row: its base terrain, the patches over it, the roads on it.
fn write_terrain(out: &mut Out, side: u8, base: Terrain, patches: &[Patch], roads: &[Road]) {
    let size = usize::from(side);
    let terrain_at = |x: u8, y: u8| {
        patches
            .iter()
            .rev()
            .find(|patch| patch.covers(x, y))
            .map_or(base, |patch| patch.terrain)
    };
    let coastal = |x: u8, y: u8| {
        let near = |c: u8| c.saturating_sub(1)..=c.saturating_add(1).min(side - 1);
        near(y).any(|ny| near(x).any(|nx| terrain_at(nx, ny) == Terrain::Water))
    };
    for (index, (y, x)) in (0..side)
        .flat_map(|y| (0..side).map(move |x| (y, x)))
        .enumerate()
    {
        let terrain = terrain_at(x, y);
        // A deterministic spread over the eight interior pictures (the editor picks randomly).
        let variant = ((index * 7 + index / size * 3) % 8) as u8;
        let road = roads
            .iter()
            .filter(|road| road.row == y)
            .find_map(|road| Some((road.kind as u8, road.picture(x)?)));
        let (kind, (picture, mut flags)) = road.unwrap_or((0, (0, 0)));
        if terrain != Terrain::Water && coastal(x, y) {
            flags |= COASTAL;
        }
        out.bytes(&[
            terrain as u8,
            terrain.interior_view() + variant,
            0,
            0,
            kind,
            picture,
            flags,
        ]);
    }
}

fn object_template(object: &MapObject) -> ObjectTemplate {
    match object {
        MapObject::Hero(hero) => ObjectTemplate::hero(hero.class),
        MapObject::Monster(monster) => monster.template.clone(),
        MapObject::Pile(pile) => pile.template.clone(),
        MapObject::Ground(ground) => ground.template.clone(),
        MapObject::Plain(plain) => plain.template.clone(),
        MapObject::Town(town) => town.template.clone(),
    }
}

/// Distinct templates in order of first use.
fn templates(map: &Map) -> Vec<ObjectTemplate> {
    let mut templates: Vec<ObjectTemplate> = Vec::new();
    for (_, object) in map.placed() {
        let template = object_template(object);
        if !templates.contains(&template) {
            templates.push(template);
        }
    }
    templates
}

/// The object's position in the file: the bottom-right corner of its picture. A hero's picture
/// has the hero one column left of that corner (template `ahNN_e.def`).
fn position(object: &MapObject) -> (u8, u8) {
    match object {
        MapObject::Hero(hero) => (hero.tile.0.saturating_add(1), hero.tile.1),
        MapObject::Monster(monster) => monster.tile,
        MapObject::Pile(pile) => pile.tile,
        MapObject::Ground(ground) => ground.tile,
        MapObject::Plain(plain) => plain.tile,
        MapObject::Town(town) => town.tile,
    }
}

fn write_hero(out: &mut Out, hero: &Hero, identifier: u32) -> Result<(), WriteError> {
    out.u32(identifier);
    out.u8(hero.owner);
    out.u8(hero.hero_type);
    out.bool(hero.name.is_some());
    if let Some(name) = &hero.name {
        out.string(name)?;
    }
    out.bool(hero.experience.is_some());
    if let Some(experience) = hero.experience {
        out.u32(experience);
    }
    out.bool(false); // no custom portrait
    out.bool(hero.skills.is_some());
    if let Some(skills) = &hero.skills {
        out.u32(len32(skills.len()));
        for &(skill, level) in skills {
            out.bytes(&[skill, level]);
        }
    }
    out.bool(hero.army.is_some());
    if let Some(army) = &hero.army {
        write_army(out, army);
    }
    out.bool(hero.tight_formation);
    out.bool(hero.artifacts.is_some());
    if let Some(artifacts) = &hero.artifacts {
        for slot in &artifacts.worn {
            out.u16(slot.unwrap_or(0xffff));
        }
        out.u16(u16::try_from(artifacts.backpack.len()).unwrap_or(u16::MAX));
        for &artifact in &artifacts.backpack {
            out.u16(artifact);
        }
    }
    out.u8(0xff); // no patrol
    out.bool(false); // no custom biography
    out.u8(0xff); // sex of the hero type
    out.bool(hero.spells.is_some());
    if let Some(spells) = &hero.spells {
        out.bytes(spells);
    }
    out.bool(hero.primary.is_some());
    if let Some(primary) = &hero.primary {
        out.bytes(primary);
    }
    out.bytes(&[0; 16]);
    Ok(())
}

fn write_monster(out: &mut Out, monster: &Monster, identifier: u32) {
    out.u32(identifier);
    out.u16(monster.count);
    out.u8(monster.disposition);
    out.bool(false); // no message or treasure
    out.bool(monster.never_flees);
    out.bool(monster.fixed_count);
    out.bytes(&[0, 0]);
}

/// A pile's body (`docs/formats/h3m.md`, classes 76 and 79): the message (unguarded: none;
/// guarded: its text, the guards' flag and army, 4 unused bytes), the amount and 4 unused bytes.
fn write_pile(out: &mut Out, pile: &Pile) -> Result<(), WriteError> {
    out.bool(pile.guards.is_some());
    if let Some(guards) = &pile.guards {
        out.string(&guards.question)?;
        out.bool(true);
        write_army(out, &guards.army);
        out.bytes(&[0; 4]);
    }
    out.u32(pile.amount);
    out.bytes(&[0; 4]);
    Ok(())
}

/// An army's seven slots, `0xFFFF` for an empty one.
fn write_army(out: &mut Out, army: &[Option<(u16, u16)>; ARMY_SLOTS]) {
    for slot in army {
        let (creature, count) = slot.unwrap_or((0xffff, 0));
        out.u16(creature);
        out.u16(count);
    }
}

/// A town's body (`docs/formats/h3m.md`, classes 77 and 98): no name, no required or banned
/// spells, no events, the alignment of its owner.
fn write_town(out: &mut Out, town: &Town, identifier: u32) {
    out.u32(identifier);
    out.u8(town.owner.unwrap_or(0xff));
    out.bool(false); // no custom name
    out.bool(town.garrison.is_some());
    if let Some(garrison) = &town.garrison {
        write_army(out, garrison);
    }
    out.bool(town.tight_formation);
    match &town.buildings {
        TownBuildings::Standard { fort } => {
            out.bool(false);
            out.bool(*fort);
        }
        TownBuildings::Custom { built, forbidden } => {
            out.bool(true);
            out.bytes(built);
            out.bytes(forbidden);
        }
    }
    out.bytes(&[0; 9]); // no required spells
    out.bytes(&[0; 9]); // no banned spells
    out.u32(0); // no events
    out.u8(0xff); // alignment: the owner's
    out.bytes(&[0; 3]);
}

fn len32(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

/// Little-endian output buffer.
#[derive(Default)]
struct Out(Vec<u8>);

impl Out {
    fn u8(&mut self, value: u8) {
        self.0.push(value);
    }
    fn bool(&mut self, value: bool) {
        self.0.push(u8::from(value));
    }
    fn u16(&mut self, value: u16) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }
    fn u32(&mut self, value: u32) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }
    fn bytes(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }
    /// A length-prefixed Windows-1252 string; characters outside Latin-1 are rejected.
    fn string(&mut self, text: &str) -> Result<(), WriteError> {
        let mut encoded = Vec::with_capacity(text.len());
        for c in text.chars() {
            let code = u32::from(c);
            // Latin-1 code points map to the same bytes in Windows-1252 except 0x80–0x9F,
            // which we do not need.
            if code > 0xff || (0x80..0xa0).contains(&code) {
                return Err(out_of_range("character in a map string", code));
            }
            encoded.push(code as u8);
        }
        if encoded.len() > MAX_STRING {
            return Err(out_of_range("map string length", encoded.len()));
        }
        self.u32(len32(encoded.len()));
        self.bytes(&encoded);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PIKEMAN: &str = "AvWPike.def 011111111111111111111111111111111111111111111111 \
                           100000000000000000000000000000000000000000000000 111111111 000000100 \
                           54 0 2 0";

    fn smoke_like_map() -> Map {
        let mut players: [Option<Player>; PLAYERS] = Default::default();
        players[0] = Some(Player {
            human: true,
            computer: true,
            towns: 1,
        });
        players[1] = Some(Player {
            human: false,
            computer: true,
            towns: 1 << 6,
        });
        let mut worn = [None; ARTIFACT_SLOTS];
        worn[3] = Some(7);
        worn[13] = Some(4);
        Map {
            name: "Test".into(),
            description: "A test map".into(),
            size: 36,
            terrain: Terrain::Grass,
            patches: vec![Patch {
                terrain: Terrain::Swamp,
                from: (2, 3),
                to: (4, 3),
            }],
            roads: vec![Road {
                kind: RoadKind::Gravel,
                row: 5,
                from: 1,
                to: 4,
            }],
            players,
            objects: vec![
                MapObject::Hero(Hero {
                    tile: (10, 10),
                    owner: 0,
                    hero_type: 0,
                    class: 0,
                    name: Some("Red".into()),
                    experience: Some(0),
                    skills: Some(vec![(1, 2), (23, 1)]),
                    army: Some([Some((2, 20)), Some((0, 10)), None, None, None, None, None]),
                    tight_formation: false,
                    artifacts: Some(HeroArtifacts {
                        worn,
                        backpack: vec![],
                    }),
                    spells: None,
                    primary: Some([5, 4, 1, 1]),
                }),
                MapObject::Monster(Monster {
                    tile: (20, 20),
                    template: ObjectTemplate::from_objects_txt(PIKEMAN).unwrap(),
                    count: 10,
                    disposition: 4,
                    never_flees: true,
                    fixed_count: true,
                }),
            ],
            underground: None,
            level_limit: 0,
        }
    }

    #[test]
    fn a_ground_needs_a_ground_class_and_writes_no_body() {
        let template = ObjectTemplate::from_objects_txt(
            "AVXcg5.def 111111111111111111111111111111111111111111111111 \
             000000000000000000000000000000000000000000000000 111111111 011111111 223 0 0 1",
        )
        .unwrap();
        let mut map = smoke_like_map();
        map.objects = vec![MapObject::Ground(Ground {
            tile: (5, 5),
            template: template.clone(),
        })];
        let bytes = write(&map).unwrap();
        // The objects end the file but for the events' count: one object at its position, template
        // 0, five unused bytes and no body.
        let tail = [1, 0, 0, 0, 5, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(&bytes[bytes.len() - tail.len()..], &tail[..]);
        map.objects = vec![MapObject::Ground(Ground {
            tile: (5, 5),
            template: ObjectTemplate {
                class: 79,
                ..template
            },
        })];
        assert_eq!(
            write(&map),
            Err(WriteError::OutOfRange {
                what: "ground object class",
                value: 79
            })
        );
    }

    #[test]
    fn a_plain_object_needs_a_plain_class_and_writes_no_body() {
        let template = ObjectTemplate::from_objects_txt(
            "AVXmaps0.def 001111111111111111111111111111111111111111111111 \
             010000000000000000000000000000000000000000000000 111111111 010111111 13 1 0 0",
        )
        .unwrap();
        let mut map = smoke_like_map();
        map.level_limit = 7;
        map.objects = vec![MapObject::Plain(Plain {
            tile: (5, 5),
            template: template.clone(),
        })];
        let bytes = write(&map).unwrap();
        let tail = [1, 0, 0, 0, 5, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(&bytes[bytes.len() - tail.len()..], &tail[..]);
        // The header: the format, whether anyone plays, the size, the levels, two strings, then
        // the difficulty (normal) and the level limit.
        let at = 10 + 4 + map.name.len() + 4 + map.description.len();
        assert_eq!(bytes[at..at + 2], [1, 7]);
        // E08 T36a: a tree and a Border Gate, the game's lines, without a body as well.
        for line in [
            "AVLtr3d0.def 011111111111111111111111111111111111111111111111 \
             000000000000000000000000000000000000000000000000 111111111 000010101 155 0 0 0",
            "avxbgt00.def 000111111111111111111111111111111111111111111111 \
             010000000000000000000000000000000000000000000000 111111111 011111111 212 0 0 0",
        ] {
            map.objects = vec![MapObject::Plain(Plain {
                tile: (5, 5),
                template: ObjectTemplate::from_objects_txt(line).unwrap(),
            })];
            let bytes = write(&map).unwrap();
            assert_eq!(&bytes[bytes.len() - tail.len()..], &tail[..]);
        }
        map.objects = vec![MapObject::Plain(Plain {
            tile: (5, 5),
            template: ObjectTemplate {
                class: 222,
                ..template
            },
        })];
        assert_eq!(
            write(&map),
            Err(WriteError::OutOfRange {
                what: "plain object class",
                value: 222
            })
        );
    }

    #[test]
    fn templates_from_objects_txt_match_the_game_s_maps() {
        let template = ObjectTemplate::from_objects_txt(PIKEMAN).unwrap();
        // The same masks as the random-monster template in the game's own SoD maps.
        assert_eq!(template.passable, [0xff, 0xff, 0xff, 0xff, 0xff, 0x7f]);
        assert_eq!(template.visitable, [0, 0, 0, 0, 0, 0x80]);
        assert_eq!((template.terrains, template.menu), (0x1ff, 4));
        assert_eq!(
            (template.class, template.subclass, template.group),
            (54, 0, 2)
        );
        assert!(ObjectTemplate::from_objects_txt("x.def 0 1").is_err());
        let bad_mask = PIKEMAN.replace("011111111111", "211111111111");
        assert!(ObjectTemplate::from_objects_txt(&bad_mask).is_err());
        assert!(ObjectTemplate::from_objects_txt(&PIKEMAN.replace(" 54 ", " x ")).is_err());
        assert!(ObjectTemplate::from_objects_txt(&PIKEMAN.replace("111111111", "11")).is_err());
    }

    #[test]
    fn header_terrain_and_objects_are_laid_out_in_order() {
        let map = smoke_like_map();
        let bytes = write(&map).unwrap();
        assert_eq!(&bytes[..4], &SOD_FORMAT.to_le_bytes());
        assert_eq!(&bytes[4..10], &[1, 36, 0, 0, 0, 0]);
        // The whole file: header strings, 8 players, 156 hero flags, 36 × 36 tiles, templates…
        let tiles = 7 * 36 * 36;
        assert!(bytes.len() > tiles + 156);
        // The file ends with the event count.
        assert_eq!(&bytes[bytes.len() - 4..], &[0, 0, 0, 0]);
        // Hero object: position one column right of its tile.
        let hero_template = ObjectTemplate::hero(0);
        assert_eq!(hero_template.def, "ah00_e.def");
        assert_eq!(position(&map.objects[0]), (11, 10));
        assert_eq!(templates(&map).len(), 2);
    }

    #[test]
    fn patches_cover_the_base_terrain_with_interior_pictures() {
        let mut map = smoke_like_map();
        map.patches.push(Patch {
            terrain: Terrain::Sand,
            from: (4, 3),
            to: (35, 35),
        });
        assert!(write(&map).is_ok());
        let mut out = Out::default();
        write_terrain(&mut out, map.size, map.terrain, &map.patches, &map.roads);
        let bytes = out.0;
        assert_eq!(bytes.len(), 7 * 36 * 36);
        let tile = |x: usize, y: usize| &bytes[7 * (y * 36 + x)..][..7];
        // Grass, then the swamp patch, then the later sand patch over the swamp's last tile.
        for (x, y, terrain, first) in [
            (0, 0, Terrain::Grass, 49),
            (1, 3, Terrain::Grass, 49),
            (2, 3, Terrain::Swamp, 49),
            (3, 3, Terrain::Swamp, 49),
            (4, 3, Terrain::Sand, 0),
            (4, 2, Terrain::Grass, 49),
            (35, 35, Terrain::Sand, 0),
        ] {
            let t = tile(x, y);
            assert_eq!(t[0], terrain as u8, "terrain of ({x}, {y})");
            assert!((first..first + 8).contains(&t[1]), "picture of ({x}, {y})");
            assert_eq!(&t[2..], &[0; 5]);
        }
        // The gravel road on row 5: 15, 12, 13, 15 mirrored; nothing beside it.
        for (x, road) in [
            (0, [0, 0, 0]),
            (1, [2, 15, 0]),
            (2, [2, 12, 0]),
            (3, [2, 13, 0]),
            (4, [2, 15, ROAD_MIRRORED]),
            (5, [0, 0, 0]),
        ] {
            assert_eq!(
                &tile(x, 5)[2..],
                &[0, 0, road[0], road[1], road[2]],
                "road at {x}"
            );
        }
        for (terrain, first) in [
            (Terrain::Dirt, 21),
            (Terrain::Water, 21),
            (Terrain::Rock, 0),
            (Terrain::Snow, 49),
            (Terrain::Rough, 49),
            (Terrain::Subterranean, 49),
            (Terrain::Lava, 49),
        ] {
            assert_eq!(terrain.interior_view(), first);
        }
    }

    #[test]
    fn invalid_maps_are_rejected() {
        let mut map = smoke_like_map();
        map.size = 40;
        assert!(write(&map).is_err());
        for road in [(1, 1, 5), (4, 4, 5), (1, 36, 5), (1, 4, 36)] {
            let mut map = smoke_like_map();
            (map.roads[0].from, map.roads[0].to, map.roads[0].row) = road;
            assert!(write(&map).is_err(), "{road:?}");
        }
        let mut map = smoke_like_map();
        map.roads.push(Road {
            kind: RoadKind::Dirt,
            row: 5,
            from: 4,
            to: 6,
        });
        assert!(write(&map).is_err());
        map.roads[1].from = 5;
        assert!(write(&map).is_ok());
        let mut map = smoke_like_map();
        map.patches[0].from = (5, 3);
        assert!(write(&map).is_err());
        let mut map = smoke_like_map();
        map.patches[0].from = (2, 4);
        assert!(write(&map).is_err());
        let mut map = smoke_like_map();
        map.patches[0].to = (36, 3);
        assert!(write(&map).is_err());
        let mut map = smoke_like_map();
        map.patches[0].to = (4, 36);
        assert!(write(&map).is_err());
        let base = smoke_like_map();
        let with_hero = |f: &dyn Fn(&mut Hero)| {
            let mut map = base.clone();
            if let MapObject::Hero(hero) = &mut map.objects[0] {
                f(hero);
            }
            write(&map)
        };
        assert!(with_hero(&|h| h.owner = 2).is_err());
        assert!(with_hero(&|h| h.hero_type = 156).is_err());
        assert!(with_hero(&|h| h.class = 18).is_err());
        assert!(with_hero(&|h| h.tile = (35, 1)).is_err());
        assert!(with_hero(&|h| h.tile = (36, 1)).is_err());
        assert!(with_hero(&|h| h.skills = Some(vec![(1, 1); 9])).is_err());
        assert!(with_hero(&|h| h.skills = Some(vec![(28, 1)])).is_err());
        assert!(with_hero(&|h| h.skills = Some(vec![(1, 4)])).is_err());
        assert!(with_hero(&|h| h.army = Some([Some((145, 1)); ARMY_SLOTS])).is_err());
        assert!(with_hero(&|h| h.army = Some([Some((1, 0)); ARMY_SLOTS])).is_err());
        assert!(with_hero(&|h| h.name = Some("Ω".into())).is_err());
        assert!(with_hero(&|h| h.name = Some("x".repeat(MAX_STRING + 1))).is_err());
        assert!(
            with_hero(&|h| {
                h.artifacts = Some(HeroArtifacts {
                    worn: [None; ARTIFACT_SLOTS],
                    backpack: vec![144],
                })
            })
            .is_err()
        );
        assert!(
            with_hero(&|h| {
                h.artifacts = Some(HeroArtifacts {
                    worn: [None; ARTIFACT_SLOTS],
                    backpack: vec![1; 65],
                })
            })
            .is_err()
        );
        let mut map = base.clone();
        if let MapObject::Monster(monster) = &mut map.objects[1] {
            monster.disposition = 5;
        }
        assert!(write(&map).is_err());
        let mut map = base.clone();
        if let MapObject::Monster(monster) = &mut map.objects[1] {
            monster.template.class = 34;
        }
        assert!(write(&map).is_err());
        let mut map = base;
        if let MapObject::Monster(monster) = &mut map.objects[1] {
            monster.tile = (36, 0);
        }
        assert!(write(&map).is_err());
    }

    const STRONGHOLD: &str = "AVCstrx0.def 000001110000011110001111111111111111111111111111 \
                              001000000000000000000000000000000000000000000000 011111111 \
                              011111111 98 6 1 0";

    #[test]
    fn towns_are_written_with_the_editor_s_body_and_checked() {
        let town = Town {
            tile: (12, 9),
            template: ObjectTemplate::from_objects_txt(STRONGHOLD).unwrap(),
            owner: Some(0),
            garrison: Some([Some((1, 25)), None, None, None, None, None, None]),
            tight_formation: true,
            buildings: TownBuildings::Custom {
                built: [0x58, 0, 0x04, 0, 0, 0x01],
                forbidden: [0; 6],
            },
        };
        let mut map = smoke_like_map();
        map.objects.push(MapObject::Town(town.clone()));
        let out = write(&map).unwrap();
        let mut body = vec![];
        body.extend_from_slice(&3u32.to_le_bytes()); // the third object's identifier
        body.extend_from_slice(&[0, 0, 1]); // red, no name, a garrison
        body.extend_from_slice(&[1, 0, 25, 0]);
        body.extend_from_slice(&[0xff, 0xff, 0, 0].repeat(6));
        body.extend_from_slice(&[1, 1, 0x58, 0, 0x04, 0, 0, 0x01]); // tight, custom, built
        body.extend_from_slice(&[0; 6 + 9 + 9 + 4]); // forbidden, spells, events
        body.extend_from_slice(&[0xff, 0, 0, 0, 0, 0, 0, 0]); // alignment, 3 unused, no events
        assert!(out.ends_with(&body));
        // A neutral town with the standard buildings and a fort, its default garrison.
        let mut map = smoke_like_map();
        map.objects.push(MapObject::Town(Town {
            owner: None,
            garrison: None,
            tight_formation: false,
            buildings: TownBuildings::Standard { fort: true },
            ..town.clone()
        }));
        let out = write(&map).unwrap();
        let mut body = 3u32.to_le_bytes().to_vec();
        body.extend_from_slice(&[0xff, 0, 0, 0, 0, 1]);
        body.extend_from_slice(&[0; 9 + 9 + 4]);
        body.extend_from_slice(&[0xff, 0, 0, 0, 0, 0, 0, 0]);
        assert!(out.ends_with(&body));
        let with_town = |f: &dyn Fn(&mut Town)| {
            let mut town = town.clone();
            f(&mut town);
            let mut map = smoke_like_map();
            map.objects.push(MapObject::Town(town));
            write(&map).map(|_| ())
        };
        let fails = |f: &dyn Fn(&mut Town), what: &str| match with_town(f) {
            Err(WriteError::OutOfRange { what: w, .. }) => assert_eq!(w, what),
            other => panic!("{what}: {other:?}"),
        };
        fails(&|t| t.template.class = 77, "town object class");
        fails(&|t| t.template.subclass = 9, "town faction");
        fails(&|t| t.owner = Some(2), "town owner (no playable player)");
        fails(&|t| t.owner = Some(8), "town owner (no playable player)");
        fails(
            &|t| t.garrison = Some([Some((145, 1)); ARMY_SLOTS]),
            "creature",
        );
        fails(
            &|t| t.garrison = Some([Some((1, 0)); ARMY_SLOTS]),
            "stack size",
        );
        let bit = |built: [u8; 6], forbidden: [u8; 6]| {
            move |t: &mut Town| t.buildings = TownBuildings::Custom { built, forbidden }
        };
        fails(&bit([0, 0, 0, 0, 0, 0x02], [0; 6]), "town building (bit)");
        fails(&bit([0; 6], [0, 0, 0, 0, 0, 0x80]), "town building (bit)");
        fails(&|t| t.tile = (36, 9), "object tile");
        assert_eq!(
            with_town(&bit([0xff, 0xff, 0xff, 0xff, 0xff, 0x01], [0; 6])),
            Ok(())
        );
    }

    const WOOD: &str = "AVTwood0.def 011111111111111111111111111111111111111111111111 \
                        100000000000000000000000000000000000000000000000 111111111 000000001 \
                        79 0 5 0";

    #[test]
    fn piles_are_written_with_the_original_s_body() {
        let mut map = smoke_like_map();
        let template = ObjectTemplate::from_objects_txt(WOOD).unwrap();
        map.objects.push(MapObject::Pile(Pile {
            tile: (7, 9),
            template: template.clone(),
            amount: 0x0006_0304,
            guards: None,
        }));
        let bytes = write(&map).unwrap();
        assert_eq!(templates(&map).len(), 3);
        assert_eq!(templates(&map)[2], template);
        assert_eq!(position(&map.objects[2]), (7, 9));
        // The last object: its position, template index, 5 zero bytes, then the body (no message,
        // the amount, 4 unused bytes), and the event count.
        let tail = &bytes[bytes.len() - 25..];
        assert_eq!(&tail[..7], &[7, 9, 0, 2, 0, 0, 0]);
        assert_eq!(&tail[7..12], &[0; 5]);
        assert_eq!(&tail[12..21], &[0, 4, 3, 6, 0, 0, 0, 0, 0]);
        assert_eq!(&tail[21..], &[0; 4]);
        let with_pile = |f: &dyn Fn(&mut Pile)| {
            let mut map = map.clone();
            if let MapObject::Pile(pile) = &mut map.objects[2] {
                f(pile);
            }
            write(&map)
        };
        assert!(with_pile(&|p| p.template.subclass = 6).is_ok());
        assert!(with_pile(&|p| p.template.subclass = 7).is_err());
        assert!(with_pile(&|p| p.template.class = 76).is_err());
        assert!(with_pile(&|p| p.tile = (36, 0)).is_err());
        assert!(with_pile(&|p| p.amount = MAX_PILE_AMOUNT).is_ok());
        assert!(with_pile(&|p| p.amount = MAX_PILE_AMOUNT + 1).is_err());
    }

    #[test]
    fn guarded_piles_carry_their_guards_in_the_message() {
        let mut map = smoke_like_map();
        let mut army = [None; ARMY_SLOTS];
        army[2] = Some((0, 12));
        map.objects.push(MapObject::Pile(Pile {
            tile: (7, 9),
            template: ObjectTemplate::from_objects_txt(WOOD).unwrap(),
            amount: 5,
            guards: Some(Guards {
                army,
                question: "Q?".into(),
            }),
        }));
        let bytes = write(&map).unwrap();
        // The body after the object's 12 bytes: the message flag, its text, the guards' flag and
        // army, 4 unused bytes; the amount and 4 unused bytes; then the event count.
        let body = &bytes[bytes.len() - 4 - 48..bytes.len() - 4];
        assert_eq!(&body[..7], &[1, 2, 0, 0, 0, b'Q', b'?']);
        assert_eq!(body[7], 1);
        let mut stacks = Vec::new();
        for slot in body[8..36].chunks(4) {
            stacks.push((
                u16::from_le_bytes([slot[0], slot[1]]),
                u16::from_le_bytes([slot[2], slot[3]]),
            ));
        }
        let empty = (0xffff, 0);
        assert_eq!(stacks, [empty, empty, (0, 12), empty, empty, empty, empty]);
        assert_eq!(&body[36..], &[0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0]);
        let with_guards = |f: &dyn Fn(&mut Guards)| {
            let mut map = map.clone();
            if let Some(MapObject::Pile(Pile {
                guards: Some(guards),
                ..
            })) = map.objects.last_mut()
            {
                f(guards);
            }
            write(&map)
        };
        // No question: an empty text, the guards fight at once.
        let at_once = with_guards(&|g| g.question.clear()).unwrap();
        assert_eq!(at_once.len(), bytes.len() - 2);
        assert!(with_guards(&|g| g.army = [None; ARMY_SLOTS]).is_err());
        assert!(with_guards(&|g| g.army[0] = Some((CREATURE_COUNT, 1))).is_err());
        assert!(with_guards(&|g| g.army[0] = Some((1, 0))).is_err());
        assert!(with_guards(&|g| g.question = "\u{100}".into()).is_err());
    }

    #[test]
    fn the_underground_follows_the_surface() {
        let mut map = smoke_like_map();
        let one_level = write(&map).unwrap();
        let MapObject::Hero(mut below) = map.objects[0].clone() else {
            panic!("the smoke map's first object is a hero")
        };
        below.tile = (3, 4);
        below.hero_type = 1;
        map.underground = Some(Underground {
            terrain: Terrain::Subterranean,
            patches: vec![Patch {
                terrain: Terrain::Rock,
                from: (0, 0),
                to: (35, 1),
            }],
            objects: vec![MapObject::Hero(below.clone())],
        });
        let bytes = write(&map).unwrap();
        // The header's flag, the second level's tiles after the first's, the hero's object.
        assert_eq!(&bytes[4..10], &[1, 36, 0, 0, 0, 1]);
        let mut out = Out::default();
        write_hero(&mut out, &below, 3).unwrap();
        // The second level's tiles, Red's second hero in its list (type and an empty name), the
        // hero's object (12 bytes and its body).
        assert_eq!(
            bytes.len(),
            one_level.len() + 7 * 36 * 36 + 5 + 12 + out.0.len()
        );
        let find = |level: &[u8]| bytes.windows(level.len()).position(|w| w == level).unwrap();
        let mut surface = Out::default();
        write_terrain(&mut surface, 36, map.terrain, &map.patches, &map.roads);
        let mut out = Out::default();
        let u = map.underground.as_ref().unwrap();
        write_terrain(&mut out, 36, u.terrain, &u.patches, &[]);
        assert_eq!(find(&out.0), find(&surface.0) + 7 * 36 * 36);
        assert_eq!((out.0[0], out.0[7 * 36 * 2]), (9, 6));
        // Red's heroes: the surface's, then the underground's.
        let mut out = Out::default();
        write_player(&mut out, &map, 0, map.players[0].as_ref()).unwrap();
        assert_eq!(
            &out.0[out.0.len() - 14..],
            &[2, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0]
        );
        assert_eq!(
            map.placed().map(|(level, _)| level).collect::<Vec<_>>(),
            [0, 0, 1]
        );
        let with_level = |f: &dyn Fn(&mut Underground)| {
            let mut map = map.clone();
            f(map.underground.as_mut().unwrap());
            write(&map)
        };
        assert!(with_level(&|u| u.patches[0].to = (36, 1)).is_err());
        assert!(with_level(&|u| u.patches[0].from = (1, 2)).is_err());
        assert!(
            with_level(&|u| {
                if let MapObject::Hero(h) = &mut u.objects[0] {
                    h.tile = (35, 4);
                }
            })
            .is_err()
        );
        assert!(with_level(&|u| u.objects.clear()).is_ok());
    }

    #[test]
    fn gzip_round_trips_and_carries_the_crc() {
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
        let data = write(&smoke_like_map()).unwrap();
        let packed = gzip(&data);
        assert_eq!(&packed[..3], &[0x1f, 0x8b, 8]);
        let body = &packed[10..packed.len() - 8];
        let unpacked = miniz_oxide::inflate::decompress_to_vec(body).unwrap();
        assert_eq!(unpacked, data);
        let tail = &packed[packed.len() - 8..];
        assert_eq!(&tail[..4], &crc32(&data).to_le_bytes());
        assert_eq!(&tail[4..], &(data.len() as u32).to_le_bytes());
    }
}
