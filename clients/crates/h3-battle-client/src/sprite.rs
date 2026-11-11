//! `h3-view sprite` (E17 M1 T6): one group of a DEF drawn by the compositor, frame by frame side
//! by side on their canvases, over an image, as a PNG. A look at the drawers on the user's files
//! (shadows over a background, mirroring, the keyed runs); the battle still comes with T7.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_assets::def::{decode_frame, parse_def};
use h3_assets::pcx::decode_pcx;
use h3_formats::lod::LodArchive;
use h3_render::blit::{Drawer, Placement, draw_frame, draw_image};
use h3_render::palette::ScreenPalette;
use h3_render::screen::{PixelFormat, Screen};

use crate::gate::find_ci;

/// The archives a DEF and an image are looked up in, in the executable's order: the base archive
/// first, so its copy of a duplicate name wins (`docs/epics/E17-design.md` §3.1).
const SPRITES: [&str; 2] = ["H3sprite.lod", "H3ab_spr.lod"];
const BITMAPS: [&str; 2] = ["H3bitmap.lod", "H3ab_bmp.lod"];

/// The fill where no image is drawn: a mid grey, so that shadows show.
const GREY: u16 = 0x8410;

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    def: String,
    group: Option<u32>,
    background: Option<String>,
    mirrored: bool,
    install: PathBuf,
    out: Option<PathBuf>,
}

impl Options {
    /// Parses `<name.def> [--group <id>] [--background <name.pcx>] [--mirror] [--install <dir>]
    /// [--out <file>]`; the installation defaults as the gate's.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let env = std::env::var_os("H3_INSTALL").map(PathBuf::from);
        Self::parse_with(args, env)
    }

    fn parse_with(args: &[String], env: Option<PathBuf>) -> Result<Self, String> {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut def = None;
        let (mut group, mut background, mut mirrored, mut install, mut out) =
            (None, None, false, None, None);
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            let mut value = || {
                rest.next()
                    .cloned()
                    .ok_or_else(|| format!("sprite: `{arg}` needs a value"))
            };
            match arg.as_str() {
                "--group" => {
                    let v = value()?;
                    group = Some(v.parse().map_err(|_| format!("sprite: bad group `{v}`"))?);
                }
                "--background" => background = Some(value()?),
                "--mirror" => mirrored = true,
                "--install" => install = Some(PathBuf::from(value()?)),
                "--out" => out = Some(PathBuf::from(value()?)),
                flag if flag.starts_with("--") => {
                    return Err(format!("sprite: unknown option `{flag}`"));
                }
                name if def.is_none() => def = Some(name.to_owned()),
                extra => return Err(format!("sprite: unexpected `{extra}`")),
            }
        }
        Ok(Self {
            def: def.ok_or("sprite: which DEF?")?,
            group,
            background,
            mirrored,
            install: install
                .or(env)
                .unwrap_or_else(|| manifest.join("../../../original-data/install")),
            out,
        })
    }

    fn out(&self) -> PathBuf {
        self.out.clone().unwrap_or_else(|| {
            let stem = self.def.rsplit_once('.').map_or(&*self.def, |(s, _)| s);
            let group = self
                .group
                .map_or_else(|| "first".to_owned(), |g| g.to_string());
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../out/sprite")
                .join(format!("{stem}-{group}.png"))
        })
    }
}

pub fn run(options: &Options) -> ExitCode {
    match sprite(options) {
        Ok(path) => {
            println!("{}", path.display());
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("sprite: {message}");
            ExitCode::FAILURE
        }
    }
}

/// The first archive of `names` (in `data`) that has `entry`, read.
fn lookup(data: &Path, names: &[&str], entry: &str) -> Result<Vec<u8>, String> {
    for name in names {
        let path = find_ci(data, name).map_err(|e| format!("{}/{name}: {e}", data.display()))?;
        let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let archive = LodArchive::parse(&bytes).map_err(|e| format!("{name}: {e:?}"))?;
        if let Some(found) = archive
            .read_by_name(entry)
            .map_err(|e| format!("{name}:{entry}: {e:?}"))?
        {
            return Ok(found);
        }
    }
    Err(format!("{entry}: in none of {}", names.join(", ")))
}

