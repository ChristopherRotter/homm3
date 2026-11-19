//! High Scores (E27 M4 T1a; `docs/epics/E27-design.md` §2.8, §34): the dialog of both tables,
//! its four buttons and their keys, the headings and the eleven rows of the tab shown, and the
//! eleven creatures of that tab stepping through their frames.
//!
//! - **The dialog** (`high_scores_dialog_init` 0x4E9880): 800 × 600 at (0, 0), no picture of its
//!   own; four DEF buttons (Exit 0x7802, Reset 0x3EB, Campaign 0x3E9, Standard 0x3EA, in that
//!   order), then the scenarios' eleven creatures (ids 1004 to 1014, 64 × 64 at (649, 26 + 50
//!   *i*)), then the campaigns' (1015 to 1025), each hidden but the tab's (the shell's layout,
//!   `[high_scores]`). The tab shown first is the manager's (+0x8D0, the scenarios', 1; only the
//!   new entry's dialog writes it, §5.6: M5).
//! - **A creature** (`high_score_creature` 0x4E9830, inlined at its three callers): the first pair
//!   of `high_score_creatures` 0x67F1FC (118 pairs of a score and a creature, the layout's
//!   `creatures`) whose score is not below the row's (a campaign's divided by 5 first, rounded
//!   toward 0); its picture the adventure map's last template of that creature (class 54) as the
//!   game's map holds them at the start, `objects.txt`'s (`class_template_last` 0x505EA0 over
//!   the lists `game_init` fills, 0x505DA0): the adapter's [`Pictures`]. A score past the last
//!   pair (above 32 767) reads past the table in the original: the client takes the last pair
//!   (*provisional*, CLT-Q157).
//! - **The draw** (`high_scores_draw` 0x4E9E50): the tab's picture copied whole onto the screen
//!   (`hiscore2.pcx` the campaigns', `hiscore.pcx` the scenarios'), the headings and the rows in
//!   `medfont.fnt` over it in their boxes ([`ScoresLook`]), colour 1 (8 for the row the new entry
//!   made, `high_score_newest` 0x67F1F4: M5's), alignment 5; then the dialog's items over them as
//!   the dialog shows: the buttons (the tab's selected, its frame 3) and the tab's creatures.
//! - **The procedure** (`high_scores_proc` 0x4EA1D0): Escape and Enter end it at their press
//!   (Exit's id 0x7802), as does Exit's click; Standard's click (S) shows the scenarios, Campaign's
//!   (C) the campaigns (the creatures of both hidden, then the tab's shown; their frames kept);
//!   Reset's (no key) asks its question ([`Scores::question`]). Any other message, once more than 200 ms of the clock have passed since the last step (the
//!   dialog's +0x104, set as it opens), steps the tab's eleven creatures a frame (their counters
//!   +0xA4, from 0 as it opens, back to 0 at their DEF's count of group 0), the table drawn again.
//!   No hint line and no right press's popup (§6.3: recorded). A key down and a command (the
//!   message 0x200) run no step. While the mouse holds a button its own loop takes every message
//!   (`def_button_message` 0x45641C), so none steps; the release that ends the hold runs the
//!   step at its own time (*recorded*, §34: `scores-tabs`' `exit-left` to `campaigns-anim`; the
//!   message that runs it unread), after its click.
//! - **Reset** (E27 M4 T1b; the procedure's 0x3EB, design §35): GENRLTXT 667 in a box of style 2;
//!   its modal loop takes every message, so no step runs while it shows. Cancel changes nothing;
//!   OK (0x7805) resets both tables to the defaults (`high_scores_reset` 0x4E8FB0), writes the
//!   scores file (the original's `DATA\HiScore.dat`; the client's own, [`Scores::take_save`]) and
//!   sets every creature's DEF again by its row's score with its frame 0, the counters kept (so
//!   the next step shows each counter's next frame); it redraws nothing itself.
//! - **The scores file** (E27 M4 T1b, *provisional*: its place and format the client's; never
//!   the installation's `HiScore.dat`): [`Tables::read`] at the start over the defaults,
//!   [`Tables::write`] at Reset's OK, a TOML file of the [`FILE_VERSION`].

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use h3_render::Bugs;
use h3_ui::input::Input;
use h3_ui::ui::{Dialog, Draw, Item, Label, Shown, State};
use serde::Deserialize;

