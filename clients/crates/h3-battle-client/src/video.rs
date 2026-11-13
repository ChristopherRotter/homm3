//! `h3-view video` (E17 M5 T4b2a): the Bink clips of a video archive decoded by `h3-bink`, each
//! written out as the file it is and as its frames in raw YUV 4:2:0 (the bytes of FFmpeg's
//! `rawvideo` output in `yuv420p`), and compared with a reference decoding of the same files
//! (the game's `BINKW32.DLL`'s since E17 M6 T6c1, read as it reads with `--original`:
//! `clients/scripts/bink-check.sh`; FFmpeg's before). The synthetic samples
//! (`h3_bink::build::sample`) go the same way, so that the reference checks the block types the
//! game's clips may not use. With `--smacker` (E27 M2 T1) the Smacker clips the same way through
//! `h3-smacker` (default archive `original-data/shell-videos.vid`), each written as its frames'
//! colour indices and palettes (`<stem>.pal8`, `h3_smacker::check::raw`) and as the original's
//! 5-6-5 copy of them (`<stem>.565`, `check::original_16`), both compared with the game's
//! `SMACKW32.DLL`'s (`clients/scripts/smacker-check.sh`).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_assets::vid::VidArchive;
use h3_bink::{Clip, Decoder, Frame, Reading};
use h3_smacker::check::{Format, original_16};

use crate::connect::printable;

/// The clips' format the command decodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Bink,
    /// `--smacker` (E27 M2 T1).
    Smacker,
}

impl Kind {
    fn extension(self) -> &'static str {
        match self {
            Self::Bink => "bik",
            Self::Smacker => "smk",
        }
    }
}

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    names: Vec<String>,
    vid: PathBuf,
    out: PathBuf,
    reference: Option<PathBuf>,
    samples: Vec<u64>,
    /// `--original`: the residues read as the DLL reads them (CLT-Q68; Bink's only).
    reading: Reading,
    kind: Kind,
}

impl Options {
    /// Parses `[<name>...] [--vid <file>] [--out <dir>] [--reference <dir>] [--sample <seed>]...
    /// [--original] [--smacker]`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let (mut vid, mut out, mut reference) = (None, None, None);
        let (mut names, mut samples) = (Vec::new(), Vec::new());
        let mut reading = Reading::Fixed;
        let mut kind = Kind::Bink;
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            let mut value = || {
                rest.next()
                    .cloned()
                    .ok_or_else(|| format!("video: `{arg}` needs a value"))
            };
            match arg.as_str() {
                "--vid" => vid = Some(PathBuf::from(value()?)),
                "--out" => out = Some(PathBuf::from(value()?)),
                "--reference" => reference = Some(PathBuf::from(value()?)),
                "--original" => reading = Reading::Original,
                "--smacker" => kind = Kind::Smacker,
                "--sample" => {
                    let v = value()?;
                    samples.push(v.parse().map_err(|_| format!("video: bad seed `{v}`"))?);
                }
                flag if flag.starts_with("--") => {
                    return Err(format!("video: unknown option `{flag}`"));
                }
                name => names.push(name.to_owned()),
            }
        }
        let archive = match kind {
            Kind::Bink => "original-data/battle-videos.vid",
            Kind::Smacker => "original-data/shell-videos.vid",
        };
        Ok(Self {
            names,
            vid: vid.unwrap_or_else(|| repo.join(archive)),
            out: out.unwrap_or_else(|| repo.join("clients/out/video")),
            reference,
            samples,
            reading,
            kind,
        })
    }

    /// Whether the archive's entry `name` is asked for: a file of the format decoded, by its
    /// name or stem when names are given (ignoring case), every one when none is and no sample
    /// either.
    fn wants(&self, name: &str) -> bool {
        let Some((stem, extension)) = name.rsplit_once('.') else {
            return false;
        };
        if !extension.eq_ignore_ascii_case(self.kind.extension()) {
            return false;
        }
        if self.names.is_empty() {
            return self.samples.is_empty();
        }
        self.names
            .iter()
            .any(|n| n.eq_ignore_ascii_case(name) || n.eq_ignore_ascii_case(stem))
    }
}

pub fn run(options: &Options) -> ExitCode {
    match video(options) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(message) => {
            // An archive's entry name may be in it (E17 M5 T8b1, R2b-6).
            eprintln!("video: {}", printable(&message));
            ExitCode::FAILURE
        }
    }
}

