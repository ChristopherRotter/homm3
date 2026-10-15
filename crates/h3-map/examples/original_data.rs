//! `h3-map`'s check against the original data (`cargo xtask verify --with-original-data`): reads
//! every map in `<install>/Maps`, imports it with the original packs (RoE to SoD) and validates
//! the setup. Fails when the reader rejects a map, when a map's static layer does not import and
//! validate, or when a map that imports fails validation; a map the current increment cannot
//! represent is counted by its reason (`Unsupported`). Prints what the input bounds were set from
//! (`docs/formats/h3m.md`, "Input bounds"), the import's counts ("Import") and the maps with a
//! tile that is the visitable tile of two or more objects (E08 T14a: which object a step there
//! visits, ADV-OBJ-003) and the typed objects of one type on one anchor (E08 T15a: what a player
//! sees names a typed object by its type and anchor, ADR-0016 point 6); since E08 T20a2a the
//! monsters' and the special grounds' counts and since E08 T21a the guarded piles'
//! (`docs/formats/h3m.md`, "Import").
//!
//! `cargo run -p h3-map --example original_data -- original-data/install`
//!
//! The installation copy is only read. Without a `Maps` folder the check is skipped with a note.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;

use std::sync::Arc;

use h3_adventure::MapSetup;
use h3_game::{DEFAULT_DIFFICULTY, GameRuleset, GameSetup, PlayerSetup};
use h3_map::model::{Artifacts, Body, Player, RawMap, SeerTask, Text, Version};
use h3_map::{ImportError, Limits, gunzip, import, import_map};
use h3_pack_adventure::{ORIGINAL_PACKS, compile};

