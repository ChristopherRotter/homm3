//! `cargo xtask check-exe-map`: the address map of `Heroes3.exe` (docs/exe/README.md) is well
//! formed, and every address of the executable that the docs or the code cite is covered by one
//! of its entries, so an agent who meets an address can look it up instead of re-reading code.
//!
//! The check needs no executable: the map holds the image's layout (`[image]`) and each
//! function's size, which `scripts/exe.py check --fix-sizes` fills from the executable.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::util::{rel, workspace_root};

/// Our readings: functions, sites, tables and globals.
pub const MAP: &str = "docs/exe/map.toml";
/// Record layouts (stack, hero, battle manager, …).
pub const STRUCTS: &str = "docs/exe/structs.toml";
/// Names ported from H3API (generated; guesses).
pub const H3API: &str = "docs/exe/h3api.toml";
/// Directories whose text files may cite addresses (the map's own directory is left out).
pub const CITING_DIRS: &[&str] = &["docs", "crates"];
/// File types scanned for citations.
pub const CITING_EXTENSIONS: &[&str] = &["md", "toml", "rs"];

const KINDS: &[&str] = &["function", "site", "table", "global"];
const CONFIDENCES: &[&str] = &["read", "inferred", "guess"];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapFile {
    pub image: Image,
    #[serde(default)]
    pub entry: Vec<Entry>,
}

/// The executable the map describes and the address range citations are checked in.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub sha256: String,
    pub text_start: u32,
    pub text_end: u32,
    pub end: u32,
    /// Numbers in the image's range that the docs use for something else.
    #[serde(default)]
    pub ignore: Vec<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub va: u32,
    pub kind: String,
    pub name: String,
    #[serde(default)]
    pub size: Option<u32>,
    pub summary: String,
    pub confidence: String,
    #[serde(default)]
    pub args: Option<String>,
    #[serde(default)]
    pub returns: Option<String>,
    #[serde(default)]
    pub refs: Vec<String>,
    #[serde(default)]
    pub h3api: Option<String>,
}

/// The generated H3API port: only what the map is checked against (its other keys are free).
#[derive(Debug, Deserialize)]
pub struct H3ApiFile {
    #[serde(default)]
    pub function: Vec<H3ApiItem>,
    #[serde(default)]
    pub data: Vec<H3ApiItem>,
}

