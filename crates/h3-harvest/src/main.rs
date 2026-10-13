//! `h3-harvest --install <dir> --out <packs dir> [--check]`
//!
//! Reads the installation (or a copy, e.g. `original-data/install`), writes
//! `<out>/{base,ab,sod}/core.toml` and `battle.toml` (pack format v35). With `--check` nothing
//! is written; the command fails if the files under `<out>` differ from a fresh harvest (used by
//! `cargo xtask verify --with-original-data`). The installation is only ever read: an `--out`
//! inside `--install` is refused.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_harvest::gapfill::{GAPFILL, GapFill};
use h3_harvest::{Inputs, harvest};

struct Args {
    install: PathBuf,
    out: PathBuf,
    check: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut install = None;
    let mut out = None;
    let mut check = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--install" => install = args.next().map(PathBuf::from),
            "--out" => out = args.next().map(PathBuf::from),
            "--check" => check = true,
            "-h" | "--help" => {
                return Err("usage: h3-harvest --install <dir> --out <packs dir> [--check]".into());
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    Ok(Args {
        install: install.ok_or("--install <dir> is required")?,
        out: out.ok_or("--out <dir> is required")?,
        check,
    })
}

fn inside(path: &Path, root: &Path) -> bool {
    match (std::fs::canonicalize(root), std::path::absolute(path)) {
        (Ok(root), Ok(path)) => {
            let path = std::fs::canonicalize(&path).unwrap_or(path);
            path.starts_with(root)
        }
        _ => false,
    }
}

fn run() -> Result<bool, String> {
    let args = parse_args()?;
    if inside(&args.out, &args.install) {
        return Err("refusing to write into the installation (--out is inside --install)".into());
    }
    let inputs = Inputs::read(&args.install)?;
    let gap = GapFill::parse(GAPFILL)?;
    let result = harvest(&inputs, &gap)?;
    for line in &result.report {
        eprintln!("note: {line}");
    }
    let mut same = true;
    let files = result
        .packs
        .iter()
        .flat_map(|(id, (core, battle))| [(id, "core.toml", core), (id, "battle.toml", battle)]);
    for (id, file, text) in files {
        let path = args.out.join(id).join(file);
        if args.check {
            let existing = std::fs::read_to_string(&path).unwrap_or_default();
            if existing.replace("\r\n", "\n") != *text {
                eprintln!("differs: {}", path.display());
                same = false;
            }
        } else {
            std::fs::create_dir_all(path.parent().unwrap_or(&args.out))
                .map_err(|e| format!("{}: {e}", path.display()))?;
            std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
            println!("wrote {}", path.display());
        }
    }
    if args.check && same {
        println!("harvest check: committed packs match the installation");
    }
    Ok(same)
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("h3-harvest: {e}");
            ExitCode::FAILURE
        }
    }
}
