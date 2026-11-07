//! `h3-pack-adventure`'s check against the original data (`cargo xtask verify
//! --with-original-data`; E08 T93a): compiles the original packs (RoE to SoD) and checks their
//! dwellings against the installation's text tables and executable. Each dwelling type is matched
//! to its original class and subclass by name (class 17 subclass `s` is `CrGen1.txt`'s row `s`,
//! class 20 `CrGen4.txt`'s; the duplicate name in pack order), and each row must have its type and
//! each type its row. Their footprints must be the class and subclass's last template in
//! `Objects.txt`, converted as the map import converts them (blocked: the passability's `0`s;
//! visitable: the visitability's `1`s; bit = character index); their creatures the executable's
//! (`dwelling_creatures` for class 17, `dwelling4_creatures` for class 20, through the core packs'
//! original creature ids). And the random dwellings' tables must equal the original's derivation
//! (design note `docs/epics/E08-design.md` §2; survey `docs/rules/surveys/E08-dwellings.md` §3.2
//! steps 3–4): a faction's level is the dwelling of its town's unupgraded creature of that level,
//! the Golem Factory for the Stone Golem and otherwise the last class-17 subclass that offers it.
//! Since E08 T103a the teleporters (design note §3; survey `docs/rules/surveys/E08-teleporters.md`
//! §1, §2): each monolith subclass of classes 43 to 45 that SoD's `Objects.txt` lists has exactly
//! one monolith type of its way with the subclass as its channel, in `base` when AB's
//! `Objects.txt` lists it too and in `sod` otherwise, named `ObjNames.txt`'s row with the
//! channel; classes 103 and 111 their one base type each, named by their rows; each teleporter
//! type entered and visited from above as the executable's class lists say
//! (`object_classes_not_entered`, `object_classes_from_above`). Fails with the mismatches of the
//! first check that has any.
//!
//! `cargo run -p h3-pack-adventure --example original_data -- original-data/install`
//!
//! The text tables are read from `<install>/../extracted/H3ab_bmp` (AB's) and, for the
//! teleporters, `<install>/../extracted/H3bitmap` (SoD's). The installation copy is only read.
//! Without its files the check is skipped with a note.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_adventure::{AdventureRuleset, MonolithWay, ObjectKind, ObjectType, Visit};
use h3_content::{CreatureTypeId, Ruleset};
use h3_pack_adventure::{ORIGINAL_PACKS, compile};

/// The executable's table of class 17's creatures, by subclass (its virtual address).
const DWELLING_CREATURES: u32 = 0x0063_D570;
/// The executable's table of class 20's four creatures, by subclass.
const DWELLING4_CREATURES: u32 = 0x0067_7938;
/// The executable's table of the towns' creatures, `faction × 14 + level` (0 to 6 unupgraded).
const TOWN_CREATURES: u32 = 0x0067_47B4;
/// The executable's factions in order, as core keys.
const FACTIONS: [&str; 9] = [
    "base:castle",
    "base:rampart",
    "base:tower",
    "base:inferno",
    "base:necropolis",
    "base:dungeon",
    "base:stronghold",
    "base:fortress",
    "ab:conflux",
];
/// The original Stone Golem, whose dwelling is class 20 subclass 1.
const STONE_GOLEM: i32 = 32;
/// The class-17 subclasses the derivation never yields.
const NEVER: [u32; 5] = [7, 13, 16, 47, 51];
/// The executable's 42 classes (u32) a hero visits from the neighbouring tile without entering.
const OBJECT_CLASSES_NOT_ENTERED: u32 = 0x0063_AA58;
/// The executable's 42 classes (u32) whose objects may be entered from above.
const OBJECT_CLASSES_FROM_ABOVE: u32 = 0x0063_AB00;
/// The monoliths' classes, their ways and their types' key prefixes (E08 T103a).
const MONOLITHS: [(u32, MonolithWay, &str); 3] = [
    (43, MonolithWay::Entrance, "monolith_entrance"),
    (44, MonolithWay::Exit, "monolith_exit"),
    (45, MonolithWay::TwoWay, "monolith_two_way"),
];

/// An original object class and subclass.
type Class = (u32, u32);

