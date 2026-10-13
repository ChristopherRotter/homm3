//! Static tables inside `Heroes3.exe` (`docs/formats/installation.md` §5, §7).
//!
//! Only builds listed in [`KNOWN_BUILDS`] are read: the table location is build-specific, so an
//! unknown executable is rejected instead of being read at a guessed offset.

use crate::{FormatError, le};

/// A build of the executable whose static tables are located.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownBuild {
    /// Human-readable name.
    pub name: &'static str,
    /// COFF header `TimeDateStamp`.
    pub timestamp: u32,
    /// File length in bytes.
    pub file_len: usize,
    /// Virtual address of the static creature table.
    pub creature_table_va: u32,
    /// Number of creature records.
    pub creature_count: usize,
    /// Hero, specialty, skill-value and spell tables (E04a T3).
    pub heroes: HeroTables,
    /// The battlefield's obstacle tables (E04c T2b).
    pub obstacles: ObstacleTables,
}

/// Where a build keeps its battlefield obstacle tables (`docs/formats/installation.md` §7.26).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObstacleTables {
    /// Virtual address of the usual obstacle types (0x14 bytes each).
    pub types_va: u32,
    /// Number of usual obstacle types.
    pub type_count: usize,
    /// Virtual address of the absolute obstacles (0x44 bytes each).
    pub absolute_va: u32,
    /// Number of absolute obstacles.
    pub absolute_count: usize,
    /// Virtual address of the hexes blocked when both heroes are in boats (`i32` each).
    pub boats_va: u32,
    /// Number of those hexes.
    pub boats_count: usize,
}

/// Where a build keeps its hero-related static tables (`docs/formats/installation.md` §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeroTables {
    /// Virtual address of the hero table (§7.1).
    pub hero_va: u32,
    /// Number of hero records.
    pub hero_count: usize,
    /// Virtual address of the specialty table (§7.2), one record per hero.
    pub specialty_va: u32,
    /// Virtual address of the spell table (§7.6).
    pub spell_va: u32,
    /// Number of spell records.
    pub spell_count: usize,
    /// The secondary-skill value tables (§7.4).
    pub skill_values: &'static [SkillValueTable],
    /// Virtual address of the `f32` per-level growth of skill specialties (§7.4).
    pub specialty_growth_va: u32,
    /// Virtual address of the artifact primary-skill table (§7.3): four signed bytes per
    /// artifact (E04a T3b).
    pub artifact_primary_va: u32,
    /// Number of artifacts in that table.
    pub artifact_count: usize,
}

/// One secondary skill's four values (levels 0–3) in the executable (§7.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillValueTable {
    /// Secondary skill id (`SSTRAITS.TXT` row).
    pub skill: u8,
    /// Virtual address of the four values.
    pub va: u32,
    /// The values are `f32` (else `i32`).
    pub float: bool,
}

/// Skill value tables of the Complete 4.0 build whose getters were identified (§7.4).
const COMPLETE_SKILL_VALUES: &[SkillValueTable] = &[
    SkillValueTable {
        skill: 9,
        va: 0x0063_E998,
        float: false,
    }, // Luck
    SkillValueTable {
        skill: 6,
        va: 0x0063_E9A8,
        float: false,
    }, // Leadership
    SkillValueTable {
        skill: 12,
        va: 0x0063_E9B8,
        float: true,
    }, // Necromancy
    SkillValueTable {
        skill: 8,
        va: 0x0063_E9C8,
        float: false,
    }, // Mysticism
    SkillValueTable {
        skill: 3,
        va: 0x0063_E9D8,
        float: false,
    }, // Scouting
    SkillValueTable {
        skill: 1,
        va: 0x0063_E9E8,
        float: true,
    }, // Archery
    SkillValueTable {
        skill: 22,
        va: 0x0063_E9F8,
        float: true,
    }, // Offense
    SkillValueTable {
        skill: 23,
        va: 0x0063_EA08,
        float: true,
    }, // Armorer
    SkillValueTable {
        skill: 13,
        va: 0x0063_EA18,
        float: false,
    }, // Estates
    SkillValueTable {
        skill: 11,
        va: 0x0063_EA28,
        float: true,
    }, // Eagle Eye
    SkillValueTable {
        skill: 4,
        va: 0x0063_EA38,
        float: true,
    }, // Diplomacy
    SkillValueTable {
        skill: 26,
        va: 0x0063_EA48,
        float: true,
    }, // Resistance
    SkillValueTable {
        skill: 21,
        va: 0x0063_EA58,
        float: true,
    }, // Learning
    SkillValueTable {
        skill: 25,
        va: 0x0063_EA78,
        float: true,
    }, // Sorcery
    SkillValueTable {
        skill: 24,
        va: 0x0063_EA88,
        float: true,
    }, // Intelligence
    SkillValueTable {
        skill: 27,
        va: 0x0063_EA98,
        float: true,
    }, // First Aid
];

/// Builds known to this reader.
pub const KNOWN_BUILDS: &[KnownBuild] = &[KnownBuild {
    name: "Complete 4.0 (GOG, English), SHA-256 aac39e8c…3d7d",
    timestamp: 0x39B8_3835, // 2000-09-08 00:52:05 UTC
    file_len: 2_732_032,
    creature_table_va: 0x0067_03B8,
    creature_count: 150,
    heroes: HeroTables {
        hero_va: 0x0067_9DD0,
        hero_count: 156,
        specialty_va: 0x0067_8420,
        spell_va: 0x0068_5450,
        spell_count: 81,
        skill_values: COMPLETE_SKILL_VALUES,
        specialty_growth_va: 0x0063_EAE4,
        artifact_primary_va: 0x0063_E758,
        artifact_count: 144,
    },
    obstacles: ObstacleTables {
        types_va: 0x0063_C7C8,
        type_count: 91,
        absolute_va: 0x0063_BEC0,
        absolute_count: 34,
        boats_va: 0x0063_D368,
        boats_count: 32,
    },
}];

