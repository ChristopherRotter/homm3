//! `h3-map-view`: the adventure map client's command line (E24). An adapter: it reads files and
//! prints; what it computes comes from the pure library.

mod files;
mod gate;
mod render;

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
  render <map.h3m> --view <x>,<y>[,<z>] [--at <ms>] [--viewer <0-7>]
         [--hidden <x0>,<y0>,<x1>,<y1>] [--bugs fixed|original] [--install <dir>] [--map <dir>]
         [--out <file.png>] [--list]
          draws the adventure screen as the original does at that view (the tile x, y of level z,
          default 0, at the view's top-left) and moment: the frame's picture in the viewer's
          colours (--viewer, default 0, red), the map's terrain, rivers, roads and edge, the
          unexplored tiles and the corner gems (objects and heroes not yet: E24 M1 T4, T5), and
          writes a PNG (default clients/out/map/<stem>-v<x>_<y>_<z>[-<ms>ms].png). The live part is
          the map's start with every tile explored but the rectangle --hidden gives on the view's
          level; the moment is --at ms (default 0) after the map's assets were loaded, on a clock
          that never runs late (a palette cycle step every 200 ms). --bugs original reproduces the
          original's drawing bugs (for the comparisons; MCL-Q2). --list prints the draw list.
          The assets come from the installation copy's archives (<dir> as for gate).
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
        Some((command, rest)) if command == "render" => match render::Options::parse(rest) {
            Ok(options) => render::run(&options),
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