/// The largest value of each bounded quantity over all maps, with the map it came from; how many
/// objects of each body kind the maps hold; how many tiles objects are visited from; each map
/// with stacked visitable tiles, with the classes on each such tile in map order; each map with
/// typed objects that share their type and anchor with an earlier one, with how many; the
/// monsters' and grounds' counts (E08 T20a2a).
#[derive(Default)]
struct Peaks(
    BTreeMap<&'static str, (usize, String)>,
    BTreeMap<&'static str, usize>,
    usize,
    BTreeMap<String, Vec<Vec<u32>>>,
    BTreeMap<String, usize>,
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
    let packs = [
        h3_pack_battle::BASE_PACK,
        h3_pack_battle::AB_PACK,
        h3_pack_battle::SOD_PACK,
    ];
    let battle = match h3_pack_battle::compile(&packs, "base:faithful") {
        Ok(battle) => battle,
        Err(e) => {
            eprintln!("h3-map: the original battle packs do not compile: {e:?}");
            return ExitCode::FAILURE;
        }
    };
    let Ok(rules) = GameRuleset::new(Arc::new(core), Arc::new(part), Arc::new(battle)) else {
        eprintln!("h3-map: the parts belong to another core");
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
    let tiles: usize = peaks.3.values().map(Vec::len).sum();
    println!(
        "  maps with a tile visited from two or more objects: {} ({tiles} of {} visitable tiles)",
        peaks.3.len(),
        peaks.2
    );
    let mut stacks = BTreeMap::<&[u32], usize>::new();
    for classes in peaks.3.values().flatten() {
        *stacks.entry(classes).or_default() += 1;
    }
    for (classes, n) in &stacks {
        println!("    classes {classes:?} (map order): {n}");
    }
    for (map, tiles) in &peaks.3 {
        println!("    {map}: {}", tiles.len());
    }
    println!(
        "  typed objects of one type on one anchor (each after the first): {} in {} maps",
        peaks.4.values().sum::<usize>(),
        peaks.4.len()
    );
    for (map, twins) in &peaks.4 {
        println!("    {map}: {twins}");
    }
    for (what, n) in &peaks.5 {
        println!("  {what}: {n}");
    }
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
    // The static layer alone, in a game of one player without heroes: its objects' contents are
    // drawn as a game's are (E08 T12b).
    let setup = GameSetup {
        players: vec![PlayerSetup { colour: 0 }],
        difficulty: DEFAULT_DIFFICULTY,
        map: static_map,
        heroes: Vec::new(),
    };
    setup
        .validate(rules)
        .map_err(|e| format!("static layer: {e}"))?;
    let (visitable, stacked) = stacked(&map, &setup.map);
    peaks.2 += visitable;
    if !stacked.is_empty() {
        peaks.3.insert(name.to_owned(), stacked);
    }
    let twins = twins(&setup.map);
    if twins > 0 {
        peaks.4.insert(name.to_owned(), twins);
    }
    census(&map, &setup.map, &mut peaks.5);
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

/// Counts the monsters' data the import reads or refuses and the special grounds (E08 T20a2a):
/// treasures with a resource below 0 (the import leaves such an amount out), dispositions
/// beyond savage, treasures with an artifact; grounds, those whose picture the import's table
/// lacks (they lie on one tile), and tiles two or more grounds lie on; (E08 T21a) the guarded
/// piles the import keeps, those whose guards ask first, and the piles with a guards record the
/// import reads as plain (no stack) or leaves unguarded (a random stack).
fn census(map: &RawMap, setup: &MapSetup, counts: &mut BTreeMap<&'static str, usize>) {
    let mut add = |what, n: usize| *counts.entry(what).or_default() += n;
    for object in &map.objects {
        if let Body::Monster(m) = &object.body {
            add("monsters", 1);
            add(
                "monsters of a disposition beyond savage",
                usize::from(m.disposition > 4),
            );
            if let Some(t) = &m.treasure {
                add("monsters with a treasure", 1);
                add(
                    "monsters with a resource below 0 in their treasure",
                    usize::from(t.resources.iter().any(|&r| r < 0)),
                );
                add(
                    "monsters with an artifact in their treasure",
                    usize::from(t.artifact.is_some()),
                );
            }
        }
    }
    let records = map.objects.iter().filter(
        |o| matches!(&o.body, Body::Resource { message: Some(m), .. } if m.guards.is_some()),
    );
    add("piles with a guards record", records.count());
    for guards in setup.objects.iter().filter_map(|o| o.guards.as_ref()) {
        add("guarded piles", 1);
        add("guarded piles that ask first", usize::from(guards.question));
        add(
            "guarded piles with two or more stacks",
            usize::from(guards.army.len() > 1),
        );
    }
    let mut lying = BTreeMap::<(u16, u16, u8), usize>::new();
    for object in setup.objects.iter().filter(|o| o.drawn.bits() != 0) {
        add("special grounds", 1);
        add(
            "special grounds of a picture the import lacks",
            usize::from(object.drawn.bits() == 1),
        );
        for (column, row) in object.drawn.offsets() {
            if let (Some(x), Some(y)) = (
                object.anchor.x.checked_sub(u16::from(column)),
                object.anchor.y.checked_sub(u16::from(row)),
            ) {
                *lying.entry((x, y, object.anchor.level)).or_default() += 1;
            }
        }
    }
    add(
        "tiles two or more special grounds lie on",
        lying.values().filter(|&&n| n > 1).count(),
    );
}

/// The object classes the static layer leaves out (hero, random hero, hero placeholder), as
/// `h3_map::import_map` does.
const HERO_CLASSES: [u32; 3] = [34, 70, 214];

/// How many tiles of the static layer objects are visited from, and each one that is the
/// visitable tile of two or more objects, with the objects' classes in map order.
fn stacked(map: &RawMap, setup: &MapSetup) -> (usize, Vec<Vec<u32>>) {
    let classes = map
        .objects
        .iter()
        .filter_map(|o| {
            usize::try_from(o.template)
                .ok()
                .and_then(|t| map.templates.get(t))
        })
        .map(|t| t.class)
        .filter(|class| !HERO_CLASSES.contains(class));
    let mut tiles = BTreeMap::<(u8, u16, u16), Vec<u32>>::new();
    for (object, class) in setup.objects.iter().zip(classes) {
        let anchor = object.anchor;
        for (column, row) in object.visitable.offsets() {
            let x = anchor.x.checked_sub(u16::from(column));
            let y = anchor.y.checked_sub(u16::from(row));
            if let (Some(x), Some(y)) = (x, y) {
                tiles.entry((anchor.level, y, x)).or_default().push(class);
            }
        }
    }
    let visitable = tiles.len();
    let stacked = tiles
        .into_values()
        .filter(|classes| classes.len() > 1)
        .collect();
    (visitable, stacked)
}

/// How many typed objects of the static layer share their type and anchor with an earlier one (a
/// random resource counted by its random type, before the draw).
fn twins(setup: &MapSetup) -> usize {
    let mut seen = BTreeMap::<_, usize>::new();
    for object in &setup.objects {
        if let Some(object_type) = object.object_type {
            let anchor = object.anchor;
            *seen
                .entry((object_type, anchor.level, anchor.y, anchor.x))
                .or_default() += 1;
        }
    }
    seen.values().map(|n| n - 1).sum()
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
