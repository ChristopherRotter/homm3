//! `h3-map-view`: the adventure map client's command line (E24). An adapter: it reads files and
//! prints; what it computes comes from the pure library.

mod audio;
mod files;
mod gate;
mod paint;
mod render;
mod sequences;
mod situation;
mod sounds;
mod window;

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
          <install>/Maps; a problem if absent or none is read) gives their code. It prints each DEF's frames, their
          encodings and the pixels that came from runs (the view skips runs in the terrains',
          rivers' and roads' frames: E24 design §1.3). Then every object picture of the game's
          Objects.txt and the shipped maps' templates: its shape and the pixels its .msk leaves
          undrawn (D8), and, listed, those a random object's picture loses drawn with its random
          template's masks (design §2.7); the templates with a visitable cell outside their
          .msk and the owners' and towns' pictures (design §2.9); the map's sounds (E24 M2 T5):
          every ambient loop and step sound in the sound archives, decoded, every terrain's music
          in <install>/MP3, decoded. It exits 1 on a problem. <dir> defaults to
          $H3_INSTALL, then to the repository's original-data/install.
  render <map.h3m> --view <x>,<y>[,<z>] [--at <ms>] [--cycles <k>] [--live <live.toml>]
         [--viewer <0-7>] [--hidden <x0>,<y0>,<x1>,<y1>]... [--bugs fixed|original]
         [--install <dir>] [--map <dir>] [--out <file.png>] [--list]
          draws the adventure screen as the original does at that view (the tile x, y of level z,
          default 0, at the view's top-left) and moment: the frame's picture in the viewer's
          colours (--viewer, default 0, red), the map's terrain, rivers, roads and edge, its
          objects and their shadows in their owners' colours, the heroes and boats, the path's
          arrows, the unexplored tiles and the corner gems, and writes a PNG (default clients/out/map/<stem>-v<x>_<y>_<z>[-<ms>ms].png). The live part is
          the map's start with every tile explored but the rectangles --hidden gives on the view's
          level; the moment is --at ms (default 0) after the map's assets were loaded, on a clock
          that never runs late (a palette cycle step every 200 ms), its cycle steps --cycles
          when given (the original's, read by the harness). --live takes the live part the
          harness read from the original's memory (h3map.py's live.toml: the view, which must
          be --view; the palette steps, the rivers' apart when the original's load set them
          apart; the frame step; the viewer; every object's anchor, phase, template and owner,
          what the random ones became; the heroes and boats; the explored tiles; the path's
          arrows); its steps and viewer override --cycles, --at and --viewer. --bugs original
          reproduces the original's drawing bugs (for the comparisons; MCL-Q2, MCL-Q3, MCL-Q4,
          MCL-Q7).
          --list prints the draw list.
          The assets come from the installation copy's archives (<dir> as for gate).
  sequences [<id>] [--frames <dir>] [--live <live.toml>] [--bugs fixed|original]
            [--install <dir>] [--map <dir>]
          replays the recorded sequences (<map>/sequences.toml, E24 M2) without the original:
          each one's situation built as `situation` builds it, its live part at 0
          (<map>/timelines/<id>-live.toml, or --live) drawn as `render --live` draws it, its
          inputs played by the map's timeline (the redraws every 180 ms, the palette steps every
          200 ms, the scroll by Ctrl and the direction keys and at the frame's edges, the cursor)
          and the screen drawn wherever it can change: the timeline (the view, the objects' and
          palette steps, the cursor and the box the screen changed in outside the panels) is
          compared line by line with <map>/timelines/<id>.txt. A walk (the selected hero the
          live part's, its paths the recorded ones: E24 M2 T3a) is drawn as the timeline has it
          (the hero lifted at the view's middle, the view slid by the walk's offset, the path's
          arrows left: E24 M2 T3b). With --frames (one sequence) the frame at each time the
          screen changed and at each time <dir>/frames/ holds the original's (t<ms>.npz,
          h3map.py frames) is written as <dir>/ours/t<ms>.png, the timeline as
          <dir>/ours.txt. --bugs defaults to original
          (the comparisons: the original's edge zones, MCL-Q6, and its cursor's even x). It
          exits 1 unless every sequence played is as recorded.
  situation <situation.toml> --out <map.h3m> [--install <dir>]
          writes a situation's map (clients/map/situations/; E24 M1 T3b2) with the adventure
          lane's writer, the shipped maps its copies name read from <dir>/Maps (a situation
          that names a shipped map whole, `map`, is the harness's to load: refused).
  window <sequence | situation> [--keys <file>] [--scale <n>] [--vsync] [--fullscreen]
         [--drawn-cursor] [--mute] [--cues <file>] [--bugs fixed|original] [--install <dir>]
         [--map <dir>] [--check <s> | --measure <s>]
          shows a recorded sequence (<map>/sequences.toml: its inputs replayed at their times,
          the cursor the recording's) or a situation (<map>/situations/<id>.toml, with its kept
          live part if any; the viewer's hero selected when it has one only) in a window at the
          original's frame times, as `sequences` draws it; past a sequence's inputs, and in a
          situation from the start, the map takes the keys and the mouse (E24 M2 T4b): Ctrl and
          a direction key (the keypad's too) scroll, a direction key steps the selected hero (a
          held key repeats), the pointer at the frame's edges scrolls with the edge cursors, a
          click shows a path the map knows and on its destination walks it; the cursor is the
          hardware one (--drawn-cursor or F9: drawn in the frame at the mouse poll's point). The
          map's sound plays (E24 M2 T5): the terrain's music, the walking hero's step sound, the
          objects' ambient loops within three tiles of the view's middle; --mute plays none,
          --cues writes each cue played with the clock's millisecond. Every key goes through a
          table of commands (E24 M2 T4a, provisional): F2 switches the music, F3 the effects, F5
          pauses, F6 shows the next frame while paused, F7 restarts, F8 turns vsync on or off, F9
          the cursor's kind, F11 fullscreen, Esc quits (the battle window's own keys too). A
          user's file (--keys, else keys.toml in the working directory when there is one; its
          [map] table, the battle window's [battle]) overrides the defaults command by command,
          e.g. `[map]` then `pause = [\"Space\"]`, `quit = [\"Escape\", \"Ctrl+F12\"]` (the
          commands: scroll-north ... scroll-north-west, hero-north ... hero-north-west, music,
          effects, pause, frame, restart, vsync, cursor, fullscreen, quit; [] unbinds one).
          --bugs defaults to fixed; original reproduces the original's bugs (the drawing's as for
          render, the edge zones, MCL-Q8's ambient loops sharing a bit, MCL-Q12's music kept on
          the wrong terrain).
          --check <s> and --measure <s> run unattended for <s> seconds (the window's own keys but Esc do
          nothing; the map's input is taken) and report when each frame showed against its time
          and how long the frames took to make;
          --check also reads every frame back and compares it with the frame composed, compares
          each frame made with the screen's own pixels widened one by one, and plays every
          action headless to check each change shown (exit 1 on a difference).
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
        Some((command, rest)) if command == "sequences" => match sequences::Options::parse(rest) {
            Ok(options) => sequences::run(&options),
            Err(message) => {
                eprintln!("{message}");
                ExitCode::from(2)
            }
        },
        Some((command, rest)) if command == "situation" => match situation::Options::parse(rest) {
            Ok(options) => situation::run(&options),
            Err(message) => {
                eprintln!("{message}");
                ExitCode::from(2)
            }
        },
        Some((command, rest)) if command == "window" => match window::Options::parse(rest) {
            Ok(options) => window::run(&options),
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
