//! `h3-map-view`: the adventure map client's command line (E24). An adapter: it reads files and
//! prints; what it computes comes from the pure library.

mod gate;

use std::process::ExitCode;

const USAGE: &str = "\
usage: h3-map-view <command> [options]

commands:
  gate [--install <dir>] [--map <dir>] [--maps <dir>]
          the map's real-asset gate: loads the map's asset tables (<map>/assets, default
          clients/map) and the adventure screen's layout (<map>/layouts/adventure.toml), finds
          every name they hold where the game searches it in the installation copy's four LOD
          archives and decodes it, and checks that each DEF's group 0 holds the frames the view
          draws: the edge's 36, the unexplored tiles' pieces', the corner gems' 8, and for the
          terrains, rivers and roads the highest picture any shipped map (<maps>, default
          <install>/Maps; skipped if absent) gives their code. It prints each DEF's frames, their
          encodings and the pixels that came from runs (the view skips runs in the terrains',
          rivers' and roads' frames: E24 design §1.3), and exits 1 on a problem. <dir> defaults
          to $H3_INSTALL, then to the repository's original-data/install.
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.split_first() {
        Some((command, rest)) if command == "gate" => match gate::Options::parse(rest) {
            Ok(options) => gate::run(&options),
            Err(message) => {
                eprintln!("{message}");
                ExitCode::from(2)
            }
        },
        _ => {
            eprint!("{USAGE}");
            ExitCode::from(2)
        }
    }
}
