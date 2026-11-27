//! `h3-harvest`'s check against the original data (`cargo xtask verify --with-original-data`; E14
//! T4b): the map generator's data that the rmg packs hold beside the text tables, read here on
//! its own path. Each `objects.txt` line of `packs/ab/rmg.toml` and each line `packs/sod/rmg.toml`
//! adds or changes carries its picture's `.msk` as `msk_load` 0x514610 finds it: the picture's
//! name with its extension replaced by `.msk`, from `Data/H3sprite.lod`, else `Data/H3ab_spr.lod`
//! (the game's sprite search order), else `default.msk`; its width and height (bytes 0 and 1) and
//! its colour mask (bytes 2 to 7: the cell `dx` columns left of and `dy` rows above the anchor is
//! bit 7 − `dx` of mask byte 5 − `dy`, mark 8 `dy` + `dx` of the pack's string). SoD's
//! `[generator]` object settings equal what `object_settings_init` 0x41B500 builds, its tables'
//! bounds and fields read from its instructions: the class count (0x41B577), each class's own
//! class as +0x8 first (0x41B554, 0x41B55E), the (class, value) pairs into +0x8 (0x41B57F to
//! 0x41B594), the classes of the three flag tables into +0x2, +0x0 and +0x1 (0x41B5B7 to
//! 0x41B5FF). Without the executable or the sprite archives the check is skipped with a note. The
//! installation copy is only read.

use std::path::Path;
use std::process::ExitCode;

use h3_formats::lod::LodArchive;

const AB_RMG: &str = include_str!("../../../packs/ab/rmg.toml");
const SOD_RMG: &str = include_str!("../../../packs/sod/rmg.toml");

/// The sprite archives in the game's search order.
const SPRITES: [&str; 2] = ["Data/H3sprite.lod", "Data/H3ab_spr.lod"];

/// The object template lines of the two packs: AB's, SoD's added, SoD's changed.
fn lines() -> Result<Vec<toml::Value>, String> {
    let mut out = Vec::new();
    for (text, keys) in [
        (AB_RMG, &["object_templates"][..]),
        (
            SOD_RMG,
            &["object_templates", "object_template_overrides"][..],
        ),
    ] {
        let value: toml::Value = toml::from_str(text).map_err(|e| e.to_string())?;
        for key in keys {
            let list = value
                .get(key)
                .and_then(toml::Value::as_array)
                .ok_or_else(|| format!("no {key}"))?;
            out.extend(list.iter().cloned());
        }
    }
    Ok(out)
}

