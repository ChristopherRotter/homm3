//! `h3-view render` (E17 M1 T7): a battle still. A record's first snapshot (or a scenario file's,
//! through [`crate::scenario`]) and the asset tables give the draw list (`h3_view::scene`); its
//! assets are loaded as one checkpoint batch (ADR-0020 point 7: the four archives read whole until
//! CLT-Q2's table-only reader lands, then the batch's entries by name); the compositor draws the
//! frame; the frame is written as a PNG.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_formats::lod::LodArchive;
use h3_view::anim::{self, Speed};
use h3_view::batch::{self, Batch};
use h3_view::compose::compose;
use h3_view::{layout, record, scene};

use crate::gate::{find_ci, timed};
use crate::{assets, motion};

/// The archives, the base ones first: a DEF is searched in the sprite archives, anything else in
/// the bitmap ones (`docs/epics/E17-design.md` §3.1).
const SPRITES: [&str; 2] = ["H3sprite.lod", "H3ab_spr.lod"];
const BITMAPS: [&str; 2] = ["H3bitmap.lod", "H3ab_bmp.lod"];

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    input: PathBuf,
    seed: u64,
    packs: PathBuf,
    install: PathBuf,
    out: Option<PathBuf>,
    list: bool,
    at: Option<u32>,
    speed: u8,
    frames: Option<PathBuf>,
}

impl Options {
    /// Parses `<scenario.toml | record.jsonl> [--seed <n>] [--packs <dir>] [--install <dir>]
    /// [--out <file.png>] [--list] [--at <ms> | --frames <dir>] [--speed <0-2>]`; the
    /// installation defaults as the gate's.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let env = std::env::var_os("H3_INSTALL").map(PathBuf::from);
        Self::parse_with(args, env)
    }

    fn parse_with(args: &[String], env: Option<PathBuf>) -> Result<Self, String> {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let (mut input, mut seed, mut packs, mut install, mut out, mut list) =
            (None, 1, None, None, None, false);
        let (mut at, mut speed, mut frames) = (None, 0, None);
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            let mut value = || {
                rest.next()
                    .cloned()
                    .ok_or_else(|| format!("render: `{arg}` needs a value"))
            };
            match arg.as_str() {
                "--seed" => {
                    let v = value()?;
                    seed = v.parse().map_err(|_| format!("render: bad seed `{v}`"))?;
                }
                "--packs" => packs = Some(PathBuf::from(value()?)),
                "--install" => install = Some(PathBuf::from(value()?)),
                "--out" => out = Some(PathBuf::from(value()?)),
                "--list" => list = true,
                "--at" => {
                    let v = value()?;
                    at = Some(v.parse().map_err(|_| format!("render: bad time `{v}`"))?);
                }
                "--frames" => frames = Some(PathBuf::from(value()?)),
                "--speed" => {
                    let v = value()?;
                    speed = v
                        .parse()
                        .ok()
                        .filter(|s| *s <= 2)
                        .ok_or(format!("render: bad speed `{v}`"))?;
                }
                flag if flag.starts_with("--") => {
                    return Err(format!("render: unknown option `{flag}`"));
                }
                path if input.is_none() => input = Some(PathBuf::from(path)),
                extra => return Err(format!("render: unexpected `{extra}`")),
            }
        }
        if at.is_some() && frames.is_some() {
            return Err("render: --at or --frames, not both".into());
        }
        Ok(Self {
            input: input.ok_or("render: which scenario or record?")?,
            seed,
            packs: packs.unwrap_or_else(|| manifest.join("../../../packs")),
            install: install
                .or(env)
                .unwrap_or_else(|| manifest.join("../../../original-data/install")),
            out,
            list,
            at,
            speed,
            frames,
        })
    }

    /// Whether the input is a scenario file (else a record).
    fn scenario(&self) -> bool {
        self.input
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("toml"))
    }

    /// The PNG's path: `--out`, or `clients/out/render/<stem>[-s<seed>].png`.
    fn png(&self) -> PathBuf {
        self.out.clone().unwrap_or_else(|| {
            let stem = self
                .input
                .file_stem()
                .map_or_else(|| "still".into(), |s| s.to_string_lossy());
            let at = self.at.map_or_else(String::new, |t| format!("-{t}ms"));
            let name = if self.scenario() {
                format!("{stem}-s{}{at}.png", self.seed)
            } else {
                format!("{stem}{at}.png")
            };
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../out/render")
                .join(name)
        })
    }
}