use super::Ended;
use super::keys::{Command, Context};
use super::menu::{Did, Hovers, Stale, outside};
use crate::message::{self, MessageBox, Open, Outcome};

/// The rows of a table.
pub const ROWS: usize = 11;
/// Exit's button (and the procedure's end, `high_scores_proc`'s 0x7802).
pub const EXIT: u16 = 0x7802;
/// Reset's button.
pub const RESET: u16 = 0x3EB;
/// Campaign's button: the campaigns' tab.
pub const CAMPAIGN: u16 = 0x3E9;
/// Standard's button: the scenarios' tab.
pub const STANDARD: u16 = 0x3EA;
/// The scores file's version (E27 M4 T1b, *provisional*): [`Tables::read`] refuses another.
pub const FILE_VERSION: i64 = 1;

/// A tab: the dialog's +0xFC (0 the campaigns, 1 the scenarios).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    /// The campaigns' table (Campaign, C).
    Campaigns = 0,
    /// The single scenarios' table (Standard, S).
    Scenarios = 1,
}

impl Tab {
    /// Both, by their number.
    pub const ALL: [Self; 2] = [Self::Campaigns, Self::Scenarios];

    /// Its button.
    #[must_use]
    pub const fn button(self) -> u16 {
        match self {
            Self::Campaigns => CAMPAIGN,
            Self::Scenarios => STANDARD,
        }
    }
}

/// A tab's look (the shell's layout `[high_scores.campaigns]`, `[high_scores.scenarios]`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TabLook {
    /// The picture copied whole onto the screen first: `hiscore2.pcx`, `hiscore.pcx`.
    pub picture: String,
    /// The columns' boxes, left edge and width, each heading's and each row's: the rank, the
    /// player, the land or campaign, then the scenarios' days and score or the campaigns' score.
    pub columns: Vec<[i32; 2]>,
    /// Its first creature's id (1015 the campaigns', 1004 the scenarios'), the others following.
    pub creatures: u16,
}

/// High Scores' look (the shell's layout `[high_scores]`; `high_scores_draw` 0x4E9E50).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScoresLook {
    /// The headings' and the rows' font: `medfont.fnt` (`medium_font` 0x698A0C).
    pub font: String,
    /// The palette its colours come from.
    pub font_palette: String,
    /// The text drawer's colour: 1.
    pub colour: u32,
    /// The newest row's: 8.
    pub newest: u32,
    /// The text drawer's alignment: 5 (centred both ways).
    pub align: u32,
    /// The headings' top edge and every box's height: 11, 26.
    pub headings: [i32; 2],
    /// The first row's top edge and the step to the next: 53, 50.
    pub rows: [i32; 2],
    /// The creatures' step: more than so many milliseconds since the last (200).
    pub step: u32,
    /// The buttons' DEFs' canvas (42 × 200): a press's redraw shows only the button's box (its
    /// size, 41 × 199), so in the original's profile its last column and row keep the frame last
    /// shown whole (as CLT-Q118's menus, decision 22: the client's profile draws it whole).
    pub drawn: [i32; 2],
    /// `high_score_creatures` 0x67F1FC: each pair a score and the creature it earns, the scores
    /// rising.
    pub creatures: Vec<[i32; 2]>,
    /// The campaigns' tab.
    pub campaigns: TabLook,
    /// The scenarios' tab.
    pub scenarios: TabLook,
    /// The dialog: its buttons, then both tabs' creatures (their DEFs the screen's).
    pub dialog: Dialog,
}

impl ScoresLook {
    /// The look of `tab`.
    #[must_use]
    pub const fn tab(&self, tab: Tab) -> &TabLook {
        match tab {
            Tab::Campaigns => &self.campaigns,
            Tab::Scenarios => &self.scenarios,
        }
    }

