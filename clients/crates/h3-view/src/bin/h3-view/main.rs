//! `h3-view`: the client's command line (E17). An adapter: it reads files, measures time, prints
//! and opens the window (through `h3-sdl`); what it computes comes from the pure crates.

mod assets;
mod audio;
mod battle;
mod bot;
mod gate;
mod harvest;
mod log;
mod motion;
mod play;
mod pointer;
mod render;
mod scenario;
mod sequences;
mod situation;
mod sounds;
mod sprite;
mod text;
mod window;

use std::process::ExitCode;

const USAGE: &str = "\
usage: h3-view <command> [options]

commands:
  gate [--install <dir>] [--out <dir>] [--tables <dir>] [--keys <file>]
          the real-asset gate: opens the four LOD archives of an installation copy, decodes every
          PCX, PAL, DEF and MSK in them (it fails if one does not decode), and writes every entry
          with its verdict to <out>/gate/entries.tsv and the inventory (findings, duplicates,
          names nothing mentions, timings) to <out>/gate/report.md, and checks that every name
          in the asset tables (<tables>, default clients/assets) is found and decodes, and that
          every key of the string catalogue (<file>, default clients/text/keys.toml) is read
          from the text tables with its arguments, and the battle log's lines compose. <dir>
          defaults to $H3_INSTALL, then to the repository's original-data/install; <out> to
          clients/out (gitignored). Build with --release for meaningful times (about 3 s,
          against 30 s).
  sprite <name.def> [--group <id>] [--background <name.pcx>] [--mirror] [--install <dir>]
         [--out <file>]
          draws every frame of one group of a DEF (the first by default) side by side, each on
          its canvas, over a PCX or grey, as the battle draws a stack (shadows over what is
          below), and writes a PNG (default clients/out/sprite/<name>-<group>.png). The base
          archives are searched before the AB ones, as the game does.
  render <scenario.toml | record.jsonl> [--seed <n>] [--packs <dir>] [--install <dir>]
         [--out <file.png>] [--list] [--at <ms> | --frames <dir>] [--speed <0-2>]
          draws a battle still: the battle at its start as the record's first snapshot shows it
          (the background, the obstacles, the heroes and their flags, every living stack
          standing on its hex with its count box; no corpses yet), with its assets loaded as one
          checkpoint batch, and writes
          a PNG (default clients/out/render/<stem>.png; <stem>-s<seed>.png from a scenario). A
          scenario file (as `h3 run` reads it, its packs from <dir>, default the repository's
          packs/; obstacles from the seed, default 1, when it asks) is run in-process as a
          spectator of h3-protocol's session first; that record is written beside the PNG
          (<stem>-s<seed>.jsonl). --list prints the draw list. --at draws the battle <ms>
          after the record's start, its commands played at the original's timing at combat
          speed --speed (default 0; h3_view::anim): each stack in its pose then
          (<stem>[-s<seed>]-<ms>ms.png). --frames draws, instead of the still, the battle at
          each time <dir>/frames.toml lists (the capture set's, `h3drive.py set`), without
          fidgets of the viewer's own and with those the file lists, into <dir>/ours/ (what
          it drew: <dir>/ours.txt).
  play <scenario.toml | record.jsonl> [--seed <n>] [--speed <0-2>] [--packs <dir>]
       [--install <dir>] [--out <dir>] [--png] [--from <ms>] [--to <ms>]
          a whole battle frame by frame, headless: a scenario is fought to its end in-process
          with a simple bot in both seats (its obstacles from the seed, default 1), or a record
          is read; its commands are played at the original's timing at combat speed --speed
          (h3_view::anim) and every distinct frame is drawn (from --from to --to ms). Writes the
          record, each frame's moment (frames.txt) and the creatures' sound cues (cues.txt) to
          <out> (default clients/out/play/<stem>[-s<seed>]); --png also writes each frame as
          f<ms>.png there.
  window <scenario.toml | record.jsonl> [--seed <n>] [--speed <0-2>] [--packs <dir>]
         [--install <dir>] [--scale <n>] [--vsync] [--drawn-cursor] [--fullscreen] [--from <ms>]
         [--mute] [--check <s> | --measure <s>]
          a battle in a window (SDL 3, h3-sdl), as `play` loads it, at the original's frame
          times from --from ms: each frame is shown when it falls due, at the largest integer
          scale that fits the display (--scale), vsync off unless --vsync, the game's cursor as
          the hardware cursor (--drawn-cursor: drawn into the frame); past the end the battle
          idles. With sound (the installation's sound archives and MP3 folder; none with
          --mute): from the battle's start its start sound plays while the clock waits, then
          the battle music loops and the creatures' sounds follow their animations. Keys: Space
          pause, Right a frame while paused, R restart, F fullscreen, V vsync, C the cursor's
          kind, M the music, E the creatures' sounds, Esc quit. --measure runs <s> seconds
          unattended and prints how late each frame showed; --check also reads every frame back
          and compares it with the frame drawn headless (exit 1 on a difference).
  situation <scenario.toml> [--packs <dir>] [--install <dir>] [--out <file.h3m>]
          writes the original's map for a scenario's battle, for the capture harness
          (clients/capture/README.md): Red's hero (human) just west of Blue's (computer) on the
          scenario's terrain, each with the scenario's stacks in their slots, its formation and its
          war machines (as artifacts), its spell book and the school skills behind its
          masteries, no other skills or artifacts; stepping Red's hero east
          starts the battle (--hot-seat: Blue a human too). The
          creatures' original ids from the installation's crtraits.txt; the map defaults to
          clients/out/situation/<stem>.h3m.
  sequences [<id>] [--speed <0-2>] [--packs <dir>] [--install <dir>] [--out <dir>]
            [--frames <dir>]
          the recorded sequences (clients/situations/sequences.toml): each sequence's scenario
          played in-process with its engine commands, timed by the viewer at the original's
          timing (h3_view::anim) and compared with the timeline the capture harness recorded in
          the original (at speed 0); writes each record and timeline to <out> (default
          clients/out/sequences) and exits 1 when one differs. --frames: also draws the viewer's
          frame at each time the harness kept a frame of the original (<dir>/<id>/frames.toml,
          from `h3drive.py frames`) into <dir>/<id>/ours/, and what it drew into ours.txt.
  text [<key> [<name>=<value>]...] [--log] [--pack <file.json>] [--install <dir>]
       [--tables <dir>] [--keys <file>]
          the string catalogue (clients/text/keys.toml; docs/epics/E17-design.md §9): without a
          key, every key's forms as the installation's text tables give them ({{name}} where an
          argument goes); with one, its text for the arguments given (a whole number is a
          number; `count` picks a plural form, `context` a context's form). --pack puts a
          language pack before the original's text (nested JSON, i18next's conventions, its
          language its file's name: de.json) and lists what in it no key names. --log prints
          the battle log's lines for a few stacks instead, as the executable composes them
          (\n where a line breaks).
  harvest [--install <dir>] [--packs <dir>] [--out <dir>]
          generates the asset tables (<out>/{base,ab,sod}.toml, default clients/out/harvest)
          from the installation copy's Heroes3.exe and H3bitmap.lod, keyed by the packs (default
          the repository's packs/); prints the gaps. The committed tables (clients/assets) are
          source edited by hand: diff a fresh harvest against them, never overwrite them.
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
            "play" => match play::Options::parse(rest) {
                Ok(options) => play::run(&options),
                Err(message) => usage_error(&message),
            },
            "render" => match render::Options::parse(rest) {
                Ok(options) => render::run(&options),
                Err(message) => usage_error(&message),
            },
            "sequences" => match sequences::Options::parse(rest) {
                Ok(options) => sequences::run(&options),
                Err(message) => usage_error(&message),
            },
            "situation" => match situation::Options::parse(rest) {
                Ok(options) => situation::run(&options),
                Err(message) => usage_error(&message),
            },
            "sprite" => match sprite::Options::parse(rest) {
                Ok(options) => sprite::run(&options),
                Err(message) => usage_error(&message),
            },
            "text" => match text::Options::parse(rest) {
                Ok(options) => text::run(&options),
                Err(message) => usage_error(&message),
            },
            "window" => match window::Options::parse(rest) {
                Ok(options) => window::run(&options),
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
