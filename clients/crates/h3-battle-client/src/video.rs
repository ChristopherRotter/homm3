//! `h3-view video` (E17 M5 T4b2a): the Bink clips of a video archive decoded by `h3-bink`, each
//! written out as the file it is and as its frames in raw YUV 4:2:0 (the bytes of FFmpeg's
//! `rawvideo` output in `yuv420p`), and compared with a reference decoding of the same files
//! (the game's `BINKW32.DLL`'s since E17 M6 T6c1, read as it reads with `--original`, each frame
//! as its `BinkCopyToBuffer` copies it since E27 M2 T2b, `h3_bink::Target`:
//! `clients/scripts/bink-check.sh`; FFmpeg's before). The synthetic samples
//! (`h3_bink::build::sample`) go the same way, so that the reference checks the block types the
//! game's clips may not use. With `--smacker` (E27 M2 T1) the Smacker clips the same way through
//! `h3-smacker` (default archive `original-data/shell-videos.vid`), each written as its frames'
//! colour indices and palettes (`<stem>.pal8`, `h3_smacker::check::raw`) and as the original's
//! 5-6-5 copy of them (`<stem>.565`, `check::original_16`), both compared with the game's
//! `SMACKW32.DLL`'s (`clients/scripts/smacker-check.sh`). Since E27 M2 T2 a clip's sound too:
//! a Bink clip's first track (`<stem>.pcm`, `h3_bink::check::sound`), each of a Smacker clip's
//! (`<stem>.t<k>.pcm`, `h3_smacker::check::sound`), compared whole; `--sound-tables <file>`
//! writes Bink's sound tables (`h3_bink::check::sound_tables`, against `binkprobe sound-tables`);
//! `--play` plays each clip's sound on the default device as the client will (`h3_view::clip_sound`
//! at the effects setting 9, `HostSound::play_clip`), one after another.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_assets::vid::VidArchive;
use h3_bink::{Clip, Decoder, Frame, Reading, Target};
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
    /// `--sound-tables <file>` (E27 M2 T2): where to write Bink's sound tables.
    tables: Option<PathBuf>,
    /// `--play` (E27 M2 T2): each clip's sound played on the default device.
    play: bool,
}