    /// The creature a row's `score` earns on `tab` (`high_score_creature` 0x4E9830): a campaign's
    /// divided by 5 first (rounded toward 0), then the first pair whose score is not below it;
    /// past the last pair the last's (*provisional*, CLT-Q157: the original reads past the
    /// table); none with no pairs.
    #[must_use]
    pub fn creature(&self, score: i32, tab: Tab) -> Option<i32> {
        let score = match tab {
            Tab::Campaigns => score / 5,
            Tab::Scenarios => score,
        };
        self.creatures
            .iter()
            .find(|&&[s, _]| score <= s)
            .or_else(|| self.creatures.last())
            .map(|&[_, creature]| creature)
    }
}

/// A row of a table (the manager's record of 100 bytes): the player (+0, 41 bytes), the land or
/// campaign (+0x29, 41), the score (+0x54) and the days (+0x58).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// The player.
    pub player: String,
    /// The land (a scenario's name) or the campaign.
    pub land: String,
    /// The days.
    pub days: i32,
    /// The score.
    pub score: i32,
}

/// Both tables (the manager's +0x38 the campaigns', +0x484 the scenarios').
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tables {
    /// The campaigns' rows.
    pub campaigns: Vec<Record>,
    /// The scenarios' rows.
    pub scenarios: Vec<Record>,
}

impl Tables {
    /// The defaults from `CampHigh.txt`'s records (`high_score_defaults_load` 0x5B91D0 and
    /// `high_scores_reset` 0x4E8FB0): records 1 to 11 the scenarios', 13 to 23 the campaigns',
    /// each its fields 1 to 4 (the player, the land or campaign, the days, the score; the numbers
    /// by `atoi`); none when a record lacks a field.
    #[must_use]
    pub fn defaults(records: &[Vec<String>]) -> Option<Self> {
        let rows = |first: usize| {
            (first..first + ROWS)
                .map(|r| {
                    let record = records.get(r)?;
                    let field = |f: usize| record.get(f);
                    Some(Record {
                        player: field(1)?.clone(),
                        land: field(2)?.clone(),
                        days: atoi(field(3)?),
                        score: atoi(field(4)?),
                    })
                })
                .collect::<Option<Vec<_>>>()
        };
        Some(Self {
            scenarios: rows(1)?,
            campaigns: rows(13)?,
        })
    }

    /// The rows of `tab`.
    #[must_use]
    pub fn of(&self, tab: Tab) -> &[Record] {
        match tab {
            Tab::Campaigns => &self.campaigns,
            Tab::Scenarios => &self.scenarios,
        }
    }

    /// The tables a scores file's `text` holds (E27 M4 T1b, *provisional*; [`Tables::write`]'s
    /// format): its `version` the [`FILE_VERSION`], then `scenarios` and `campaigns`, each
    /// [`ROWS`] rows of a `player`, a `land` (a campaign's name in the campaigns'), the `days` and
    /// the `score`.
    ///
    /// # Errors
    /// Not TOML, another version or none, a table or a row not as above.
    pub fn read(text: &str) -> Result<Self, String> {
        let file: toml::Table = toml::from_str(text).map_err(|e| e.to_string())?;
        match file.get("version").map(toml::Value::as_integer) {
            Some(Some(FILE_VERSION)) => {}
            Some(Some(v)) => {
                return Err(format!(
                    "version {v}: this client reads version {FILE_VERSION}"
                ));
            }
            _ => return Err(String::from("no `version` number")),
        }
        let file: File = toml::from_str(text).map_err(|e| e.to_string())?;
        let rows = |name: &str, rows: Vec<Row>| {
            if rows.len() == ROWS {
                Ok(rows.into_iter().map(Record::from).collect())
            } else {
                Err(format!("`{name}`: {} rows, not {ROWS}", rows.len()))
            }
        };
        Ok(Self {
            scenarios: rows("scenarios", file.scenarios)?,
            campaigns: rows("campaigns", file.campaigns)?,
        })
    }