/// Size of one usual obstacle type record (§7.26).
pub const OBSTACLE_TYPE_SIZE: usize = 0x14;
/// Size of one absolute obstacle record (§7.26).
pub const ABSOLUTE_OBSTACLE_SIZE: usize = 0x44;
/// Most hexes a usual obstacle type blocks (its offsets field holds eight).
const MAX_TYPE_HEXES: usize = 8;
/// Most hexes an absolute obstacle record holds (+0xC to its name pointer at +0x40).
const MAX_ABSOLUTE_HEXES: usize = 26;

/// One usual obstacle type (§7.26): where it may appear and the hexes it blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObstacleTypeRecord {
    /// Terrain bits (bit `t` for terrain `t` of `TERRNAME.txt`; bit 8 water, the ship's deck).
    pub terrains: u16,
    /// Special field bits (bit 0 the shore, 1–9 the magic fields, §7.26).
    pub fields: u16,
    /// Rows it needs above its anchor.
    pub height: u8,
    /// Columns it needs from its anchor.
    pub width: u8,
    /// The blocked hexes as offsets from the anchor on the 17-column field (the original's
    /// numbering: −17 is the hex above in the same column parity).
    pub blocked: Vec<i8>,
}

/// One absolute obstacle (§7.26): where it may appear and its fixed hexes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbsoluteObstacleRecord {
    /// Terrain bits, as [`ObstacleTypeRecord::terrains`].
    pub terrains: u16,
    /// Special field bits, as [`ObstacleTypeRecord::fields`].
    pub fields: u16,
    /// The hexes it blocks, by hex number.
    pub blocked: Vec<u16>,
}

/// The battlefield's obstacle tables of a build (§7.26).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObstacleData {
    /// The usual obstacle types, by type number.
    pub types: Vec<ObstacleTypeRecord>,
    /// The absolute obstacles, by record number.
    pub absolute: Vec<AbsoluteObstacleRecord>,
    /// The hexes blocked when both heroes are in boats.
    pub boats: Vec<u16>,
}

/// Reads the obstacle tables of a known build.
///
/// # Errors
/// Unknown build, unmapped tables, or values outside the ranges the tables use.
pub fn obstacle_data(data: &[u8]) -> Result<ObstacleData, FormatError> {
    let (build, image) = identify(data)?;
    obstacle_data_at(&image, &build.obstacles)
}

/// Reads the tables described by `t` (exposed for tests with synthetic images).
///
/// # Errors
/// As [`obstacle_data`], without the build check.
pub fn obstacle_data_at(
    image: &PeImage<'_>,
    t: &ObstacleTables,
) -> Result<ObstacleData, FormatError> {
    let at = |va: u32, what: &'static str| {
        image.file_offset(va).ok_or(FormatError::OutOfRange {
            what,
            value: u64::from(va),
        })
    };
    let byte = |off: usize, what: &'static str| {
        image
            .data
            .get(off)
            .copied()
            .ok_or(FormatError::Truncated { what, offset: off })
    };
    let base = at(t.types_va, "obstacle type table address")?;
    let mut types = Vec::with_capacity(t.type_count.min(1024));
    for i in 0..t.type_count {
        let r = base + i * OBSTACLE_TYPE_SIZE;
        let count = usize::from(byte(r + 6, "obstacle type")?);
        if count == 0 || count > MAX_TYPE_HEXES {
            return Err(FormatError::OutOfRange {
                what: "obstacle type hex count",
                value: count as u64,
            });
        }
        let blocked = (0..count)
            .map(|k| byte(r + 8 + k, "obstacle type").map(|b| i8::from_le_bytes([b])))
            .collect::<Result<_, _>>()?;
        types.push(ObstacleTypeRecord {
            terrains: le::u16_at(image.data, r, "obstacle type")?,
            fields: le::u16_at(image.data, r + 2, "obstacle type")?,
            height: byte(r + 4, "obstacle type")?,
            width: byte(r + 5, "obstacle type")?,
            blocked,
        });
    }
    let base = at(t.absolute_va, "absolute obstacle table address")?;
    let mut absolute = Vec::with_capacity(t.absolute_count.min(1024));
    for i in 0..t.absolute_count {
        let r = base + i * ABSOLUTE_OBSTACLE_SIZE;
        // The hexes end at 0xFFFF or where the record's name pointer begins (+0x40).
        let mut blocked = Vec::new();
        for k in 0..MAX_ABSOLUTE_HEXES {
            let hex = le::u16_at(image.data, r + 0xC + 2 * k, "absolute obstacle")?;
            if hex == u16::MAX {
                break;
            }
            blocked.push(hex);
        }
        absolute.push(AbsoluteObstacleRecord {
            terrains: le::u16_at(image.data, r, "absolute obstacle")?,
            fields: le::u16_at(image.data, r + 2, "absolute obstacle")?,
            blocked,
        });
    }
    let base = at(t.boats_va, "boats hex table address")?;
    let boats = (0..t.boats_count)
        .map(|i| {
            let hex = le::i32_at(image.data, base + 4 * i, "boats hex table")?;
            u16::try_from(hex).map_err(|_| FormatError::OutOfRange {
                what: "boats hex",
                value: u64::from(hex.unsigned_abs()),
            })
        })
        .collect::<Result<_, _>>()?;
    Ok(ObstacleData {
        types,
        absolute,
        boats,
    })
}

