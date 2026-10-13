//! `h3-view`: the client's command line (E17). An adapter: it reads files, measures time and
//! prints; what it computes comes from the pure crates.

mod assets;
mod gate;
mod harvest;
mod sprite;

use std::process::ExitCode;

const USAGE: &str = "\
usage: h3-view <command> [options]

commands:
  gate [--install <dir>] [--out <dir>]
          the real-asset gate: opens the four LOD archives of an installation copy, decodes every
          PCX, PAL, DEF and MSK in them (it fails if one does not decode), and writes every entry
          with its verdict to <out>/gate/entries.tsv and the inventory (findings, duplicates,
          names nothing mentions, timings) to <out>/gate/report.md, and checks that every name
          in the asset tables (clients/assets) is found and decodes. <dir> defaults to
          $H3_INSTALL, then to the repository's original-data/install; <out> to clients/out
          (gitignored). Build with --release for meaningful times (about 3 s, against 30 s).
  sprite <name.def> [--group <id>] [--background <name.pcx>] [--mirror] [--install <dir>]
         [--out <file>]
          draws every frame of one group of a DEF (the first by default) side by side, each on
          its canvas, over a PCX or grey, as the battle draws a stack (shadows over what is
          below), and writes a PNG (default clients/out/sprite/<name>-<group>.png). The base
          archives are searched before the AB ones, as the game does.
  harvest [--install <dir>] [--packs <dir>] [--out <dir>]
          generates the asset tables (<out>/{base,ab,sod}.toml, default clients/assets) from the
          installation copy's Heroes3.exe and H3bitmap.lod, keyed by the packs (default the
          repository's packs/); prints the gaps. Run once; the tables are then edited as source.
  help    show this message
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.split_first() {
        None => help(),
        Some((command, rest)) => match command.as_str() {
            "help" | "--help" | "-h" => help(),
            "gate" => match gate::Options::parse(rest) {
                Ok(options) => gate::run(&options),
                Err(message) => usage_error(&message),
            },
            "harvest" => match harvest::Options::parse(rest) {
                Ok(options) => harvest::run(&options),
                Err(message) => usage_error(&message),
            },
            "sprite" => match sprite::Options::parse(rest) {
                Ok(options) => sprite::run(&options),
                Err(message) => usage_error(&message),
            },
            other => usage_error(&format!("unknown command `{other}`")),
        },
    }
}

fn help() -> ExitCode {
    print!("{USAGE}");
    ExitCode::SUCCESS
}

fn usage_error(message: &str) -> ExitCode {
    eprint!("h3-view: {message}\n\n{USAGE}");
    ExitCode::from(2)
}