    /// The scores file's text (E27 M4 T1b, *provisional*): the [`FILE_VERSION`], then each
    /// table's rows ([`Tables::read`]).
    #[must_use]
    pub fn write(&self) -> String {
        let mut text = format!(
            "# High Scores' tables (h3-view shell; E27 M4 T1b): read at the start over\n\
             # CampHigh.txt's defaults, written as Reset's OK resets them to those.\n\
             version = {FILE_VERSION}\n"
        );
        for (name, rows) in [
            ("scenarios", &self.scenarios),
            ("campaigns", &self.campaigns),
        ] {
            for r in rows {
                text.push_str(&format!(
                    "\n[[{name}]]\nplayer = {}\nland = {}\ndays = {}\nscore = {}\n",
                    quoted(&r.player),
                    quoted(&r.land),
                    r.days,
                    r.score
                ));
            }
        }
        text
    }
}

/// A scores file as [`Tables::read`] takes it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    /// Checked before (a number of any version).
    #[serde(rename = "version")]
    _version: i64,
    scenarios: Vec<Row>,
    campaigns: Vec<Row>,
}

/// A row of a scores file.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    player: String,
    land: String,
    days: i32,
    score: i32,
}

impl From<Row> for Record {
    fn from(r: Row) -> Self {
        Self {
            player: r.player,
            land: r.land,
            days: r.days,
            score: r.score,
        }
    }
}

/// `text` as a TOML basic string: `"` and `\` escaped, each control character as `\uXXXX`.
fn quoted(text: &str) -> String {
    let mut quoted = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            c if c.is_control() => quoted.push_str(&format!("\\u{:04X}", u32::from(c))),
            c => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted
}

/// C's `atoi` (0x617F09): leading blanks, a sign, the digits up to the first other character
/// (none: 0); wrapping as the 32-bit sum does.
#[must_use]
pub fn atoi(text: &str) -> i32 {
    let text = text.trim_start_matches([' ', '\t', '\n', '\r', '\x0B', '\x0C']);
    let (negative, digits) = match text.as_bytes().first() {
        Some(b'-') => (true, &text[1..]),
        Some(b'+') => (false, &text[1..]),
        _ => (false, text),
    };
    let value = digits
        .bytes()
        .take_while(u8::is_ascii_digit)
        .fold(0i32, |n, d| {
            n.wrapping_mul(10).wrapping_add(i32::from(d - b'0'))
        });
    if negative {
        value.wrapping_neg()
    } else {
        value
    }
}

/// A creature's picture: the adventure map's DEF and its group 0's frame count (the counter's
/// wrap; 0 for a DEF without one, which never steps).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picture {
    /// The DEF.
    pub def: String,
    /// Its group 0's frames.
    pub frames: u32,
}

/// The creatures' pictures by creature (the adapter's, from `objects.txt`'s templates): a
/// creature without one shows none (the original's the record before the list, *provisional*).
pub type Pictures = BTreeMap<i32, Picture>;

/// High Scores' texts: the headings (GENRLTXT 434 to 437, 673 and 76, the user's tables').
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Texts {
    /// Rank (434).
    pub rank: String,
    /// Player (435).
    pub player: String,
    /// Land (436).
    pub land: String,
    /// Days (437).
    pub days: String,
    /// Campaign (673).
    pub campaign: String,
    /// Score (76).
    pub score: String,
}

impl Texts {
    /// The headings of `tab`, a column each.
    fn headings(&self, tab: Tab) -> Vec<&str> {
        match tab {
            Tab::Campaigns => alloc::vec![&self.rank, &self.player, &self.campaign, &self.score],
            Tab::Scenarios => {
                alloc::vec![
                    &self.rank,
                    &self.player,
                    &self.land,
                    &self.days,
                    &self.score
                ]
            }
        }
        .into_iter()
        .map(String::as_str)
        .collect()
    }
}

