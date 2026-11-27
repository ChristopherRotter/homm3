//! `h3-pack-rmg`'s check against the original data (`cargo xtask verify --with-original-data`;
//! E14 T2b): the harvested tables equal the text tables. It compiles the original packs' rmg part
//! (AB's alone, `base` and `ab`; SoD's, all three) and compares it, field by field, with
//! `Rmg.txt`, `Objects.txt` and `Rand_trn.txt` of `<install>/../extracted/H3ab_bmp` and
//! `H3bitmap`, read here by a reader of its own as survey `docs/rules/surveys/E14-generator.md`
//! §3.1 describes them (rows from 3; a template at a row of two or more cells with column 0 set, a
//! zone at 76 or more with column 3 set, a connection at 85 or more with 76 set and 77 not empty;
//! *set*: not empty, no leading space; numbers as `c_atoi`; quoted cells as the game's loader).
//! In `faithful` the part must equal the tables; in `strict-fixed` it differs only by 8XM8's six
//! decimal first zones, 6, 6, 7, 7, 8, 8 (RMG-QUIRK-22). The object templates in SoD's line
//! order are the generator's table, whose index the phase oracle's dump gives (E14 T4b). An
//! object template's `.msk` fields
//! (width, height, colour mask; E14 T4b) are not in the text tables: they are taken from the part
//! here and checked against the sprite archives by `h3-harvest`'s own `original_data` example.
//! Without the text tables the check is skipped with a note. The installation copy is only read.

use std::path::Path;
use std::process::ExitCode;

use h3_pack_rmg::{ORIGINAL_PACKS, compile};
use h3_rmg::{
    Connection, ObjectTemplate, Obstacle, RmgRuleset, Strength, Template, Zone, ZoneKind,
};

/// The records of a tab-separated table, a cell starting with `"` running to the next `"` before a
/// tab, a line break or the end.
fn records(bytes: &[u8]) -> Vec<Vec<String>> {
    let text: Vec<char> = bytes.iter().map(|&b| char::from(b)).collect();
    let (mut rows, mut row, mut cell) = (Vec::new(), Vec::new(), String::new());
    let mut i = 0;
    let mut start = true;
    while i < text.len() {
        let c = text[i];
        if start && c == '"' {
            let mut j = i + 1;
            while j < text.len()
                && !(text[j] == '"' && matches!(text.get(j + 1), None | Some('\t' | '\r' | '\n')))
            {
                cell.push(text[j]);
                j += 1;
            }
            i = j + 1;
            start = false;
            continue;
        }
        start = false;
        match c {
            '\t' => {
                row.push(std::mem::take(&mut cell));
                start = true;
            }
            '\r' => {}
            '\n' => {
                row.push(std::mem::take(&mut cell));
                rows.push(std::mem::take(&mut row));
                start = true;
            }
            _ => cell.push(c),
        }
        i += 1;
    }
    if !cell.is_empty() || !row.is_empty() {
        row.push(cell);
        rows.push(row);
    }
    rows
}

fn set(row: &[String], c: usize) -> bool {
    row.get(c)
        .is_some_and(|v| !v.is_empty() && !v.starts_with(' '))
}

fn atoi(row: &[String], c: usize) -> i32 {
    let v = row.get(c).map_or("", |v| v.trim_start());
    let (sign, digits) = match v.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, v.strip_prefix('+').unwrap_or(v)),
    };
    let n: String = digits.chars().take_while(char::is_ascii_digit).collect();
    sign * n.parse::<i32>().unwrap_or(0)
}

fn atois<const N: usize>(row: &[String], first: usize) -> [i32; N] {
    std::array::from_fn(|k| atoi(row, first + k))
}

fn bits(row: &[String], first: usize, n: usize) -> u16 {
    (0..n)
        .filter(|&k| set(row, first + k))
        .fold(0, |b, k| b | 1 << k)
}