fn main() -> ExitCode {
    let Some(install) = std::env::args().nth(1) else {
        eprintln!("usage: original_data <installation copy>");
        return ExitCode::FAILURE;
    };
    let install = Path::new(&install);
    let texts = install.join("..").join("extracted").join("H3ab_bmp");
    let sod_texts = install.join("..").join("extracted").join("H3bitmap");
    let files = [
        install.join("Heroes3.exe"),
        texts.join("Objects.txt"),
        texts.join("CrGen1.txt"),
        texts.join("CrGen4.txt"),
        sod_texts.join("Objects.txt"),
        sod_texts.join("ObjNames.txt"),
    ];
    if let Some(missing) = files.iter().find(|f| !f.is_file()) {
        println!(
            "h3-pack-adventure: skipped ({} is not there)",
            missing.display()
        );
        return ExitCode::SUCCESS;
    }
    match check(&files) {
        Ok(summary) => {
            println!("h3-pack-adventure: {summary}");
            ExitCode::SUCCESS
        }
        Err(mismatches) => {
            for m in &mismatches {
                eprintln!("h3-pack-adventure: {m}");
            }
            ExitCode::FAILURE
        }
    }
}

/// Runs the checks; the summary, or the first check's mismatches.
fn check(files: &[PathBuf; 6]) -> Result<String, Vec<String>> {
    let read = |path: &PathBuf| {
        std::fs::read(path).map_err(|e| vec![format!("{} does not read: {e}", path.display())])
    };
    let exe = Exe::parse(read(&files[0])?)
        .ok_or_else(|| vec![format!("{} is not a PE32 image", files[0].display())])?;
    let lines = |path: &PathBuf| -> Result<Vec<String>, Vec<String>> {
        let text = String::from_utf8(read(path)?)
            .map_err(|_| vec![format!("{} is not text", path.display())])?;
        Ok(text
            .split('\n')
            .map(|l| l.trim_end_matches('\r').to_owned())
            .collect())
    };
    let objects = lines(&files[1])?;
    let rows = [(17, lines(&files[2])?), (20, lines(&files[3])?)].map(|(class, mut rows)| {
        while rows.last().is_some_and(String::is_empty) {
            rows.pop();
        }
        (class, rows)
    });
    let table = |va: u32, n: usize, what: &str| {
        exe.i32s(va, n)
            .ok_or_else(|| vec![format!("{what} ({va:#x}) is not in the executable's image")])
    };
    let dwelling_creatures = table(DWELLING_CREATURES, rows[0].1.len(), "dwelling_creatures")?;
    let dwelling4_creatures = table(DWELLING4_CREATURES, 4 * rows[1].1.len(), "dwelling4")?;
    let town_creatures = table(TOWN_CREATURES, 14 * FACTIONS.len(), "town_creatures")?;

    let (core, part) = compile(&ORIGINAL_PACKS, "base:faithful")
        .map_err(|e| vec![format!("the original packs do not compile: {e:?}")])?;
    let creatures = original_creatures(&core)?;

    // 1. The types by name.
    let types = part
        .object_types()
        .iter()
        .filter(|t| matches!(t.kind, ObjectKind::Dwelling { .. }))
        .collect::<Vec<_>>();
    let mut by_name = BTreeMap::<&str, Vec<Class>>::new();
    for (class, rows) in &rows {
        for (row, name) in (0..).zip(rows) {
            by_name.entry(name).or_default().push((*class, row));
        }
    }
    let mut mismatches = Vec::new();
    let mut matched = BTreeMap::<Class, &ObjectType>::new();
    for ty in &types {
        match by_name.get_mut(ty.name.as_str()) {
            Some(left) if !left.is_empty() => {
                matched.insert(left.remove(0), ty);
            }
            _ => mismatches.push(format!(
                "the dwelling type {} ({:?}) has no row in CrGen1.txt or CrGen4.txt left",
                ty.key.as_str(),
                ty.name
            )),
        }
    }
    for (name, left) in &by_name {
        for (class, sub) in left {
            mismatches.push(format!(
                "class {class} subclass {sub} ({name:?}) has no dwelling type"
            ));
        }
    }
    done(mismatches)?;

    // 2. The footprints.
    let mut templates = BTreeMap::<Class, (&str, &str)>::new();
    for line in &objects {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        if let [_, passable, visitable, _, _, class, sub, ..] = fields[..]
            && let (Ok(class), Ok(sub)) = (class.parse(), sub.parse())
        {
            templates.insert((class, sub), (passable, visitable));
        }
    }
    let mask = |chars: &str, set: u8| {
        (0..)
            .zip(chars.bytes())
            .filter(|&(_, c)| c == set)
            .fold(0u64, |acc, (i, _)| acc | 1 << i)
    };
    let mut mismatches = Vec::new();
    for (&(class, sub), ty) in &matched {
        let ObjectKind::Dwelling {
            blocked, visitable, ..
        } = &ty.kind
        else {
            unreachable!("only dwellings are matched");
        };
        let Some(&(passable, visits)) = templates.get(&(class, sub)) else {
            mismatches.push(format!(
                "class {class} subclass {sub} ({}) has no template in Objects.txt",
                ty.key.as_str()
            ));
            continue;
        };
        let original = (mask(passable, b'0'), mask(visits, b'1'));
        let pack = (blocked.bits(), visitable.bits());
        if pack != original {
            mismatches.push(format!(
                "class {class} subclass {sub} ({}): blocked, visitable {:#x}, {:#x} in the pack, \
                 {:#x}, {:#x} in Objects.txt",
                ty.key.as_str(),
                pack.0,
                pack.1,
                original.0,
                original.1
            ));
        }
    }
    done(mismatches)?;

    // 3. The creatures.
    let key = |id: i32| match creatures.get(&id) {
        Some(&c) => creature_key(&core, c),
        None => format!("original id {id}"),
    };
    let mut mismatches = Vec::new();
    for (&(class, sub), ty) in &matched {
        let ObjectKind::Dwelling {
            creatures: offered, ..
        } = &ty.kind
        else {
            unreachable!("only dwellings are matched");
        };
        let s = usize::try_from(sub).expect("a row index fits");
        let original = if class == 17 {
            &dwelling_creatures[s..=s]
        } else {
            &dwelling4_creatures[4 * s..4 * s + 4]
        };
        let expected = original
            .iter()
            .map(|id| creatures.get(id).copied())
            .collect::<Option<Vec<_>>>();
        if expected.as_ref() != Some(offered) {
            mismatches.push(format!(
                "class {class} subclass {sub} ({}): creatures {:?} in the pack, {:?} in the \
                 executable",
                ty.key.as_str(),
                offered
                    .iter()
                    .map(|&c| creature_key(&core, c))
                    .collect::<Vec<_>>(),
                original.iter().map(|&id| key(id)).collect::<Vec<_>>()
            ));
        }
    }
    done(mismatches)?;

    // 4. The tables against the original's derivation.
    let mut mismatches = Vec::new();
    let mut outcomes = BTreeSet::<Class>::new();
    for (f, faction) in FACTIONS.iter().enumerate() {
        let Some(id) = core.faction_by_key(faction) else {
            mismatches.push(format!("the core has no faction {faction}"));
            continue;
        };
        let Some(row) = part.random_dwelling(id) else {
            mismatches.push(format!("{faction} has no random dwelling table"));
            continue;
        };
        for (level0, &actual) in row.iter().enumerate() {
            let c = town_creatures[f * 14 + level0];
            let expected = if c == STONE_GOLEM {
                Some((20, 1))
            } else {
                (0..)
                    .zip(&dwelling_creatures)
                    .filter(|&(_, &d)| d == c)
                    .last()
                    .map(|(s, _)| (17, s))
            };
            let Some(expected) = expected else {
                mismatches.push(format!(
                    "{faction} level {}: no dwelling offers {}",
                    level0 + 1,
                    key(c)
                ));
                continue;
            };
            outcomes.insert(expected);
            let actual = type_key(&part, actual);
            let wanted = matched.get(&expected).map(|t| t.key.as_str());
            if wanted != Some(actual.as_str()) {
                mismatches.push(format!(
                    "{faction} level {}: {actual} in the table, class {} subclass {} ({}) by \
                     the derivation",
                    level0 + 1,
                    expected.0,
                    expected.1,
                    wanted.unwrap_or("no type")
                ));
            }
        }
    }
    if outcomes.len() != 63 {
        mismatches.push(format!(
            "the derivation has {} distinct outcomes, not 63",
            outcomes.len()
        ));
    }
    for sub in NEVER {
        if outcomes.contains(&(17, sub)) {
            mismatches.push(format!(
                "class 17 subclass {sub} is an outcome of the derivation"
            ));
        }
    }
    done(mismatches)?;

    // 5. The teleporters (E08 T103a).
    let not_entered = table(OBJECT_CLASSES_NOT_ENTERED, 42, "object_classes_not_entered")?;
    let from_above = table(OBJECT_CLASSES_FROM_ABOVE, 42, "object_classes_from_above")?;
    let names = lines(&files[5])?;
    let teleporters = teleporters(
        &part,
        [&objects, &lines(&files[4])?],
        &names,
        [&not_entered, &from_above],
    )?;

    Ok(format!(
        "{} dwelling types match {} CrGen1 + {} CrGen4 rows, their Objects.txt footprints and \
         the executable's creatures; the {} factions' random dwelling tables match the \
         original's derivation ({} outcomes); {teleporters}",
        types.len(),
        rows[0].1.len(),
        rows[1].1.len(),
        FACTIONS.len(),
        outcomes.len()
    ))
}

