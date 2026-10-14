//! The sound archives in the real-asset gate (E17 M2 T5b; `docs/epics/E17-design.md` §8.2):
//! every entry of `Heroes3.snd` and `H3ab_ahd.snd` decodes with the pure decoder
//! (`h3_assets::wav`) to exactly the samples SDL's own loader gives with the hints for the whole
//! sound (`h3_sdl::load_wav`, `dropframe` and `truncate`); SDL's defaults, which drop a cut last
//! block, are listed beside them. Every entry with its verdict goes to `<out>/gate/sounds.tsv`.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use h3_assets::snd::SndArchive;
use h3_assets::wav::{Pcm, decode_wav};

use crate::gate::find_ci;

/// The sound archives, in the order the viewer searches them (their common entries are equal in
/// the user's files: the game's order is not read).
pub const SOUND_ARCHIVES: [&str; 2] = ["Heroes3.snd", "H3ab_ahd.snd"];

/// One entry's verdict.
#[derive(Debug)]
struct Verdict {
    archive: &'static str,
    name: String,
    ours: Result<Pcm, String>,
    sdl: Result<(u32, u16, Vec<i16>), String>,
    sdl_default: Result<(u32, u16, Vec<i16>), String>,
}

impl Verdict {
    /// Ours equal SDL's for the whole sound, and SDL's defaults' samples as far as both go (the
    /// defaults ignore the `fact` chunk: 18 entries end inside their whole blocks).
    fn equal(&self) -> bool {
        matches!((&self.ours, &self.sdl, &self.sdl_default), (Ok(o), Ok((r, c, s)), Ok((_, _, d)))
            if o.rate == *r && o.channels == *c && o.samples == *s
                && s.iter().zip(d).all(|(a, b)| a == b))
    }
}

/// The frames of SDL's samples.
fn frames(sdl: &Result<(u32, u16, Vec<i16>), String>) -> String {
    sdl.as_ref().map_or_else(
        |e| format!("error {e}"),
        |(_, c, s)| (s.len() / usize::from((*c).max(1))).to_string(),
    )
}

/// Checks every entry of the sound archives under `data`, and that every name of `names` (the
/// asset tables' sounds, upper-cased) is one of them; the summary's lines, or what failed.
/// Archives that are missing are named in the summary and skipped.
pub fn check(data: &Path, out: &Path, names: &BTreeSet<String>) -> Result<String, String> {
    let mut summary = String::new();
    let mut verdicts = Vec::new();
    for archive in SOUND_ARCHIVES {
        let Ok(path) = find_ci(data, archive) else {
            let _ = writeln!(summary, "sounds: no {archive}");
            continue;
        };
        let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let snd = SndArchive::parse(&bytes).map_err(|e| format!("{archive}: {e:?}"))?;
        for entry in snd.entries() {
            let wav = snd.data(entry);
            verdicts.push(Verdict {
                archive,
                name: entry.name.clone(),
                ours: decode_wav(wav).map_err(|e| format!("{e:?}")),
                sdl: h3_sdl::load_wav(wav, true),
                sdl_default: h3_sdl::load_wav(wav, false),
            });
        }
    }
    if verdicts.is_empty() {
        return Ok(summary);
    }
    let mut listing = String::from(
        "archive\tname\tchannels\trate\tframes\tsdl_frames\tsdl_default_frames\tverdict\n",
    );
    let (mut ours_total, mut default_total) = (0usize, 0usize);
    for v in &verdicts {
        let (channels, rate, ours) = v.ours.as_ref().map_or_else(
            |e| (String::from("-"), String::from("-"), format!("error {e}")),
            |p| {
                ours_total += p.frames();
                (
                    p.channels.to_string(),
                    p.rate.to_string(),
                    p.frames().to_string(),
                )
            },
        );
        if let Ok((_, c, s)) = &v.sdl_default {
            default_total += s.len() / usize::from((*c).max(1));
        }
        let verdict = if v.equal() { "equal" } else { "different" };
        let _ = writeln!(
            listing,
            "{}\t{}\t{channels}\t{rate}\t{ours}\t{}\t{}\t{verdict}",
            v.archive,
            v.name,
            frames(&v.sdl),
            frames(&v.sdl_default),
        );
    }
    let path = out.join("gate").join("sounds.tsv");
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    fs::write(&path, listing).map_err(|e| format!("{}: {e}", path.display()))?;
    let path = fs::canonicalize(&path).unwrap_or(path);
    let present: BTreeSet<String> = verdicts
        .iter()
        .map(|v| v.name.to_ascii_uppercase())
        .collect();
    let missing: Vec<&str> = names.difference(&present).map(String::as_str).collect();
    if !missing.is_empty() {
        return Err(format!(
            "{summary}sounds: the asset tables name sounds in no archive: {}",
            missing.join(", ")
        ));
    }
    let _ = writeln!(
        summary,
        "sounds: the asset tables' {} sounds are found",
        names.len()
    );
    let different: Vec<&str> = verdicts
        .iter()
        .filter(|v| !v.equal())
        .map(|v| v.name.as_str())
        .collect();
    let _ = writeln!(
        summary,
        "sounds: {} entries of {}; {} decode to SDL's samples ({} frames; SDL's defaults: {}); \
         every entry in {}",
        verdicts.len(),
        SOUND_ARCHIVES.join(" and "),
        verdicts.len() - different.len(),
        ours_total,
        default_total,
        path.display(),
    );
    if different.is_empty() {
        Ok(summary)
    } else {
        Err(format!(
            "{summary}sounds: {} entries differ from SDL's: {}",
            different.len(),
            different.join(", ")
        ))
    }
}