/// The mismatches of the lines' `.msk` fields with the archives; the lines with their own `.msk`.
fn masks(install: &Path) -> Result<(Vec<String>, usize, usize), String> {
    let data = SPRITES
        .iter()
        .map(|a| std::fs::read(install.join(a)).map_err(|e| format!("{a}: {e}")))
        .collect::<Result<Vec<_>, _>>()?;
    let lods = data
        .iter()
        .map(|d| LodArchive::parse(d).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let find = |name: &str| -> Result<Option<Vec<u8>>, String> {
        for lod in &lods {
            if let Some(bytes) = lod.read_by_name(name).map_err(|e| e.to_string())? {
                return Ok(Some(bytes));
            }
        }
        Ok(None)
    };
    let (mut mismatches, mut own) = (Vec::new(), 0);
    let lines = lines()?;
    for line in &lines {
        let field = |key: &str| line.get(key).ok_or_else(|| format!("a line without {key}"));
        let picture = field("picture")?.as_str().unwrap_or_default();
        let name = match picture.rfind('.') {
            Some(dot) => format!("{}.msk", &picture[..dot]),
            None => format!("{picture}.msk"),
        };
        let msk = match find(&name)? {
            Some(msk) => {
                own += 1;
                msk
            }
            None => find("default.msk")?.ok_or("no default.msk")?,
        };
        let colour: String = (0..48)
            .map(|k| {
                let (dy, dx) = (k / 8, k % 8);
                if msk[2 + 5 - dy] >> (7 - dx) & 1 == 1 {
                    '1'
                } else {
                    '0'
                }
            })
            .collect();
        let want = (i64::from(msk[0]), i64::from(msk[1]), colour.as_str());
        let got = (
            field("width")?.as_integer().unwrap_or(-1),
            field("height")?.as_integer().unwrap_or(-1),
            field("colour")?.as_str().unwrap_or_default(),
        );
        if got != want {
            mismatches.push(format!("{picture}: the pack's {got:?}, {name}'s {want:?}"));
        }
    }
    Ok((mismatches, lines.len(), own))
}

/// The object settings `object_settings_init` builds: per field offset 0, 1, 2 the flags by class,
/// and the classes' +0x8.
fn settings(exe: &[u8]) -> Result<([Vec<bool>; 3], Vec<u32>), String> {
    let (_, image) = h3_formats::exe::identify(exe).map_err(|e| e.to_string())?;
    let bytes = |va: u32, n: usize| -> Result<&[u8], String> {
        let off = image
            .file_offset(va)
            .ok_or_else(|| format!("{va:#x} unmapped"))?;
        exe.get(off..off + n)
            .ok_or_else(|| format!("{va:#x} past the file"))
    };
    let word = |va: u32| -> Result<u32, String> {
        let b = bytes(va, 4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    let expect = |va: u32, code: &[u8]| -> Result<(), String> {
        if bytes(va, code.len())? == code {
            Ok(())
        } else {
            Err(format!("{va:#x} is not the instruction read"))
        }
    };
    // cmp ecx, classes; mov eax, settings + 2; mov [eax + 6], ecx (each class's own as +0x8).
    expect(0x41_B577, &[0x81, 0xF9])?;
    let classes = usize::try_from(word(0x41_B579)?).map_err(|e| e.to_string())?;
    expect(0x41_B554, &[0xB8])?;
    let base = word(0x41_B555)? - 2;
    expect(0x41_B55E, &[0x89, 0x48, 0x06])?;
    // A table from `mov eax, start` to `cmp eax, end`.
    let table = |start: u32, end: u32| -> Result<Vec<u32>, String> {
        expect(start, &[0xB8])?;
        expect(end, &[0x3D])?;
        (word(start + 1)?..word(end + 1)?)
            .step_by(4)
            .map(word)
            .collect()
    };
    let slot = |class: u32| usize::try_from(class).ok().filter(|&c| c < classes);
    let mut groups: Vec<u32> = (0..u32::try_from(classes).map_err(|e| e.to_string())?).collect();
    // mov [ecx + settings + 8], edx.
    expect(0x41_B594, &[0x89, 0x91])?;
    if word(0x41_B596)? != base + 8 {
        return Err(String::from("the pairs' store is not +0x8"));
    }
    for pair in table(0x41_B57F, 0x41_B58F)?.chunks(2) {
        let class = slot(pair[0]).ok_or_else(|| format!("pair class {}", pair[0]))?;
        groups[class] = pair[1];
    }
    let mut flags = [
        vec![false; classes],
        vec![false; classes],
        vec![false; classes],
    ];
    for (start, end, store) in [
        (0x41_B5B7, 0x41_B5C4, 0x41_B5C9),
        (0x41_B5D2, 0x41_B5DF, 0x41_B5E4),
        (0x41_B5ED, 0x41_B5FA, 0x41_B5FF),
    ] {
        // mov byte [reg + settings + field], 1.
        expect(store, &[0xC6])?;
        expect(store + 6, &[1])?;
        let field = usize::try_from(word(store + 2)? - base).map_err(|e| e.to_string())?;
        let marks = flags
            .get_mut(field)
            .ok_or_else(|| format!("{store:#x} stores field {field}"))?;
        for class in table(start, end)? {
            marks[slot(class).ok_or_else(|| format!("flag class {class}"))?] = true;
        }
    }
    Ok((flags, groups))
}

/// The mismatches of SoD's `[generator]` object settings with the executable's.
fn compare_settings(exe: &[u8]) -> Result<Vec<String>, String> {
    let (flags, groups) = settings(exe)?;
    let value: toml::Value = toml::from_str(SOD_RMG).map_err(|e| e.to_string())?;
    let generator = value.get("generator").ok_or("no [generator]")?;
    let mut out = Vec::new();
    for (key, marks) in [
        ("object_not_entered", &flags[0]),
        ("object_from_above", &flags[1]),
        ("object_removable", &flags[2]),
    ] {
        let want: String = marks.iter().map(|&m| if m { '1' } else { '0' }).collect();
        if generator.get(key).and_then(toml::Value::as_str) != Some(want.as_str()) {
            out.push(format!("{key} differs from the executable's"));
        }
    }
    let want: Vec<toml::Value> = groups
        .iter()
        .zip(0..)
        .filter(|&(&group, class)| group != class)
        .map(|(&group, class)| {
            toml::Value::Array(vec![
                toml::Value::Integer(i64::from(class)),
                toml::Value::Integer(i64::from(group)),
            ])
        })
        .collect();
    if generator
        .get("object_class_groups")
        .and_then(toml::Value::as_array)
        != Some(&want)
    {
        out.push(String::from(
            "object_class_groups differ from the executable's",
        ));
    }
    Ok(out)
}

fn check(install: &Path) -> Result<String, Vec<String>> {
    let (mut mismatches, lines, own) = masks(install).map_err(|e| vec![e])?;
    let exe = std::fs::read(install.join("Heroes3.exe")).map_err(|e| vec![e.to_string()])?;
    mismatches.extend(compare_settings(&exe).map_err(|e| vec![e])?);
    if mismatches.is_empty() {
        Ok(format!(
            "the rmg packs' {lines} object template lines carry their .msk ({own} their own, {} \
             default.msk) and SoD's object settings are object_settings_init's",
            lines - own
        ))
    } else {
        Err(mismatches)
    }
}

fn main() -> ExitCode {
    let Some(install) = std::env::args().nth(1) else {
        eprintln!("usage: original_data <installation copy>");
        return ExitCode::FAILURE;
    };
    let install = Path::new(&install);
    let needed = ["Heroes3.exe", SPRITES[0], SPRITES[1]].map(|f| install.join(f));
    if let Some(missing) = needed.iter().find(|f| !f.is_file()) {
        println!("h3-harvest: skipped ({} is not there)", missing.display());
        return ExitCode::SUCCESS;
    }
    match check(install) {
        Ok(summary) => {
            println!("h3-harvest: {summary}");
            ExitCode::SUCCESS
        }
        Err(mismatches) => {
            for m in &mismatches {
                eprintln!("h3-harvest: {m}");
            }
            ExitCode::FAILURE
        }
    }
}
