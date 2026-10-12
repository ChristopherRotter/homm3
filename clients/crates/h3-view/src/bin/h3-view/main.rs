//! `h3-view`: the client's command line (E17). An adapter: it reads files, measures time and
//! prints; what it computes comes from the pure crates.

mod gate;

use std::process::ExitCode;

const USAGE: &str = "\
usage: h3-view <command> [options]

commands:
  gate [--install <dir>] [--out <dir>]
          the real-asset gate: opens the four LOD archives of an installation copy, reports their
          entries by extension and the time to open them, and lists every entry in
          <out>/gate/entries.tsv. <dir> defaults to $H3_INSTALL, then to the repository's
          original-data/install; <out> to clients/out (gitignored).
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