/// Size of one hero record (§7.1).
pub const HERO_RECORD_SIZE: usize = 92;
/// Size of one specialty record (§7.2).
pub const SPECIALTY_RECORD_SIZE: usize = 40;
/// Size of one spell record (§7.6).
pub const SPELL_RECORD_SIZE: usize = 0x88;

/// The static part of a hero record (§7.1); names and army sizes are filled from the text
/// tables at start-up and are zero in the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeroRecord {
    /// Hero class (`HCTRAITS.TXT` row).
    pub class: i32,
    /// Starting secondary skills as (skill id, level 1–3); skill `-1` = none.
    pub skills: [(i32, i32); 2],
    /// Starts with a spell book.
    pub spell_book: bool,
    /// Starting spell id, `-1` = none.
    pub spell: i32,
    /// Starting creature types of the three army entries.
    pub creatures: [i32; 3],
    /// Flag byte 0: a hero of The Restoration of Erathia.
    pub roe: bool,
    /// Flag byte 1: available with the expansions.
    pub expansions: bool,
    /// Flag byte 2: a campaign hero.
    pub campaign: bool,
}

/// A specialty record (§7.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpecialtyRecord {
    /// Kind: −1 none, 0 skill, 1 creature, 2 resource, 3 spell, 4 creature stats, 5 speed,
    /// 6 upgrade, 7 dragons.
    pub kind: i32,
    /// Skill, creature, resource or spell id (kind 5: the speed bonus).
    pub id: i32,
    /// Attack bonus (kinds 4 and 7).
    pub attack: i32,
    /// Defense bonus (kinds 4 and 7).
    pub defense: i32,
    /// Damage bonus (kind 4).
    pub damage: i32,
    /// Upgrade targets (kind 6).
    pub upgrades: [i32; 2],
}

/// A skill table value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SkillValue {
    /// An integer value (levels, points).
    Integer(i32),
    /// A fraction (0.25 = 25 %).
    Float(f32),
}

/// The hero-related static tables of an executable.
#[derive(Debug, Clone, PartialEq)]
pub struct HeroData {
    /// Hero records by hero id.
    pub heroes: Vec<HeroRecord>,
    /// Specialty records by hero id.
    pub specialties: Vec<SpecialtyRecord>,
    /// Spell type by spell id: −1 negative, 0 neutral, 1 positive.
    pub spell_types: Vec<i32>,
    /// Spell flag word by spell id (§7.6, decoded in E04b T1).
    pub spell_flags: Vec<u32>,
    /// Values of levels 0–3 per identified skill table, in table order.
    pub skill_values: Vec<(u8, [SkillValue; 4])>,
    /// Growth of a skill specialty per hero level (0.05).
    pub specialty_growth: f32,
    /// Attack, defense, power and knowledge per artifact id (§7.3).
    pub artifact_primary: Vec<[i8; 4]>,
}

/// Size of one creature record.
pub const CREATURE_RECORD_SIZE: usize = 116;

/// One record of the static creature table. The numeric fields of the record are zero in the file
/// (the game fills them from `CRTRAITS.TXT`), so only the static part is returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureRecord {
    /// Creature id (index in the table).
    pub id: u16,
    /// Town index, `-1` for neutral.
    pub town: i32,
    /// Level, 0-based; `-1` for unused ids.
    pub level: i32,
    /// Sound-file prefix.
    pub sound: String,
    /// Battle sprite (DEF) name.
    pub def_name: String,
    /// Ability flag bits (meanings in the format catalogue).
    pub flags: u32,
}

/// A section of the PE image.
#[derive(Debug, Clone, Copy)]
struct Section {
    virtual_address: u32,
    raw_size: u32,
    raw_offset: u32,
}

/// Minimal PE32 view: image base, sections and link timestamp.
#[derive(Debug)]
pub struct PeImage<'a> {
    data: &'a [u8],
    timestamp: u32,
    image_base: u32,
    sections: Vec<Section>,
}

/// Upper bound for the section count (the COFF field is 16 bits; real images have a handful).
const MAX_SECTIONS: u16 = 96;

impl<'a> PeImage<'a> {
    /// Parses the DOS/PE headers and the section table.
    ///
    /// # Errors
    /// Missing signatures, a non-PE32 optional header, too many sections, or truncation.
    pub fn parse(data: &'a [u8]) -> Result<Self, FormatError> {
        if data.get(..2) != Some(b"MZ".as_slice()) {
            return Err(FormatError::BadMagic("DOS executable"));
        }
        let pe = le::u32_at(data, 0x3C, "DOS header")? as usize;
        let sig_end = pe.checked_add(4).ok_or(FormatError::Truncated {
            what: "PE header",
            offset: pe,
        })?;
        if data.get(pe..sig_end) != Some(b"PE\0\0".as_slice()) {
            return Err(FormatError::BadMagic("PE image"));
        }
        let coff = sig_end;
        let sections = le::u16_at(data, coff + 2, "COFF header")?;
        if sections > MAX_SECTIONS {
            return Err(FormatError::OutOfRange {
                what: "PE section count",
                value: u64::from(sections),
            });
        }
        let timestamp = le::u32_at(data, coff + 4, "COFF header")?;
        let optional_size = le::u16_at(data, coff + 16, "COFF header")? as usize;
        let optional = coff + 20;
        if le::u16_at(data, optional, "optional header")? != 0x10B {
            return Err(FormatError::BadMagic("PE32 optional header"));
        }
        let image_base = le::u32_at(data, optional + 28, "optional header")?;
        let table = optional + optional_size;
        let mut out = Vec::with_capacity(usize::from(sections));
        for i in 0..usize::from(sections) {
            let at = table + i * 40;
            out.push(Section {
                virtual_address: le::u32_at(data, at + 12, "section header")?,
                raw_size: le::u32_at(data, at + 16, "section header")?,
                raw_offset: le::u32_at(data, at + 20, "section header")?,
            });
        }
        Ok(Self {
            data,
            timestamp,
            image_base,
            sections: out,
        })
    }

