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
//! - **Delete** (E27 M4 T4b2; `saved_game_delete` 0x58DFB0, design §38.1, §40): over a selected
//!   save GENRLTXT 688 with its file's name in a box of style 2 ([`Asking`]); Cancel keeps it (the
//!   box takes its pixels back), OK deletes it: the file the caller's to delete from its folder
//!   ([`Scenario::take_deleted`]; the original's `DeleteFileA`, whose failure it does not check),
//!   the entry gone from both lists, the bar's range; with saves left the full list sorted by the
//!   version in the direction the last sort left (`scenario_list_sort`(2), the recorded key kept),
//!   the first selected with the select's partial redraw (CLT-Q138: the difficulty's buttons and
//!   the rating stale); with none the whole redraw, nothing selected (the panel as an empty
//!   list's) and Load disabled (`scenario_select`(−1), 0x585922: Begin disabled over an empty full
//!   list).
//! - **No saves** (`scenario_buttons_show` 0x57FE68, `scenario_list_finish` 0x583044,
//!   `scenario_dialog_proc` 0x587D1B): an opening with no save disables Load and shows GENRLTXT
//!   686 in an OK box once; its OK disables Load again and redraws whole.
//! - **Load** (E27 M4 T4b2): over a selected save it ends the screen as Begin does
//!   ([`Scenario::loaded`] the save's file, the adapter's to load: E27 M4 T3b's `save::load`, then
//!   play's start), over none nothing.

use alloc::boxed::Box;
use alloc::string::String;

use h3_assets::fnt::Font;
use h3_render::Bugs;
use h3_ui::ui::State;

use super::{BEGIN, BUILT_KEY, Did, Ended, Open, Over, Scenario};
use crate::message::{self, MessageBox, MessageLayout, Style};
use crate::shell::menu::BOX_PLAYER;

/// Delete's question (E27 M4 T4b2; `saved_game_delete` 0x58DFB0: GENRLTXT 688, its `%s` the
/// selected save's file's name, in a box of style 2), laid out at Delete's press.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asking {
    /// The box's look (the layout's `[message]`).
    pub layout: MessageLayout,
    /// The box's medium and small fonts (their metrics).
    pub fonts: [Font; 2],
    /// The question, [`FILE`] where the file's name goes.
    pub text: String,
}

/// Where [`Asking::text`] takes the file's name.
pub const FILE: &str = "{{file}}";

impl Asking {
    /// The question about the save `file`, laid out in the profile `bugs` (framed as the shell's
    /// boxes, [`BOX_PLAYER`]).
    #[must_use]
    pub fn about(&self, file: &str, bugs: Bugs) -> MessageBox {
        let [medium, small] = &self.fonts;
        message::layout(
            &self.layout,
            (medium, small),
            &self.text.replace(FILE, file),
            Style::OkCancel,
            None,
            (BOX_PLAYER, bugs),
        )
    }
}

/// What the box over the screen is (E27 M4 T4b2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum Asked {
    /// A message of Begin's or the client's stub's notice: its OK closes it.
    #[default]
    Told,
    /// Delete's question: its OK deletes the save.
    Delete,
    /// The notice with no saves: its OK disables Load.
    NoSaves,
}

impl Scenario {
    /// The save `file` deleted by Delete's OK since the last time it was taken (E27 M4 T4b2): the
    /// caller deletes it from its own folder (the original's `DeleteFileA` in `games`, 0x58E08B).
    pub fn take_deleted(&mut self) -> Option<String> {
        self.deleted.take()
    }

    /// The save the load mode's Load ended the screen with (E27 M4 T4b2): its file's name, for the
    /// adapter to load; none in a new game's screen or with no save selected.
    #[must_use]
    pub fn loaded(&self) -> Option<&str> {
        self.loading.as_ref()?;
        self.entry().map(|e| e.file.as_str())
    }

    /// The load mode's Load (`scenario_begin` 0x58BCE0): over a selected save the screen ends
    /// as at Begin ([`Scenario::loaded`]); over none nothing (Load is disabled there).
    pub(super) fn load(&self) -> Did {
        if self.entry().is_none() {
            return Did::default();
        }
        Did {
            ended: Some(Ended::Scenario(true)),
            ..Did::default()
        }
    }

    /// The box `message` opened over the screen as `asked`.
    pub(super) fn notice(&mut self, message: MessageBox, asked: Asked) -> Did {
        self.over = Over::Notice(Box::new(Open::new(message)));
        self.asked = asked;
        Did::CHANGED
    }

    /// Delete in the load mode over a selected save (`saved_game_delete` 0x58DFB0): its question
    /// about the save's file; nothing without its text.
    pub(super) fn ask_delete(&mut self) -> Did {
        let message = match (&self.texts.delete, self.entry()) {
            (Some(asking), Some(entry)) => asking.about(&entry.file, self.bugs),
            _ => return Did::default(),
        };
        self.notice(message, Asked::Delete)
    }

    /// An opening of the load mode over no saves: Load disabled (`scenario_buttons_show`
    /// 0x57FE68: an empty shown list in the load mode; recorded, `scn-load-none` notice) and the
    /// notice with no saves (`scenario_list_finish` 0x583044 sets it, the procedure's next call
    /// shows it once; none without its text).
    pub(super) fn no_saves(&mut self) {
        if !self.entries.is_empty() {
            return;
        }
        self.enable_load(false);
        if let Some(message) = self.texts.no_saves.clone() {
            self.notice(message, Asked::NoSaves);
        }
    }

    /// The box over the screen closed by its button `id` (the asked's end): Delete's OK deletes
    /// the save; the no saves notice's OK disables Load and redraws whole (0x587D57 to
    /// 0x587D81); any other close takes the box's pixels back alone.
    pub(super) fn closed(&mut self, id: u16) {
        match core::mem::take(&mut self.asked) {
            Asked::Delete if id == message::YES => self.delete(),
            Asked::NoSaves => {
                self.enable_load(false);
                self.whole();
            }
            Asked::Told | Asked::Delete => {}
        }
    }

    /// Delete's OK (0x58E07D to 0x58E2C9; the module's docs).
    fn delete(&mut self) {
        let Some(at) = self
            .selected
            .and_then(|i| self.listed.get(i).copied())
            .filter(|&at| at < self.entries.len())
        else {
            return;
        };
        self.deleted = Some(self.entries.remove(at).file);
        self.relist();
        self.bar.reset(self.ticks());
        if self.listed.is_empty() {
            self.selected = None;
            self.select();
            self.enable_load(!self.entries.is_empty());
            self.whole();
        } else {
            // The sort gives its first row's save to the game and redraws whole, then the select
            // of that row, which it already is: the partial redraw (as a column's press).
            self.sort_list(BUILT_KEY, self.descending);
            self.give_to_game();
            self.whole();
            self.select();
            self.partial = true;
        }
    }

    /// Load (Begin, 186) enabled or disabled.
    fn enable_load(&mut self, enabled: bool) {
        let state = self.shown.state(BEGIN);
        self.shown.states.insert(
            BEGIN,
            State {
                disabled: !enabled,
                ..state
            },
        );
    }
}

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
