//! The real-asset gate (E17 M0 T3; M1 T2 and T3: every PCX, PAL, DEF and MSK decodes, the
//! inventory report in `<out>/gate/report.md`, every entry with its verdict in `entries.tsv`;
//! M2 T5b: every sound decodes to SDL's samples, `sounds.tsv`; M3 T4b-1: every key of the string
//! catalogue is read from the text tables). It
//! runs only where the user's files are (ADR-0009): a copy of the installation, never the
//! installation itself, which it would only read anyway. It reads whole archives, since
//! `h3-formats` parses only a whole archive; with the table-only reader (CLT-Q2) it measures the
//! client's real start instead: the directories only (ADR-0020 point 7).

use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use std::collections::{BTreeMap, BTreeSet};

use h3_assets::archive;
use h3_assets::check::{Decoded, EntryCheck, check_entry};
use h3_assets::survey::{self, Mention, Mentions};
use h3_formats::lod::LodArchive;
use h3_ui::text::Catalogue;
use h3_ui::text::pack::Pack;

use crate::{assets, sounds, text};

/// The four LOD archives of the Complete edition, in the order the gate reports them.
const ARCHIVES: [&str; 4] = [
    "H3bitmap.lod",
    "H3sprite.lod",
    "H3ab_bmp.lod",
    "H3ab_spr.lod",
];

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    install: PathBuf,
    out: PathBuf,
    tables: PathBuf,
    keys: PathBuf,
}

impl Options {
    /// Parses `[--install <dir>] [--out <dir>] [--tables <dir>] [--keys <file>]`; the defaults
    /// are `$H3_INSTALL`, then the repository's `original-data/install`, `clients/out`,
    /// `clients/assets` and `clients/text/keys.toml`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        // An adapter's ambient input, read once here.
        let env = std::env::var_os("H3_INSTALL").map(PathBuf::from);
        Self::parse_with(args, env)
    }

    fn parse_with(args: &[String], env: Option<PathBuf>) -> Result<Self, String> {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut install = None;
        let mut out = None;
        let mut tables = None;
        let mut keys = None;
        let mut rest = args.iter();
        while let Some(flag) = rest.next() {
            let slot = match flag.as_str() {
                "--install" => &mut install,
                "--out" => &mut out,
                "--tables" => &mut tables,
                "--keys" => &mut keys,
                other => return Err(format!("gate: unknown option `{other}`")),
            };
            let value = rest
                .next()
                .ok_or_else(|| format!("gate: `{flag}` needs a path"))?;
            *slot = Some(PathBuf::from(value));
        }
        Ok(Self {
            install: install
                .or(env)
                .unwrap_or_else(|| manifest.join("../../../original-data/install")),
            out: out.unwrap_or_else(|| manifest.join("../../out")),
            tables: tables.unwrap_or_else(assets::default_dir),
            keys: keys.unwrap_or_else(text::default_keys),
        })
    }
}

/// One archive, read.
struct Opened {
    name: &'static str,
    path: PathBuf,
    bytes: Vec<u8>,
    read: Duration,
}

/// One entry and what the gate found.
struct Checked {
    archive: usize,
    name: String,
    tag: u32,
    size: u32,
    compressed_size: u32,
    check: Result<EntryCheck, String>,
}

/// Wall time per step of the decoding pass.
#[derive(Default)]
struct Times {
    inflate: Duration,
    decode: Duration,
    compare: Duration,
}

pub fn run(options: &Options) -> ExitCode {
    match gate(options) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("gate: {message}");
            ExitCode::FAILURE
        }
    }
}

