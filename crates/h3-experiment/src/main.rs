//! `h3-experiment <spec.toml> --install <dir> --map <out.h3m> --template <out.md>`
//!
//! Compiles an experiment spec into a SoD map and the results template (E04a T2,
//! `docs/experiments/README.md`). The installation (or its copy, `original-data/install`) is only
//! read; outputs inside it are refused.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_experiment::{Tables, build, parse};

const USAGE: &str = "usage: h3-experiment <spec.toml> --install <dir> --map <out.h3m> --template <out.md>\n       h3-experiment --install <dir> --list heroes|creatures|skills|artifacts";

struct Args {
    spec: PathBuf,
    install: PathBuf,
    map: PathBuf,
    template: PathBuf,
}

fn parse_args() -> Result<Args, String> {
    let mut spec = None;
    let (mut install, mut map, mut template) = (None, None, None);
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--install" => install = args.next().map(PathBuf::from),
            "--map" => map = args.next().map(PathBuf::from),
            "--template" => template = args.next().map(PathBuf::from),
            "-h" | "--help" => return Err(USAGE.into()),
            other if !other.starts_with("--") && spec.is_none() => {
                spec = Some(PathBuf::from(other))
            }
            other => return Err(format!("unknown argument `{other}`\n{USAGE}")),
        }
    }
    Ok(Args {
        spec: spec.ok_or(USAGE)?,
        install: install.ok_or(USAGE)?,
        map: map.ok_or(USAGE)?,
        template: template.ok_or(USAGE)?,
    })
}

fn inside(path: &Path, dir: &Path) -> bool {
    let canon = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let parent = path.parent().map_or_else(|| PathBuf::from("."), canon);
    parent.starts_with(canon(dir))
}

/// Prints a table's ids and names (for spec authors).
fn list(install: &Path, what: &str) -> Result<(), String> {
    let tables = Tables::read(install)?;
    let names: Vec<String> = match what {
        "heroes" => (0..tables.heroes.len())
            .map(|i| {
                format!(
                    "{} (class {}, specialty {})",
                    tables.heroes[i],
                    tables.hero_classes[i],
                    tables.specialty_text(i)
                )
            })
            .collect(),
        "creatures" => tables.creatures.clone(),
        "skills" => tables.skills.clone(),
        "artifacts" => tables.artifacts.iter().map(|a| a.name.clone()).collect(),
        other => return Err(format!("cannot list `{other}`\n{USAGE}")),
    };
    for (i, n) in names.iter().enumerate() {
        println!("{i:3} {n}");
    }
    Ok(())
}

fn run() -> Result<(), String> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    if let Some(pos) = raw.iter().position(|a| a == "--list") {
        let what = raw.get(pos + 1).ok_or(USAGE)?;
        let install = raw
            .iter()
            .position(|a| a == "--install")
            .and_then(|i| raw.get(i + 1))
            .ok_or(USAGE)?;
        return list(Path::new(install), what);
    }
    let args = parse_args()?;
    for out in [&args.map, &args.template] {
        if inside(out, &args.install) {
            return Err(format!(
                "{} is inside the installation, which is read-only",
                out.display()
            ));
        }
    }
    let text =
        std::fs::read_to_string(&args.spec).map_err(|e| format!("{}: {e}", args.spec.display()))?;
    let spec = parse(&text).map_err(|e| format!("{}: {e}", args.spec.display()))?;
    let tables = Tables::read(&args.install)?;
    let file = args
        .map
        .file_name()
        .map_or_else(String::new, |f| f.to_string_lossy().into_owned());
    let (map, template) = build(&spec, &tables, &file)?;
    let bytes = h3_formats::h3m::write(&map).map_err(|e| format!("map: {e}"))?;
    for out in [&args.map, &args.template] {
        if let Some(dir) = out.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
    }
    std::fs::write(&args.map, h3_formats::h3m::gzip(&bytes))
        .map_err(|e| format!("{}: {e}", args.map.display()))?;
    std::fs::write(&args.template, template)
        .map_err(|e| format!("{}: {e}", args.template.display()))?;
    println!(
        "wrote {} ({} encounters, {} bytes uncompressed) and {}",
        args.map.display(),
        spec.encounter.len(),
        bytes.len(),
        args.template.display()
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
