//! The video archives (E17 M5 T4b2b): where the game finds the result window's clips and reading
//! them entry by entry. The Complete edition (3) looks a clip up as `<name>.bik`, ignoring case,
//! in `data\Video.vid`, then in the CD's archive when one is open, then in `data\h3ab_ahd.vid`
//! (`video_open` 0x44D5A0; the archives opened at 0x598210); the viewer skips the CD's. Only the
//! directories are read at the start (`VIDEO.VID` is 629 MB), a clip's bytes when a batch takes
//! it. An installation copy without `VIDEO.VID` (a cloud sandbox's: the 629 MB stay on the user's
//! computer) takes the repository's `original-data/battle-videos.vid` in its place, the user's
//! copy of the clips the window plays (a `VIDEO.VID` of its own, as the capture harness's run
//! directory has it).

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use h3_assets::vid::{VidEntry, directory, directory_bytes};

use crate::gate::find_ci;

/// The video archives found, in the order the game searches them.
#[derive(Debug, Clone, Default)]
pub struct Videos {
    archives: Vec<(PathBuf, Vec<VidEntry>)>,
}

/// The stand-in for an installation copy's missing `VIDEO.VID`.
fn stand_in() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../original-data/battle-videos.vid")
}

/// An archive's directory, reading only its head.
fn read_directory(path: &Path) -> Result<Vec<VidEntry>, String> {
    let failed = |e: std::io::Error| format!("{}: {e}", path.display());
    let mut file = File::open(path).map_err(failed)?;
    let len = file.metadata().map_err(failed)?.len();
    let mut count = [0; 4];
    file.read_exact(&mut count).map_err(failed)?;
    let size = directory_bytes(&count)
        .filter(|&n| u64::try_from(n).is_ok_and(|n| n <= len))
        .ok_or_else(|| format!("{}: not a video archive", path.display()))?;
    let mut head = vec![0; size];
    file.seek(SeekFrom::Start(0)).map_err(failed)?;
    file.read_exact(&mut head).map_err(failed)?;
    let end = u32::try_from(len).map_err(|_| format!("{}: too large", path.display()))?;
    directory(&head, end).map_err(|e| format!("{}: {e:?}", path.display()))
}

impl Videos {
    /// The archives of `install`'s `Data` folder the game reads (`Video.vid`, else the stand-in;
    /// then `h3ab_ahd.vid`); one missing or unreadable is left out, so that a copy without
    /// videos shows the window's slot bare, as the original does then.
    pub fn open(install: &Path) -> Self {
        let data = find_ci(install, "Data").unwrap_or_else(|_| install.join("Data"));
        let main = find_ci(&data, "Video.vid").unwrap_or_else(|_| stand_in());
        let expansion = find_ci(&data, "h3ab_ahd.vid").ok();
        let archives = [Some(main), expansion]
            .into_iter()
            .flatten()
            .filter_map(|path| read_directory(&path).ok().map(|d| (path, d)))
            .collect();
        Self { archives }
    }

    /// The archive and entry that hold `name`, the first in the search order.
    fn find(&self, name: &str) -> Option<(&Path, &VidEntry)> {
        self.archives.iter().find_map(|(path, entries)| {
            let entry = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))?;
            Some((path.as_path(), entry))
        })
    }

    /// Whether an archive holds `name`.
    pub fn has(&self, name: &str) -> bool {
        self.find(name).is_some()
    }

    /// The bytes of `name`.
    pub fn read(&self, name: &str) -> Result<Vec<u8>, String> {
        let (path, entry) = self
            .find(name)
            .ok_or_else(|| format!("{name}: in no video archive"))?;
        let failed = |e: std::io::Error| format!("{}:{name}: {e}", path.display());
        let mut file = File::open(path).map_err(failed)?;
        file.seek(SeekFrom::Start(u64::from(entry.offset)))
            .map_err(failed)?;
        let mut bytes = vec![0; usize::try_from(entry.size).unwrap_or(0)];
        file.read_exact(&mut bytes).map_err(failed)?;
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clips_are_read_entry_by_entry_in_the_search_order() {
        let dir = std::env::temp_dir().join(format!("h3-view-videos-{}", std::process::id()));
        let data = dir.join("DATA");
        std::fs::create_dir_all(&data).unwrap();
        let main = h3_assets::build::vid(&[("Win3.BIK", b"main"), ("rtloop.bik", b"loop")]);
        let ab = h3_assets::build::vid(&[("WIN3.bik", b"ab"), ("extra.bik", b"x")]);
        std::fs::write(data.join("video.VID"), main).unwrap();
        std::fs::write(data.join("H3AB_AHD.vid"), ab).unwrap();
        let videos = Videos::open(&dir);
        assert_eq!(videos.read("win3.bik").unwrap(), b"main");
        assert_eq!(videos.read("RTLOOP.BIK").unwrap(), b"loop");
        assert_eq!(videos.read("extra.bik").unwrap(), b"x");
        assert!(videos.has("Extra.bik") && !videos.has("none.bik"));
        assert!(videos.read("none.bik").is_err());
        std::fs::write(data.join("H3AB_AHD.vid"), [9, 0, 0, 0]).unwrap();
        assert!(!Videos::open(&dir).has("extra.bik"));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