/// Check 5 (E08 T103a): the teleporter types against AB's and SoD's `Objects.txt` (`objects`),
/// SoD's `ObjNames.txt` and the executable's class lists (`lists`: not entered, from above).
fn teleporters(
    part: &AdventureRuleset,
    objects: [&[String]; 2],
    names: &[String],
    lists: [&[i32]; 2],
) -> Result<String, Vec<String>> {
    let listed = |lines: &[String]| {
        lines
            .iter()
            .filter_map(|line| {
                let fields = line.split_whitespace().collect::<Vec<_>>();
                let (class, sub) = (fields.get(5)?.parse().ok()?, fields.get(6)?.parse().ok()?);
                [43, 44, 45, 103, 111]
                    .contains(&class)
                    .then_some((class, sub))
            })
            .collect::<BTreeSet<Class>>()
    };
    let [ab, sod] = objects.map(listed);
    let mut mismatches = Vec::new();
    let mut checked = Vec::new();
    let row = |class: u32| names.get(class as usize).map_or("", String::as_str);
    for &(class, sub) in &sod {
        let (pack, key, kind, name) = match MONOLITHS.iter().find(|m| m.0 == class) {
            Some(&(_, way, prefix)) => (
                if ab.contains(&(class, sub)) {
                    "base"
                } else {
                    "sod"
                },
                format!("{prefix}_{sub}"),
                ObjectKind::Monolith {
                    channel: u8::try_from(sub).unwrap_or(u8::MAX),
                    way,
                },
                format!("{} {sub}", row(class)),
            ),
            // The gate and the Whirlpool: one type whatever the subclass (subclass 0 only).
            None if class == 103 => (
                "base",
                "subterranean_gate".to_owned(),
                ObjectKind::SubterraneanGate,
                row(class).to_owned(),
            ),
            None => (
                "base",
                "whirlpool".to_owned(),
                ObjectKind::Whirlpool { divisor: 2 },
                row(class).to_owned(),
            ),
        };
        let key = format!("{pack}:{key}");
        let Some(ty) = part
            .object_type_by_key(&key)
            .and_then(|id| part.object_type(id))
        else {
            mismatches.push(format!("class {class} subclass {sub} has no type {key}"));
            continue;
        };
        let class = i32::try_from(class).expect("a class fits");
        let visit = if lists[0].contains(&class) {
            Visit::Neighbour
        } else {
            Visit::Enter
        };
        let expected = (&kind, name.as_str(), visit, lists[1].contains(&class));
        if (&ty.kind, ty.name.as_str(), ty.visit, ty.from_above) != expected {
            mismatches.push(format!(
                "{key}: {:?} {:?} {:?} from above {} in the pack, {expected:?} in the original",
                ty.kind, ty.name, ty.visit, ty.from_above
            ));
        }
        checked.push(ty.key.as_str().to_owned());
    }
    // No other teleporter type, and AB's subclasses a subset of SoD's.
    for ty in part.object_types() {
        let teleporter = matches!(
            ty.kind,
            ObjectKind::Monolith { .. }
                | ObjectKind::SubterraneanGate
                | ObjectKind::Whirlpool { .. }
        );
        if teleporter && !checked.iter().any(|k| k == ty.key.as_str()) {
            mismatches.push(format!(
                "{} is a teleporter type of no listed class and subclass",
                ty.key.as_str()
            ));
        }
    }
    for (class, sub) in ab.difference(&sod) {
        mismatches.push(format!(
            "class {class} subclass {sub} is in AB's Objects.txt, not SoD's"
        ));
    }
    done(mismatches)?;
    Ok(format!(
        "{} teleporter types match SoD's {} listed teleporter classes and subclasses ({} in AB's), \
         ObjNames.txt and the executable's class lists",
        checked.len(),
        sod.len(),
        ab.len()
    ))
}

