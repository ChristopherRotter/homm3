//! The towns' static tables of `Heroes3.exe` (`docs/formats/installation.md` §7.61; E09 T5b2b1):
//! each faction's buildings with the buildings each requires, the supersede lists, the map's
//! building slots, the creatures by level, the hordes, the Resource Silo, the neutral garrison and
//! the standard set's level-2 dwelling chance. Read as the file holds them, before start-up: what
//! `building_tables_init` 0x4EB730 and `horde_table_init` 0x5BDF60 compute from them (the closed
//! masks, the hordes' levels and upgrades) is the harvest's to derive.

use super::{Immediate, PeImage};
use crate::{FormatError, le};

/// Building ids of the towns' tables are below this (0 to 43); the map's building slots write it
/// for "none".
pub const BUILDING_IDS: u8 = 44;
/// Slots of the map's building ids a faction (the map's town building bits, `docs/formats/h3m.md`).
pub const MAP_SLOTS: usize = 41;
/// Creatures of a faction: the seven levels' creatures, then their upgrades.
pub const TOWN_CREATURES: usize = 14;
/// Resources of a Resource Silo row, in `RESTYPES.TXT`'s order: wood, mercury, ore, sulfur,
/// crystal, gems, gold.
pub const SILO_RESOURCES: usize = 7;
/// Horde records of a faction, in the horde buildings' order: the horde 18, its upgrade 19, the
/// horde 24, its upgrade 25.
pub const HORDE_RECORDS: usize = 4;
/// Size of one horde record: the creature (`i32`), its growth (`i16`) and its level (`i16`).
pub const HORDE_RECORD_SIZE: usize = 8;
/// Levels of a neutral town's drawn garrison.
pub const GARRISON_LEVELS: usize = 4;

/// Ends an entry of a building or supersede list.
const ENTRY_END: i32 = -1;
/// Ends a list, in an entry's place.
const LIST_END: i32 = -100;
/// The most entries of a list, and the most ids of an entry (a faction's mask holds 64 bits).
const MAX_LIST: usize = 64;

/// Where a build keeps the towns' tables (§7.61).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TownTables {
    /// Virtual address of each faction's building list, by faction (their number is the
    /// factions'): per building its id, the ids it requires (direct) and −1; −100 ends the list.
    pub building_lists: &'static [u32],
    /// Virtual address of the common supersede list (the same layout: a building, the ids it
    /// supersedes, −1).
    pub common_supersedes: u32,
    /// Virtual address of each faction's supersede list, by faction.
    pub supersede_lists: &'static [u32],
    /// Virtual address of the map's building ids: [`MAP_SLOTS`] building ids (`i32`) a faction.
    pub map_building_ids: u32,
    /// Virtual address of the factions' creatures: [`TOWN_CREATURES`] creature ids (`i32`) a
    /// faction.
    pub creatures: u32,
    /// Virtual address of the hordes: [`HORDE_RECORDS`] records of [`HORDE_RECORD_SIZE`] bytes a
    /// faction.
    pub hordes: u32,
    /// Virtual address of the hordes' buildings: [`HORDE_RECORDS`] building ids (`i32`).
    pub horde_buildings: u32,
    /// Virtual address of the Resource Silo's resources a day: [`SILO_RESOURCES`] amounts (`i32`)
    /// a faction.
    pub silo: u32,
    /// Virtual addresses of the neutral garrison's chances in 100, its least and its most counts:
    /// [`GARRISON_LEVELS`] values (`i32`) each.
    pub garrison: [u32; 3],
    /// The standard set's level-2 dwelling chance in 100 (`cmp eax, 30`, 0x5C0C60).
    pub dwelling_2_chance: Immediate,
}

/// One entry of a building or supersede list: a building and the ids listed after it, the
/// buildings it requires or those it supersedes (direct, in the file's order).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildingEntry {
    /// The building's id.
    pub building: u8,
    /// The ids listed after it.
    pub listed: Vec<u8>,
}

/// A horde record as the file holds it: the base hordes' creature and growth; the levels and the
/// upgrades' records are filled at start-up (`horde_table_init` 0x5BDF60), so their fields are
/// not read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HordeRecord {
    /// The creature, or none (−1: the faction has no such horde, or the record is an upgrade's).
    pub creature: Option<u16>,
    /// The creatures it adds a week (increment 3's).
    pub growth: u16,
}