/// The dialog on the screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scores {
    look: ScoresLook,
    texts: Texts,
    tables: Tables,
    /// `CampHigh.txt`'s, which Reset's OK sets again.
    defaults: Tables,
    pictures: Pictures,
    tab: Tab,
    /// The row the new entry made on each tab, by the tab's number (`high_score_newest`; M5).
    newest: [Option<usize>; 2],
    /// Each creature's frame counter (+0xA4), by the tab's number and the row.
    counters: [[u32; ROWS]; 2],
    /// The clock at the last step (+0x104): the opening's first reading.
    last: Option<u32>,
    /// The clock's last reading.
    now: u32,
    shown: Shown,
    /// Each button's state when the dialog was last shown whole (the opening, a tab's switch, a
    /// step): outside its box it shows so after a press's redraw (the original's profile).
    whole: BTreeMap<u16, State>,
    input: Input,
    /// Reset's question (GENRLTXT 667, style 2), laid out.
    question: MessageBox,
    /// The question when open.
    open: Option<Box<Open>>,
    /// The tables Reset's OK set, not yet taken for the scores file ([`Scores::take_save`]).
    unsaved: bool,
    /// The menus' last items as the main menu left them (the screen writes none).
    hovers: Hovers,
    bugs: Bugs,
}

impl Scores {
    /// The dialog as it opens (`high_scores_dialog_init`): the scenarios' tab, every counter 0,
    /// the tables the `defaults` (`CampHigh.txt`'s; a scores file's over them,
    /// [`Scores::with_tables`]), the creatures' DEFs by their scores and `pictures`, Reset's
    /// `question` laid out, in the profile `bugs`.
    #[must_use]
    pub fn new(
        look: ScoresLook,
        texts: Texts,
        defaults: Tables,
        pictures: Pictures,
        question: MessageBox,
        bugs: Bugs,
    ) -> Self {
        let mut scores = Self {
            look,
            texts,
            tables: defaults.clone(),
            defaults,
            pictures,
            tab: Tab::Scenarios,
            newest: [None; 2],
            counters: [[0; ROWS]; 2],
            last: None,
            now: 0,
            shown: Shown::default(),
            whole: BTreeMap::new(),
            input: Input::new(bugs),
            question,
            open: None,
            unsaved: false,
            hovers: Hovers::default(),
            bugs,
        };
        scores.set_creatures();
        scores.show_tab();
        scores.shown_whole();
        scores
    }

    /// The dialog opened anew (its tab the manager's first, the scenarios'; the counters 0, the
    /// clock's first reading its start; its tables as they are), the menus' last items `hovers`
    /// kept for the main menu.
    #[must_use]
    pub fn opened(&self, hovers: Hovers) -> Self {
        let mut scores = Self::new(
            self.look.clone(),
            self.texts.clone(),
            self.defaults.clone(),
            self.pictures.clone(),
            self.question.clone(),
            self.bugs,
        )
        .with_tables(self.tables.clone());
        scores.newest = self.newest;
        scores.hovers = hovers;
        scores
    }

    /// The dialog with the tables `tables` (a scores file's, read over the defaults at the start:
    /// `high_score_manager_load` 0x4E90A0), the creatures' DEFs by their scores.
    #[must_use]
    pub fn with_tables(mut self, tables: Tables) -> Self {
        self.tables = tables;
        self.set_creatures();
        self
    }

    /// The tables `left` ended with kept, for the next opening (the manager's, which outlives the
    /// dialog).
    pub fn keep(&mut self, left: &Self) {
        self.tables.clone_from(&left.tables);
    }

    /// The scores file's text ([`Tables::write`]) when Reset's OK set the tables since the last
    /// time it was taken: the caller writes it (the original's `HiScore.dat` at 0x4EA3E3).
    pub fn take_save(&mut self) -> Option<String> {
        core::mem::take(&mut self.unsaved).then(|| self.tables.write())
    }

    /// The dialog with the row `row` of `tab` the newest (drawn in colour 8; the new entry's,
    /// M5).
    #[must_use]
    pub const fn with_newest(mut self, tab: Tab, row: usize) -> Self {
        self.newest[tab as usize] = Some(row);
        self
    }

    /// The tab shown.
    #[must_use]
    pub const fn tab(&self) -> Tab {
        self.tab
    }

    /// The tables.
    #[must_use]
    pub const fn tables(&self) -> &Tables {
        &self.tables
    }

    /// The creatures' frames on `tab`, a row each.
    #[must_use]
    pub const fn frames(&self, tab: Tab) -> [u32; ROWS] {
        self.counters[tab as usize]
    }