/// `Err` with the mismatches if there are any.
fn done(mismatches: Vec<String>) -> Result<(), Vec<String>> {
    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(mismatches)
    }
}

/// The creatures by original id: the core packs' `id = "…"  # original id N` lines in their
/// `[[creature]]` tables, as keys of the pack's namespace.
fn original_creatures(core: &Ruleset) -> Result<BTreeMap<i32, CreatureTypeId>, Vec<String>> {
    let mut map = BTreeMap::new();
    let mut mismatches = Vec::new();
    for (text, _) in ORIGINAL_PACKS {
        let mut section = "";
        let mut namespace = "";
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                section = line;
                continue;
            }
            let Some(rest) = line.strip_prefix("id = \"") else {
                continue;
            };
            let Some((id, comment)) = rest.split_once('"') else {
                continue;
            };
            if section == "[pack]" {
                namespace = id;
            } else if section == "[[creature]]"
                && let Some(original) = comment.trim().strip_prefix("# original id ")
            {
                let key = format!("{namespace}:{id}");
                match (original.trim().parse(), core.creature_by_key(&key)) {
                    (Ok(original), Some(c)) => {
                        map.insert(original, c);
                    }
                    _ => mismatches.push(format!("the creature {key} ({original}) does not map")),
                }
            }
        }
    }
    done(mismatches).map(|()| map)
}

