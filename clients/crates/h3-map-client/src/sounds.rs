//! The map's sound files (E24 M2 T5; design §3.11): the sound archives' WAVs (`h3_assets::snd`,
//! `wav`) and the music's MP3s (`nanomp3`), found in the installation copy as the battle viewer
//! finds them (`h3-view`'s `sounds` and `audio`, E17 design §8.2: `Heroes3.snd` first, then
//! `H3ab_ahd.snd`; `MP3/<name>.mp3`; names ignoring case), and the gate's check of every name the
//! map's asset tables give. An adapter: it reads files.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use h3_assets::snd::SndArchive;
use h3_assets::wav::{Pcm, decode_wav};
use h3_map_view::tables::Tables;

/// The sound archives in the order the viewers search them (`h3-view`'s `SOUND_ARCHIVES`).
pub const SOUND_ARCHIVES: [&str; 2] = ["Heroes3.snd", "H3ab_ahd.snd"];

/// A decoded MP3: its rate, channels and samples.
pub type Music = (u32, u16, Vec<i16>);

/// The entry of `dir` named `name`, ignoring ASCII case (installations differ in case; copied
/// from `h3-view`'s gate).
pub fn find_ci(dir: &Path, name: &str) -> io::Result<PathBuf> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry
            .file_name()
            .to_string_lossy()
            .eq_ignore_ascii_case(name)
        {
            return Ok(entry.path());
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "not found"))
}

/// The sound archives' bytes found in the installation's `Data`, in search order (one missing is
/// left out).
pub fn read_archives(install: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    let Ok(data) = find_ci(install, "Data") else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for name in SOUND_ARCHIVES {
        if let Ok(path) = find_ci(&data, name) {
            let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            out.push((name.to_owned(), bytes));
        }
    }
    Ok(out)
}

/// The archives parsed.
pub fn parse_archives(bytes: &[(String, Vec<u8>)]) -> Result<Vec<SndArchive<'_>>, String> {
    bytes
        .iter()
        .map(|(name, b)| SndArchive::parse(b).map_err(|e| format!("{name}: {e:?}")))
        .collect()
}

/// A WAV's stem as the archives name it (`LoopAnim.wav` → `LOOPANIM`).
#[must_use]
pub fn stem(name: &str) -> String {
    let upper = name.to_ascii_uppercase();
    upper.strip_suffix(".WAV").unwrap_or(&upper).to_owned()
}

/// The first archive's entry named `name` (a WAV name or its stem), decoded; none when no archive
/// holds it.
pub fn decode(archives: &[SndArchive<'_>], name: &str) -> Option<Result<Pcm, String>> {
    let stem = stem(name);
    let bytes = archives.iter().find_map(|a| a.find(&stem))?;
    Some(decode_wav(bytes).map_err(|e| format!("{e:?}")))
}

/// The music track `name` of the installation's `MP3` folder (`<name>.mp3`, found ignoring case),
/// decoded; the reason when it is missing or holds no audio.
pub fn mp3(install: &Path, name: &str) -> Result<Music, String> {
    let file = format!("{name}.mp3");
    let path = find_ci(install, "MP3")
        .and_then(|d| find_ci(&d, &file))
        .map_err(|e| format!("MP3/{file}: {e}"))?;
    let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let decoded = nanomp3::decode_all::<i16>(&bytes);
    let channels = decoded.channels.map_or(0, |c| c as u16);
    // A track whose frames do not decode holds no samples (M2 T6a, the reviewer's R2-1: the music's
    // fade waited for them for ever).
    if channels == 0 || decoded.sample_rate == 0 || decoded.samples.is_empty() {
        return Err(format!("{}: holds no audio", path.display()));
    }
    Ok((decoded.sample_rate, channels, decoded.samples))
}

/// The gate's check (E24 M2 T5): every sound the tables name found in the sound archives and
/// decoded, every music track found in `MP3` and decoded; a line each, the problems added.
pub fn check(install: &Path, tables: &Tables, problems: &mut Vec<String>) -> Result<(), String> {
    println!("\nsounds (the map's: the ambient loops, the step sounds): name, archive, length");
    let bytes = read_archives(install)?;
    let archives = parse_archives(&bytes)?;
    if archives.is_empty() {
        problems.push("sounds: no sound archive in Data".into());
    }
    for name in tables.sounds() {
        let archive = archives
            .iter()
            .zip(&bytes)
            .find(|(a, _)| a.find(&name).is_some())
            .map(|(_, (n, _))| n.as_str());
        match (archive, decode(&archives, &name)) {
            (Some(archive), Some(Ok(pcm))) => {
                let ms = pcm.frames() as u64 * 1000 / u64::from(pcm.rate.max(1));
                println!(
                    "  {name:<10} {archive:<13} {ms} ms, {} Hz x {}",
                    pcm.rate, pcm.channels
                );
            }
            (_, Some(Err(e))) => problems.push(format!("sound {name}: does not decode ({e})")),
            _ => problems.push(format!("sound {name}: in no sound archive")),
        }
    }
    println!("\nmusic (the terrains'): name, length");
    for name in tables.tracks() {
        match mp3(install, &name) {
            Ok((rate, channels, samples)) => {
                let seconds = samples.len() / usize::from(channels.max(1)) / rate as usize;
                println!("  {name:<12} {seconds} s, {rate} Hz x {channels}");
            }
            Err(e) => problems.push(format!("music {name}: {e}")),
        }
    }
    Ok(())
}