    /// COFF link timestamp.
    #[must_use]
    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }

    /// Maps a virtual address to a file offset inside a section's raw data.
    #[must_use]
    pub fn file_offset(&self, va: u32) -> Option<usize> {
        let rva = va.checked_sub(self.image_base)?;
        self.sections.iter().find_map(|s| {
            let delta = rva.checked_sub(s.virtual_address)?;
            (delta < s.raw_size).then(|| s.raw_offset as usize + delta as usize)
        })
    }

    /// Reads a NUL-terminated string at a virtual address (at most 256 bytes, printable ASCII).
    ///
    /// # Errors
    /// The address is not mapped, the string is unterminated, too long or not printable.
    pub fn c_string(&self, va: u32) -> Result<String, FormatError> {
        let start = self.file_offset(va).ok_or(FormatError::OutOfRange {
            what: "string address",
            value: u64::from(va),
        })?;
        let tail = self.data.get(start..).unwrap_or_default();
        let len = tail
            .iter()
            .take(256)
            .position(|&b| b == 0)
            .ok_or(FormatError::OutOfRange {
                what: "string length",
                value: u64::from(va),
            })?;
        let bytes = &tail[..len];
        if !bytes.iter().all(|b| b.is_ascii_graphic() || *b == b' ') {
            return Err(FormatError::OutOfRange {
                what: "string contents",
                value: u64::from(va),
            });
        }
        Ok(bytes.iter().map(|&b| char::from(b)).collect())
    }
}

