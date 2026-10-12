//! The real-asset gate (E17 M0 T3; extended by M1: every entry decodes, the inventory report). It
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

use h3_assets::archive::{self, Inventory};
use h3_formats::lod::LodArchive;

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
}

impl Options {
    /// Parses `[--install <dir>] [--out <dir>]`; the defaults are `$H3_INSTALL`, then the
    /// repository's `original-data/install`, and `clients/out`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        // An adapter's ambient input, read once here.
        let env = std::env::var_os("H3_INSTALL").map(PathBuf::from);
        Self::parse_with(args, env)
    }

    fn parse_with(args: &[String], env: Option<PathBuf>) -> Result<Self, String> {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut install = None;
        let mut out = None;
        let mut rest = args.iter();
        while let Some(flag) = rest.next() {
            let slot = match flag.as_str() {
                "--install" => &mut install,
                "--out" => &mut out,
                other => return Err(format!("gate: unknown option `{other}`")),
            };
            let value = rest
                .next()
                .ok_or_else(|| format!("gate: `{flag}` needs a directory"))?;
            *slot = Some(PathBuf::from(value));
        }
        Ok(Self {
            install: install
                .or(env)
                .unwrap_or_else(|| manifest.join("../../../original-data/install")),
            out: out.unwrap_or_else(|| manifest.join("../../out")),
        })
    }
}

/// One archive, opened.
struct Opened {
    name: &'static str,
    path: PathBuf,
    bytes: u64,
    read: Duration,
    parse: Duration,
    inventory: Inventory,
    listing: String,
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
    print!("{}", report(&options.install, &opened));
    let listing_path = options.out.join("gate").join("entries.tsv");
    let mut listing = String::from("archive\tname\ttype\tsize\tcompressed_size\n");
    for archive in &opened {
        listing.push_str(&archive.listing);
    }
    write(&listing_path, &listing)?;
    let shown = fs::canonicalize(&listing_path).unwrap_or(listing_path);
    println!("gate: every entry listed in {}", shown.display());
    Ok(())
}

/// Reads and parses one archive, timing both.
fn open(data: &Path, name: &'static str) -> Result<Opened, String> {
    let path = find_ci(data, name).map_err(|e| format!("{}/{name}: {e}", data.display()))?;
    let (bytes, read) = timed(|| fs::read(&path));
    let bytes = bytes.map_err(|e| format!("{}: {e}", path.display()))?;
    let (archive, parse) = timed(|| LodArchive::parse(&bytes));
    let archive = archive.map_err(|e| format!("{}: {e:?}", path.display()))?;
    let mut listing = String::new();
    for e in archive.entries() {
        // Writing into a String cannot fail.
        let _ = writeln!(
            listing,
            "{name}\t{}\t{}\t{}\t{}",
            e.name, e.type_tag, e.size, e.compressed_size
        );
    }
    Ok(Opened {
        name,
        path,
        bytes: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
        read,
        parse,
        inventory: archive::inventory(&archive),
        listing,
    })
}

/// The gate's report: one block per archive, then the totals.
fn report(install: &Path, opened: &[Opened]) -> String {
    let mut out = format!("gate: {}\n", install.display());
    let mut entries = 0u64;
    let mut total = Duration::ZERO;
    for a in opened {
        let inv = &a.inventory;
        entries += u64::from(inv.total.entries);
        total += a.read + a.parse;
        let _ = writeln!(
            out,
            "{:<13} kind {:>3}  {:>5} entries ({} compressed, {} MiB unpacked)  {} MiB read in {} ms, \
             parsed in {} ms  [{}]",
            a.name,
            inv.kind,
            inv.total.entries,
            inv.total.compressed,
            inv.total.bytes >> 20,
            a.bytes >> 20,
            a.read.as_millis(),
            a.parse.as_millis(),
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
    out
}

/// The entry of `dir` named `name`, ignoring ASCII case (installations differ in case).
fn find_ci(dir: &Path, name: &str) -> io::Result<PathBuf> {
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
fn timed<T>(f: impl FnOnce() -> T) -> (T, Duration) {
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
        let o = Options::parse_with(
            &args(&["--install", "/a", "--out", "/b"]),
            Some("/e".into()),
        )
        .unwrap();
        assert_eq!(
            (o.install, o.out),
            (PathBuf::from("/a"), PathBuf::from("/b"))
        );
    }

    #[test]
    fn options_reject_unknown_flags_and_missing_values() {
        assert!(Options::parse_with(&args(&["--bogus"]), None).is_err());
        assert!(Options::parse_with(&args(&["--out"]), None).is_err());
    }

    #[test]
    fn the_gate_opens_a_synthetic_installation() {
        let root = std::env::temp_dir().join(format!("h3-view-gate-{}", std::process::id()));
        let data = root.join("install").join("DATA");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&data).unwrap();
        for (i, name) in ARCHIVES.iter().enumerate() {
            let lod = build(
                200,
                &[("X.DEF", &[1, 2, 3], i % 2 == 0), ("y.pcx", &[], false)],
            );
            fs::write(data.join(name.to_ascii_lowercase()), lod).unwrap();
        }
        let options = Options {
            install: root.join("install"),
            out: root.join("out"),
        };
        gate(&options).unwrap();
        let listing = fs::read_to_string(root.join("out/gate/entries.tsv")).unwrap();
        assert_eq!(listing.lines().count(), 1 + 2 * ARCHIVES.len());
        assert!(listing.contains("H3ab_spr.lod\ty.pcx\t"));
        fs::remove_file(data.join("h3sprite.lod")).unwrap();
        assert!(gate(&options).unwrap_err().contains("H3sprite.lod"));
        fs::remove_dir_all(&root).unwrap();
    }
}
