//! `h3-map`'s check against the original data (`cargo xtask verify --with-original-data`): reads
//! every map in `<install>/Maps` and fails on the first one the reader rejects. Prints what the
//! input bounds were set from (`docs/formats/h3m.md`, "Input bounds").
//!
//! `cargo run -p h3-map --example original_data -- original-data/install`
//!
//! The installation copy is only read. Without a `Maps` folder the check is skipped with a note.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;

use h3_map::model::{CustomHero, Player, RawMap, Version};
use h3_map::{Limits, gunzip};

/// The largest value of each bounded quantity over all maps, with the map it came from.
#[derive(Default)]
struct Peaks(BTreeMap<&'static str, (usize, String)>);

impl Peaks {
    fn note(&mut self, what: &'static str, value: usize, map: &str) {
        let entry = self.0.entry(what).or_insert((0, String::new()));
        if value > entry.0 {
            *entry = (value, map.to_owned());
        }
    }
}

fn main() -> ExitCode {
    let Some(install) = std::env::args().nth(1) else {
        eprintln!("usage: original_data <installation copy>");
        return ExitCode::FAILURE;
    };
    let dir = Path::new(&install).join("Maps");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        println!("h3-map: skipped ({} is not there)", dir.display());
        return ExitCode::SUCCESS;
    };
    let mut files: Vec<_> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("h3m") || e.eq_ignore_ascii_case("tut"))
        })
        .collect();
    files.sort();
    let limits = Limits::default();
    let mut versions = BTreeMap::<Version, usize>::new();
    let mut peaks = Peaks::default();
    let mut failures = 0;
    for path in &files {
        let name = path
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        match check(path, &limits, &mut peaks, &name) {
            Ok(version) => *versions.entry(version).or_default() += 1,
            Err(e) => {
                println!("h3-map: {name}: {e}");
                failures += 1;
            }
        }
    }
    println!(
        "h3-map: {} of {} maps read ({versions:?})",
        files.len() - failures,
        files.len()
    );
    for (what, (value, map)) in &peaks.0 {
        println!("  largest {what}: {value} ({map})");
    }
    if failures == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Reads one map and checks that its terrain codes are in range (a misaligned read shows there).
fn check(path: &Path, limits: &Limits, peaks: &mut Peaks, name: &str) -> Result<Version, String> {
    let file = std::fs::read(path).map_err(|e| e.to_string())?;
    peaks.note("file", file.len(), name);
    let inflated = gunzip(&file, limits.max_map_bytes).map_err(|e| e.to_string())?;
    peaks.note("inflated map", inflated.len(), name);
    let map = h3_map::read(&file, limits).map_err(|e| e.to_string())?;
    if let Some((i, t)) = map
        .terrain
        .tiles
        .iter()
        .enumerate()
        .find(|(_, t)| t.terrain > 9 || t.river > 4 || t.road > 3)
    {
        return Err(format!("tile {i} has codes out of range: {t:?}"));
    }
    note(&map, peaks, name);
    Ok(map.header.version)
}

fn note(map: &RawMap, peaks: &mut Peaks, name: &str) {
    let mut string = |len: usize| peaks.note("string", len, name);
    string(map.header.name.as_bytes().len());
    string(map.header.description.as_bytes().len());
    for r in &map.rumors {
        string(r.name.as_bytes().len());
        string(r.text.as_bytes().len());
    }
    for d in map
        .players
        .iter()
        .filter_map(|p: &Player| p.details.as_ref())
    {
        peaks.note("player heroes", d.heroes.len(), name);
        for h in &d.heroes {
            peaks.note("string", h.name.as_bytes().len(), name);
        }
    }
    peaks.note("placeholders", map.placeholders.len(), name);
    peaks.note("disposed heroes", map.disposed_heroes.len(), name);
    peaks.note("rumors", map.rumors.len(), name);
    peaks.note("custom heroes", map.custom_heroes.len(), name);
    for h in &map.custom_heroes {
        let CustomHero {
            secondary_skills,
            artifacts,
            biography,
            ..
        } = h;
        peaks.note(
            "secondary skills",
            secondary_skills.as_ref().map_or(0, Vec::len),
            name,
        );
        peaks.note(
            "backpack",
            artifacts.as_ref().map_or(0, |a| a.backpack.len()),
            name,
        );
        peaks.note(
            "string",
            biography.as_ref().map_or(0, |b| b.as_bytes().len()),
            name,
        );
    }
}
