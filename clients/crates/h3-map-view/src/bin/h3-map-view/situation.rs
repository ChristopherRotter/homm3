//! `situation`: a situation's map (E24 M1 T3b2): its file read (`h3_map_view::situation`), the
//! shipped maps its copies name read from the installation copy's `Maps`, the game's object
//! templates (`Objects.txt` in its bitmap archives) when it gives objects by their DEF (T4a), the
//! map written with the adventure lane's writer.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_map::Limits;
use h3_map::writer::{self, ObjectTemplate};
use h3_map_view::situation::Situation;

use crate::files;

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    file: PathBuf,
    install: PathBuf,
    out: PathBuf,
}

impl Options {
    /// Parses `<situation.toml> --out <map.h3m> [--install <dir>]`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let (mut file, mut install, mut out) = (None, None, None);
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            let slot = match arg.as_str() {
                "--install" => &mut install,
                "--out" => &mut out,
                other if !other.starts_with("--") && file.is_none() => {
                    file = Some(PathBuf::from(other));
                    continue;
                }
                other => return Err(format!("situation: unexpected `{other}`")),
            };
            let value = rest
                .next()
                .ok_or_else(|| format!("situation: `{arg}` needs a path"))?;
            *slot = Some(PathBuf::from(value));
        }
        Ok(Self {
            file: file.ok_or("situation: which situation?")?,
            install: install.unwrap_or_else(files::default_install),
            out: out.ok_or("situation: --out is required")?,
        })
    }
}

pub fn run(options: &Options) -> ExitCode {
    match write(options) {
        Ok(()) => {
            println!("situation: wrote {}", options.out.display());
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("situation: {message}");
            ExitCode::FAILURE
        }
    }
}

fn write(options: &Options) -> Result<(), String> {
    let path = &options.file;
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let situation = Situation::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let templates = if situation.needs_templates() {
        let bytes = files::read_archives(&options.install)?;
        files::templates(&files::parse_archives(&bytes)?)?
    } else {
        Vec::new()
    };
    let bytes = map_bytes(&situation, &options.install, &templates)?;
    if let Some(dir) = options.out.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    fs::write(&options.out, bytes).map_err(|e| format!("{}: {e}", options.out.display()))
}

/// A situation's map as the writer writes it: the shipped maps its copies name read from
/// `<install>/Maps`, the game's object templates given (`files::templates`; only a situation that
/// gives objects by their DEF needs them). Also the sequences' (E24 M2 T2).
pub fn map_bytes(
    situation: &Situation,
    install: &Path,
    templates: &[ObjectTemplate],
) -> Result<Vec<u8>, String> {
    let mut sources = BTreeMap::new();
    for name in situation.sources() {
        let map = install.join("Maps").join(&name);
        let bytes = fs::read(&map).map_err(|e| format!("{}: {e}", map.display()))?;
        let raw = h3_map::read(&bytes, &Limits::default())
            .map_err(|e| format!("{}: {e}", map.display()))?;
        sources.insert(name, raw);
    }
    let (map, extras) = situation.build(&sources, templates)?;
    writer::write_with(&map, &extras)
        .map_err(|e| format!("the writer refuses the situation's map: {e}"))
}