fn gate(options: &Options) -> Result<(), String> {
    let data = find_ci(&options.install, "Data")
        .map_err(|e| format!("{}: {e}", options.install.display()))?;
    let mut opened = Vec::new();
    for name in ARCHIVES {
        opened.push(open(&data, name)?);
    }
    let mut archives = Vec::new();
    let mut parse_times = Vec::new();
    for a in &opened {
        let (archive, parse) = timed(|| LodArchive::parse(&a.bytes));
        archives.push(archive.map_err(|e| format!("{}: {e:?}", a.path.display()))?);
        parse_times.push(parse);
    }
    let mut times = Times::default();
    let mut mentions = Mentions::default();
    let mut checked = Vec::new();
    for (i, archive) in archives.iter().enumerate() {
        for e in archive.entries() {
            let (bytes, inflate) = timed(|| archive.read(e));
            times.inflate += inflate;
            let check = match bytes {
                Ok(bytes) => {
                    if archive::extension(&e.name) == "TXT" {
                        mentions.scan(&bytes);
                    }
                    let (check, decode) = timed(|| check_entry(&e.name, e.type_tag, &bytes));
                    times.decode += decode;
                    Ok(check)
                }
                Err(error) => Err(format!("inflate {error:?}")),
            };
            checked.push(Checked {
                archive: i,
                name: e.name.clone(),
                tag: e.type_tag,
                size: e.size,
                compressed_size: e.compressed_size,
                check,
            });
        }
    }
    // The game's and the map editor's executables: an entry only the editor names is not dead.
    let mut scanned = vec![String::from("the text tables")];
    for exe in ["Heroes3.exe", "h3maped.exe"] {
        match find_ci(&options.install, exe).and_then(fs::read) {
            Ok(bytes) => {
                mentions.scan(&bytes);
                scanned.push(String::from(exe));
            }
            Err(e) => scanned.push(format!("not {exe} ({e})")),
        }
    }
    let exe_note = scanned.join(", ");
    let listings: Vec<Vec<String>> = archives
        .iter()
        .map(|a| a.entries().iter().map(|e| e.name.clone()).collect())
        .collect();
    let dups = survey::duplicates(&listings);
    let (same, compare) = timed(|| identical(&archives, &dups));
    times.compare = compare;
    let all: BTreeSet<String> = dups
        .keys()
        .cloned()
        .chain(listings.iter().flatten().map(|n| n.to_ascii_uppercase()))
        .collect();

    let summary = summary(
        &options.install,
        &opened,
        &archives,
        &parse_times,
        &checked,
        &times,
    );
    print!("{summary}");
    let mut listing = String::from(
        "archive\tname\ttype\tsize\tcompressed_size\tdecoded\tfindings\tmention\tduplicate\n",
    );
    for c in &checked {
        let upper = c.name.to_ascii_uppercase();
        let duplicate = same
            .get(&upper)
            .map_or("-", |&s| if s { "identical" } else { "different" });
        let _ = writeln!(
            listing,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:?}\t{duplicate}",
            ARCHIVES[c.archive],
            c.name,
            c.tag,
            c.size,
            c.compressed_size,
            decoded_text(&c.check),
            findings_text(&c.check),
            mentions.mention(&c.name, &all),
        );
    }
    let gate_dir = options.out.join("gate");
    write(&gate_dir.join("entries.tsv"), &listing)?;
    let report = report(&summary, &exe_note, &checked, &mentions, &all, &dups, &same);
    write(&gate_dir.join("report.md"), &report)?;
    let shown = fs::canonicalize(&gate_dir).unwrap_or(gate_dir);
    println!(
        "gate: every entry listed in {0}/entries.tsv, the inventory in {0}/report.md",
        shown.display()
    );
    let failed = checked
        .iter()
        .filter(|c| !matches!(&c.check, Ok(k) if k.decoded.is_ok()))
        .count();
    if failed > 0 {
        return Err(format!(
            "{failed} entries did not decode (entries.tsv, column `decoded`)"
        ));
    }
    // The asset tables (E17 M1 T4): every name found where the game searches it, and decoded.
    let tables = assets::load(&options.tables, assets::ALL)?;
    let decoded = checked
        .iter()
        .map(|c| {
            let ok = matches!(&c.check, Ok(k) if k.decoded.is_ok());
            ((c.archive, c.name.to_ascii_uppercase()), ok)
        })
        .collect();
    let (names, problems) = assets::check(&tables, &decoded);
    if !problems.is_empty() {
        return Err(format!("asset tables: {}", problems.join("; ")));
    }
    println!("gate: the asset tables' {names} names are found and decode");
    // The string catalogue (E17 M3 T4b-1): every original form of every key read from the text
    // tables, where the game searches them, its specifiers as the key names them.
    let keys = text::read_keys(&options.keys, &tables)?;
    let entry = |name: &str| {
        for &a in assets::search(&name.to_ascii_uppercase()) {
            if let Some(bytes) = archives[a]
                .read_by_name(name)
                .map_err(|e| format!("{}:{name}: {e:?}", ARCHIVES[a]))?
            {
                return Ok(bytes);
            }
        }
        Err(format!("{name}: in no archive"))
    };
    let exe = text::executable(&options.install)?;
    let texts = text::tables(&keys, entry, &exe)?;
    let (original, problems) = Pack::original(&keys, &texts);
    if !problems.is_empty() {
        let shown: Vec<String> = problems.iter().map(ToString::to_string).collect();
        return Err(format!("the key catalogue: {}", shown.join("; ")));
    }
    println!(
        "gate: the key catalogue's {} keys: {} forms read from {} text tables",
        keys.keys.len(),
        original.messages.len(),
        texts.len()
    );
    // The battle log's lines (E17 M3 T4b-2a): composed from the original's text, every argument
    // in its place (a catalogue without the log's keys, as the tests' own, has none to compose).
    if keys.keys.contains_key("battle.log.damage") {
        let samples = text::log_samples(&Catalogue {
            packs: vec![original],
        });
        let problems = text::log_problems(&samples);
        if !problems.is_empty() {
            return Err(format!("the battle log: {}", problems.join("; ")));
        }
        println!("gate: the battle log: {} lines composed", samples.len());
    }
    // The sound archives (E17 M2 T5b): every WAV decodes to SDL's samples.
    print!(
        "{}",
        sounds::check(&data, &options.out, &tables.sound_names())?
    );
    Ok(())
}