    /// The menus' last items: the main menu's as it left them.
    #[must_use]
    pub const fn hovers(&self) -> Hovers {
        self.hovers
    }

    /// Where it takes its keys: Reset's question's, else its own.
    #[must_use]
    pub const fn context(&self) -> Context {
        if self.open.is_some() {
            Context::Question
        } else {
            Context::HighScores
        }
    }

    /// Reset's question, while it shows.
    #[must_use]
    pub fn question(&self) -> Option<&Open> {
        self.open.as_deref()
    }

    /// The picture copied whole onto the screen first: the tab's.
    #[must_use]
    pub fn background(&self) -> &str {
        &self.look.tab(self.tab).picture
    }

    /// The draws over the picture: the headings and the rows (`high_scores_draw`), then the
    /// dialog's items (its buttons, the tab's creatures).
    #[must_use]
    pub fn draws(&self, players: &str) -> Vec<Draw> {
        let mut draws = self.text_draws();
        draws.extend(self.look.dialog.draws(&self.shown, players));
        draws
    }

    /// In the original's profile, each button last shown whole in another frame than it is drawn
    /// in now: its canvas outside its box keeps that frame (a press's redraw shows the box alone,
    /// `button_press` 0x456860; as CLT-Q118's menus); none in the client's.
    #[must_use]
    pub fn stale(&self, players: &str) -> Vec<Stale> {
        if self.bugs != Bugs::Original {
            return Vec::new();
        }
        let dialog = &self.look.dialog;
        let [px, py] = dialog.place;
        let [dw, dh] = self.look.drawn;
        dialog
            .items
            .iter()
            .filter_map(|item| {
                let Item::Button(b) = item else {
                    return None;
                };
                let state = self.whole.get(&b.id).copied().unwrap_or_default();
                if b.frame(state) == b.frame(self.shown.state(b.id)) {
                    return None;
                }
                let [x, y] = [px + b.at[0], py + b.at[1]];
                let rects = outside([x, y, dw, dh], &[[x, y, b.size[0], b.size[1]]]);
                let mut shown = Shown::default();
                shown.states.insert(b.id, state);
                let alone = Dialog {
                    items: alloc::vec![item.clone()],
                    ..dialog.clone()
                };
                Some(Stale {
                    rects,
                    draws: alone.draws(&shown, players),
                })
            })
            .collect()
    }

    /// Reset's question over the dialog, when it shows.
    #[must_use]
    pub fn box_draws(&self, players: &str) -> Vec<Draw> {
        self.open
            .as_ref()
            .map(|o| o.message.dialog.draws(&o.message.shown, players))
            .unwrap_or_default()
    }

    /// The headings and the rows of the tab shown.
    fn text_draws(&self) -> Vec<Draw> {
        let look = &self.look;
        let tab = self.tab;
        let columns = &look.tab(tab).columns;
        let [heading_y, height] = look.headings;
        let label = |[x, width]: [i32; 2], y: i32, colour: u32, text: &str| Draw::Label {
            item: Label {
                id: 0,
                at: [x, y],
                size: [width, height],
                font: look.font.clone(),
                font_palette: look.font_palette.clone(),
                colour,
                align: look.align,
                hint: None,
                repeat: None,
            },
            x,
            y,
            text: String::from(text),
            disabled: false,
        };
        let mut draws: Vec<Draw> = columns
            .iter()
            .zip(self.texts.headings(tab))
            .map(|(&column, text)| label(column, heading_y, look.colour, text))
            .collect();
        let [first, step] = look.rows;
        let mut y = first;
        for (i, record) in self.tables.of(tab).iter().enumerate().take(ROWS) {
            let colour = if self.newest[tab as usize] == Some(i) {
                look.newest
            } else {
                look.colour
            };
            let mut fields = alloc::vec![
                (i + 1).to_string(),
                record.player.clone(),
                record.land.clone(),
            ];
            match tab {
                Tab::Campaigns => fields.push(record.score.to_string()),
                Tab::Scenarios => {
                    fields.extend([record.days.to_string(), record.score.to_string()]);
                }
            }
            draws.extend(
                columns
                    .iter()
                    .zip(&fields)
                    .map(|(&column, text)| label(column, y, colour, text)),
            );
            y = y.saturating_add(step);
        }
        draws
    }