#[derive(Debug, Deserialize)]
pub struct H3ApiItem {
    pub va: u32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructsFile {
    #[serde(default, rename = "struct")]
    pub structs: Vec<Struct>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Struct {
    pub name: String,
    pub size: u32,
    pub summary: String,
    #[serde(default)]
    pub refs: Vec<String>,
    #[serde(default)]
    pub h3api: Option<String>,
    #[serde(default)]
    pub fields: Vec<Field>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    pub offset: u32,
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub summary: String,
    pub confidence: String,
    #[serde(default)]
    pub refs: Vec<String>,
}

fn snake(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Whether an optional text or a reference is present but empty.
fn blank(texts: &[&Option<String>], refs: &[String]) -> bool {
    texts
        .iter()
        .any(|t| t.as_ref().is_some_and(|t| t.trim().is_empty()))
        || refs.iter().any(|r| r.trim().is_empty())
}

/// The bytes an entry covers for citations: a site only its own address.
fn span(e: &Entry) -> (u32, u32) {
    let size = match e.kind.as_str() {
        "site" => 1,
        "global" => e.size.unwrap_or(4),
        _ => e.size.unwrap_or(1),
    };
    (e.va, e.va.saturating_add(size.max(1)))
}

/// What is wrong with the map's entries; empty if nothing.
pub fn map_problems(map: &MapFile) -> Vec<String> {
    let mut problems = Vec::new();
    let img = &map.image;
    if img.sha256.len() != 64 || !img.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
        problems.push(format!("{MAP}: [image] sha256 is not a SHA-256"));
    }
    if !(img.text_start < img.text_end && img.text_end <= img.end) {
        problems.push(format!("{MAP}: [image] needs text_start < text_end <= end"));
    }
    let mut names = BTreeSet::new();
    let mut previous: Option<u32> = None;
    let mut function_end: Option<(u32, &str)> = None;
    for e in &map.entry {
        let at = format!("{MAP}: 0x{:06X} {}", e.va, e.name);
        if previous.is_some_and(|p| e.va <= p) {
            problems.push(format!(
                "{at}: entries must be sorted by address, one per address"
            ));
        }
        previous = Some(e.va);
        if !snake(&e.name) {
            problems.push(format!("{at}: names are snake_case"));
        }
        if !names.insert(e.name.as_str()) {
            problems.push(format!("{at}: the name is used twice"));
        }
        if !KINDS.contains(&e.kind.as_str()) {
            problems.push(format!("{at}: kind must be one of {KINDS:?}"));
        }
        if !CONFIDENCES.contains(&e.confidence.as_str()) {
            problems.push(format!("{at}: confidence must be one of {CONFIDENCES:?}"));
        }
        if e.summary.trim().is_empty() {
            problems.push(format!("{at}: the summary is empty"));
        }
        if blank(&[&e.args, &e.returns, &e.h3api], &e.refs) {
            problems.push(format!(
                "{at}: args, returns, h3api and refs may not be empty"
            ));
        }
        let in_text = img.text_start <= e.va && e.va < img.text_end;
        let in_image = img.text_start <= e.va && e.va < img.end;
        match e.kind.as_str() {
            "function" | "site" if !in_text => {
                problems.push(format!("{at}: a {} lies in .text", e.kind));
            }
            "table" | "global" if !in_image => problems.push(format!("{at}: outside the image")),
            _ => {}
        }
        if matches!(e.kind.as_str(), "function" | "table") && e.size.unwrap_or(0) == 0 {
            problems.push(format!(
                "{at}: a {} needs its size (functions: scripts/exe.py check --fix-sizes)",
                e.kind
            ));
        }
        if e.kind == "function" {
            if let Some((end, name)) = function_end
                && e.va < end
            {
                problems.push(format!("{at}: overlaps the function {name}"));
            }
            function_end = Some((span(e).1, e.name.as_str()));
        }
    }
    problems
}

/// Entries at an address H3API also names must say whether the reading agrees: `h3api = "…"`
/// when it does, else a summary that mentions H3API and says it does not (docs/exe/README.md,
/// "Recording a reading"). Without this, a name invented from one site inside a function stood
/// next to a stronger guess without anyone noticing (0x468860, once `faerie_spell_reader`).
pub fn h3api_problems(map: &MapFile, h3api: &H3ApiFile) -> Vec<String> {
    let names: BTreeMap<u32, &str> = h3api
        .function
        .iter()
        .chain(&h3api.data)
        .map(|i| (i.va, i.name.as_str()))
        .collect();
    map.entry
        .iter()
        .filter_map(|e| {
            let guess = names.get(&e.va)?;
            let says = e.h3api.is_some() || e.summary.to_ascii_lowercase().contains("h3api");
            (!says).then(|| {
                format!(
                    "{MAP}: 0x{:06X} {}: H3API names it {guess}; add h3api = \"...\" if the \
                     reading agrees, else say in the summary that it does not",
                    e.va, e.name
                )
            })
        })
        .collect()
}

/// What is wrong with the record layouts; empty if nothing.
pub fn struct_problems(file: &StructsFile) -> Vec<String> {
    let mut problems = Vec::new();
    let mut names = BTreeSet::new();
    for s in &file.structs {
        let at = format!("{STRUCTS}: {}", s.name);
        if !snake(&s.name) || !names.insert(s.name.as_str()) {
            problems.push(format!("{at}: struct names are unique and snake_case"));
        }
        if s.summary.trim().is_empty() || blank(&[&s.h3api], &s.refs) {
            problems.push(format!(
                "{at}: the summary, h3api and refs may not be empty"
            ));
        }
        let mut fields = BTreeSet::new();
        let mut previous: Option<u32> = None;
        for f in &s.fields {
            let fat = format!("{at}+0x{:X} {}", f.offset, f.name);
            if previous.is_some_and(|p| f.offset <= p) {
                problems.push(format!(
                    "{fat}: fields must be sorted by offset, one per offset"
                ));
            }
            previous = Some(f.offset);
            if f.offset >= s.size {
                problems.push(format!("{fat}: beyond the struct's size 0x{:X}", s.size));
            }
            if !snake(&f.name) || !fields.insert(f.name.as_str()) {
                problems.push(format!("{fat}: field names are unique and snake_case"));
            }
            if !CONFIDENCES.contains(&f.confidence.as_str()) {
                problems.push(format!("{fat}: confidence must be one of {CONFIDENCES:?}"));
            }
            if f.summary.trim().is_empty() || f.ty.trim().is_empty() || blank(&[], &f.refs) {
                problems.push(format!("{fat}: type and summary are needed"));
            }
        }
    }
    problems
}

/// The addresses `0x` + six hex digits in `text` that lie in the image's range.
pub fn cited(text: &str, img: &Image) -> BTreeSet<u32> {
    let bytes = text.as_bytes();
    let mut out = BTreeSet::new();
    let word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut i = 0;
    while i + 8 <= bytes.len() {
        let starts = bytes[i] == b'0'
            && bytes[i + 1] == b'x'
            && (i == 0 || !word(bytes[i - 1]))
            && bytes[i + 2..i + 8].iter().all(u8::is_ascii_hexdigit)
            && bytes.get(i + 8).is_none_or(|&b| !word(b));
        if starts {
            let digits = std::str::from_utf8(&bytes[i + 2..i + 8]).unwrap_or("0");
            let va = u32::from_str_radix(digits, 16).unwrap_or(0);
            if img.text_start <= va && va < img.end && !img.ignore.contains(&va) {
                out.insert(va);
            }
            i += 8;
        } else {
            i += 1;
        }
    }
    out
}

/// Cited addresses no entry covers, with the files citing them.
pub fn uncovered(
    map: &MapFile,
    citations: &BTreeMap<u32, BTreeSet<String>>,
) -> Vec<(u32, BTreeSet<String>)> {
    let spans: Vec<(u32, u32)> = map.entry.iter().map(span).collect();
    citations
        .iter()
        .filter(|(va, _)| !spans.iter().any(|(s, e)| *s <= **va && **va < *e))
        .map(|(va, files)| (*va, files.clone()))
        .collect()
}

fn text_files(dir: &Path, skip: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if !dir.is_dir() || dir == skip || dir.file_name().is_some_and(|n| n == "target") {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let path = entry.map_err(|e| format!("{}: {e}", dir.display()))?.path();
        if path.is_dir() {
            text_files(&path, skip, out)?;
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| CITING_EXTENSIONS.contains(&e))
        {
            out.push(path);
        }
    }
    Ok(())
}

/// Every citation under [`CITING_DIRS`] of `root`.
pub fn citations(root: &Path, img: &Image) -> Result<BTreeMap<u32, BTreeSet<String>>, String> {
    let skip = root.join("docs/exe");
    let mut files = Vec::new();
    for dir in CITING_DIRS {
        text_files(&root.join(dir), &skip, &mut files)?;
    }
    files.sort();
    let mut out: BTreeMap<u32, BTreeSet<String>> = BTreeMap::new();
    for file in files {
        let text = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        for va in cited(&text, img) {
            out.entry(va).or_default().insert(rel(root, &file));
        }
    }
    Ok(out)
}

fn load<T: for<'de> Deserialize<'de>>(root: &Path, file: &str) -> Result<T, String> {
    let text = fs::read_to_string(root.join(file)).map_err(|e| format!("{file}: {e}"))?;
    toml::from_str(&text).map_err(|e| format!("{file}: {e}"))
}

/// All problems of the map files under `root`.
pub fn check(root: &Path) -> Result<Vec<String>, String> {
    let map: MapFile = load(root, MAP)?;
    let structs: StructsFile = load(root, STRUCTS)?;
    // The generated H3API file's names are guesses; only their overlap with the map is checked.
    let h3api: H3ApiFile = load(root, H3API)?;
    let mut problems = map_problems(&map);
    problems.extend(struct_problems(&structs));
    problems.extend(h3api_problems(&map, &h3api));
    for (va, files) in uncovered(&map, &citations(root, &map.image)?) {
        let files: Vec<&str> = files.iter().map(String::as_str).collect();
        problems.push(format!(
            "0x{va:06X} is cited in {} but no entry of {MAP} covers it (docs/exe/README.md, \
             \"Recording a reading\"; `scripts/exe.py cited`)",
            files.join(", ")
        ));
    }
    Ok(problems)
}

pub fn run(_args: &[String]) -> Result<(), String> {
    let root = workspace_root();
    let problems = check(&root)?;
    if problems.is_empty() {
        let map: MapFile = load(&root, MAP)?;
        println!(
            "check-exe-map: ok ({} entries, every cited address covered)",
            map.entry.len()
        );
        Ok(())
    } else {
        Err(problems.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::test_support::TempDir;

    const IMAGE: &str = "[image]\nsha256 = \"aac39e8cf7ff46fb009057bd53716f4be46a88a49b857165b8d19e04df9d3d7d\"\ntext_start = 0x401000\ntext_end = 0x639612\nend = 0x6B6000\nignore = [0x400000]\n";

    fn map(entries: &str) -> MapFile {
        toml::from_str(&format!("{IMAGE}\n{entries}")).unwrap()
    }

    fn entry(va: &str, kind: &str, name: &str, size: &str) -> String {
        format!(
            "[[entry]]\nva = {va}\nkind = \"{kind}\"\nname = \"{name}\"\n{size}summary = \"s\"\nconfidence = \"read\"\n"
        )
    }

    #[test]
    fn a_good_map_passes() {
        let m = map(&format!(
            "{}{}{}",
            entry("0x401000", "function", "first", "size = 0x10\n"),
            entry("0x401004", "site", "first_site", ""),
            entry("0x63B850", "table", "weights", "size = 0x48\n"),
        ));
        assert!(map_problems(&m).is_empty(), "{:?}", map_problems(&m));
    }

    #[test]
    fn bad_entries_fail() {
        let m = map(&format!(
            "{}{}{}{}{}",
            entry("0x401000", "function", "Bad", "size = 4\n"),
            entry("0x401000", "routine", "bad", "size = 4\n"),
            entry("0x401002", "function", "bad", "size = 4\n"),
            entry("0x401010", "function", "no_size", "args = \"\"\n"),
            entry("0x6B7000", "table", "outside", "size = 4\n"),
        ));
        let problems = map_problems(&m).join("\n");
        for needle in [
            "snake_case",
            "needs its size",
            "sorted",
            "kind must",
            "used twice",
            "overlaps",
            "outside",
            "may not be empty",
        ] {
            assert!(problems.contains(needle), "{needle} missing in {problems}");
        }
        let m = map(&entry("0x63B850", "site", "data_site", ""));
        assert!(map_problems(&m).join("").contains("lies in .text"));
        let m: MapFile = toml::from_str(&format!(
            "{}{}",
            IMAGE
                .replace("aac39e8c", "zz")
                .replace("0x639612", "0x300000"),
            entry("0x401000", "function", "f", "size = 1\n")
                .replace("\"read\"", "\"sure\"")
                .replace("\"s\"", "\" \"")
        ))
        .unwrap();
        let problems = map_problems(&m).join("\n");
        for needle in [
            "not a SHA-256",
            "text_start <",
            "confidence",
            "summary is empty",
        ] {
            assert!(problems.contains(needle), "{needle} missing in {problems}");
        }
    }

    #[test]
    fn structs_are_checked() {
        let good: StructsFile = toml::from_str(
            "[[struct]]\nname = \"stack\"\nsize = 0x548\nsummary = \"s\"\nfields = [\n{ offset = 0x4C, name = \"count\", type = \"i32\", summary = \"alive\", confidence = \"read\" },\n]\n",
        )
        .unwrap();
        assert!(struct_problems(&good).is_empty());
        let bad: StructsFile = toml::from_str(
            "[[struct]]\nname = \"Stack\"\nsize = 0x10\nsummary = \"\"\nfields = [\n{ offset = 0x20, name = \"a\", type = \"\", summary = \"x\", confidence = \"maybe\", refs = [\"\"] },\n{ offset = 0x4, name = \"a\", type = \"i32\", summary = \"x\", confidence = \"read\" },\n]\n",
        )
        .unwrap();
        let problems = struct_problems(&bad).join("\n");
        for needle in [
            "snake_case",
            "may not be empty",
            "beyond",
            "sorted",
            "confidence",
            "type and summary",
        ] {
            assert!(problems.contains(needle), "{needle} missing in {problems}");
        }
    }

    #[test]
    fn h3api_overlaps_must_be_tagged_or_disputed() {
        let m = map(&format!(
            "{}{}{}{}",
            entry(
                "0x401000",
                "function",
                "agrees",
                "size = 0x10\nh3api = \"H3A::X\"\n"
            ),
            entry("0x401010", "function", "disputes", "size = 0x10\n")
                .replace("summary = \"s\"", "summary = \"not what H3API calls it\""),
            entry("0x401020", "function", "silent", "size = 0x10\n"),
            entry("0x63B850", "global", "silent_data", ""),
        ));
        let h3api: H3ApiFile = toml::from_str(
            "function = [\n{ va = 0x401000, name = \"H3A::X\", conv = \"thiscall\" },\n{ va = 0x401010, name = \"H3A::Y\" },\n{ va = 0x401020, name = \"H3A::Z\" },\n]\ndata = [ { va = 0x63B850, name = \"H3Table\" } ]\n",
        )
        .unwrap();
        let problems = h3api_problems(&m, &h3api);
        assert_eq!(problems.len(), 2, "{problems:?}");
        assert!(problems[0].contains("0x401020 silent: H3API names it H3A::Z"));
        assert!(problems[1].contains("0x63B850 silent_data: H3API names it H3Table"));
    }

    #[test]
    fn finds_citations_in_the_image_only() {
        let img = map("").image;
        let text = "0x5A8090, (0x4448F0) 0x400000 0x5A8090A x0x401000 0x40100 0x6B6000 0x63b850.";
        let found: Vec<u32> = cited(text, &img).into_iter().collect();
        assert_eq!(found, vec![0x4448F0, 0x5A8090, 0x63B850]);
        assert!(cited("0x401000", &img).contains(&0x401000));
    }

    #[test]
    fn reports_uncovered_citations() {
        let m = map(&format!(
            "{}{}{}",
            entry("0x401000", "function", "f", "size = 0x10\n"),
            entry("0x402000", "site", "s", ""),
            entry("0x63B850", "global", "g", ""),
        ));
        let mut citations = BTreeMap::new();
        for va in [0x40100F, 0x401010, 0x402000, 0x402001, 0x63B853, 0x63B854] {
            citations.insert(va, BTreeSet::from(["docs/a.md".to_owned()]));
        }
        let missing: Vec<u32> = uncovered(&m, &citations)
            .into_iter()
            .map(|(va, _)| va)
            .collect();
        assert_eq!(missing, vec![0x401010, 0x402001, 0x63B854]);
    }

    #[test]
    fn checks_a_tree() {
        let dir = TempDir::new("exemap");
        dir.write(
            MAP,
            &format!(
                "{IMAGE}\n{}",
                entry("0x401000", "function", "f", "size = 0x10\n")
            ),
        );
        dir.write(STRUCTS, "");
        dir.write(H3API, "function = []\n");
        dir.write("docs/a.md", "see 0x401004 and 0x5A8090\n");
        dir.write("docs/exe/README.md", "0x50C7C0 is H3API's\n");
        dir.write("crates/c/src/lib.rs", "// 0x401008\n");
        dir.write("crates/c/notes.txt", "0x5A8091\n");
        let problems = check(dir.path()).unwrap();
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(problems[0].starts_with("0x5A8090 is cited in docs/a.md"));
        dir.write(H3API, "not toml");
        assert!(check(dir.path()).is_err());
    }

    #[test]
    fn the_real_map_passes() {
        let problems = check(&workspace_root()).unwrap();
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }
}