/// A faction's town tables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactionTowns {
    /// Its buildings, each with the buildings it requires.
    pub buildings: Vec<BuildingEntry>,
    /// Its own supersede list (the common one is [`TownData::common_supersedes`]).
    pub supersedes: Vec<BuildingEntry>,
    /// The building of each of the map's [`MAP_SLOTS`] slots, or none.
    pub map_building_ids: Vec<Option<u8>>,
    /// Its [`TOWN_CREATURES`] creatures: levels 1 to 7, then their upgrades.
    pub creatures: Vec<u16>,
    /// Its [`HORDE_RECORDS`] horde records.
    pub hordes: Vec<HordeRecord>,
    /// The Resource Silo's [`SILO_RESOURCES`] amounts a day.
    pub silo: Vec<u32>,
}

/// One level of a neutral town's drawn garrison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GarrisonLevel {
    /// The chance in 100 that the level is drawn.
    pub chance: u32,
    /// The fewest creatures of the drawn stack.
    pub least: u32,
    /// The most creatures of the drawn stack.
    pub most: u32,
}

/// The towns' tables of a build (§7.61).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TownData {
    /// Each faction's tables, by faction.
    pub factions: Vec<FactionTowns>,
    /// The supersede list every faction starts from.
    pub common_supersedes: Vec<BuildingEntry>,
    /// The buildings of the [`HORDE_RECORDS`] horde records, in their order.
    pub horde_buildings: Vec<u8>,
    /// The neutral garrison's [`GARRISON_LEVELS`] levels.
    pub garrison: Vec<GarrisonLevel>,
    /// The standard set's level-2 dwelling chance in 100.
    pub dwelling_2_chance: u8,
}

/// Reads the towns' tables of a known build.
///
/// # Errors
/// Unknown build, unmapped or truncated tables, an unterminated or too long list, a building id
/// outside 0 to 43, a negative creature, growth or amount, a chance above 100, an instruction that
/// is not the expected one.
pub fn town_data(data: &[u8]) -> Result<TownData, FormatError> {
    let (build, image) = super::identify(data)?;
    town_data_at(&image, &build.towns)
}

/// Reads the tables described by `t` (exposed for tests with synthetic images).
///
/// # Errors
/// As [`town_data`], without the build check; also a supersede list's count other than the
/// building lists'.
pub fn town_data_at(image: &PeImage<'_>, t: &TownTables) -> Result<TownData, FormatError> {
    if t.supersede_lists.len() != t.building_lists.len() {
        return Err(FormatError::OutOfRange {
            what: "town supersede lists",
            value: t.supersede_lists.len() as u64,
        });
    }
    let factions = t
        .building_lists
        .iter()
        .zip(t.supersede_lists)
        .enumerate()
        .map(|(f, (&buildings, &supersedes))| {
            Ok(FactionTowns {
                buildings: list(image, buildings, "town building list")?,
                supersedes: list(image, supersedes, "town supersede list")?,
                map_building_ids: ints(
                    image,
                    t.map_building_ids,
                    f,
                    MAP_SLOTS,
                    "map building ids",
                )?
                .into_iter()
                .map(|id| {
                    if id == i32::from(BUILDING_IDS) {
                        Ok(None)
                    } else {
                        building(id, "map building ids").map(Some)
                    }
                })
                .collect::<Result<_, _>>()?,
                creatures: ints(image, t.creatures, f, TOWN_CREATURES, "town creatures")?
                    .into_iter()
                    .map(|c| creature(c, "town creatures"))
                    .collect::<Result<_, _>>()?,
                hordes: hordes(image, t.hordes, f)?,
                silo: ints(image, t.silo, f, SILO_RESOURCES, "town silo")?
                    .into_iter()
                    .map(|n| amount(n, "town silo"))
                    .collect::<Result<_, _>>()?,
            })
        })
        .collect::<Result<_, FormatError>>()?;
    let horde_buildings = ints(
        image,
        t.horde_buildings,
        0,
        HORDE_RECORDS,
        "horde buildings",
    )?
    .into_iter()
    .map(|b| building(b, "horde buildings"))
    .collect::<Result<_, _>>()?;
    let [chances, least, most] = t.garrison.map(|va| {
        ints(image, va, 0, GARRISON_LEVELS, "neutral garrison")?
            .into_iter()
            .map(|n| amount(n, "neutral garrison"))
            .collect::<Result<Vec<_>, _>>()
    });
    let garrison = chances?
        .into_iter()
        .zip(least?)
        .zip(most?)
        .map(|((chance, least), most)| {
            Ok(GarrisonLevel {
                chance: chance_in_100(chance, "neutral garrison chance")?,
                least,
                most,
            })
        })
        .collect::<Result<_, FormatError>>()?;
    let dwelling_2_chance = immediate(image, t.dwelling_2_chance)?;
    chance_in_100(u32::from(dwelling_2_chance), "level-2 dwelling chance")?;
    Ok(TownData {
        factions,
        common_supersedes: list(image, t.common_supersedes, "town supersede list")?,
        horde_buildings,
        garrison,
        dwelling_2_chance,
    })
}

