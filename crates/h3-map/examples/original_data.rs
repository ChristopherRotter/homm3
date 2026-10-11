//! `h3-map`'s check against the original data (`cargo xtask verify --with-original-data`): reads
//! every map in `<install>/Maps`, imports it with the original packs (RoE to SoD) and validates
//! the setup. Fails when the reader rejects a map, when a map's static layer does not import and
//! validate, or when a map that imports fails validation; a map the current increment cannot
//! represent is counted by its reason (`Unsupported`). Prints what the input bounds were set from
//! (`docs/formats/h3m.md`, "Input bounds") and the import's counts ("Import").
//!
//! `cargo run -p h3-map --example original_data -- original-data/install`
//!
//! The installation copy is only read. Without a `Maps` folder the check is skipped with a note.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;

use std::sync::Arc;

use h3_adventure::{AdventureSetup, AdventureState};
use h3_game::GameRuleset;
use h3_map::model::{Artifacts, Body, Player, RawMap, SeerTask, Text, Version};
use h3_map::{ImportError, Limits, gunzip, import, import_map};
use h3_pack_adventure::{ORIGINAL_PACKS, compile};

/// The largest value of each bounded quantity over all maps, with the map it came from, and how
/// many objects of each body kind the maps hold.
#[derive(Default)]
struct Peaks(
    BTreeMap<&'static str, (usize, String)>,
    BTreeMap<&'static str, usize>,
);

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
    let (core, part) = match compile(&ORIGINAL_PACKS, "base:faithful") {
        Ok(rules) => rules,
        Err(e) => {
            eprintln!("h3-map: the original packs do not compile: {e:?}");
            return ExitCode::FAILURE;
        }
    };
    let Ok(rules) = GameRuleset::new(Arc::new(core), Arc::new(part)) else {
        eprintln!("h3-map: the adventure part belongs to another core");
        return ExitCode::FAILURE;
    };
    let mut imports = BTreeMap::<String, Vec<String>>::new();
    let limits = Limits::default();
    let mut versions = BTreeMap::<Version, usize>::new();
    let mut peaks = Peaks::default();
    let mut failures = 0;
    for path in &files {
        let name = path
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        match check(path, &limits, &mut peaks, &name, &rules) {
            Ok((version, outcome)) => {
                *versions.entry(version).or_default() += 1;
                imports.entry(outcome).or_default().push(name);
            }
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
    let kinds: Vec<String> = peaks.1.iter().map(|(k, n)| format!("{k} {n}")).collect();
    println!("  objects by body: {}", kinds.join(", "));
    println!("h3-map: every static layer imports and validates; the import by outcome:");
    for (outcome, maps) in &imports {
        println!("  {outcome}: {}", maps.len());
        if outcome == IMPORTED {
            println!("    {}", maps.join(", "));
        }
    }
    if failures == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// The outcome of a map that imports and validates.
const IMPORTED: &str = "imported and valid";

/// Reads one map and checks that its terrain codes are in range (a misaligned read shows there),
/// then imports it: its static layer must import and validate; the whole map imports and
/// validates, or the import reports what is not supported yet (the outcome).
fn check(
    path: &Path,
    limits: &Limits,
    peaks: &mut Peaks,
    name: &str,
    rules: &GameRuleset,
) -> Result<(Version, String), String> {
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
    let static_map = import_map(&map, rules).map_err(|e| format!("static layer: {e}"))?;
    let setup = AdventureSetup {
        players: 1,
        map: static_map,
        heroes: Vec::new(),
    };
    AdventureState::new(rules.core(), rules.adventure(), &setup)
        .map_err(|e| format!("static layer: {e}"))?;
    let outcome = match import(&map, rules) {
        Ok(setup) => {
            setup
                .validate(rules)
                .map_err(|e| format!("imported, but invalid: {e}"))?;
            IMPORTED.to_owned()
        }
        Err(ImportError::Unsupported(what)) => format!("not supported: {}", what.reason()),
        Err(e) => return Err(format!("import: {e}")),
    };
    Ok((map.header.version, outcome))
}

fn note(map: &RawMap, peaks: &mut Peaks, name: &str) {
    let mut texts: Vec<&Text> = vec![&map.header.name, &map.header.description];
    for r in &map.rumors {
        texts.extend([&r.name, &r.text]);
    }
    for d in map
        .players
        .iter()
        .filter_map(|p: &Player| p.details.as_ref())
    {
        peaks.note("player heroes", d.heroes.len(), name);
        texts.extend(d.heroes.iter().map(|h| &h.name));
    }
    peaks.note("placeholders", map.placeholders.len(), name);
    peaks.note("disposed heroes", map.disposed_heroes.len(), name);
    peaks.note("rumors", map.rumors.len(), name);
    peaks.note("custom heroes", map.custom_heroes.len(), name);
    let hero =
        |peaks: &mut Peaks, skills: Option<&Vec<(u8, u8)>>, artifacts: Option<&Artifacts>| {
            peaks.note("secondary skills", skills.map_or(0, Vec::len), name);
            peaks.note("backpack", artifacts.map_or(0, |a| a.backpack.len()), name);
        };
    for h in &map.custom_heroes {
        hero(peaks, h.secondary_skills.as_ref(), h.artifacts.as_ref());
        texts.extend(h.biography.as_ref());
    }
    for o in &map.objects {
        *peaks.1.entry(kind(&o.body)).or_default() += 1;
        match &o.body {
            Body::Hero(h) => {
                hero(peaks, h.secondary_skills.as_ref(), h.artifacts.as_ref());
                texts.extend(h.name.iter().chain(h.biography.iter()));
            }
            Body::Event(e) => texts.extend(e.message.as_ref().map(|m| &m.text)),
            Body::Pandora { message, .. }
            | Body::Artifact(message)
            | Body::Scroll { message, .. }
            | Body::Resource { message, .. } => texts.extend(message.as_ref().map(|m| &m.text)),
            Body::Monster(m) => texts.extend(m.treasure.as_ref().map(|t| &t.text)),
            Body::Sign(t) => texts.push(t),
            Body::SeerHut(Some(SeerTask { quest, .. })) | Body::QuestGuard(Some(quest)) => {
                texts.extend([&quest.first_visit, &quest.next_visit, &quest.completed]);
            }
            Body::Town(t) => {
                peaks.note("town events", t.events.len(), name);
                texts.extend(t.name.iter());
                for e in &t.events {
                    texts.extend([&e.event.name, &e.event.message]);
                }
            }
            _ => {}
        }
    }
    for e in &map.events {
        texts.extend([&e.name, &e.message]);
    }
    peaks.note("templates", map.templates.len(), name);
    peaks.note("objects", map.objects.len(), name);
    peaks.note("timed events", map.events.len(), name);
    let longest = texts.iter().map(|t| t.as_bytes().len()).max().unwrap_or(0);
    peaks.note("string", longest, name);
}

/// The body's kind, for the summary.
const fn kind(body: &Body) -> &'static str {
    match body {
        Body::None => "none",
        Body::Event(_) => "event",
        Body::Hero(_) => "hero",
        Body::Monster(_) => "monster",
        Body::Sign(_) => "sign",
        Body::SeerHut(_) => "seer hut",
        Body::WitchHut(_) => "witch hut",
        Body::Scholar { .. } => "scholar",
        Body::Garrison { .. } => "garrison",
        Body::Artifact(_) => "artifact",
        Body::Scroll { .. } => "scroll",
        Body::Resource { .. } => "resource",
        Body::Town(_) => "town",
        Body::Mine { .. } => "mine",
        Body::AbandonedMine(_) => "abandoned mine",
        Body::Owned { .. } => "owned",
        Body::RandomDwelling(_) => "random dwelling",
        Body::Shrine { .. } => "shrine",
        Body::Pandora { .. } => "pandora",
        Body::Grail { .. } => "grail",
        Body::QuestGuard(_) => "quest guard",
        Body::HeroPlaceholder { .. } => "hero placeholder",
    }
}