/// Whether an archive's entry `name` is a plain file name, which `--out` may take as it is (E17
/// M5 T8b1, R2b-2): no directory (`../x.bik`, `/tmp/x.bik`) and nothing Windows reads as one
/// (`\`, a drive's `:`).
fn plain(name: &str) -> bool {
    Path::new(name).file_name() == Some(name.as_ref()) && !name.contains(['\\', ':'])
}

/// A clip's frames as FFmpeg's `rawvideo` writes them: each plane's picture row by row.
fn raw(frames: &[Frame]) -> Vec<u8> {
    let mut out = Vec::new();
    for frame in frames {
        for plane in frame.planes() {
            for y in 0..plane.height() {
                out.extend_from_slice(plane.row(y));
            }
        }
    }
    out
}

/// Decodes one file; prints its line and each failing frame; the frames decoded.
fn decode(name: &str, bytes: &[u8], reading: Reading) -> Result<(Vec<Frame>, bool), String> {
    // Printed without its control characters (E17 M5 T8b1, R2b-6).
    let name = printable(name);
    let clip = Clip::parse(bytes).map_err(|e| format!("{name}: {e:?}"))?;
    let mut decoder = Decoder::new(&clip)
        .map_err(|e| format!("{name}: {e:?}"))?
        .reading(reading);
    let mut frames = Vec::new();
    let mut clean = true;
    for n in 0..clip.frames().len() {
        match clip
            .video(n)
            .and_then(|packet| decoder.decode(packet).cloned())
        {
            Ok(frame) => frames.push(frame),
            Err(e) => {
                eprintln!("{name}: frame {n}: {e:?}");
                clean = false;
            }
        }
    }
    println!(
        "{name}: {}x{}, {} of {} frames, {}/{} fps, hash {:016x}",
        clip.width,
        clip.height,
        frames.len(),
        clip.frames().len(),
        clip.rate_num,
        clip.rate_den,
        h3_bink::check::hash(&frames)
    );
    Ok((frames, clean))
}

/// Decodes one Smacker file; prints its line and each failing frame; the frames decoded.
fn decode_smacker(name: &str, bytes: &[u8]) -> Result<(Vec<h3_smacker::Frame>, bool), String> {
    let name = printable(name);
    let clip = h3_smacker::Clip::parse(bytes).map_err(|e| format!("{name}: {e:?}"))?;
    let mut decoder = h3_smacker::Decoder::new(&clip).map_err(|e| format!("{name}: {e:?}"))?;
    let mut frames = Vec::new();
    let mut clean = true;
    for (n, packet) in clip.reader().enumerate() {
        match packet.and_then(|packet| decoder.decode(&packet).cloned()) {
            Ok(frame) => frames.push(frame),
            Err(e) => {
                eprintln!("{name}: frame {n}: {e:?}");
                clean = false;
            }
        }
    }
    // The frame time: negative in 10 µs, else in ms.
    let micros = if clip.rate < 0 {
        -i64::from(clip.rate) * 10
    } else {
        i64::from(clip.rate) * 1000
    };
    println!(
        "{name}: {}x{}, {} of {} frames, {micros} µs a frame, hash {:016x}",
        clip.width,
        clip.height,
        frames.len(),
        clip.frames(),
        h3_smacker::check::hash(&frames)
    );
    Ok((frames, clean))
}

/// Compares `ours` (`count` frames) with the reference's bytes frame by frame; prints the
/// verdict.
fn compare(name: &str, count: usize, ours: &[u8], reference: &[u8]) -> bool {
    let name = printable(name);
    let size = ours.len() / count.max(1);
    if ours.len() != reference.len() || size == 0 {
        println!(
            "{name}: {} bytes, the reference {} ({count} frames of {size})",
            ours.len(),
            reference.len(),
        );
        return ours == reference;
    }
    let mut differing = Vec::new();
    let mut pixels = 0;
    for (n, (a, b)) in ours.chunks(size).zip(reference.chunks(size)).enumerate() {
        let wrong = a.iter().zip(b).filter(|(x, y)| x != y).count();
        if wrong > 0 {
            differing.push(n);
            pixels += wrong;
        }
    }
    if differing.is_empty() {
        println!("{name}: as the reference, all {count} frames");
        return true;
    }
    println!(
        "{name}: {} of {count} frames differ from the reference ({pixels} bytes), the first {:?}",
        differing.len(),
        &differing[..differing.len().min(8)]
    );
    false
}

