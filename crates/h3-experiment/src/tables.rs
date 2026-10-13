//! What a spec's names resolve against: the original tables of the user's installation (read
//! only; nothing is copied into the repo, ADR-0009).

use std::path::Path;

use h3_formats::exe;
use h3_formats::lod::LodArchive;
use h3_formats::text::{TextTable, decode_windows_1252};
use h3_map::writer::{ARTIFACT_SLOTS, GROUND_OBJECTS, ObjectTemplate};

/// Object class of a fixed wandering monster (subclass = creature id).
const MONSTER_CLASS: u32 = 54;
/// Object class of a resource pile (subclass = resource id, [`RESOURCES`] order).
const PILE_CLASS: u32 = 79;
/// The group rows of `SPTRAITS.TXT`, which are not spells.
const SPELL_GROUPS: [&str; 3] = ["Adventure Spells", "Combat Spells", "Creature Abilities"];
/// Resource names in `RESTYPES.TXT` order (specialty descriptions, resource piles).
pub const RESOURCES: [&str; 7] = [
    "wood", "mercury", "ore", "sulfur", "crystal", "gems", "gold",
];

/// An artifact row: its name and the slots it may be worn in (map slot order, see
/// [`h3_map::writer::HeroArtifacts`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    /// Name (`ARTRAITS.TXT`).
    pub name: String,
    /// Allowed slots.
    pub slots: [bool; ARTIFACT_SLOTS],
}

/// The text tables [`Tables::from_tables`] reads (`H3bitmap.lod`).
#[derive(Debug, Clone, Copy)]
pub struct TextTables<'a> {
    /// `CRTRAITS.TXT`.
    pub creatures: &'a TextTable,
    /// `HOTRAITS.TXT`.
    pub heroes: &'a TextTable,
    /// `SSTRAITS.TXT`.
    pub skills: &'a TextTable,
    /// `ARTRAITS.TXT`.
    pub artifacts: &'a TextTable,
    /// `SPTRAITS.TXT`.
    pub spells: &'a TextTable,
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
    /// Spell names by id (`SPTRAITS.TXT`, without its group rows).
    pub spells: Vec<String>,
    /// Wandering-monster templates (`Objects.txt`, class 54) as (creature id, template).
    pub monsters: Vec<(u32, ObjectTemplate)>,
    /// Resource-pile templates (`Objects.txt`, class 79) as (resource id, template; E08 T16b).
    pub piles: Vec<(u32, ObjectTemplate)>,
    /// Special-ground templates (`Objects.txt`, the classes of [`GROUND_OBJECTS`]; E04c T3), in
    /// the file's order.
    pub grounds: Vec<ObjectTemplate>,
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
            &TextTables {
                creatures: &table("CRTRAITS.TXT")?,
                heroes: &table("HOTRAITS.TXT")?,
                skills: &table("SSTRAITS.TXT")?,
                artifacts: &table("ARTRAITS.TXT")?,
                spells: &table("SPTRAITS.TXT")?,
            },
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
        texts: &TextTables<'_>,
        objects_txt: &str,
        hero_classes: Vec<u8>,
        specialties: Vec<Specialty>,
    ) -> Result<Self, String> {
        let TextTables {
            creatures,
            heroes,
            skills,
            artifacts,
            spells,
        } = *texts;
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
        let templates: Vec<ObjectTemplate> = objects_txt
            .lines()
            .filter_map(|line| ObjectTemplate::from_objects_txt(line).ok())
            .collect();
        let of_class = |class: u32| {
            templates
                .iter()
                .filter(|t| t.class == class)
                .map(|t| (t.subclass, t.clone()))
                .collect::<Vec<_>>()
        };
        let (monsters, piles) = (of_class(MONSTER_CLASS), of_class(PILE_CLASS));
        let grounds = templates
            .iter()
            .filter(|t| GROUND_OBJECTS.contains(&t.class))
            .cloned()
            .collect();
        Ok(Self {
            creatures: names(creatures),
            heroes,
            hero_classes,
            specialties,
            skills: names(skills),
            artifacts,
            spells: names(spells)
                .into_iter()
                .filter(|n| !SPELL_GROUPS.contains(&n.as_str()))
                .collect(),
            monsters,
            piles,
            grounds,
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
            3 => format!("spell {}", name(&self.spells)),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A table of two header rows and these one-column names.
    fn table(names: &[&str]) -> TextTable {
        let mut records = vec![vec!["header".to_owned()], vec!["header".to_owned()]];
        records.extend(names.iter().map(|n| vec![(*n).to_owned()]));
        TextTable { records }
    }

    #[test]
    fn objects_txt_gives_the_monster_pile_and_ground_templates() {
        let mut artifact = vec!["Ballista".to_owned(), "x".to_owned()];
        artifact.extend((0..ARTIFACT_SLOTS).map(|i| if i == 4 { "x" } else { "" }.to_owned()));
        let artifacts = TextTable {
            records: vec![vec!["h".into()], vec!["h".into()], artifact],
        };
        let mask = "011111111111111111111111111111111111111111111111 \
                    100000000000000000000000000000000000000000000000 111111111 000000001";
        let objects = format!(
            "header\nAvWPike.def {mask} 54 0 2 0\nAVTwood0.def {mask} 79 0 5 0\n\
             AVTmerc0.def {mask} 79 1 5 0\nAVA0079.def {mask} 5 79 4 0\nbroken line\n\
             AVXcf0.def {mask} 222 0 0 1\nAVXcg5.def {mask} 223 0 0 1\n"
        );
        let (heroes, creatures) = (table(&["Orrin"]), table(&["Pikeman"]));
        let (skills, spells) = (table(&["Archery"]), table(&["Combat Spells", "Bless"]));
        let texts = TextTables {
            creatures: &creatures,
            heroes: &heroes,
            skills: &skills,
            artifacts: &artifacts,
            spells: &spells,
        };
        let specialty = Specialty { kind: 2, id: 1 };
        let t = Tables::from_tables(&texts, &objects, vec![0], vec![specialty]).unwrap();
        let ids = |list: &[(u32, ObjectTemplate)]| {
            list.iter()
                .map(|(id, t)| (*id, t.def.clone()))
                .collect::<Vec<_>>()
        };
        assert_eq!(ids(&t.monsters), [(0, "AvWPike.def".to_owned())]);
        assert_eq!(
            ids(&t.piles),
            [
                (0, "AVTwood0.def".to_owned()),
                (1, "AVTmerc0.def".to_owned())
            ]
        );
        let grounds: Vec<_> = t
            .grounds
            .iter()
            .map(|g| (g.def.as_str(), g.class))
            .collect();
        assert_eq!(grounds, [("AVXcf0.def", 222), ("AVXcg5.def", 223)]);
        assert_eq!(t.spells, ["Bless"]);
        assert_eq!(t.specialty_text(0), "resource mercury");
        // Slot column 4 (right ring, read in reverse order from field 2) holds the "x".
        assert!(t.artifacts[0].slots.iter().filter(|s| **s).count() == 1);
        let mismatch = Tables::from_tables(&texts, &objects, vec![], vec![specialty]);
        assert!(mismatch.unwrap_err().contains("1 heroes, 0 classes"));
    }
}