    /// The id of row `row`'s creature on `tab`.
    fn creature_id(&self, tab: Tab, row: usize) -> u16 {
        self.look
            .tab(tab)
            .creatures
            .saturating_add(u16::try_from(row).unwrap_or(u16::MAX))
    }

    /// Every creature's DEF by its row's score and its frame 0 (the constructor's and Reset's
    /// `dialog_def_set_def`, `dialog_def_set_frame`: the counters are left as they are).
    fn set_creatures(&mut self) {
        for tab in Tab::ALL {
            for row in 0..ROWS {
                let id = self.creature_id(tab, row);
                let picture = self
                    .tables
                    .of(tab)
                    .get(row)
                    .and_then(|r| self.look.creature(r.score, tab))
                    .and_then(|c| self.pictures.get(&c));
                match picture {
                    Some(p) => self.shown.defs.insert(id, p.def.clone()),
                    None => self.shown.defs.remove(&id),
                };
                self.shown.frames.insert(id, 0);
            }
        }
    }

    /// The dialog shown whole: each button's state as it is.
    fn shown_whole(&mut self) {
        for item in &self.look.dialog.items {
            if let Item::Button(b) = item {
                self.whole.insert(b.id, self.shown.state(b.id));
            }
        }
    }

    /// The tab's button selected and its creatures shown, the other's hidden and not selected
    /// (`high_scores_draw`'s state 0x10, the procedure's messages 6).
    fn show_tab(&mut self) {
        for tab in Tab::ALL {
            let on = tab == self.tab;
            let button = self.shown.states.entry(tab.button()).or_default();
            button.selected = on;
            for row in 0..ROWS {
                let id = self.creature_id(tab, row);
                self.shown.states.insert(
                    id,
                    State {
                        hidden: !on,
                        ..State::default()
                    },
                );
            }
        }
    }

    /// The shell's clock is `now`: the first reading starts the creatures' timer; once more than
    /// [`ScoresLook::step`] ms have passed since the last step, the tab's creatures step a frame
    /// (each back to 0 at its DEF's count) and the dialog shows whole; none while the mouse holds
    /// a button (its own loop takes the messages, `def_button_message` 0x45641C: recorded,
    /// `scores-tabs`' `reset-press`) or Reset's question shows (its modal loop's). Whether what
    /// shows changed.
    pub fn clock(&mut self, now: u32) -> bool {
        self.now = now;
        self.step_due()
    }

    /// The procedure's step at the clock's last reading, when one is due ([`Scores::clock`]).
    fn step_due(&mut self) -> bool {
        let now = self.now;
        let Some(last) = self.last else {
            self.last = Some(now);
            return false;
        };
        if self.input.holds_button()
            || self.open.is_some()
            || now.wrapping_sub(last) as i32 <= self.look.step as i32
        {
            return false;
        }
        self.last = Some(now);
        let tab = self.tab;
        for row in 0..ROWS {
            let frames = self
                .tables
                .of(tab)
                .get(row)
                .and_then(|r| self.look.creature(r.score, tab))
                .and_then(|c| self.pictures.get(&c))
                .map_or(0, |p| p.frames);
            let counter = &mut self.counters[tab as usize][row];
            *counter += 1;
            if *counter >= frames {
                *counter = 0;
            }
            let frame = *counter;
            let id = self.creature_id(tab, row);
            self.shown.frames.insert(id, frame);
        }
        self.shown_whole();
        true
    }

    /// The mouse moved to `point`: the button it holds follows it.
    pub fn moved(&mut self, point: [i32; 2]) -> Did {
        match &mut self.open {
            Some(open) => Did::of_box(open.moved(point)),
            None => Did::held(self.input.moved(&self.look.dialog, &mut self.shown, point)),
        }
    }

    /// The left button pressed at `point`: a button there goes down.
    pub fn press(&mut self, point: [i32; 2]) -> Did {
        match &mut self.open {
            Some(open) => Did::box_press(open.press(point)),
            None => Did::held(self.input.press(&self.look.dialog, &mut self.shown, point)),
        }
    }