/// Reads one archive, timing it.
fn open(data: &Path, name: &'static str) -> Result<Opened, String> {
    let path = find_ci(data, name).map_err(|e| format!("{}/{name}: {e}", data.display()))?;
    let (bytes, read) = timed(|| fs::read(&path));
    let bytes = bytes.map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Opened {
        name,
        path,
        bytes,
        read,
    })
}

/// For each duplicate name, whether every copy has the same bytes.
fn identical(
    archives: &[LodArchive<'_>],
    dups: &BTreeMap<String, Vec<usize>>,
) -> BTreeMap<String, bool> {
    let mut copies: BTreeMap<&str, Vec<Option<Vec<u8>>>> = BTreeMap::new();
    for archive in archives {
        for e in archive.entries() {
            if let Some((name, _)) = dups.get_key_value(&e.name.to_ascii_uppercase()) {
                copies.entry(name).or_default().push(archive.read(e).ok());
            }
        }
    }
    copies
        .into_iter()
        .map(|(name, bytes)| {
            let same = bytes.windows(2).all(|w| w[0].is_some() && w[0] == w[1]);
            (name.to_owned(), same)
        })
        .collect()
}

fn decoded_text(check: &Result<EntryCheck, String>) -> String {
    match check {
        Err(e) => format!("error {e}"),
        Ok(c) => match &c.decoded {
            Err(e) => format!("error {e:?}"),
            Ok(Decoded::Pcx { width, height, rgb }) => {
                format!("pcx {}-bit {width}x{height}", if *rgb { 24 } else { 8 })
            }
            Ok(Decoded::Pal { chunks }) => format!("pal +{chunks} chunks"),
            Ok(Decoded::Def {
                kind,
                groups,
                frames,
                encodings,
            }) => format!(
                "def 0x{kind:02X} {groups} groups {frames} frames enc {}/{}/{}/{}",
                encodings[0], encodings[1], encodings[2], encodings[3]
            ),
            Ok(Decoded::Fnt { height, glyphs }) => format!("fnt height {height}, {glyphs} glyphs"),
            Ok(Decoded::Msk { width, height }) => format!("msk {width}x{height}"),
            Ok(Decoded::Skipped) => String::from("skipped"),
        },
    }
}

fn findings_text(check: &Result<EntryCheck, String>) -> String {
    match check {
        Ok(c) if !c.findings.is_empty() => c
            .findings
            .iter()
            .map(|f| format!("{f:?}"))
            .collect::<Vec<_>>()
            .join("; "),
        _ => String::from("-"),
    }
}

/// The printed summary: one block per archive, then the decoding pass.
fn summary(
    install: &Path,
    opened: &[Opened],
    archives: &[LodArchive<'_>],
    parse_times: &[Duration],
    checked: &[Checked],
    times: &Times,
) -> String {
    let mut out = format!("gate: {}\n", install.display());
    let mut entries = 0u64;
    let mut total = Duration::ZERO;
    for ((a, parsed), &parse) in opened.iter().zip(archives).zip(parse_times) {
        let inv = archive::inventory(parsed);
        entries += u64::from(inv.total.entries);
        total += a.read + parse;
        let _ = writeln!(
            out,
            "{:<13} kind {:>3}  {:>5} entries ({} compressed, {} MiB unpacked)  {} MiB read in {} ms, \
             parsed in {} ms  [{}]",
            a.name,
            inv.kind,
            inv.total.entries,
            inv.total.compressed,
            inv.total.bytes >> 20,
            a.bytes.len() >> 20,
            a.read.as_millis(),
            parse.as_millis(),
            a.path.display(),
        );
        let by_ext: Vec<String> = inv
            .by_extension
            .iter()
            .map(|(ext, t)| {
                let ext = if ext.is_empty() { "(none)" } else { ext };
                format!("{ext} {}", t.entries)
            })
            .collect();
        let _ = writeln!(out, "{:<13} {}", "", by_ext.join(", "));
    }
    let _ = writeln!(
        out,
        "gate: {} archives, {entries} entries, opened in {} ms",
        opened.len(),
        total.as_millis()
    );
    let mut per_ext: BTreeMap<String, (u32, u32, u32)> = BTreeMap::new();
    for c in checked {
        let slot = per_ext.entry(archive::extension(&c.name)).or_default();
        match &c.check {
            Ok(k) if k.decoded == Ok(Decoded::Skipped) => slot.2 += 1,
            Ok(k) if k.decoded.is_ok() => slot.0 += 1,
            _ => slot.1 += 1,
        }
    }
    let line: Vec<String> = per_ext
        .iter()
        .filter(|(_, (ok, bad, _))| ok + bad > 0)
        .map(|(ext, (ok, bad, _))| format!("{ext} {ok} ok, {bad} failed"))
        .collect();
    let _ = writeln!(
        out,
        "gate: decoded {}; every entry inflated in {} ms, decoded in {} ms; duplicates compared in {} ms",
        line.join("; "),
        times.inflate.as_millis(),
        times.decode.as_millis(),
        times.compare.as_millis()
    );
    out
}

/// The inventory report (Markdown, local only: it names the user's entries).
fn report(
    summary: &str,
    exe_note: &str,
    checked: &[Checked],
    mentions: &Mentions,
    all: &BTreeSet<String>,
    dups: &BTreeMap<String, Vec<usize>>,
    same: &BTreeMap<String, bool>,
) -> String {
    let mut out = String::from("# The real-asset gate's inventory (E17 M1 T3)\n\n```\n");
    out.push_str(summary);
    out.push_str("```\n\n## Findings\n\n");
    let mut findings: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut failures = Vec::new();
    for c in checked {
        match &c.check {
            Ok(k) => {
                for f in &k.findings {
                    let kind = format!("{f:?}");
                    let kind = kind.split([' ', '{']).next().unwrap_or_default().to_owned();
                    findings
                        .entry(kind)
                        .or_default()
                        .push(format!("{}:{} ({f:?})", ARCHIVES[c.archive], c.name));
                }
                if let Err(e) = &k.decoded {
                    failures.push(format!("{}:{} ({e:?})", ARCHIVES[c.archive], c.name));
                }
            }
            Err(e) => failures.push(format!("{}:{} ({e})", ARCHIVES[c.archive], c.name)),
        }
    }
    let _ = writeln!(out, "- Entries that did not decode: {}", failures.len());
    for f in &failures {
        let _ = writeln!(out, "  - {f}");
    }
    for (kind, list) in &findings {
        let _ = writeln!(out, "- {kind}: {}", list.len());
        for item in list.iter().take(40) {
            let _ = writeln!(out, "  - {item}");
        }
    }
    let mut tags: BTreeMap<(String, u32), u32> = BTreeMap::new();
    let mut def_kinds: BTreeMap<u32, u32> = BTreeMap::new();
    let mut encodings = [0u32; 4];
    for c in checked {
        *tags
            .entry((archive::extension(&c.name), c.tag))
            .or_default() += 1;
        if let Ok(EntryCheck {
            decoded: Ok(Decoded::Def {
                kind, encodings: e, ..
            }),
            ..
        }) = &c.check
        {
            *def_kinds.entry(*kind).or_default() += 1;
            for (sum, n) in encodings.iter_mut().zip(e) {
                *sum += n;
            }
        }
    }
    out.push_str("\n## Extensions and type tags\n\n| extension | tag | entries |\n|---|---|---|\n");
    for ((ext, tag), n) in &tags {
        let _ = writeln!(out, "| {ext} | {tag} | {n} |");
    }
    out.push_str("\nDEF types (header): ");
    let kinds: Vec<String> = def_kinds
        .iter()
        .map(|(k, n)| format!("0x{k:02X} × {n}"))
        .collect();
    out.push_str(&kinds.join(", "));
    let _ = writeln!(
        out,
        ".\nDEF frames by encoding (distinct frames): 0 × {}, 1 × {}, 2 × {}, 3 × {}.",
        encodings[0], encodings[1], encodings[2], encodings[3]
    );
    let identical = same.values().filter(|&&s| s).count();
    let _ = writeln!(
        out,
        "\n## Duplicates\n\n{} names appear more than once ({identical} with identical bytes, {} different).\n",
        dups.len(),
        dups.len() - identical
    );
    let mut pairs: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    for (name, holders) in dups {
        let key = holders
            .iter()
            .map(|&i| ARCHIVES[i])
            .collect::<Vec<_>>()
            .join(" + ");
        let slot = pairs.entry(key).or_default();
        if same.get(name) == Some(&true) {
            slot.0 += 1;
        } else {
            slot.1 += 1;
        }
    }
    for (key, (ident, diff)) in &pairs {
        let _ = writeln!(out, "- {key}: {ident} identical, {diff} different");
    }
    out.push_str("\nDifferent copies (the list in entries.tsv, column `duplicate`): ");
    let different: Vec<&str> = same
        .iter()
        .filter(|(_, s)| !**s)
        .map(|(n, _)| n.as_str())
        .collect();
    out.push_str(
        &different
            .iter()
            .take(60)
            .copied()
            .collect::<Vec<_>>()
            .join(", "),
    );
    if different.len() > 60 {
        let _ = write!(out, ", … ({} more)", different.len() - 60);
    }
    let _ = writeln!(
        out,
        "\n\n## Mentions\n\nScanned: {exe_note}: {} exact names, {} patterns ({}).\n",
        mentions.exact_count(),
        mentions.patterns().len(),
        mentions
            .patterns()
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    );
    let mut by_mention: BTreeMap<(String, String), u32> = BTreeMap::new();
    let mut unmentioned: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for name in all {
        let m = mentions.mention(name, all);
        let ext = archive::extension(name);
        *by_mention
            .entry((ext.clone(), format!("{m:?}")))
            .or_default() += 1;
        if m == Mention::None {
            unmentioned.entry(ext).or_default().push(name.clone());
        }
    }
    out.push_str("| extension | mention | names |\n|---|---|---|\n");
    for ((ext, m), n) in &by_mention {
        let _ = writeln!(out, "| {ext} | {m} | {n} |");
    }
    out.push_str("\nNot mentioned (candidates for dead entries; names built at run time and data the survey does not read are not covered):\n\n");
    for (ext, names) in &unmentioned {
        let _ = writeln!(out, "- {ext} ({}): {}", names.len(), names.join(", "));
    }
    let broken: Vec<&String> = all.iter().filter(|n| survey::breaks_pattern(n)).collect();
    let _ = writeln!(
        out,
        "\n## Names\n\n{} distinct names; {} break the 8.3 pattern{}",
        all.len(),
        broken.len(),
        if broken.is_empty() {
            String::from(".")
        } else {
            format!(
                ": {}",
                broken
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    );
    out
}

/// The entry of `dir` named `name`, ignoring ASCII case (installations differ in case).
pub fn find_ci(dir: &Path, name: &str) -> io::Result<PathBuf> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry
            .file_name()
            .to_string_lossy()
            .eq_ignore_ascii_case(name)
        {
            return Ok(entry.path());
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "not found"))
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Runs `f` and measures its wall time.
#[expect(
    clippy::disallowed_types,
    reason = "an adapter's clock: the gate reports load times (clippy.toml forbids it in logic)"
)]
pub(crate) fn timed<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let start = std::time::Instant::now();
    let value = f();
    (value, start.elapsed())
}

