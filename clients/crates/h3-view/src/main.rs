//! `h3-view`: the client's command line (E17). An adapter: it reads files, measures time and
//! prints; everything it computes comes from the pure crates.

use std::process::ExitCode;

const USAGE: &str = "\
usage: h3-view <command> [options]

commands:
  help    show this message
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None | Some("help" | "--help" | "-h") => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprint!("h3-view: unknown command `{other}`\n\n{USAGE}");
            ExitCode::from(2)
        }
    }
}
