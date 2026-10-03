//! What a spec's names resolve against: the original tables of the user's installation (read
//! only; nothing is copied into the repo, ADR-0009).

use std::path::Path;

use h3_formats::exe;
use h3_formats::h3m::{ARTIFACT_SLOTS, ObjectTemplate};
use h3_formats::lod::LodArchive;
use h3_formats::text::{TextTable, decode_windows_1252};

/// Object class of a fixed wandering monster (subclass = creature id).
const MONSTER_CLASS: u32 = 54;
/// Resource names in `RESTYPES.TXT` order (for specialty descriptions).
const RESOURCES: [&str; 7] = [
    "wood", "mercury", "ore", "sulfur", "crystal", "gems", "gold",
];

/// An artifact row: its name and the slots it may be worn in (map slot order, see
/// [`h3_formats::h3m::HeroArtifacts`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    /// Name (`ARTRAITS.TXT`).
    pub name: String,
    /// Allowed slots.
    pub slots: [bool; ARTIFACT_SLOTS],
}

/// A hero type's specialty as stored in the executable (kind, id).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Specialty {
    /// Kind (−1 none, 0 skill, 1 creature, 2 resource, 3 spell, 4 creature stats, 5 speed,
    /// 6 upgrade, 7 dragons).
    pub kind: i32,
    /// Skill, creature, resource or spell id, depending on the kind.
    pub id: i32,
}

/// Names, classes, specialties and monster templates.
#[derive(Debug, Clone, Default)]
pub struct Tables {
    /// Creature names by id (`CRTRAITS.TXT`, singular).
    pub creatures: Vec<String>,
    /// Hero names by hero type (`HOTRAITS.TXT`).
    pub heroes: Vec<String>,
    /// Hero class by hero type (executable).
    pub hero_classes: Vec<u8>,
    /// Specialty by hero type (executable).
    pub specialties: Vec<Specialty>,
    /// Secondary skill names by id (`SSTRAITS.TXT`).
    pub skills: Vec<String>,
    /// Artifacts by id (`ARTRAITS.TXT`).
    pub artifacts: Vec<Artifact>,
    /// Wandering-monster templates (`Objects.txt`, class 54) as (creature id, template).
    pub monsters: Vec<(u32, ObjectTemplate)>,
}

impl Tables {
    /// Reads the tables from an installation or its copy (`Heroes3.exe`, `Data/H3bitmap.lod`).
    ///
    /// # Errors
    /// Missing files, an unknown executable build, malformed archives or tables.
    pub fn read(install: &Path) -> Result<Self, String> {
        let read = |rel: &str| {
            let path = install.join(rel);
            std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))
        };
        let exe_data = read("Heroes3.exe")?;
        let data = exe::hero_data(&exe_data).map_err(|e| format!("Heroes3.exe: {e}"))?;
        let hero_classes = data
            .heroes
            .iter()
            .enumerate()
            .map(|(id, h)| {
                u8::try_from(h.class)
                    .map_err(|_| format!("hero {id}: class {} out of range", h.class))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let specialties = data
            .specialties
            .iter()
            .map(|s| Specialty {
                kind: s.kind,
                id: s.id,
            })
            .collect();
        let lod_data = read("Data/H3bitmap.lod")?;
        let lod = LodArchive::parse(&lod_data).map_err(|e| format!("H3bitmap.lod: {e}"))?;
        let entry = |name: &str| -> Result<Vec<u8>, String> {
            lod.read_by_name(name)
                .map_err(|e| format!("H3bitmap.lod/{name}: {e}"))?
                .ok_or_else(|| format!("H3bitmap.lod: no {name}"))
        };
        let table = |name: &str| -> Result<TextTable, String> {
            TextTable::parse(&entry(name)?).map_err(|e| format!("{name}: {e}"))
        };
        Self::from_tables(
            &table("CRTRAITS.TXT")?,
            &table("HOTRAITS.TXT")?,
            &table("SSTRAITS.TXT")?,
            &table("ARTRAITS.TXT")?,
            &decode_windows_1252(&entry("OBJECTS.TXT")?),
            hero_classes,
            specialties,
        )
    }

    /// Builds the tables from parsed text tables and the executable's hero data.
    ///
    /// # Errors
    /// Missing names or slot columns, or class/specialty lists that do not match the heroes.
    pub fn from_tables(
        creatures: &TextTable,
        heroes: &TextTable,
        skills: &TextTable,
        artifacts: &TextTable,
        objects_txt: &str,
        hero_classes: Vec<u8>,
        specialties: Vec<Specialty>,
    ) -> Result<Self, String> {
        let heroes = names(heroes);
        if heroes.len() != hero_classes.len() || heroes.len() != specialties.len() {
            return Err(format!(
                "{} heroes, {} classes, {} specialties",
                heroes.len(),
                hero_classes.len(),
                specialties.len()
            ));
        }
        let artifacts = rows(artifacts)
            .map(|r| {
                // Fields 2..=20: Spell Book, War Machine 4..1, Misc 5..1, Feet, Left Ring,
                // Right Ring, Torso, Left Hand, Right Hand, Neck, Shoulders, Head.
                const COLUMN_SLOTS: [usize; ARTIFACT_SLOTS] = [
                    17, 16, 15, 14, 13, 18, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
                ];
                let mut slots = [false; ARTIFACT_SLOTS];
                for (i, &slot) in COLUMN_SLOTS.iter().enumerate() {
                    let cell = r
                        .get(2 + i)
                        .ok_or_else(|| format!("{}: no slot columns", r[0]))?;
                    slots[slot] = cell.trim().eq_ignore_ascii_case("x");
                }
                Ok(Artifact {
                    name: r[0].trim().to_owned(),
                    slots,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let monsters = objects_txt
            .lines()
            .filter_map(|line| ObjectTemplate::from_objects_txt(line).ok())
            .filter(|t| t.class == MONSTER_CLASS)
            .map(|t| (t.subclass, t))
            .collect();
        Ok(Self {
            creatures: names(creatures),
            heroes,
            hero_classes,
            specialties,
            skills: names(skills),
            artifacts,
            monsters,
        })
    }

    /// Describes a hero type's specialty in words ("resource gold", "skill Logistics", …).
    pub fn specialty_text(&self, hero_type: usize) -> String {
        let Some(s) = self.specialties.get(hero_type) else {
            return "unknown".into();
        };
        let name = |list: &[String]| {
            usize::try_from(s.id)
                .ok()
                .and_then(|i| list.get(i).cloned())
                .unwrap_or_else(|| format!("#{}", s.id))
        };
        match s.kind {
            -1 => "none".into(),
            0 => format!("skill {}", name(&self.skills)),
            1 => format!("creature {}", name(&self.creatures)),
            2 => format!(
                "resource {}",
                usize::try_from(s.id)
                    .ok()
                    .and_then(|i| RESOURCES.get(i))
                    .map_or_else(|| format!("#{}", s.id), |r| (*r).to_owned())
            ),
            3 => format!("spell #{}", s.id),
            4 => format!("creature stats {}", name(&self.creatures)),
            5 => "speed".into(),
            6 => "upgrade".into(),
            7 => "dragons".into(),
            k => format!("kind {k}"),
        }
    }
}

/// Data rows of a table: the two header rows and blank separator rows dropped.
fn rows(table: &TextTable) -> impl Iterator<Item = &Vec<String>> {
    table
        .records
        .iter()
        .skip(2)
        .filter(|r| !TextTable::is_blank(r))
}

/// First column of the data rows.
fn names(table: &TextTable) -> Vec<String> {
    rows(table).map(|r| r[0].trim().to_owned()).collect()
}