impl Options {
    /// Parses `[<name>...] [--vid <file>] [--out <dir>] [--reference <dir>] [--sample <seed>]...
    /// [--original] [--smacker] [--sound-tables <file>] [--play]`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let (mut vid, mut out, mut reference, mut tables) = (None, None, None, None);
        let (mut names, mut samples) = (Vec::new(), Vec::new());
        let mut reading = Reading::Fixed;
        let mut kind = Kind::Bink;
        let mut play = false;
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
                "--sound-tables" => tables = Some(PathBuf::from(value()?)),
                "--play" => play = true,
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
            tables,
            play,
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
    // Read as the DLL, each frame as the game gets it (E27 M2 T2b, CLT-Q122).
    let mut target = Target::new(&decoder);
    let mut frames = Vec::new();
    let mut clean = true;
    for n in 0..clip.frames().len() {
        match clip.video(n).and_then(|packet| decoder.decode(packet)) {
            Ok(_) if reading == Reading::Original => frames.push(target.copy(&decoder).clone()),
            Ok(frame) => frames.push(frame.clone()),
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

/// Where `SmackToBuffer` puts a clip for its rectangles' check (E27 M2 T5a2), each with its
/// file's extension: at the buffer's corner, and at a left edge off the DLL's 8-pixel grid
/// (`H3Intro`'s top). `smacker-dll.py` runs `smackprobe rects` at the same places.
const RECTS_AT: [(&str, [u32; 2]); 2] = [("rects", [0, 0]), ("shifted.rects", [83, 187])];

/// The clock steps (ms) `smackprobe wait` and the model play a clip with its sound at (E27 M2
/// T5b1); `smacker-dll.py` runs the probe at the same steps, the effects setting 5.
const WAIT_STEPS: [u32; 5] = [1, 5, 7, 10, 16];

/// A Smacker clip played full screen with its sound on (E27 M2 T5b1;
/// `h3_view::shell::start::SmackSound`), a reading every `step` ms from its open at 0, in
/// `smacker-dll.py`'s text of `smackprobe wait`'s log: `0 volume <v>` (the DirectSound volume's
/// index, `clip_sound::volume` at the effects setting 5), then `<t> play` (the buffer's Play),
/// `<t> frame <k>` (a frame decoded) and `<t> stop <bytes>` (the close: the track's bytes
/// played). None for a clip without a track, or one whose first frame's sound does not fill the
/// DLL's buffer (half a second): the model's limit (`SmackSound`: the buffer never runs dry, as
/// in every clip of the game; the synthetic samples' few hundred bytes a frame run it dry, the
/// DLL's underrun path, not modelled).
fn wait_text(clip: &h3_smacker::Clip<'_>, step: u32) -> Option<Vec<u8>> {
    use h3_view::shell::start::{SmackSound, smack_period};
    use std::fmt::Write as _;
    // The first track present is the one the frames wait on (`SmackOpen`'s +0x444); its bytes
    // a second as the DLL takes them (16-bit and stereo doubling the rate).
    let (k, &track) = clip
        .tracks
        .iter()
        .enumerate()
        .find(|(_, t)| t.rate != 0 && t.flags & 0x40 != 0)?;
    let rate =
        track.rate * (1 + u32::from(track.flags >> 5 & 1)) * (1 + u32::from(track.flags >> 4 & 1));
    let first = clip.reader().next()?.ok()?;
    let filled = match h3_smacker::sound::decode(track, first.audio[k]?).ok()?? {
        h3_smacker::Samples::Sixteen(s) => s.len() * 2,
        h3_smacker::Samples::Eight(s) => s.len(),
    };
    if filled < usize::try_from(((rate >> 1) + 0xFFF) & !0xFFF).ok()? {
        return None;
    }
    let frames = u32::try_from(clip.frames() - usize::from(clip.ring())).ok()?;
    let mut sound = SmackSound::new(rate, smack_period(clip.rate));
    let volume = h3_view::clip_sound::volume(
        h3_view::clip_sound::Format::Smacker,
        5,
        h3_render::Bugs::Original,
    );
    let mut text = format!("0 volume {volume}\n");
    let mut now = 0u32;
    let mut k = 0;
    loop {
        // smacker_tick at a reading, again until SmackWait holds a frame back (the probe's 64).
        for _ in 0..64 {
            let (due, started) = sound.wait(now);
            if started {
                let _ = writeln!(text, "{now} play");
            }
            if !due {
                break;
            }
            sound.frame(now);
            let _ = writeln!(text, "{now} frame {k}");
            if k + 1 >= frames {
                let _ = writeln!(text, "{now} stop {}", sound.heard_by(now));
                return Some(text.into_bytes());
            }
            k += 1;
        }
        now += step;
        if now > 600_000 {
            return None;
        }
    }
}

/// A clip's rectangles in `smackprobe rects`' text: a line a frame, its number, then for each
/// call whether it took a rectangle and the last one after it, `<took>:<x>,<y>,<w>,<h>`.
fn rects_text(calls: &[Vec<(bool, h3_smacker::Rect)>]) -> Vec<u8> {
    let mut out = String::new();
    for (n, frame) in calls.iter().enumerate() {
        out.push_str(&n.to_string());
        for &(took, [x, y, w, h]) in frame {
            out.push_str(&format!(" {}:{x},{y},{w},{h}", u8::from(took)));
        }
        out.push('\n');
    }
    out.into_bytes()
}

/// Compares two texts line by line (a frame a line); prints the outcome and the first line
/// that differs.
fn compare_lines(name: &str, ours: &[u8], reference: &[u8]) -> bool {
    let (ours, reference) = (
        String::from_utf8_lossy(ours),
        String::from_utf8_lossy(reference),
    );
    let (a, b): (Vec<&str>, Vec<&str>) = (ours.lines().collect(), reference.lines().collect());
    let differ: Vec<usize> = (0..a.len().max(b.len()))
        .filter(|&n| a.get(n) != b.get(n))
        .collect();
    match differ.first() {
        None => println!("{name}: as the reference, all {} frames", a.len()),
        Some(&n) => println!(
            "{name}: {} of {} frames differ from the reference, the first {n}: {:?}, the \
             reference {:?}",
            differ.len(),
            a.len().max(b.len()),
            a.get(n).copied().unwrap_or(""),
            b.get(n).copied().unwrap_or(""),
        ),
    }
    differ.is_empty()
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
        if count == 1 {
            println!("{name}: as the reference, all {} bytes", ours.len());
        } else {
            println!("{name}: as the reference, all {count} frames");
        }
        return true;
    }
    println!(
        "{name}: {} of {count} frames differ from the reference ({pixels} bytes), the first {:?}",
        differing.len(),
        &differing[..differing.len().min(8)]
    );
    false
}

/// Plays the clip's sounds on the default device, waiting until each ends.
fn play(name: &str, bytes: &[u8], kind: Kind) -> Result<(), String> {
    use h3_view::clip_sound::{ClipSound, Format, volume};
    let (sounds, format) = match kind {
        Kind::Bink => (
            ClipSound::bink(bytes)
                .map_err(|e| format!("{e:?}"))?
                .into_iter()
                .collect::<Vec<_>>(),
            Format::Bink,
        ),
        Kind::Smacker => (
            ClipSound::smacker(bytes).map_err(|e| format!("{e:?}"))?,
            Format::Smacker,
        ),
    };
    if sounds.is_empty() {
        return Ok(());
    }
    let mut host = crate::audio::HostSound::open(h3_render::Bugs::Fixed)?;
    for sound in &sounds {
        println!(
            "{}: playing {} Hz × {}, {} ms",
            printable(name),
            sound.rate,
            sound.channels,
            sound.samples.len() as u64 * 1000
                / (u64::from(sound.rate) * u64::from(sound.channels)).max(1)
        );
        let stream = host.play_clip(sound, volume(format, 9, h3_render::Bugs::Fixed))?;
        while host.clip_playing(stream) {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        host.stop_clip(stream);
    }
    Ok(())
}

fn video(options: &Options) -> Result<bool, String> {
    if let Some(path) = &options.tables {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        fs::write(path, h3_bink::check::sound_tables())
            .map_err(|e| format!("{}: {e}", path.display()))?;
        if options.names.is_empty() && options.samples.is_empty() {
            return Ok(true);
        }
    }
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
                let mut outputs = vec![("yuv".to_owned(), raw(&frames))];
                // The first track's sound (E27 M2 T2), compared whole.
                if Clip::parse(bytes).is_ok_and(|clip| !clip.tracks.is_empty()) {
                    let sound = h3_bink::check::sound(bytes).map_err(|e| format!("{e:?}"))?;
                    outputs.push(("pcm".to_owned(), sound));
                }
                (outputs, frames.len())
            }
            Kind::Smacker => {
                let (frames, clean) = decode_smacker(name, bytes)?;
                good &= clean;
                let mut outputs = vec![
                    ("pal8".to_owned(), h3_smacker::check::raw(&frames)),
                    ("565".to_owned(), original_16(&frames, Format::Rgb565)),
                ];
                // Each track's sound (E27 M2 T2), compared whole.
                for (k, sound) in h3_smacker::check::sound(bytes).map_err(|e| format!("{e:?}"))? {
                    outputs.push((format!("t{k}.pcm"), sound));
                }
                // `SmackToBufferRect`'s rectangles (E27 M2 T5a2), compared frame by frame.
                for (extension, at) in RECTS_AT {
                    let calls =
                        h3_smacker::check::rects(bytes, at).map_err(|e| format!("{e:?}"))?;
                    outputs.push((extension.to_owned(), rects_text(&calls)));
                }
                // The frames' times with the sound on (E27 M2 T5b1), line by line.
                let clip = h3_smacker::Clip::parse(bytes).map_err(|e| format!("{e:?}"))?;
                for step in WAIT_STEPS {
                    if let Some(text) = wait_text(&clip, step) {
                        outputs.push((format!("wait{step}"), text));
                    }
                }
                (outputs, frames.len())
            }
        };
        if options.play {
            play(name, bytes, options.kind)?;
        }
        for (extension, ours) in outputs {
            let count = if extension.ends_with("pcm") { 1 } else { count };
            write(options.out.join(format!("{stem}.{extension}")), &ours)?;
            if let Some(dir) = &options.reference {
                let path = dir.join(format!("{stem}.{extension}"));
                let reference = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
                let label = match options.kind {
                    Kind::Bink if extension == "yuv" => name.clone(),
                    _ => format!("{name} ({extension})"),
                };
                good &= if extension.ends_with("rects") || extension.starts_with("wait") {
                    compare_lines(&label, &ours, &reference)
                } else {
                    compare(&label, count, &ours, &reference)
                };
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
            tables: Some(dir.join("tables")),
            play: false,
        };
        assert_eq!(video(&options), Ok(true));
        // The sound (E27 M2 T2) and the sound tables beside the frames.
        assert_eq!(
            fs::read(dir.join("sample-3.pcm")).unwrap(),
            h3_bink::check::sound(&h3_bink::build::sample(3)).unwrap()
        );
        assert_eq!(
            fs::read(dir.join("tables")).unwrap(),
            h3_bink::check::sound_tables()
        );
        let tables_only = Options {
            samples: Vec::new(),
            ..Options::parse(&args(&[
                "--sound-tables",
                &dir.join("t").display().to_string(),
            ]))
            .unwrap()
        };
        assert_eq!(video(&tables_only), Ok(true));
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
            tables: None,
            play: false,
        };
        assert_eq!(video(&options), Ok(true));
        let frames = h3_smacker::check::frames(&h3_smacker::build::sample(5)).unwrap();
        let pal8 = fs::read(dir.join("sample-5.pal8")).unwrap();
        assert_eq!(pal8, h3_smacker::check::raw(&frames));
        let sixteen = fs::read(dir.join("sample-5.565")).unwrap();
        assert_eq!(sixteen, original_16(&frames, Format::Rgb565));
        let sound = h3_smacker::check::sound(&h3_smacker::build::sample(5)).unwrap();
        assert_eq!(fs::read(dir.join("sample-5.t0.pcm")).unwrap(), sound[0].1);
        // SmackToBufferRect's calls (E27 M2 T5a2), at the corner and shifted.
        for (extension, at) in RECTS_AT {
            let calls = h3_smacker::check::rects(&h3_smacker::build::sample(5), at).unwrap();
            let text = fs::read(dir.join(format!("sample-5.{extension}"))).unwrap();
            assert_eq!(text, rects_text(&calls));
            assert!(text.starts_with(b"0 1:"));
        }
        // Compared line by line (the probe's CRLF read as LF): a line that differs or lacks
        // fails.
        assert!(compare_lines("x", b"0 1:0,0,4,4\n", b"0 1:0,0,4,4\r\n"));
        assert!(!compare_lines("x", b"0 1\n1 0\n", b"0 1\n1 1\n"));
        assert!(!compare_lines("x", b"0 1\n1 0\n", b"0 1\n"));
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
            tables: None,
            play: false,
        };
        assert_eq!(video(&options), Ok(false));
        assert!(out.join("\u{1b}[2Jgood.bik").exists());
        assert!(!dir.join("escaped.bik").exists());
        fs::remove_dir_all(&dir).unwrap();
    }
}