pub fn run(options: &Options) -> ExitCode {
    match render(options) {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("render: {message}");
            ExitCode::FAILURE
        }
    }
}

pub fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
}

/// The archives of `install`, read whole (CLT-Q2), each with its name.
pub fn read_archives(install: &Path) -> Result<Vec<(&'static str, Vec<u8>)>, String> {
    let data = find_ci(install, "Data").map_err(|e| format!("{}: {e}", install.display()))?;
    SPRITES
        .iter()
        .chain(&BITMAPS)
        .map(|&name| {
            let path =
                find_ci(&data, name).map_err(|e| format!("{}/{name}: {e}", data.display()))?;
            let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            Ok((name, bytes))
        })
        .collect()
}

/// The entry `name` from the first archive of its search list that holds it.
pub fn lookup(archives: &[(&str, LodArchive<'_>)], name: &str) -> Result<Vec<u8>, String> {
    let list: &[&str] = if name.to_ascii_uppercase().ends_with(".DEF") {
        &SPRITES
    } else {
        &BITMAPS
    };
    for (archive_name, archive) in archives.iter().filter(|(n, _)| list.contains(n)) {
        if let Some(bytes) = archive
            .read_by_name(name)
            .map_err(|e| format!("{archive_name}:{name}: {e:?}"))?
        {
            return Ok(bytes);
        }
    }
    Err(format!("{name}: in none of {}", list.join(", ")))
}

/// Renders the still; the lines to print: the PNG, the record written (from a scenario), the
/// counts and the times, and with `--list` the draw list.
fn render(options: &Options) -> Result<Vec<String>, String> {
    let mut lines = Vec::new();
    let text = if options.scenario() {
        let text = crate::scenario::record(&options.input, &options.packs, options.seed)?;
        let path = options.png().with_extension("jsonl");
        write(&path, text.as_bytes())?;
        lines.push(format!("record: {}", path.display()));
        text
    } else {
        fs::read_to_string(&options.input)
            .map_err(|e| format!("{}: {e}", options.input.display()))?
    };
    let record = record::parse(&text).map_err(|e| e.to_string())?;
    let observation = record
        .first_snapshot()
        .ok_or("the record holds no snapshot")?;
    let manifest = &record.header.manifest;
    // The tables of the game's topmost pack, as the ruleset names its packs.
    let top = manifest.packs.last().map_or("", |p| p.id.as_str());
    let tables = assets::load(&assets::default_dir(), top)?;
    let layout_path = assets::default_dir().join("../layouts/battle.toml");
    let layout = layout::parse_battle(
        &fs::read_to_string(&layout_path).map_err(|e| format!("{}: {e}", layout_path.display()))?,
    )
    .map_err(|e| format!("{}: {e}", layout_path.display()))?;
    // The checkpoint: the archives (whole, CLT-Q2), then the batch.
    let (bytes, read) = timed(|| read_archives(&options.install));
    let bytes = bytes?;
    let archives = bytes
        .iter()
        .map(|(name, b)| {
            LodArchive::parse(b)
                .map(|a| (*name, a))
                .map_err(|e| format!("{name}: {e:?}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if let Some(dir) = &options.frames {
        // The capture set's frames by time (E17 M3 T1b-2, `h3drive.py set`): the record's
        // commands at the original's timing without fidgets of the viewer's own, each frame with
        // the fidgets the original showed then, as the frame check draws a sequence's.
        let looks = motion::looks(&record, &tables, &archives)?;
        let stage = motion::stage(&record, &tables, &layout);
        let commands = anim::commands(&record.messages);
        let timeline = anim::timeline(
            observation,
            &commands,
            &looks,
            stage,
            Speed::new(options.speed),
        )
        .map_err(|e| e.to_string())?
        .with_heroes(motion::heroes(&record, &tables, &archives, None)?);
        let drawn =
            crate::sequences::draw_frames(dir, &record, &timeline, &tables, &layout, &archives)?;
        lines.push(format!("{drawn} frames drawn in {}", dir.display()));
        return Ok(lines);
    }
    let list = match options.at {
        None => scene::battle(observation, manifest, &tables, &layout),
        Some(t) => {
            // The record's commands at the original's timing (E17 M2 T4a, `h3_view::anim`).
            let looks = motion::looks(&record, &tables, &archives)?;
            let stage = motion::stage(&record, &tables, &layout);
            let commands = anim::commands(&record.messages);
            // With the viewer's own fidgets, from the seed (T4b-2a), and its heroes' (E17 M3
            // T3a).
            let seed = u32::try_from(options.seed & u64::from(u32::MAX)).unwrap_or(1);
            let timeline = anim::fidgeting(
                observation,
                &commands,
                &looks,
                stage,
                Speed::new(options.speed),
                seed,
            )
            .map_err(|e| e.to_string())?
            .with_heroes(motion::heroes(&record, &tables, &archives, Some(seed))?);
            lines.push(format!("the timeline ends at {} ms", timeline.end));
            let moment = scene::Moment::at(&timeline, t);
            scene::battle_at(observation, &moment, manifest, &tables, &layout)
        }
    }
    .map_err(|e| e.to_string())?;
    let names = batch::names(&list, &layout);
    let (batch, loaded) = timed(|| {
        let mut batch = Batch::default();
        for name in &names {
            batch
                .add(name, &lookup(&archives, name)?)
                .map_err(|e| e.to_string())?;
        }
        Ok::<_, String>(batch)
    });
    let batch = batch?;
    let screen = compose(&list, &batch, &layout).map_err(|e| e.to_string())?;
    let png = options.png();
    write(&png, &h3_render::png::encode(&screen))?;
    lines.insert(0, png.display().to_string());
    lines.push(format!(
        "{} draws; batch of {} assets; archives read in {} ms, batch decoded in {} ms",
        list.len(),
        names.len(),
        read.as_millis(),
        loaded.as_millis()
    ));
    if options.list {
        lines.extend(list.iter().map(|d| format!("{d:?}")));
    }
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use h3_assets::build;
    use h3_assets::palette::Palette;
    use h3_protocol::codec;
    use h3_protocol::wire::{
        Algorithm, Field, KeyTables, Manifest, PackRef, RecordHeader, Seat, WireU64,
    };

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn options_are_parsed() {
        let o = Options::parse_with(&args(&["a/duel.toml"]), None).unwrap();
        assert_eq!((o.seed, o.list), (1, false));
        assert!(o.install.ends_with("original-data/install"));
        assert!(o.packs.ends_with("packs"));
        assert!(o.png().ends_with("out/render/duel-s1.png"));
        let o = Options::parse_with(
            &args(&[
                "--seed", "7", "r.JSONL", "--list", "--packs", "/p", "--out", "/o.png",
            ]),
            Some("/env".into()),
        )
        .unwrap();
        assert_eq!((o.seed, o.list, o.scenario()), (7, true, false));
        assert_eq!(o.png(), PathBuf::from("/o.png"));
        assert_eq!(
            (o.packs, o.install),
            (PathBuf::from("/p"), PathBuf::from("/env"))
        );
        let o = Options::parse_with(&args(&["r.jsonl", "--install", "/i"]), None).unwrap();
        assert!(o.png().ends_with("out/render/r.png"));
        assert_eq!(o.install, PathBuf::from("/i"));
        let o =
            Options::parse_with(&args(&["a.toml", "--at", "250", "--speed", "2"]), None).unwrap();
        assert_eq!((o.at, o.speed), (Some(250), 2));
        assert!(o.png().ends_with("out/render/a-s1-250ms.png"));
        let o = Options::parse_with(&args(&["a.toml", "--frames", "/f"]), None).unwrap();
        assert_eq!((o.at, o.frames), (None, Some(PathBuf::from("/f"))));
        for bad in [
            &["a.toml", "--at", "1", "--frames", "/f"][..],
            &["--seed"],
            &["--seed", "x", "a.toml"],
            &["--at", "soon", "a.toml"],
            &["--speed", "3", "a.toml"],
            &["--what"],
            &["a.toml", "b.toml"],
            &[],
        ] {
            assert!(Options::parse_with(&args(bad), None).is_err(), "{bad:?}");
        }
    }

    /// A record of an empty battle of the `base` packs: it draws nothing, so its batch is only the
    /// outline palette.
    fn empty_record() -> String {
        use h3_protocol::wire::ServerMessage;
        let header = RecordHeader {
            format: h3_protocol::RECORD_FORMAT_VERSION,
            game: 1,
            seat: Seat::Spectator,
            manifest: Manifest {
                protocol: h3_protocol::PROTOCOL_VERSION,
                engine: "test".into(),
                packs: vec![PackRef {
                    id: "base".into(),
                    version: "1".into(),
                }],
                profile: "base:faithful".into(),
                ruleset_hash: WireU64(0),
                battle_ruleset_hash: WireU64(0),
                rng: Algorithm {
                    name: "r".into(),
                    version: 1,
                },
                hash: Algorithm {
                    name: "h".into(),
                    version: 1,
                },
                battle_state_hash_version: 1,
                field: Field {
                    width: 17,
                    height: 11,
                },
                keys: KeyTables::default(),
            },
            first: 1,
        };
        let text = crate::scenario::record(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../packs/synthetic/scenarios/duel.toml"),
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../packs"),
            1,
        )
        .unwrap();
        let mut snapshot = record::parse(&text).unwrap().messages.remove(0);
        if let ServerMessage::Snapshot { observation, .. } = &mut snapshot {
            observation.stacks.clear();
            observation.removable_obstacles.clear();
            observation.absolute_obstacle = None;
            observation.terrain = None;
        }
        [codec::encode_text(&header), codec::encode_text(&snapshot)].join("\n")
    }

    #[test]
    fn a_record_renders_from_a_synthetic_installation() {
        use h3_formats::lod::build as lod;
        let root = std::env::temp_dir().join(format!("h3-view-render-{}", std::process::id()));
        let data = root.join("install/Data");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&data).unwrap();
        let pal = build::pal(&Palette::default(), &[0; 256], 0, &[]);
        for (name, items) in [
            ("H3sprite.lod", vec![]),
            ("H3ab_spr.lod", vec![]),
            ("H3bitmap.lod", vec![]),
            ("h3ab_bmp.lod", vec![("GAME.PAL", &pal[..], true)]),
        ] {
            fs::write(data.join(name), lod(0x200, &items)).unwrap();
        }
        let input = root.join("empty.jsonl");
        fs::write(&input, empty_record()).unwrap();
        let options = Options {
            out: Some(root.join("out/still.png")),
            list: true,
            ..Options::parse_with(&args(&["x.jsonl"]), Some(root.join("install"))).unwrap()
        };
        let options = Options { input, ..options };
        let lines = render(&options).unwrap();
        assert_eq!(lines[0], root.join("out/still.png").display().to_string());
        assert!(
            lines[1].starts_with("0 draws; batch of 1 assets"),
            "{lines:?}"
        );
        let png = fs::read(root.join("out/still.png")).unwrap();
        assert_eq!(png[16..24], [0, 0, 3, 32, 0, 0, 2, 88]); // 800 x 600
        // At a time: the record's (no) commands played first.
        let later = Options {
            at: Some(500),
            ..Options { ..options }
        };
        let lines = render(&later).unwrap();
        assert!(
            lines.contains(&"the timeline ends at 0 ms".to_owned()),
            "{lines:?}"
        );
        let options = Options { at: None, ..later };
        // A record without a snapshot, and an archive without the palette.
        let header = empty_record().lines().next().unwrap().to_owned();
        fs::write(root.join("bare.jsonl"), &header).unwrap();
        let bare = Options {
            input: root.join("bare.jsonl"),
            ..Options::parse_with(&args(&["x"]), Some(root.join("install"))).unwrap()
        };
        assert_eq!(render(&bare).unwrap_err(), "the record holds no snapshot");
        fs::write(data.join("h3ab_bmp.lod"), lod(0x200, &[])).unwrap();
        assert!(
            render(&options)
                .unwrap_err()
                .contains("GAME.PAL: in none of")
        );
        fs::remove_dir_all(&root).unwrap();
    }
}