fn zone(r: &[String]) -> Zone {
    let kind = match (4..=7).rev().find(|&c| set(r, c)) {
        Some(4) => ZoneKind::HumanStart,
        Some(5) => ZoneKind::ComputerStart,
        Some(7) => ZoneKind::Junction,
        _ => ZoneKind::Treasure,
    };
    let strength = match r[55].chars().next().map(|c| c.to_ascii_lowercase()) {
        Some('n') => Strength::None,
        Some('w') => Strength::Weak,
        Some('s') => Strength::Strong,
        _ => Strength::Average,
    };
    Zone {
        id: atoi(r, 3),
        kind,
        base_size: atoi(r, 8),
        humans: atois(r, 9),
        players: atois(r, 11),
        ownership: atoi(r, 13),
        player_towns: atois(r, 14),
        neutral_towns: atois(r, 18),
        same_type: set(r, 22),
        town_types: bits(r, 23, 9),
        mines: atois(r, 32),
        mine_density: atois(r, 39),
        terrain_match: set(r, 46),
        terrains: u8::try_from(bits(r, 47, 8)).unwrap_or(0),
        strength,
        monster_match: set(r, 56),
        neutral_monsters: set(r, 57),
        monster_towns: bits(r, 58, 9),
        treasure: [atois(r, 67), atois(r, 70), atois(r, 73)],
    }
}

fn templates(rows: &[Vec<String>]) -> Vec<Template> {
    let mut out: Vec<Template> = Vec::new();
    for r in &rows[3..] {
        if r.len() >= 2 && set(r, 0) {
            out.push(Template {
                name: r[0].clone(),
                sizes: [atoi(r, 1), atoi(r, 2)],
                zones: Vec::new(),
                connections: Vec::new(),
            });
        }
        let Some(t) = out.last_mut() else { continue };
        if r.len() >= 76 && set(r, 3) {
            t.zones.push(zone(r));
        }
        if r.len() >= 85 && set(r, 76) && !r[77].is_empty() {
            t.connections.push(Connection {
                zones: [atoi(r, 76), atoi(r, 77)],
                value: atoi(r, 78),
                wide: set(r, 79),
                border_guard: set(r, 80),
                humans: atois(r, 81),
                players: atois(r, 83),
            });
        }
    }
    out
}

fn mask(text: &str) -> u64 {
    text.bytes()
        .enumerate()
        .filter(|&(_, c)| c == b'1')
        .fold(0, |b, (k, _)| b | 1 << k)
}

fn objects(bytes: &[u8]) -> Vec<ObjectTemplate> {
    let text = String::from_utf8_lossy(bytes);
    let mut lines = text.lines();
    let count: usize = lines
        .next()
        .and_then(|l| l.trim().parse().ok())
        .unwrap_or(0);
    lines
        .take(count)
        .map(|l| {
            let f: Vec<&str> = l.split(' ').collect();
            let n = |k: usize| f[k].parse::<u32>().unwrap_or(u32::MAX);
            ObjectTemplate {
                picture: f[0].to_owned(),
                passability: mask(f[1]),
                actions: mask(f[2]),
                editor_terrains: u16::try_from(mask(f[3])).unwrap_or(0),
                terrains: u16::try_from(mask(f[4])).unwrap_or(0),
                class: n(5),
                subclass: n(6),
                group: u8::try_from(n(7)).unwrap_or(u8::MAX),
                overlay: u8::try_from(n(8)).unwrap_or(u8::MAX),
                width: 0,
                height: 0,
                colour: 0,
            }
        })
        .collect()
}

fn obstacles(rows: &[Vec<String>]) -> Vec<Obstacle> {
    let rows: Vec<_> = rows[3..].iter().take_while(|r| set(r, 0)).collect();
    let n = rows.len();
    rows.iter()
        .map(|r| Obstacle {
            object_type: atoi(r, 3),
            subtype: atoi(r, 4),
            terrain: atoi(r, 6),
            modifiers: atois(r, 7),
            adjacent: (16..16 + n).map(|c| atoi(r, c)).collect(),
            overlap: (16 + n..16 + 2 * n).map(|c| atoi(r, c)).collect(),
        })
        .collect()
}