fn sprite(options: &Options) -> Result<PathBuf, String> {
    let data = find_ci(&options.install, "Data")
        .map_err(|e| format!("{}: {e}", options.install.display()))?;
    let bytes = lookup(&data, &SPRITES, &options.def)?;
    let def = parse_def(&bytes).map_err(|e| format!("{}: {e:?}", options.def))?;
    let group = match options.group {
        Some(id) => def.groups.iter().find(|g| g.id == id),
        None => def.groups.first(),
    }
    .ok_or_else(|| format!("{}: no such group", options.def))?;
    let frames = group
        .frames
        .iter()
        .map(|f| decode_frame(&bytes, f.offset).map_err(|e| format!("{}: {e:?}", f.name)))
        .collect::<Result<Vec<_>, _>>()?;
    let cell = frames.iter().map(|f| f.full_width).max().unwrap_or(0);
    let height = frames.iter().map(|f| f.full_height).max().unwrap_or(0);
    let count = u32::try_from(frames.len()).unwrap_or(u32::MAX);
    let format = PixelFormat::RGB565;
    let mut screen =
        Screen::new(cell.saturating_mul(count), height, format).ok_or("the sheet is too large")?;
    screen.fill(GREY);
    if let Some(name) = &options.background {
        let image =
            decode_pcx(&lookup(&data, &BITMAPS, name)?).map_err(|e| format!("{name}: {e:?}"))?;
        draw_image(&mut screen, &image, 0, 0);
    }
    let palette = ScreenPalette::new(&def.palette, format);
    for (k, frame) in frames.iter().enumerate() {
        let x = i32::try_from(k)
            .unwrap_or(i32::MAX)
            .saturating_mul(cell.cast_signed());
        let at = Placement {
            x,
            y: 0,
            mirrored: options.mirrored,
        };
        draw_frame(
            &mut screen,
            frame,
            &palette,
            at,
            Drawer::Shadowed { colour: None },
        );
    }
    let path = options.out();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::write(&path, h3_render::png::encode(&screen))
        .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use h3_assets::build;
    use h3_assets::def::{Encoding, Frame};
    use h3_assets::palette::Palette;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn options_are_parsed() {
        let o = Options::parse_with(&args(&["A.def"]), None).unwrap();
        assert_eq!((o.group, o.mirrored), (None, false));
        assert!(o.install.ends_with("original-data/install"));
        assert!(o.out().ends_with("out/sprite/A-first.png"));
        let o = Options::parse_with(
            &args(&[
                "--group",
                "2",
                "B.def",
                "--mirror",
                "--background",
                "X.pcx",
                "--out",
                "/o.png",
            ]),
            Some("/env".into()),
        )
        .unwrap();
        assert_eq!(o.group, Some(2));
        assert_eq!(o.background.as_deref(), Some("X.pcx"));
        assert_eq!(o.out(), PathBuf::from("/o.png"));
        assert_eq!(o.install, PathBuf::from("/env"));
        for bad in [
            &["--group"][..],
            &["--group", "x", "A.def"],
            &["--what"],
            &["A.def", "B.def"],
            &[],
        ] {
            assert!(Options::parse_with(&args(bad), None).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn a_group_is_drawn_from_a_synthetic_installation() {
        use h3_formats::lod::build as lod;
        let root = std::env::temp_dir().join(format!("h3-view-sprite-{}", std::process::id()));
        let data = root.join("install/data");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&data).unwrap();
        let frame = Frame {
            encoding: Encoding::Rows,
            full_width: 3,
            full_height: 1,
            left: 0,
            top: 0,
            width: 3,
            height: 1,
            pixels: vec![9, 4, 4],
            runs: vec![false, true, true],
            short_header: false,
        };
        let def = build::def(
            0x42,
            (3, 1),
            &Palette::default(),
            &[(2, &[("a", 0), ("b", 0)])],
            &[frame],
        );
        let pcx = build::pcx_indexed(1, 1, &[0], &Palette::default());
        // The AB archive's copy must lose to the base one's.
        let other = build::def(0x42, (1, 1), &Palette::default(), &[(2, &[])], &[]);
        let bare = build::def(0x42, (1, 1), &Palette::default(), &[], &[]);
        for (name, items) in [
            ("h3sprite.lod", vec![("S.DEF", &def[..], true)]),
            (
                "H3AB_SPR.LOD",
                vec![("S.DEF", &other[..], false), ("T.DEF", &bare[..], false)],
            ),
            ("H3bitmap.lod", vec![]),
            ("H3ab_bmp.lod", vec![("BG.PCX", &pcx[..], true)]),
        ] {
            fs::write(data.join(name), lod(0x200, &items)).unwrap();
        }
        let out = root.join("s.png");
        let options = Options::parse_with(
            &args(&["s.def", "--group", "2", "--background", "bg.pcx"]),
            Some(root.join("install")),
        )
        .map(|o| Options {
            out: Some(out.clone()),
            ..o
        })
        .unwrap();
        assert_eq!(sprite(&options), Ok(out.clone()));
        let png = fs::read(&out).unwrap();
        assert_eq!(png[16..24], [0, 0, 0, 6, 0, 0, 0, 1]); // two frames of 3 x 1
        for (def, group, error) in [
            ("S.DEF", Some(7), "no such group"),
            ("T.DEF", None, "no such group"),
            ("U.DEF", None, "in none of"),
        ] {
            let o = Options {
                def: def.to_owned(),
                group,
                out: Some(root.join("e.png")),
                ..Options::parse_with(&args(&["x"]), Some(root.join("install"))).unwrap()
            };
            assert!(sprite(&o).unwrap_err().contains(error), "{def}");
        }
        fs::remove_dir_all(&root).unwrap();
    }
}