    /// The left button released: the button held clicks over it.
    pub fn release(&mut self) -> Did {
        match &mut self.open {
            Some(open) => {
                let outcome = open.release();
                self.answered(outcome)
            }
            None => {
                let held = self.input.holds_button();
                let did = self.input.release(&mut self.shown);
                let mut did = self.clicked(did.changed, did.clicked);
                did.changed |= held && self.step_due();
                did
            }
        }
    }

    /// The right button pressed: no popup (recorded, §6.3).
    #[must_use]
    pub fn right(&self) -> Did {
        Did::default()
    }

    /// The right button released: a button the mouse holds released as by the left button.
    pub fn right_release(&mut self) -> Did {
        match &mut self.open {
            Some(open) if open.holds_mouse() => {
                let outcome = open.release();
                self.answered(outcome)
            }
            Some(_) => Did::default(),
            None => {
                let held = self.input.holds_button();
                let did = self.input.right_release(&mut self.shown);
                let mut did = self.clicked(did.changed, did.clicked);
                did.changed |= held && self.step_due();
                did
            }
        }
    }

    /// A key down: its command `command` (the key table's in [`Scores::context`]), its name `by`
    /// (whose release clicks). Exit's keys end the dialog at their press; a tab's key presses its
    /// button.
    pub fn key(&mut self, command: Option<Command>, by: &str) -> Did {
        match (&mut self.open, command) {
            (Some(open), Some(Command::Message(key))) => Did::box_press(open.key(key, by)),
            (Some(_), _) => Did::default(),
            (None, Some(Command::Dialog(EXIT))) => Did {
                ended: Some(Ended::Done),
                ..Did::default()
            },
            (None, Some(Command::Dialog(id @ (CAMPAIGN | STANDARD)))) => {
                Did::held(self.input.key(&mut self.shown, id, by))
            }
            (None, _) => Did::default(),
        }
    }

    /// The key `by` released: the button it held clicks.
    pub fn key_up(&mut self, by: &str) -> Did {
        match &mut self.open {
            Some(open) => {
                let outcome = open.key_up(by);
                self.answered(outcome)
            }
            None => {
                let did = self.input.key_up(&mut self.shown, by);
                self.clicked(did.changed, did.clicked)
            }
        }
    }

    /// A hold ended (`changed`: a button went up) with the click `clicked`, if any: Exit's ends
    /// the dialog, a tab's shows its tab, Reset's asks its question.
    fn clicked(&mut self, changed: bool, clicked: Option<u16>) -> Did {
        let mut did = Did {
            changed,
            ..Did::default()
        };
        match clicked {
            Some(EXIT) => did.ended = Some(Ended::Done),
            Some(STANDARD) => self.switch(Tab::Scenarios, &mut did),
            Some(CAMPAIGN) => self.switch(Tab::Campaigns, &mut did),
            Some(RESET) => {
                self.open = Some(Box::new(Open::new(self.question.clone())));
                did.changed = true;
            }
            _ => {}
        }
        did
    }

    /// The procedure's tab switch: `tab` shown, the table drawn again.
    fn switch(&mut self, tab: Tab, did: &mut Did) {
        self.tab = tab;
        self.show_tab();
        self.shown_whole();
        did.changed = true;
    }

    /// Reset's question answered: closed by either button, its OK (0x7805) resetting the tables
    /// (`high_scores_reset` 0x4E8FB0), the scores file due and every creature set again; then the
    /// step due at the close's time runs, as after a button's hold (*recorded*, `scores-reset`:
    /// Cancel's release at 400 stepped, so Reset's OK at 600 did not and the tick at 700 did).
    fn answered(&mut self, outcome: Outcome) -> Did {
        match outcome {
            Outcome::Nothing => Did::default(),
            Outcome::Changed => Did::CHANGED,
            Outcome::Closed(id) => {
                self.open = None;
                if id == message::YES {
                    self.tables.clone_from(&self.defaults);
                    self.unsaved = true;
                    self.set_creatures();
                }
                self.step_due();
                Did::CHANGED
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests;