fn video(options: &Options) -> Result<bool, String> {
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    let mut refused = false;
    if options.samples.is_empty() || !options.names.is_empty() {
        let archive_bytes =
            fs::read(&options.vid).map_err(|e| format!("{}: {e}", options.vid.display()))?;
        let archive = VidArchive::parse(&archive_bytes)
            .map_err(|e| format!("{}: {e:?}", options.vid.display()))?;
        for entry in archive.entries() {
            if !options.wants(&entry.name) {
                continue;
            }
            if plain(&entry.name) {
                files.push((entry.name.clone(), archive.data(entry).to_vec()));
            } else {
                eprintln!(
                    "video: {}: not a plain file name, skipped",
                    printable(&entry.name)
                );
                refused = true;
            }
        }
    }
    for &seed in &options.samples {
        files.push(match options.kind {
            Kind::Bink => (format!("sample-{seed}.bik"), h3_bink::build::sample(seed)),
            Kind::Smacker => (
                format!("sample-{seed}.smk"),
                h3_smacker::build::sample(seed),
            ),
        });
    }
    if files.is_empty() {
        let format = match options.kind {
            Kind::Bink => "Bink",
            Kind::Smacker => "Smacker",
        };
        return Err(format!("no {format} file asked for is in the archive"));
    }
    fs::create_dir_all(&options.out).map_err(|e| format!("{}: {e}", options.out.display()))?;
    let mut good = !refused;
    for (name, bytes) in &files {
        let stem = name.rsplit_once('.').map_or(name.as_str(), |(s, _)| s);
        let write = |path: PathBuf, bytes: &[u8]| {
            fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))
        };
        write(options.out.join(name), bytes)?;
        // Each output: its extension, its bytes, the frames it holds.
        let (outputs, count) = match options.kind {
            Kind::Bink => {
                let (frames, clean) = decode(name, bytes, options.reading)?;
                good &= clean;
                (vec![("yuv", raw(&frames))], frames.len())
            }
            Kind::Smacker => {
                let (frames, clean) = decode_smacker(name, bytes)?;
                good &= clean;
                let outputs = vec![
                    ("pal8", h3_smacker::check::raw(&frames)),
                    ("565", original_16(&frames, Format::Rgb565)),
                ];
                (outputs, frames.len())
            }
        };
        for (extension, ours) in outputs {
            write(options.out.join(format!("{stem}.{extension}")), &ours)?;
            if let Some(dir) = &options.reference {
                let path = dir.join(format!("{stem}.{extension}"));
                let reference = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
                let label = match options.kind {
                    Kind::Bink => name.clone(),
                    Kind::Smacker => format!("{name} ({extension})"),
                };
                good &= compare(&label, count, &ours, &reference);
            }
        }
    }
    Ok(good)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn options_are_parsed() {
        let o = Options::parse(&[]).unwrap();
        assert!(o.vid.ends_with("original-data/battle-videos.vid"));
        assert!(o.out.ends_with("clients/out/video"));
        assert!(o.wants("WIN3.BIK") && o.wants("x.bik") && !o.wants("PGTRNLFT.SMK"));
        assert!(!o.wants("noextension"));
        let o = Options::parse(&args(&[
            "win3",
            "LBLOOP.BIK",
            "--vid",
            "v.vid",
            "--out",
            "o",
            "--reference",
            "r",
            "--sample",
            "7",
            "--original",
        ]))
        .unwrap();
        assert_eq!(
            (o.vid.as_path(), o.out.as_path()),
            (Path::new("v.vid"), Path::new("o"))
        );
        assert_eq!(o.reference.as_deref(), Some(Path::new("r")));
        assert_eq!(
            (o.samples.as_slice(), o.reading),
            ([7].as_slice(), Reading::Original)
        );
        assert!(o.wants("WIN3.BIK") && o.wants("lbloop.bik") && !o.wants("RTLOOP.BIK"));
        let samples_only = Options::parse(&args(&["--sample", "1"])).unwrap();
        assert!(!samples_only.wants("WIN3.BIK"));
        assert_eq!(samples_only.reading, Reading::Fixed);
        assert!(Options::parse(&args(&["--sample", "x"])).is_err());
        assert!(Options::parse(&args(&["--vid"])).is_err());
        assert!(Options::parse(&args(&["--what"])).is_err());
        let smacker = Options::parse(&args(&["--smacker", "HSLOOP"])).unwrap();
        assert!(smacker.vid.ends_with("original-data/shell-videos.vid"));
        assert_eq!(smacker.kind, Kind::Smacker);
        assert!(smacker.wants("HSLOOP.SMK") && !smacker.wants("HSANIM.SMK"));
        assert!(!smacker.wants("hsloop.bik"));
    }

    #[test]
    fn a_sample_is_written_and_compared() {
        let dir = std::env::temp_dir().join(format!("h3-view-video-{}", std::process::id()));
        let options = Options {
            names: Vec::new(),
            vid: dir.join("none.vid"),
            out: dir.clone(),
            reference: None,
            samples: vec![3],
            reading: Reading::Fixed,
            kind: Kind::Bink,
        };
        assert_eq!(video(&options), Ok(true));
        let ours = fs::read(dir.join("sample-3.yuv")).unwrap();
        let frame = 256 * 48 + 2 * 128 * 24;
        assert_eq!(ours.len(), h3_bink::build::SAMPLE_FRAMES * frame);
        // Against itself, then against a reference one byte off.
        let reference = Options {
            reference: Some(dir.clone()),
            ..options
        };
        assert_eq!(video(&reference), Ok(true));
        let mut wrong = ours.clone();
        wrong[frame + 5] ^= 1;
        let frames = h3_bink::check::frames(&h3_bink::build::sample(3)).unwrap();
        assert!(!compare("x", frames.len(), &ours, &wrong));
        assert!(!compare("x", frames.len(), &ours, &wrong[1..]));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_smacker_sample_is_written_and_compared() {
        let dir = std::env::temp_dir().join(format!("h3-view-smacker-{}", std::process::id()));
        let options = Options {
            names: Vec::new(),
            vid: dir.join("none.vid"),
            out: dir.clone(),
            reference: None,
            samples: vec![5],
            reading: Reading::Fixed,
            kind: Kind::Smacker,
        };
        assert_eq!(video(&options), Ok(true));
        let frames = h3_smacker::check::frames(&h3_smacker::build::sample(5)).unwrap();
        let pal8 = fs::read(dir.join("sample-5.pal8")).unwrap();
        assert_eq!(pal8, h3_smacker::check::raw(&frames));
        let sixteen = fs::read(dir.join("sample-5.565")).unwrap();
        assert_eq!(sixteen, original_16(&frames, Format::Rgb565));
        let reference = Options {
            reference: Some(dir.clone()),
            ..options
        };
        assert_eq!(video(&reference), Ok(true));
        // A damaged file: its failing frames are printed and the run fails.
        let mut bytes = h3_smacker::build::sample(5);
        let last = bytes.len() - 1;
        bytes.truncate(last);
        assert!(matches!(decode_smacker("x", &bytes), Ok((_, false))));
        assert!(decode_smacker("x", b"BIKb").is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn only_plain_entry_names_are_written() {
        // E17 M5 T8b1, R2b-2: an entry's name never leads out of `--out`.
        for name in ["WIN3.BIK", "x.bik", "a b.bik"] {
            assert!(plain(name), "{name}");
        }
        for name in [
            "../../x.bik",
            "/tmp/x.bik",
            "C:\\x.bik",
            "C:x.bik",
            "a\\b.bik",
            "d/x.bik",
            "x.bik/",
            "..",
            ".",
            "",
        ] {
            assert!(!plain(name), "{name}");
        }
        // Such an entry is skipped and the run fails; the others are written (E17 M5 T8b1,
        // R2b-6: its name printed without the escape).
        let dir = std::env::temp_dir().join(format!("h3-view-names-{}", std::process::id()));
        let out = dir.join("out");
        fs::create_dir_all(&dir).unwrap();
        let sample = h3_bink::build::sample(1);
        let archive =
            h3_assets::build::vid(&[("../escaped.bik", &sample), ("\u{1b}[2Jgood.bik", &sample)]);
        fs::write(dir.join("v.vid"), archive).unwrap();
        let options = Options {
            names: Vec::new(),
            vid: dir.join("v.vid"),
            out: out.clone(),
            reference: None,
            samples: Vec::new(),
            reading: Reading::Original,
            kind: Kind::Bink,
        };
        assert_eq!(video(&options), Ok(false));
        assert!(out.join("\u{1b}[2Jgood.bik").exists());
        assert!(!dir.join("escaped.bik").exists());
        fs::remove_dir_all(&dir).unwrap();
    }
}
