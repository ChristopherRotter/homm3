//! The scenario screen's load mode (E27 M4 T4b1; `docs/epics/E27-design.md` §3.1, §3.3 to §3.6,
//! §38): what a save adds to its map's entry, the date the panel shows and the autosave's name.
//!
//! - **A save's entry** ([`Saved`]): the map's header the save's replay names (the list's columns,
//!   the panel's name, description, victory and loss), the save's difficulty (the disabled
//!   buttons show it, the rating is its) and the file's last-write time; the rows show the save's
//!   file's name (`scenario_entry_name` 0x584010 in the load mode).
//! - **The date** (`scenario_info_draw` 0x5841C8 to 0x58426B, design §38.1): the file's time
//!   through `FileTimeToLocalFileTime` and `FileTimeToSystemTime`, "%d/%d/%d - %d:%02d" of the
//!   month, the day, the year, the hour (0 to 23) and the minute: the zone's present offset,
//!   whatever the file's date ([`local`]; the adapter gives the offset, `h3_sdl::local_offset`).
//! - **The autosave** (§3.3, §3.4): a file whose name begins with GENRLTXT 77's text shows that
//!   text as its name in the panel ([`Texts::autosave`](super::Texts::autosave)); the name's sort
//!   takes one whose name begins `AUTOSAVE.` as a space and its file's name, so it leads
//!   (`scenario_compare_name` 0x5903B0, [`AUTOSAVE`]).

/// What a save adds to its map's entry (the original's `scenario_info` of a save file: the save's
/// header, `sub_4BC750`): its difficulty and its file's time.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Saved {
    /// The save's difficulty (0 easy to 4 impossible): the difficulty's buttons show it, disabled,
    /// and the rating is its.
    pub difficulty: u8,
    /// The file's last-write time, in seconds from 1970-01-01 UTC.
    pub time: i64,
}

/// The date's text item (the client's id: the info draw's own, §3.5): "%d/%d/%d - %d:%02d" at
/// (422, 27, 278, 18), colour 4, alignment 6, in the load and save modes with a selection.
pub const DATE: u16 = 1015;

/// The file name's beginning the name's sort takes as the autosave's (`scenario_compare_name`
/// 0x5903B0, the executable's own: the autosave's file `AUTOSAVE.GM<n>`), compared
/// case-insensitively (*provisional*: the compare's case not read).
pub const AUTOSAVE: &str = "AUTOSAVE.";

/// The save's file sorts as the autosave's (it begins with [`AUTOSAVE`], any case).
#[must_use]
pub fn sorts_first(file: &str) -> bool {
    file.get(..AUTOSAVE.len())
        .is_some_and(|start| start.eq_ignore_ascii_case(AUTOSAVE))
}

/// The panel shows the file `file` as the autosave: its name begins with `autosave` (GENRLTXT 77's
/// text), any case (*provisional*: the original's compare's case not read); never for an empty
/// text.
#[must_use]
pub fn is_autosave(file: &str, autosave: &str) -> bool {
    !autosave.is_empty()
        && file
            .get(..autosave.len())
            .is_some_and(|start| start.eq_ignore_ascii_case(autosave))
}

/// The local date of `time` (seconds from 1970-01-01 UTC) at the zone's present offset `offset`
/// (seconds east of UTC), as `FileTimeToSystemTime` gives it: the month (1 to 12), the day, the
/// year, the hour (0 to 23) and the minute.
#[must_use]
pub fn local(time: i64, offset: i32) -> [i64; 5] {
    let seconds = time.saturating_add(i64::from(offset));
    let days = seconds.div_euclid(86_400);
    let of_day = seconds.rem_euclid(86_400);
    let [year, month, day] = civil(days);
    [month, day, year, of_day / 3600, of_day % 3600 / 60]
}

/// The civil date (the year, the month 1 to 12, the day) of the day `days` from 1970-01-01, in
/// the proleptic Gregorian calendar (Howard Hinnant's `civil_from_days`).
fn civil(days: i64) -> [i64; 3] {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    [year, month, day]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_s_time_is_shown_at_the_present_offset() {
        // The recordings' files (design §38.2) under Asia/Kolkata (+5:30).
        let kolkata = 5 * 3600 + 1800;
        assert_eq!(local(1_003_826_100, kolkata), [10, 23, 2001, 14, 5]);
        assert_eq!(local(1_009_838_340, kolkata), [1, 1, 2002, 4, 9]);
        assert_eq!(local(1_016_336_220, kolkata), [3, 17, 2002, 9, 7]);
        // UTC: the autosave's last day of 2001.
        assert_eq!(local(1_009_838_340, 0), [12, 31, 2001, 22, 39]);
        // West of UTC across a year's end, the epoch, a leap day, before 1970.
        assert_eq!(local(0, -3600), [12, 31, 1969, 23, 0]);
        assert_eq!(local(0, 0), [1, 1, 1970, 0, 0]);
        assert_eq!(local(951_782_400, 0), [2, 29, 2000, 0, 0]);
        assert_eq!(local(-86_400, 0), [12, 31, 1969, 0, 0]);
        assert_eq!(local(4_107_542_400, 0), [3, 1, 2100, 0, 0]);
    }

    #[test]
    fn the_autosave_is_known_by_its_file_s_name() {
        assert!(sorts_first("AUTOSAVE.GM1"));
        assert!(sorts_first("autosave.gm2"));
        assert!(!sorts_first("AUTOSAVE1.GM1"));
        assert!(!sorts_first("AUTO"));
        assert!(!sorts_first("small.GM1"));
        assert!(is_autosave("AUTOSAVE.GM1", "AUTOSAVE"));
        assert!(is_autosave("Autosave2.GM1", "AUTOSAVE"));
        assert!(!is_autosave("AUTO.GM1", "AUTOSAVE"));
        assert!(!is_autosave("AUTOSAVE.GM1", ""));
        // A multi-byte character at the cut: no panic, no match.
        assert!(!is_autosave("AUTOSAVé", "AUTOSAVE"));
    }
}