/// Identifies the build of an executable.
///
/// # Errors
/// [`FormatError::UnknownBuild`] if the image is valid but not in [`KNOWN_BUILDS`], or a PE error.
pub fn identify(data: &[u8]) -> Result<(&'static KnownBuild, PeImage<'_>), FormatError> {
    let image = PeImage::parse(data)?;
    KNOWN_BUILDS
        .iter()
        .find(|b| b.timestamp == image.timestamp() && b.file_len == data.len())
        .map(|b| (b, image))
        .ok_or_else(|| {
            FormatError::UnknownBuild(format!(
                "timestamp {:#010x}, {} bytes; known: {}",
                PeImage::parse(data)
                    .map(|i| i.timestamp())
                    .unwrap_or_default(),
                data.len(),
                KNOWN_BUILDS
                    .iter()
                    .map(|b| b.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })
}

/// Reads the static creature table of a known build.
///
/// # Errors
/// Unknown build, unmapped table or strings, or values outside the ranges seen in the table
/// (town `-1..=8`, level `-1..=6`).
pub fn creature_table(data: &[u8]) -> Result<Vec<CreatureRecord>, FormatError> {
    let (build, image) = identify(data)?;
    creature_table_at(&image, build.creature_table_va, build.creature_count)
}

/// Reads `count` creature records at virtual address `va` (exposed for tests with synthetic
/// images).
///
/// # Errors
/// As [`creature_table`], without the build check.
pub fn creature_table_at(
    image: &PeImage<'_>,
    va: u32,
    count: usize,
) -> Result<Vec<CreatureRecord>, FormatError> {
    let base = image.file_offset(va).ok_or(FormatError::OutOfRange {
        what: "creature table address",
        value: u64::from(va),
    })?;
    let mut out = Vec::with_capacity(count.min(1024));
    for i in 0..count {
        let at = base + i * CREATURE_RECORD_SIZE;
        let town = le::i32_at(image.data, at, "creature record")?;
        let level = le::i32_at(image.data, at + 4, "creature record")?;
        if !(-1..=8).contains(&town) {
            return Err(FormatError::OutOfRange {
                what: "creature town",
                value: u64::from(town.unsigned_abs()),
            });
        }
        if !(-1..=6).contains(&level) {
            return Err(FormatError::OutOfRange {
                what: "creature level",
                value: u64::from(level.unsigned_abs()),
            });
        }
        // Make sure the whole record is present even though only the static part is decoded.
        if image.data.len() < at + CREATURE_RECORD_SIZE {
            return Err(FormatError::Truncated {
                what: "creature record",
                offset: at,
            });
        }
        out.push(CreatureRecord {
            id: u16::try_from(i).map_err(|_| FormatError::OutOfRange {
                what: "creature id",
                value: i as u64,
            })?,
            town,
            level,
            sound: image.c_string(le::u32_at(image.data, at + 8, "creature record")?)?,
            def_name: image.c_string(le::u32_at(image.data, at + 12, "creature record")?)?,
            flags: le::u32_at(image.data, at + 16, "creature record")?,
        });
    }
    Ok(out)
}

/// Reads the hero-related static tables of a known build.
///
/// # Errors
/// Unknown build, unmapped tables, or values outside the ranges the tables use.
pub fn hero_data(data: &[u8]) -> Result<HeroData, FormatError> {
    let (build, image) = identify(data)?;
    hero_data_at(&image, &build.heroes)
}

fn range(what: &'static str, v: i32, lo: i32, hi: i32) -> Result<i32, FormatError> {
    if (lo..=hi).contains(&v) {
        Ok(v)
    } else {
        Err(FormatError::OutOfRange {
            what,
            value: u64::from(v.unsigned_abs()),
        })
    }
}

/// Reads the tables described by `t` (exposed for tests with synthetic images).
///
/// # Errors
/// As [`hero_data`], without the build check.
pub fn hero_data_at(image: &PeImage<'_>, t: &HeroTables) -> Result<HeroData, FormatError> {
    let at = |va: u32, what: &'static str| {
        image.file_offset(va).ok_or(FormatError::OutOfRange {
            what,
            value: u64::from(va),
        })
    };
    let i32_at = |off: usize| le::i32_at(image.data, off, "hero table");
    let base = at(t.hero_va, "hero table address")?;
    let mut heroes = Vec::with_capacity(t.hero_count.min(1024));
    for i in 0..t.hero_count {
        let r = base + i * HERO_RECORD_SIZE;
        let byte = |o: usize| {
            image
                .data
                .get(r + o)
                .copied()
                .ok_or(FormatError::Truncated {
                    what: "hero record",
                    offset: r,
                })
        };
        let skill = |o: usize| -> Result<(i32, i32), FormatError> {
            Ok((
                range("hero skill", i32_at(r + o)?, -1, 63)?,
                range("hero skill level", i32_at(r + o + 4)?, 0, 3)?,
            ))
        };
        let creature = |o: usize| range("hero creature", i32_at(r + o)?, -1, 1023);
        heroes.push(HeroRecord {
            class: range("hero class", i32_at(r + 8)?, 0, 63)?,
            skills: [skill(0x0C)?, skill(0x14)?],
            spell_book: byte(0x1C)? != 0,
            spell: range("hero spell", i32_at(r + 0x20)?, -1, 1023)?,
            creatures: [creature(0x24)?, creature(0x28)?, creature(0x2C)?],
            roe: byte(0x38)? != 0,
            expansions: byte(0x39)? != 0,
            campaign: byte(0x3A)? != 0,
        });
        // The whole record must be present.
        byte(HERO_RECORD_SIZE - 1)?;
    }
    let base = at(t.specialty_va, "specialty table address")?;
    let mut specialties = Vec::with_capacity(t.hero_count.min(1024));
    for i in 0..t.hero_count {
        let r = base + i * SPECIALTY_RECORD_SIZE;
        let v = |k: usize| le::i32_at(image.data, r + 4 * k, "specialty record");
        specialties.push(SpecialtyRecord {
            kind: range("specialty kind", v(0)?, -1, 7)?,
            id: v(1)?,
            attack: v(2)?,
            defense: v(3)?,
            damage: v(4)?,
            upgrades: [v(5)?, v(6)?],
        });
    }
    let base = at(t.spell_va, "spell table address")?;
    let mut spell_types = Vec::with_capacity(t.spell_count.min(1024));
    let mut spell_flags = Vec::with_capacity(t.spell_count.min(1024));
    for i in 0..t.spell_count {
        let r = base + i * SPELL_RECORD_SIZE;
        spell_types.push(range(
            "spell type",
            le::i32_at(image.data, r, "spell record")?,
            -1,
            1,
        )?);
        spell_flags.push(le::u32_at(image.data, r + 0xC, "spell record")?);
    }
    let mut skill_values = Vec::with_capacity(t.skill_values.len());
    for table in t.skill_values {
        let base = at(table.va, "skill table address")?;
        let mut values = [SkillValue::Integer(0); 4];
        for (k, v) in values.iter_mut().enumerate() {
            let bits = le::u32_at(image.data, base + 4 * k, "skill table")?;
            *v = if table.float {
                SkillValue::Float(f32::from_bits(bits))
            } else {
                SkillValue::Integer(i32::from_le_bytes(bits.to_le_bytes()))
            };
        }
        skill_values.push((table.skill, values));
    }
    let growth = le::u32_at(
        image.data,
        at(t.specialty_growth_va, "specialty growth address")?,
        "specialty growth",
    )?;
    let base = at(t.artifact_primary_va, "artifact table address")?;
    let mut artifact_primary = Vec::with_capacity(t.artifact_count.min(1024));
    for i in 0..t.artifact_count {
        let r = base + 4 * i;
        let bytes = image.data.get(r..r + 4).ok_or(FormatError::Truncated {
            what: "artifact primary skills",
            offset: r,
        })?;
        artifact_primary.push([0, 1, 2, 3].map(|k| i8::from_le_bytes([bytes[k]])));
    }
    Ok(HeroData {
        heroes,
        specialties,
        spell_types,
        spell_flags,
        skill_values,
        specialty_growth: f32::from_bits(growth),
        artifact_primary,
    })
}

/// Builds a minimal PE32 image with one section that holds `payload` at RVA `0x1000`, image base
/// `0x400000` (tests only; the layout is the standard PE layout, not game data).
#[must_use]
pub fn build_test_image(timestamp: u32, payload: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; 0x200];
    out[0..2].copy_from_slice(b"MZ");
    out[0x3C..0x40].copy_from_slice(&0x80u32.to_le_bytes());
    out[0x80..0x84].copy_from_slice(b"PE\0\0");
    let coff = 0x84;
    out[coff..coff + 2].copy_from_slice(&0x14Cu16.to_le_bytes());
    out[coff + 2..coff + 4].copy_from_slice(&1u16.to_le_bytes());
    out[coff + 4..coff + 8].copy_from_slice(&timestamp.to_le_bytes());
    out[coff + 16..coff + 18].copy_from_slice(&0xE0u16.to_le_bytes());
    let opt = coff + 20;
    out[opt..opt + 2].copy_from_slice(&0x10Bu16.to_le_bytes());
    out[opt + 28..opt + 32].copy_from_slice(&0x0040_0000u32.to_le_bytes());
    let sec = opt + 0xE0;
    out[sec..sec + 5].copy_from_slice(b".data");
    let raw_size = u32::try_from(payload.len()).unwrap_or(u32::MAX);
    out[sec + 8..sec + 12].copy_from_slice(&raw_size.to_le_bytes());
    out[sec + 12..sec + 16].copy_from_slice(&0x1000u32.to_le_bytes());
    out[sec + 16..sec + 20].copy_from_slice(&raw_size.to_le_bytes());
    out[sec + 20..sec + 24].copy_from_slice(&0x200u32.to_le_bytes());
    out.extend_from_slice(payload);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A payload with strings at the start and two creature records after them.
    fn payload() -> (Vec<u8>, u32) {
        let mut p = Vec::new();
        p.extend_from_slice(b"snd\0");
        p.extend_from_slice(b"unit.def\0\0\0\0");
        let table_rva = 0x1000 + u32::try_from(p.len()).unwrap();
        for (town, level, flags) in [(0i32, 0i32, 0x10u32), (-1, 6, 0x8000_0003)] {
            let mut r = vec![0u8; CREATURE_RECORD_SIZE];
            r[0..4].copy_from_slice(&town.to_le_bytes());
            r[4..8].copy_from_slice(&level.to_le_bytes());
            r[8..12].copy_from_slice(&0x0040_1000u32.to_le_bytes());
            r[12..16].copy_from_slice(&0x0040_1004u32.to_le_bytes());
            r[16..20].copy_from_slice(&flags.to_le_bytes());
            p.extend_from_slice(&r);
        }
        (p, 0x0040_0000 + table_rva)
    }

    #[test]
    fn reads_records_from_a_synthetic_image() {
        let (p, va) = payload();
        let img = build_test_image(1, &p);
        let pe = PeImage::parse(&img).unwrap();
        assert_eq!(pe.timestamp(), 1);
        let recs = creature_table_at(&pe, va, 2).unwrap();
        assert_eq!(recs.len(), 2);
        assert_eq!(
            recs[0],
            CreatureRecord {
                id: 0,
                town: 0,
                level: 0,
                sound: "snd".into(),
                def_name: "unit.def".into(),
                flags: 0x10
            }
        );
        assert_eq!(
            (recs[1].town, recs[1].level, recs[1].flags),
            (-1, 6, 0x8000_0003)
        );
        // Reading past the section is an error, not a panic.
        assert!(creature_table_at(&pe, va, 3).is_err());
        assert!(creature_table_at(&pe, 0x10, 1).is_err());
    }

    /// A payload with two usual obstacle types, two absolute obstacles (one with a full hex
    /// list) and three boats' hexes, from RVA 0x1000.
    fn obstacle_payload() -> (Vec<u8>, ObstacleTables) {
        let mut p = Vec::new();
        for (terrains, fields, height, width, hexes) in
            [(0x63u16, 1u16, 2u8, 3u8, &[0i8, 1, 2][..]), (0, 0x200, 2, 4, &[0, 1, -14, -15])]
        {
            let mut r = vec![0u8; OBSTACLE_TYPE_SIZE];
            r[0..2].copy_from_slice(&terrains.to_le_bytes());
            r[2..4].copy_from_slice(&fields.to_le_bytes());
            r[4] = height;
            r[5] = width;
            r[6] = u8::try_from(hexes.len()).unwrap();
            for (k, h) in hexes.iter().enumerate() {
                r[8 + k] = h.to_le_bytes()[0];
            }
            p.extend_from_slice(&r);
        }
        let absolute_va = 0x0040_1000 + u32::try_from(p.len()).unwrap();
        for (terrains, fields, hexes) in [
            (1u16, 0u16, vec![80u16, 94, 95, 0xFFFF]),
            (0, 0x20, (100..126).collect::<Vec<_>>()),
        ] {
            let mut r = vec![0u8; ABSOLUTE_OBSTACLE_SIZE];
            r[0..2].copy_from_slice(&terrains.to_le_bytes());
            r[2..4].copy_from_slice(&fields.to_le_bytes());
            for (k, h) in hexes.iter().enumerate() {
                r[0xC + 2 * k..0xE + 2 * k].copy_from_slice(&h.to_le_bytes());
            }
            p.extend_from_slice(&r);
        }
        let boats_va = 0x0040_1000 + u32::try_from(p.len()).unwrap();
        for hex in [6i32, 7, 180] {
            p.extend_from_slice(&hex.to_le_bytes());
        }
        let tables = ObstacleTables {
            types_va: 0x0040_1000,
            type_count: 2,
            absolute_va,
            absolute_count: 2,
            boats_va,
            boats_count: 3,
        };
        (p, tables)
    }

    #[test]
    fn reads_the_obstacle_tables_from_a_synthetic_image() {
        let (p, t) = obstacle_payload();
        let img = build_test_image(1, &p);
        let pe = PeImage::parse(&img).unwrap();
        let data = obstacle_data_at(&pe, &t).unwrap();
        assert_eq!(
            data.types[1],
            ObstacleTypeRecord {
                terrains: 0,
                fields: 0x200,
                height: 2,
                width: 4,
                blocked: vec![0, 1, -14, -15],
            }
        );
        assert_eq!((data.types[0].terrains, data.types[0].fields), (0x63, 1));
        assert_eq!(data.absolute[0].blocked, [80, 94, 95]);
        assert_eq!(data.absolute[1].blocked.len(), 26);
        assert_eq!(data.absolute[1].fields, 0x20);
        assert_eq!(data.boats, [6, 7, 180]);
        // Out of range: a hex count of 0 or 9, a negative boats hex, an unmapped table.
        for (at, value) in [(6, 0u8), (6, 9)] {
            let mut bad = p.clone();
            bad[at] = value;
            let img = build_test_image(1, &bad);
            let pe = PeImage::parse(&img).unwrap();
            assert!(obstacle_data_at(&pe, &t).is_err());
        }
        let mut bad = p.clone();
        let at = usize::try_from(t.boats_va - 0x0040_1000).unwrap();
        bad[at..at + 4].copy_from_slice(&(-1i32).to_le_bytes());
        let img = build_test_image(1, &bad);
        assert!(obstacle_data_at(&PeImage::parse(&img).unwrap(), &t).is_err());
        for t in [
            ObstacleTables { types_va: 0x10, ..t },
            ObstacleTables { absolute_va: 0x10, ..t },
            ObstacleTables { boats_va: 0x10, ..t },
            ObstacleTables { type_count: 99, ..t },
        ] {
            assert!(obstacle_data_at(&pe, &t).is_err());
        }
    }

    /// A payload with one hero record, one specialty record, two spell records, two skill
    /// tables and the growth constant, at known offsets from RVA 0x1000.
    fn hero_payload() -> (Vec<u8>, HeroTables) {
        const SKILLS: &[SkillValueTable] = &[
            SkillValueTable {
                skill: 9,
                va: 0x0040_1000 + 92 + 40 + 2 * 0x88,
                float: false,
            },
            SkillValueTable {
                skill: 1,
                va: 0x0040_1000 + 92 + 40 + 2 * 0x88 + 16,
                float: true,
            },
        ];
        let mut p = vec![0u8; HERO_RECORD_SIZE];
        let put = |p: &mut Vec<u8>, at: usize, v: i32| {
            p[at..at + 4].copy_from_slice(&v.to_le_bytes());
        };
        put(&mut p, 8, 3); // class
        put(&mut p, 0x0C, 6);
        put(&mut p, 0x10, 1);
        put(&mut p, 0x14, -1);
        p[0x1C] = 1;
        put(&mut p, 0x20, 46);
        put(&mut p, 0x24, 0);
        put(&mut p, 0x28, 2);
        put(&mut p, 0x2C, 146);
        p[0x38] = 1;
        p[0x39] = 1;
        let mut spec = vec![0u8; SPECIALTY_RECORD_SIZE];
        for (k, v) in [4i32, 96, 5, 5, 10, 0, 0].into_iter().enumerate() {
            put(&mut spec, 4 * k, v);
        }
        p.extend_from_slice(&spec);
        for (t, flags) in [(-1i32, 0x8211), (1, 0x4_4845)] {
            let mut r = vec![0u8; SPELL_RECORD_SIZE];
            put(&mut r, 0, t);
            put(&mut r, 0xC, flags);
            p.extend_from_slice(&r);
        }
        for v in [0i32, 1, 2, 3] {
            p.extend_from_slice(&v.to_le_bytes());
        }
        for v in [0.0f32, 0.1, 0.25, 0.5] {
            p.extend_from_slice(&v.to_bits().to_le_bytes());
        }
        p.extend_from_slice(&0.05f32.to_bits().to_le_bytes());
        p.extend_from_slice(&[2, 0, 0, 0, 0xFD, 12, 0, 0]);
        let tables = HeroTables {
            hero_va: 0x0040_1000,
            hero_count: 1,
            specialty_va: 0x0040_1000 + 92,
            spell_va: 0x0040_1000 + 92 + 40,
            spell_count: 2,
            skill_values: SKILLS,
            specialty_growth_va: 0x0040_1000 + 92 + 40 + 2 * 0x88 + 32,
            artifact_primary_va: 0x0040_1000 + 92 + 40 + 2 * 0x88 + 36,
            artifact_count: 2,
        };
        (p, tables)
    }

    #[test]
    fn reads_hero_tables_from_a_synthetic_image() {
        let (p, t) = hero_payload();
        let img = build_test_image(1, &p);
        let pe = PeImage::parse(&img).unwrap();
        let d = hero_data_at(&pe, &t).unwrap();
        assert_eq!(
            d.heroes,
            vec![HeroRecord {
                class: 3,
                skills: [(6, 1), (-1, 0)],
                spell_book: true,
                spell: 46,
                creatures: [0, 2, 146],
                roe: true,
                expansions: true,
                campaign: false,
            }]
        );
        assert_eq!(
            d.specialties[0],
            SpecialtyRecord {
                kind: 4,
                id: 96,
                attack: 5,
                defense: 5,
                damage: 10,
                upgrades: [0, 0]
            }
        );
        assert_eq!(d.spell_types, vec![-1, 1]);
        assert_eq!(d.spell_flags, vec![0x8211, 0x4_4845]);
        assert_eq!(
            d.skill_values[0],
            (9, [0, 1, 2, 3].map(SkillValue::Integer))
        );
        assert_eq!(
            d.skill_values[1],
            (1, [0.0, 0.1, 0.25, 0.5].map(SkillValue::Float))
        );
        assert_eq!(d.specialty_growth, 0.05);
        assert_eq!(d.artifact_primary, vec![[2, 0, 0, 0], [-3, 12, 0, 0]]);
        assert!(matches!(hero_data(&img), Err(FormatError::UnknownBuild(_))));
    }

    #[test]
    fn hero_table_errors() {
        let (p, t) = hero_payload();
        let img = build_test_image(1, &p);
        let pe = PeImage::parse(&img).unwrap();
        // Unmapped addresses and truncated tables.
        for bad in [
            HeroTables { hero_va: 0x10, ..t },
            HeroTables {
                specialty_va: 0x10,
                ..t
            },
            HeroTables {
                spell_va: 0x10,
                ..t
            },
            HeroTables {
                specialty_growth_va: 0x10,
                ..t
            },
            HeroTables { hero_count: 9, ..t },
            HeroTables {
                artifact_primary_va: 0x10,
                ..t
            },
            HeroTables {
                artifact_count: 3,
                ..t
            },
            HeroTables {
                spell_count: 9,
                ..t
            },
            HeroTables {
                specialty_growth_va: t.artifact_primary_va + 6,
                ..t
            },
            HeroTables {
                skill_values: &[SkillValueTable {
                    skill: 0,
                    va: 0x10,
                    float: false,
                }],
                ..t
            },
            HeroTables {
                skill_values: &[SkillValueTable {
                    skill: 0,
                    va: 0x0040_1000 + 92 + 40 + 2 * 0x88 + 36,
                    float: false,
                }],
                ..t
            },
        ] {
            assert!(hero_data_at(&pe, &bad).is_err(), "{bad:?}");
        }
        // Out-of-range values.
        let at_spec = HERO_RECORD_SIZE;
        let at_spell = HERO_RECORD_SIZE + SPECIALTY_RECORD_SIZE;
        for (offset, value) in [
            (8, 64),
            (0x0C, 64),
            (0x10, 4),
            (0x14, -2),
            (0x18, -1),
            (0x20, 1024),
            (0x24, -2),
            (0x28, 1024),
            (0x2C, -5),
            (at_spec, 8),
            (at_spell, 2),
        ] {
            let mut q = p.clone();
            q[offset..offset + 4].copy_from_slice(&i32::to_le_bytes(value));
            let img = build_test_image(1, &q);
            let pe = PeImage::parse(&img).unwrap();
            assert!(
                matches!(hero_data_at(&pe, &t), Err(FormatError::OutOfRange { .. })),
                "{offset:#x} = {value}"
            );
        }
        // A hero record cut short by the end of the section.
        let img = build_test_image(1, &p[..60]);
        let pe = PeImage::parse(&img).unwrap();
        assert!(hero_data_at(&pe, &t).is_err());
        let img = build_test_image(1, &p[..80]);
        let pe = PeImage::parse(&img).unwrap();
        assert!(hero_data_at(&pe, &t).is_err());
    }

    #[test]
    fn unknown_builds_are_rejected() {
        let (p, _) = payload();
        let img = build_test_image(1, &p);
        assert!(matches!(identify(&img), Err(FormatError::UnknownBuild(_))));
        assert!(matches!(
            creature_table(&img),
            Err(FormatError::UnknownBuild(_))
        ));
    }

    #[test]
    fn out_of_range_values_and_bad_strings() {
        let (mut p, va) = payload();
        let rec = p.len() - 2 * CREATURE_RECORD_SIZE;
        p[rec..rec + 4].copy_from_slice(&9i32.to_le_bytes());
        let img = build_test_image(1, &p);
        let pe = PeImage::parse(&img).unwrap();
        assert!(matches!(
            creature_table_at(&pe, va, 1),
            Err(FormatError::OutOfRange {
                what: "creature town",
                ..
            })
        ));

        let (mut p, va) = payload();
        p[rec + 4..rec + 8].copy_from_slice(&7i32.to_le_bytes());
        let img = build_test_image(1, &p);
        let pe = PeImage::parse(&img).unwrap();
        assert!(creature_table_at(&pe, va, 1).is_err());

        let (mut p, _) = payload();
        p[0] = 0x01; // unprintable string
        let img = build_test_image(1, &p);
        let pe = PeImage::parse(&img).unwrap();
        assert!(pe.c_string(0x0040_1000).is_err());
        let (mut p, _) = payload();
        p[3] = b'x'; // no NUL until the next string: still terminated, so check a long run
        p.splice(0..0, std::iter::repeat_n(b'a', 300));
        let img = build_test_image(1, &p);
        let pe = PeImage::parse(&img).unwrap();
        assert!(pe.c_string(0x0040_1000).is_err());
    }

    #[test]
    fn header_errors() {
        assert!(matches!(
            PeImage::parse(b"ZZ"),
            Err(FormatError::BadMagic(_))
        ));
        let (p, _) = payload();
        let mut img = build_test_image(1, &p);
        img[0x80] = b'X';
        assert!(matches!(
            PeImage::parse(&img),
            Err(FormatError::BadMagic(_))
        ));
        let mut img = build_test_image(1, &p);
        img[0x84 + 20] = 0x0B;
        img[0x84 + 21] = 0x02; // PE32+ magic
        assert!(matches!(
            PeImage::parse(&img),
            Err(FormatError::BadMagic(_))
        ));
        let mut img = build_test_image(1, &p);
        img[0x86..0x88].copy_from_slice(&200u16.to_le_bytes());
        assert!(matches!(
            PeImage::parse(&img),
            Err(FormatError::OutOfRange { .. })
        ));
        let img = build_test_image(1, &p);
        assert!(PeImage::parse(&img[..0x90]).is_err());
    }
}