/// The mismatches of `part` with the tables in `dir`.
fn compare(part: &RmgRuleset, dir: &Path, what: &str) -> Result<Vec<String>, String> {
    let read = |name: &str| {
        let path = dir.join(name);
        std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))
    };
    let mut out = Vec::new();
    let expected = templates(&records(&read("Rmg.txt")?));
    if part.templates().len() != expected.len() {
        out.push(format!(
            "{what}: {} templates, {} in Rmg.txt",
            part.templates().len(),
            expected.len()
        ));
    }
    for (i, (got, want)) in part.templates().iter().zip(&expected).enumerate() {
        if got != want {
            out.push(format!(
                "{what}: template {i} `{}` differs from Rmg.txt's",
                want.name
            ));
        }
    }
    // The `.msk` fields are the part's (h3-harvest's example checks them).
    let mut lines = objects(&read("Objects.txt")?);
    for (line, got) in lines.iter_mut().zip(part.object_templates()) {
        (line.width, line.height, line.colour) = (got.width, got.height, got.colour);
    }
    if part.object_templates() != lines.as_slice() {
        out.push(format!(
            "{what}: the object templates differ from Objects.txt's"
        ));
    }
    if part.obstacles() != obstacles(&records(&read("Rand_trn.txt")?)).as_slice() {
        out.push(format!("{what}: the obstacles differ from Rand_trn.txt's"));
    }
    Ok(out)
}

fn check(extracted: &Path) -> Result<String, Vec<String>> {
    let mut mismatches = Vec::new();
    let mut zones = 0;
    for (what, packs, dir) in [
        ("AB", &ORIGINAL_PACKS[..2], "H3ab_bmp"),
        ("SoD", &ORIGINAL_PACKS[..], "H3bitmap"),
    ] {
        let faithful = compile(packs, "base:faithful").map_err(|e| vec![format!("{what}: {e}")])?;
        let fixed =
            compile(packs, "base:strict-fixed").map_err(|e| vec![format!("{what}: {e}")])?;
        mismatches.extend(compare(&faithful, &extracted.join(dir), what).map_err(|e| vec![e])?);
        zones += faithful
            .templates()
            .iter()
            .map(|t| t.zones.len())
            .sum::<usize>();
        // strict-fixed: 8XM8's decimal first zones continue the series, nothing else differs.
        let mut want = faithful.templates().to_vec();
        if let Some(t) = want.iter_mut().find(|t| t.name == "8XM8") {
            for (c, first) in t.connections[10..16].iter_mut().zip([6, 6, 7, 7, 8, 8]) {
                c.zones[0] = first;
            }
        }
        if fixed.templates() != want.as_slice() {
            mismatches.push(format!(
                "{what}: strict-fixed differs by more than 8XM8's six ids"
            ));
        }
    }
    if mismatches.is_empty() {
        Ok(format!(
            "AB's and SoD's tables equal their text tables ({zones} zones in both parts; \
             strict-fixed only 8XM8's ids)"
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
    let extracted = Path::new(&install).join("..").join("extracted");
    let needed = ["H3ab_bmp", "H3bitmap"].iter().flat_map(|d| {
        ["Rmg.txt", "Objects.txt", "Rand_trn.txt"].map(|f| extracted.join(d).join(f))
    });
    if let Some(missing) = needed.clone().find(|f| !f.is_file()) {
        println!("h3-pack-rmg: skipped ({} is not there)", missing.display());
        return ExitCode::SUCCESS;
    }
    match check(&extracted) {
        Ok(summary) => {
            println!("h3-pack-rmg: {summary}");
            ExitCode::SUCCESS
        }
        Err(mismatches) => {
            for m in &mismatches {
                eprintln!("h3-pack-rmg: {m}");
            }
            ExitCode::FAILURE
        }
    }
}