#[cfg(test)]
mod tests {
    use super::*;
    use h3_formats::lod::build;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn options_default_to_the_environment_then_the_repository() {
        let o = Options::parse_with(&[], Some(PathBuf::from("/env"))).unwrap();
        assert_eq!(o.install, PathBuf::from("/env"));
        assert!(o.out.ends_with("../../out"));
        let o = Options::parse_with(&[], None).unwrap();
        assert!(o.install.ends_with("original-data/install"));
        assert!(o.tables.ends_with("../../assets"));
        assert!(o.keys.ends_with("text/keys.toml"));
        let o = Options::parse_with(
            &args(&[
                "--install",
                "/a",
                "--out",
                "/b",
                "--tables",
                "/c",
                "--keys",
                "/d",
            ]),
            Some("/e".into()),
        )
        .unwrap();
        assert_eq!(
            (o.install, o.out, o.tables, o.keys),
            (
                PathBuf::from("/a"),
                PathBuf::from("/b"),
                PathBuf::from("/c"),
                PathBuf::from("/d")
            )
        );
    }

    #[test]
    fn options_reject_unknown_flags_and_missing_values() {
        assert!(Options::parse_with(&args(&["--bogus"]), None).is_err());
        assert!(Options::parse_with(&args(&["--out"]), None).is_err());
    }

