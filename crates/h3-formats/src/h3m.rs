//! Writer for the subset of the SoD map format (`.h3m`, version `0x1C`) that staged experiments
//! need (E04a T2, OPEN-QUESTIONS Q28): one terrain, players without towns, heroes with custom
//! armies, skills, artifacts, spells and primary skills, and wandering monsters.
//!
//! Layout notes are in `docs/formats/h3m.md`. The field order follows VCMI 1.7.5's map reader
//! (`lib/mapping/MapFormatH3M.cpp`, `MapReaderH3M.cpp`, `ObjectTemplate::readMap`; GPL-2.0-or-later,
//! the order and sizes were ported, no code), and was checked by parsing SoD maps shipped with the
//! game. The output is a whole map file: [`write`] produces the uncompressed bytes, [`gzip`]
//! wraps them the way the game stores maps.
//!
//! Every value is validated before anything is written; [`write`] never panics.

use crate::FormatError;

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
    /// The first of eight interior tile pictures ("view" in map files) of this terrain, for the
    /// terrains whose pictures were observed in the game's own maps (dirt and water 21–28, grass
    /// 49–56). Other terrains are added when a map shows their numbers.
    const fn interior_view(self) -> Option<u8> {
        match self {
            Self::Dirt | Self::Water => Some(21),
            Self::Grass => Some(49),
            Self::Sand
            | Self::Snow
            | Self::Swamp
            | Self::Rough
            | Self::Subterranean
            | Self::Lava
            | Self::Rock => None,
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
    /// [`FormatError::Text`] if the line is malformed.
    pub fn from_objects_txt(line: &str) -> Result<Self, FormatError> {
        let bad = |message| FormatError::Text { record: 0, message };
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

    fn write(&self, out: &mut Out) -> Result<(), FormatError> {
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

/// `Objects.txt` masks are 6 rows of 8 characters, top left first; map files store them as 6
/// bytes whose bit `j` of byte `i` is character `47 − (8i + j)` (VCMI `ObjectTemplate::readTxt`
/// and `readMap` agree on this mapping).
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

/// An object placed on the map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapObject {
    /// A hero.
    Hero(Hero),
    /// A wandering monster.
    Monster(Monster),
}

/// A map: one level, one terrain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map {
    /// Name shown in the scenario list.
    pub name: String,
    /// Description shown in the scenario list.
    pub description: String,
    /// Width and height (36, 72, 108 or 144).
    pub size: u8,
    /// Terrain of every tile.
    pub terrain: Terrain,
    /// Players (index = colour); `None` = nobody plays this colour.
    pub players: [Option<Player>; PLAYERS],
    /// Objects.
    pub objects: Vec<MapObject>,
}

/// Writes `map` as an uncompressed SoD map.
///
/// # Errors
/// [`FormatError::OutOfRange`] if a value does not fit the format or the map (tile outside the
/// map, unknown hero, too many skills, an owner without a playable player, …).
pub fn write(map: &Map) -> Result<Vec<u8>, FormatError> {
    validate(map)?;
    let mut out = Out::default();
    out.u32(SOD_FORMAT);
    out.bool(map.players.iter().any(Option::is_some));
    out.u32(u32::from(map.size));
    out.bool(false); // no underground
    out.string(&map.name)?;
    out.string(&map.description)?;
    out.u8(1); // difficulty: normal
    out.u8(0); // no level limit
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
    write_terrain(&mut out, map);
    let templates = templates(map);
    out.u32(len32(templates.len()));
    for template in &templates {
        template.write(&mut out)?;
    }
    out.u32(len32(map.objects.len()));
    for (index, object) in map.objects.iter().enumerate() {
        let template = object_template(object);
        let template_index = templates.iter().position(|t| *t == template).unwrap_or(0);
        let (x, y) = position(object);
        out.bytes(&[x, y, 0]);
        out.u32(len32(template_index));
        out.bytes(&[0; 5]);
        let identifier = len32(index).saturating_add(1);
        match object {
            MapObject::Hero(hero) => write_hero(&mut out, hero, identifier)?,
            MapObject::Monster(monster) => write_monster(&mut out, monster, identifier),
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

/// CRC-32 (IEEE 802.3, reflected, polynomial 0xEDB88320), as gzip uses it.
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

fn out_of_range(what: &'static str, value: impl TryInto<u64>) -> FormatError {
    FormatError::OutOfRange {
        what,
        value: value.try_into().unwrap_or(u64::MAX),
    }
}

fn validate(map: &Map) -> Result<(), FormatError> {
    if ![36, 72, 108, 144].contains(&map.size) {
        return Err(out_of_range("map size", map.size));
    }
    if map.terrain.interior_view().is_none() {
        return Err(out_of_range(
            "terrain without known tile pictures",
            map.terrain as u8,
        ));
    }
    for object in &map.objects {
        let (x, y) = match object {
            MapObject::Hero(hero) => hero.tile,
            MapObject::Monster(monster) => monster.tile,
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
        }
    }
    Ok(())
}

fn validate_hero(map: &Map, hero: &Hero) -> Result<(), FormatError> {
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
        for &(creature, count) in army.iter().flatten() {
            if creature >= CREATURE_COUNT {
                return Err(out_of_range("creature", creature));
            }
            if count == 0 {
                return Err(out_of_range("stack size", count));
            }
        }
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

fn validate_monster(monster: &Monster) -> Result<(), FormatError> {
    if !MONSTER_OBJECTS.contains(&monster.template.class) {
        return Err(out_of_range("monster object class", monster.template.class));
    }
    if monster.disposition > 4 {
        return Err(out_of_range("monster disposition", monster.disposition));
    }
    Ok(())
}

fn write_player(
    out: &mut Out,
    map: &Map,
    colour: usize,
    player: Option<&Player>,
) -> Result<(), FormatError> {
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
        .objects
        .iter()
        .filter_map(|object| match object {
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

fn write_terrain(out: &mut Out, map: &Map) {
    let size = usize::from(map.size);
    let view = map.terrain.interior_view().unwrap_or(0);
    for index in 0..size * size {
        // A deterministic spread over the eight interior pictures (the editor picks randomly).
        let variant = ((index * 7 + index / size * 3) % 8) as u8;
        out.bytes(&[map.terrain as u8, view + variant, 0, 0, 0, 0, 0]);
    }
}

fn object_template(object: &MapObject) -> ObjectTemplate {
    match object {
        MapObject::Hero(hero) => ObjectTemplate::hero(hero.class),
        MapObject::Monster(monster) => monster.template.clone(),
    }
}

/// Distinct templates in order of first use.
fn templates(map: &Map) -> Vec<ObjectTemplate> {
    let mut templates: Vec<ObjectTemplate> = Vec::new();
    for object in &map.objects {
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
    }
}

fn write_hero(out: &mut Out, hero: &Hero, identifier: u32) -> Result<(), FormatError> {
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
        for slot in army {
            let (creature, count) = slot.unwrap_or((0xffff, 0));
            out.u16(creature);
            out.u16(count);
        }
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
    fn string(&mut self, text: &str) -> Result<(), FormatError> {
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
        }
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
    fn invalid_maps_are_rejected() {
        let mut map = smoke_like_map();
        map.size = 40;
        assert!(write(&map).is_err());
        let mut map = smoke_like_map();
        map.terrain = Terrain::Lava;
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
