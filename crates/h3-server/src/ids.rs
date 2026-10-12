//! Game ids unique for the server's life (E16 M4 T3b; `E16-design.md` §11): `<data>/next-game`
//! holds an id higher than any a game was given. The lobby hands ids out under the registry's lock,
//! from the file's number on at start; a game's own thread records its id here before it creates
//! its journal ([`GameIds::reach`]), so every id a journal, a replay or a ladder's results line
//! names is below the file's number. An id handed out just before a crash, its journal not created
//! yet, may be handed out again: nothing names it. At start, a journal of a server from before the
//! file raises it too (`registry::restore`).
//!
//! Provisional (E16 M4 T3b): the file's name and form (the number in decimal and a newline, written
//! whole: a `.part` file renamed), one write per game created.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use crate::{Error, journal};

/// The ids handed out, as `<data>/next-game` records them.
#[derive(Debug)]
pub(crate) struct GameIds {
    /// The file.
    path: PathBuf,
    /// Its number: higher than every id a game was given (1 for none).
    next: Mutex<u64>,
}

impl GameIds {
    /// Reads the data directory `data`'s `next-game` (none: no game was given an id yet). A file
    /// that does not hold a number stops the start.
    pub(crate) fn open(data: &Path) -> Result<Self, Error> {
        let path = data.join("next-game");
        let next = match std::fs::read_to_string(&path) {
            Ok(text) => text
                .trim()
                .parse::<u64>()
                .ok()
                .filter(|&next| next > 0)
                .ok_or_else(|| Error(format!("{}: not a game id", path.display())))?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => 1,
            Err(e) => return Err(Error(format!("{}: {e}", path.display()))),
        };
        Ok(Self {
            path,
            next: Mutex::new(next),
        })
    }

    /// The highest id given so far (0 for none): the lobby hands out higher ones.
    pub(crate) fn last(&self) -> u32 {
        let next = *self.next.lock().unwrap_or_else(PoisonError::into_inner);
        u32::try_from(next - 1).unwrap_or(u32::MAX)
    }

    /// Game `game` exists from now on: the file's number is beyond it, written and synced when
    /// this returns. Called on the game's thread, never under the registry's lock (file I/O).
    pub(crate) fn reach(&self, game: u32) -> io::Result<()> {
        let mut next = self.next.lock().unwrap_or_else(PoisonError::into_inner);
        let beyond = u64::from(game) + 1;
        if beyond > *next {
            journal::replace(&self.path, &format!("{beyond}\n"))?;
            *next = beyond;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn data(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("h3-ids-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn ids_are_recorded_beyond_every_game_and_read_back() {
        let dir = data("read-back");
        let ids = GameIds::open(&dir).unwrap();
        assert_eq!(ids.last(), 0, "no file: no game yet");
        ids.reach(7).unwrap();
        ids.reach(3).unwrap();
        assert_eq!(ids.last(), 7, "a lower id changes nothing");
        assert_eq!(fs::read_to_string(dir.join("next-game")).unwrap(), "8\n");
        assert!(!dir.join("next-game.part").exists());
        assert_eq!(GameIds::open(&dir).unwrap().last(), 7);
        ids.reach(u32::MAX).unwrap();
        assert_eq!(GameIds::open(&dir).unwrap().last(), u32::MAX);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_file_that_holds_no_id_stops_the_start() {
        let dir = data("bad");
        for text in ["", "0\n", "-1\n", "seven\n"] {
            fs::write(dir.join("next-game"), text).unwrap();
            let e = GameIds::open(&dir).unwrap_err();
            assert!(e.0.ends_with("next-game: not a game id"), "{e}");
        }
        fs::remove_file(dir.join("next-game")).unwrap();
        fs::create_dir(dir.join("next-game")).unwrap();
        assert!(GameIds::open(&dir).is_err(), "a directory");
        fs::remove_dir_all(&dir).unwrap();
    }
}