/// The file offset of a table.
fn offset(image: &PeImage<'_>, va: u32, what: &'static str) -> Result<usize, FormatError> {
    image.file_offset(va).ok_or(FormatError::OutOfRange {
        what,
        value: u64::from(va),
    })
}

/// The `count` values (`i32`) of the `index`th row of a table.
fn ints(
    image: &PeImage<'_>,
    va: u32,
    index: usize,
    count: usize,
    what: &'static str,
) -> Result<Vec<i32>, FormatError> {
    let base = offset(image, va, what)? + 4 * count * index;
    (0..count)
        .map(|i| le::i32_at(image.data, base + 4 * i, what))
        .collect()
}

/// A building or supersede list.
fn list(
    image: &PeImage<'_>,
    va: u32,
    what: &'static str,
) -> Result<Vec<BuildingEntry>, FormatError> {
    let mut at = offset(image, va, what)?;
    let mut next = || {
        let v = le::i32_at(image.data, at, what);
        at += 4;
        v
    };
    let too_long = FormatError::OutOfRange {
        what,
        value: MAX_LIST as u64 + 1,
    };
    let mut entries = Vec::new();
    loop {
        let first = next()?;
        if first == LIST_END {
            return Ok(entries);
        }
        if entries.len() == MAX_LIST {
            return Err(too_long);
        }
        let mut entry = BuildingEntry {
            building: building(first, what)?,
            listed: Vec::new(),
        };
        loop {
            let id = next()?;
            if id == ENTRY_END {
                break;
            }
            if entry.listed.len() == MAX_LIST {
                return Err(too_long);
            }
            entry.listed.push(building(id, what)?);
        }
        entries.push(entry);
    }
}

/// A faction's horde records.
fn hordes(image: &PeImage<'_>, va: u32, faction: usize) -> Result<Vec<HordeRecord>, FormatError> {
    let what = "town hordes";
    let base = offset(image, va, what)? + HORDE_RECORDS * HORDE_RECORD_SIZE * faction;
    (0..HORDE_RECORDS)
        .map(|r| {
            let at = base + HORDE_RECORD_SIZE * r;
            let id = le::i32_at(image.data, at, what)?;
            let growth = le::u16_at(image.data, at + 4, what)?;
            Ok(HordeRecord {
                creature: if id == -1 {
                    None
                } else {
                    Some(creature(id, what)?)
                },
                growth: Some(growth)
                    .filter(|&g| g <= i16::MAX.unsigned_abs())
                    .ok_or(FormatError::OutOfRange {
                        what: "horde growth",
                        value: u64::from(growth),
                    })?,
            })
        })
        .collect()
}

/// The one-byte operand of an instruction, checked against its expected bytes.
fn immediate(image: &PeImage<'_>, i: Immediate) -> Result<u8, FormatError> {
    let what = "the level-2 dwelling's instruction";
    let at = offset(image, i.va, what)?;
    let code = image
        .data
        .get(at..at + i.opcode.len() + 1)
        .ok_or(FormatError::Truncated { what, offset: at })?;
    if code[..i.opcode.len()] != *i.opcode {
        return Err(FormatError::BadMagic(what));
    }
    Ok(code[i.opcode.len()])
}

/// A building id, 0 to 43.
fn building(v: i32, what: &'static str) -> Result<u8, FormatError> {
    u8::try_from(v)
        .ok()
        .filter(|&b| b < BUILDING_IDS)
        .ok_or(FormatError::OutOfRange {
            what,
            value: u64::from(v.unsigned_abs()),
        })
}

/// A creature id, not negative.
fn creature(v: i32, what: &'static str) -> Result<u16, FormatError> {
    u16::try_from(v).map_err(|_| FormatError::OutOfRange {
        what,
        value: u64::from(v.unsigned_abs()),
    })
}

/// An amount, not negative.
fn amount(v: i32, what: &'static str) -> Result<u32, FormatError> {
    u32::try_from(v).map_err(|_| FormatError::OutOfRange {
        what,
        value: u64::from(v.unsigned_abs()),
    })
}

/// A chance in 100.
fn chance_in_100(v: u32, what: &'static str) -> Result<u32, FormatError> {
    if v > 100 {
        return Err(FormatError::OutOfRange {
            what,
            value: u64::from(v),
        });
    }
    Ok(v)
}

#[cfg(test)]
mod tests;
