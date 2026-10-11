//! The archive of finished games (E16 M2 T5d, SRV-Q15): the replays of the games that verified,
//! public. Any client lists them (`ListReplays`, newest first, a page at a time) and downloads one
//! by its id (`FetchArchived`) without joining a game. A game its host had closed to spectators is
//! archived apart and unlisted, for the operator (`<data>/replays/closed/`). Beyond
//! `[limits] replays_kept` replays in either, the oldest file goes. At start the archive is read
//! back from the files, oldest first by their last write.
//!
//! Provisional: the id (the file's stem: the offer, the game's id and a random suffix, so ids stay
//! unique across restarts), the page of [`PAGE`], the order.

use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use h3_protocol::wire::ReplayInfo;

/// The replays a `Replays` page lists at most.
pub(crate) const PAGE: usize = 50;

/// One archived replay.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Archived {
    info: ReplayInfo,
    path: PathBuf,
}

/// The archive: the listed replays and the unlisted ones, each oldest first.
#[derive(Debug, Default)]
pub(crate) struct Archive {
    listed: VecDeque<Archived>,
    closed: VecDeque<Archived>,
    kept: usize,
}

/// Where the replays of game `game` go: the listed ones in `<data>/replays/`, those of a game
/// closed to spectators in `<data>/replays/closed/`.
pub(crate) fn dir(data: &Path, listed: bool) -> PathBuf {
    let replays = data.join("replays");
    if listed {
        replays
    } else {
        replays.join("closed")
    }
}

/// Writes a replay to `path` so that a crash leaves the whole file or none (E16 M2 T5e: the
/// reviewer's finding): to a `.part` file beside it, synced, then renamed and its directory synced.
/// Only then may the game's journal go.
pub(crate) fn write(path: &Path, text: &str) -> io::Result<()> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(dir)?;
    let part = path.with_extension("toml.part");
    let mut file = File::create(&part)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    fs::rename(&part, path)?;
    #[cfg(unix)]
    File::open(dir)?.sync_all()?;
    Ok(())
}

/// The replay's info from its file name, `<offer>-game<id>-<random>.toml`.
fn info(path: &Path) -> Option<ReplayInfo> {
    let stem = path
        .extension()
        .is_some_and(|e| e == "toml")
        .then(|| path.file_stem())??
        .to_str()?;
    let (rest, random) = stem.rsplit_once('-')?;
    let (offer, game) = rest.rsplit_once("-game")?;
    (!offer.is_empty() && !random.is_empty()).then_some(())?;
    Some(ReplayInfo {
        id: stem.to_owned(),
        offer: offer.to_owned(),
        game: game.parse().ok()?,
    })
}

/// The replays in `dir`, oldest first by their last write (then by name). A `.part` file is a
/// replay a crash cut short ([`write`]): it goes, and the game's journal, still there, writes the
/// replay again at this start.
fn read(dir: &Path) -> VecDeque<Archived> {
    let mut found: Vec<_> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension().is_some_and(|e| e == "part") {
                let _ = fs::remove_file(&path);
                return None;
            }
            let written = fs::metadata(&path).and_then(|m| m.modified()).ok()?;
            let info = info(&path)?;
            Some((written, Archived { info, path }))
        })
        .collect();
    found.sort_by(|(a, x), (b, y)| a.cmp(b).then_with(|| x.info.id.cmp(&y.info.id)));
    found.into_iter().map(|(_, archived)| archived).collect()
}

impl Archive {
    /// The archive the data directory holds, at most `kept` replays in each part (the oldest
    /// beyond go).
    pub(crate) fn load(data: &Path, kept: usize) -> Self {
        let mut archive = Self {
            listed: read(&dir(data, true)),
            closed: read(&dir(data, false)),
            kept,
        };
        archive.trim();
        archive
    }

    /// The replay just written to `path`, listed or not (`listed`).
    pub(crate) fn add(&mut self, path: PathBuf, listed: bool) {
        if let Some(info) = info(&path) {
            let part = if listed {
                &mut self.listed
            } else {
                &mut self.closed
            };
            part.push_back(Archived { info, path });
            self.trim();
        }
    }

    /// The oldest go, files and all, while a part holds more than `kept`.
    fn trim(&mut self) {
        for part in [&mut self.listed, &mut self.closed] {
            while part.len() > self.kept
                && let Some(oldest) = part.pop_front()
            {
                if let Err(e) = fs::remove_file(&oldest.path) {
                    eprintln!("h3-server: replay {}: {e}", oldest.path.display());
                }
            }
        }
    }

    /// How many listed replays there are, and the page of them from the `from`-th newest.
    pub(crate) fn page(&self, from: u32) -> (u32, Vec<ReplayInfo>) {
        let total = u32::try_from(self.listed.len()).unwrap_or(u32::MAX);
        let from = usize::try_from(from).unwrap_or(usize::MAX);
        let page = self.listed.iter().rev().skip(from).take(PAGE);
        (total, page.map(|a| a.info.clone()).collect())
    }

    /// The file of the listed replay `id`.
    pub(crate) fn path(&self, id: &str) -> Option<PathBuf> {
        let found = self.listed.iter().find(|a| a.info.id == id);
        found.map(|a| a.path.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_replays_name_gives_its_info() {
        let named = |name: &str| info(Path::new(name));
        assert_eq!(
            named("/d/duel-bot-game12-00000000000000ff.toml"),
            Some(ReplayInfo {
                id: String::from("duel-bot-game12-00000000000000ff"),
                offer: String::from("duel-bot"),
                game: 12,
            })
        );
        for other in [
            "duel-game12-ff.txt",
            "duel-gameX-ff.toml",
            "-game1-ff.toml",
            "duel-game1-.toml",
            "notes.toml",
        ] {
            assert_eq!(named(other), None, "{other}");
        }
    }
}