    #[test]
    fn the_gate_checks_a_synthetic_installation() {
        use h3_assets::build;
        use h3_assets::def::{Encoding, Frame};
        use h3_assets::palette::Palette;

        let root = std::env::temp_dir().join(format!("h3-view-gate-{}", std::process::id()));
        let install = root.join("install");
        let data = install.join("DATA");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&data).unwrap();
        let frame = Frame {
            encoding: Encoding::Rows,
            full_width: 2,
            full_height: 1,
            left: 0,
            top: 0,
            width: 2,
            height: 1,
            pixels: vec![1, 2],
            runs: vec![false, false],
            short_header: false,
        };
        let def = build::def(
            0x42,
            (2, 1),
            &Palette::default(),
            &[(0, &[("f", 0)])],
            &[frame],
        );
        let pcx = build::pcx_indexed(1, 1, &[0], &Palette::default());
        for (i, name) in ARCHIVES.iter().enumerate() {
            let lod = build(
                200,
                &[
                    ("X.DEF", &def, i % 2 == 0),
                    ("y.pcx", &pcx, false),
                    ("t.txt", b"see y.pcx", true),
                ],
            );
            fs::write(data.join(name.to_ascii_lowercase()), lod).unwrap();
        }
        fs::write(install.join("HEROES3.EXE"), b"\0x.def\0").unwrap();
        let tables = root.join("tables");
        fs::create_dir_all(&tables).unwrap();
        let header =
            |id: &str, depends: &str| format!("[table]\nid = \"{id}\"\ndepends = [{depends}]\n");
        let base = header("base", "")
            + "[battle]\ngroups = { moving = 0, mouse_over = 1, standing = 2, hit = 3, defend = 4, death = 5, strike = [11, 12, 13], shoot = [14, 15, 16], breath = [17, 18, 19], move_start = 20, move_end = 21, turn_left = [7, 10], turn_right = [9, 8] }\n\
               hero_groups = { standing = 0, idle = 1, defeat = 2, victory = 3, cast = 4 }\n\
               flags = [\"x.def\", \"x.def\"]\n\
               magic = [{ def = \"x.def\", place = 1 }]\nheal_sound = \"x\"\n\
               [[creature]]\nkey = \"x\"\noriginal = 0\ndef = \"x.def\"\ntimes = [0, 0, 0, 0]\n\
               [[battlefield]]\nkey = \"dirt\"\nbackgrounds = [\"Y.PCX\"]\n";
        fs::write(tables.join("base.toml"), &base).unwrap();
        fs::write(tables.join("ab.toml"), header("ab", "\"base\"")).unwrap();
        fs::write(tables.join("sod.toml"), header("sod", "\"ab\"")).unwrap();
        // The key catalogue: a key and the creatures' names in the archives' `t.txt`.
        let keys = root.join("keys.toml");
        fs::write(
            &keys,
            "[[key]]\nid = \"a\"\noriginal = [{ at = \"T:0\" }]\n\
             [[family]]\nid = \"creature\"\ntable = \"T\"\nfirst = 0\nforms = { one = 0 }\n",
        )
        .unwrap();
        let options = Options {
            install,
            out: root.join("out"),
            tables: tables.clone(),
            keys: keys.clone(),
        };
        gate(&options).unwrap();
        let listing = fs::read_to_string(root.join("out/gate/entries.tsv")).unwrap();
        assert_eq!(listing.lines().count(), 1 + 3 * ARCHIVES.len());
        assert!(listing.contains("H3ab_spr.lod\ty.pcx\t2\t"));
        assert!(listing.contains("def 0x42 1 groups 1 frames enc 0/1/0/0\tTypeTag"));
        assert!(listing.contains("\tExact\tidentical"));
        let report = fs::read_to_string(root.join("out/gate/report.md")).unwrap();
        assert!(
            report.contains("3 names appear more than once (3 with identical bytes, 0 different)")
        );
        // A key the text tables lack fails the gate, and so does a table no archive holds.
        fs::write(
            &keys,
            "[[key]]\nid = \"a\"\noriginal = [{ at = \"T:9\" }]\n",
        )
        .unwrap();
        assert!(
            gate(&options)
                .unwrap_err()
                .contains("T.TXT has no such record")
        );
        fs::write(
            &keys,
            "[[key]]\nid = \"a\"\noriginal = [{ at = \"U:0\" }]\n",
        )
        .unwrap();
        assert!(gate(&options).unwrap_err().contains("U.TXT: in no archive"));
        fs::write(&keys, "[[key]]\nid = \"a\"\n").unwrap();
        // A name of the tables the archives lack fails the gate.
        fs::write(tables.join("base.toml"), base.replace("x.def", "z.def")).unwrap();
        assert!(gate(&options).unwrap_err().contains("Z.DEF: in no archive"));
        // A broken entry fails the gate; the listing is still written.
        let lod = build(200, &[("bad.def", &[1, 2, 3], false)]);
        fs::write(data.join("h3sprite.lod"), lod).unwrap();
        assert!(
            gate(&options)
                .unwrap_err()
                .contains("1 entries did not decode")
        );
        let listing = fs::read_to_string(root.join("out/gate/entries.tsv")).unwrap();
        assert!(listing.contains("bad.def\t2\t3\t0\terror Truncated"));
        fs::remove_file(data.join("h3sprite.lod")).unwrap();
        assert!(gate(&options).unwrap_err().contains("H3sprite.lod"));
        fs::remove_dir_all(&root).unwrap();
    }
}