/// A creature's key, for messages.
fn creature_key(core: &Ruleset, c: CreatureTypeId) -> String {
    core.creature_key(c)
        .map_or_else(|| format!("{c:?}"), |key| key.as_str().to_owned())
}

/// An object type's key, for messages.
fn type_key(part: &AdventureRuleset, id: h3_adventure::ObjectTypeId) -> String {
    part.object_type(id)
        .map_or_else(|| format!("{id:?}"), |t| t.key.as_str().to_owned())
}

/// A PE32 image, read only as far as its sections' places.
struct Exe {
    bytes: Vec<u8>,
    /// The image base.
    base: u32,
    /// Each section's virtual address, raw size and raw offset.
    sections: Vec<(u32, u32, u32)>,
}

impl Exe {
    fn parse(bytes: Vec<u8>) -> Option<Self> {
        let u16_at = |at: usize| {
            bytes
                .get(at..at + 2)
                .map(|b| u16::from_le_bytes([b[0], b[1]]))
        };
        let u32_at = |at: usize| {
            bytes
                .get(at..at + 4)
                .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        };
        let pe = usize::try_from(u32_at(0x3C)?).ok()?;
        if bytes.get(pe..pe + 4)? != b"PE\0\0" {
            return None;
        }
        let coff = pe + 4;
        let count = usize::from(u16_at(coff + 2)?);
        let optional = coff + 20;
        if u16_at(optional)? != 0x10B {
            return None;
        }
        let base = u32_at(optional + 28)?;
        let table = optional + usize::from(u16_at(coff + 16)?);
        let sections = (0..count)
            .map(|i| {
                let at = table + 40 * i;
                Some((u32_at(at + 12)?, u32_at(at + 16)?, u32_at(at + 20)?))
            })
            .collect::<Option<Vec<_>>>()?;
        Some(Self {
            bytes,
            base,
            sections,
        })
    }

    /// `n` little-endian `i32`s from the virtual address `va`, if a section's raw data holds them.
    fn i32s(&self, va: u32, n: usize) -> Option<Vec<i32>> {
        let rva = va.checked_sub(self.base)?;
        let len = u32::try_from(4 * n).ok()?;
        let &(start, _, raw) = self
            .sections
            .iter()
            .find(|&&(start, size, _)| start <= rva && rva - start + len <= size)?;
        let at = usize::try_from(raw + (rva - start)).ok()?;
        let bytes = self.bytes.get(at..at + 4 * n)?;
        Some(
            bytes
                .chunks_exact(4)
                .map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect(),
        )
    }
}
