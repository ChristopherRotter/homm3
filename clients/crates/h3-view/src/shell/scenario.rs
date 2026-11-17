//! The scenario screen in its new-game mode (E27 M3 T1a; `docs/epics/E27-design.md` §3.1, §3.2,
//! §3.5, §3.8, §18): `scenario_dialog_create` 0x579960 with its argument 0, a single scenario,
//! the list hidden (a new game opens with no left-hand panel, the background showing there).
//!
//! - **The frame** (§3.1): `gamselb1.pcx` when `random_between(1, 100)` is below 51, else
//!   `gamselb0.pcx` (the client's own draw, a look: [`Scenario::with_background`]; the replay
//!   takes the original's draws from its recipe), `GSelPop1.pcx` over it, then the items in the
//!   order the constructor adds them (the shell's layout, `[scenario]`).
//! - **The information panel** (§3.5, `scenario_info_draw` 0x5840F0 and `scenario_team_flags`
//!   0x585300) of the entry selected: the name (GENRLTXT 509 when empty), the description, the
//!   victory's and the loss's texts and marks, the map's size mark, the allies' and the enemies'
//!   flags, the map's difficulty and the rating of the chosen difficulty, whose button shows
//!   selected. The selection is the kept map's entry (`scenario_last_map` 0x683238,
//!   `Arrogance.h3m` at the start), else the first.
//! - **The buttons:** Show Available Scenarios (128, S), Random Map (130, R) and Show Advanced
//!   Options (129, A) are caption buttons (`h3_ui::ui::Caption`); Begin (186, Enter) and Back
//!   (188, Escape) DEF buttons; a press holds a button in its own loop (`h3_ui::input`), its
//!   sample with it, the command at the release over it. Back ends the screen (0x7801: New
//!   Game's menu again). The hover changes nothing (no status line).
//! - **A right press** (`scenario_dialog_right_click` 0x5822D0, §3.8): the item's `Help.txt`
//!   right-click text (record id − 83) in a popup of style 4 while the right button is held, the
//!   cursor hidden; none for an item without one, nor while the mouse holds a button.
//!
//! - **The list** (E27 M3 T1b1; §3.3, §3.4, §19): Show Available Scenarios (128) shows its panel,
//!   title, sizes' label, rows, filters, columns and scroll bar (`scenario_list_toggle` 0x580A70,
//!   the bar's ticks the count less the rows plus one, its value the top), a second press hides
//!   them (`scenario_list_hide` 0x5819A0). Its first showing builds it (`scenario_list_finish`
//!   0x582E90): sorted by the version (key 2), the kept map's row selected and scrolled to the
//!   top. The redraw draws each row ([`ListLook`]): the players "%d/%d", the size's letter, the
//!   version's mark, the name, the victory's and the loss's marks, the selection in colour 5.
//!   The columns sort (`scenario_list_sort` 0x585050; keys 3, 1, 2, 0, 4, 5) and flip on a second
//!   press, the filters keep the maps of a size (`scenario_list_filter` 0x585D90), each with the
//!   top and the selection 0; Up and Down move the selection by a row, the top following, Page
//!   Up and Page Down the top by a page less one, the selection keeping its row
//!   (`scenario_dialog_proc` 0x587D00); the bar moves the top (`scenario_list_scrolled` 0x57C990,
//!   `h3_ui::scrollbar`); a row's release selects it (`text_item_message` 0x5BC440: at a release
//!   anywhere in the original's profile, over the row in the client's), a second within 400 ms
//!   begins (`scenario_row_click` 0x5872E4: in the original's profile any row's, the opening's
//!   time too; the client's needs the same row, CLT-Q91 (c)). The order's ties: a stable sort,
//!   the name then the file's name (CLT-Q90); the size's first press after the build descending
//!   in the original's profile (its recorded key 1), ascending in the client's (CLT-Q91 (c)).
//!
//! *Not built yet:* GENRLTXT 503 while the list is read, the right press's popups on the list,
//! its labels and texts, and the flags' strip's teams dialog (387: M3 T1b2), the difficulty's
//! presses and Begin (T2: a double press begins nothing yet), the advanced options (129: T3), the
//! random map (130: T5), a description too long for its box (its scroll bar: T2), the hot seat's
//! and the load's modes (the client's stub until T4 and M4).

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;

use h3_render::Bugs;
use h3_ui::input::Input;
use h3_ui::scrollbar::{self, Scrollbar};
use h3_ui::ui::{Def, Dialog, Draw, Label, Shown, State};
use serde::Deserialize;

use super::Ended;
use super::keys::{Command, Context, Scroll};
use super::menu::{Arrow, Did, Hovers};
use crate::message::MessageBox;

/// Show Available Scenarios (128: S).
pub const LIST: u16 = 128;
/// Show Advanced Options (129: A).
pub const ADVANCED: u16 = 129;
/// Random Map (130: R).
pub const RANDOM: u16 = 130;
/// Begin (186: Enter).
pub const BEGIN: u16 = 186;
/// Back (188: Escape).
pub const BACK: u16 = 188;
/// The map's size mark (189, `scnrmpsz.def`).
pub const SIZE: u16 = 189;
/// The difficulty's buttons, easy to impossible (107 to 111).
pub const DIFFICULTIES: [u16; 5] = [107, 108, 109, 110, 111];
/// The allies' first flag (112 to 119, `itgflags.def`).
pub const ALLIES: u16 = 112;
/// The enemies' first flag (120 to 127).
pub const ENEMIES: u16 = 120;
/// The client's own ids start here: the original's unnamed items (its 100s, the description's
/// lines) and what the redraw draws itself, each an item of the layout (*provisional*). None
/// takes a right press or has a hint.
pub const OWN: u16 = 1000;
/// The name (`bigfont.fnt`, colour 8, at (422, 45, 324, 30)).
pub const NAME: u16 = 1001;
/// The description (a scrolling text, `smalfont.fnt`, colour 4, at (422, 155, 319, 115)).
pub const DESCRIPTION: u16 = 1002;
/// The victory's text (at (456, 305, 288, 32), colour 4, alignment 4).
pub const VICTORY: u16 = 1003;
/// The loss's text (at (456, 364, 288, 32)).
pub const LOSS: u16 = 1004;
/// The victory's mark (`scnrvict.def` at (420, 308)).
pub const VICTORY_MARK: u16 = 1005;
/// The loss's mark (`scnrloss.def` at (420, 365)).
pub const LOSS_MARK: u16 = 1006;
/// The map's difficulty (at (414, 454, 89, 48), colour 4, centred).
pub const MAP_DIFFICULTY: u16 = 1007;
/// The rating (at (666, 454, 83, 48)).
pub const RATING: u16 = 1008;
/// The list's sizes label (GENRLTXT 511, drawn by the redraw with the list: at (25, 52, 132, 32)).
pub const SIZES: u16 = 1014;
/// The list's panel (`SCSelBck.pcx`).
pub const PANEL: u16 = 101;
/// The list's title (`ArrayTxt.txt` 229 for a new game).
pub const TITLE: u16 = 361;
/// The list's first row (142 to 159: text items with no text that take the press).
pub const ROW: u16 = 142;
/// The size filters S, M, L, XL and All (137 to 141) with the sizes they keep (0 every one).
pub const FILTERS: [(u16, u32); 5] = [(137, 36), (138, 72), (139, 108), (140, 144), (141, 0)];
/// The columns players, size, version, name, victory and loss (190 to 195) with their sort keys
/// (`scenario_sort_click` 0x5870FA).
pub const COLUMNS: [(u16, u8); 6] = [(190, 3), (191, 1), (192, 2), (193, 0), (194, 4), (195, 5)];
/// The right-hand panel's picture (`GSelPop1.pcx`, drawn into the screen at once).
pub const PANEL_BACK: u16 = 1020;
/// The enemies' label ("%s:" of GENRLTXT 392).
pub const ENEMIES_LABEL: u16 = 386;
/// The flags' strip `scenario_team_flags` 0x585300 restores and shows (456, 402, 290, 25).
pub const STRIP: [i32; 4] = [456, 402, 290, 25];
/// The key the list is built in (`scenario_list_finish` 0x582E90: the version, then the name).
pub const BUILT_KEY: u8 = 2;
/// A second row press within this many milliseconds of the last begins (`scenario_row_click`
/// 0x5872E4: 0x190).
pub const DOUBLE_MS: u32 = 400;

/// The labels and captions whose text the screen sets as it opens, by id, with their names in
/// the key catalogue (`shell.scenario.<name>`): the constructor's texts (GENRLTXT's; the title
/// ArrayTxt.txt's) and the list's sizes label the redraw draws.
pub const LABELS: [(u16, &str); 14] = [
    (TITLE, "title"),
    (SIZES, "sizes"),
    (132, "player-difficulty"),
    (133, "rating-label"),
    (134, "map-difficulty-label"),
    (1010, "name-label"),
    (105, "description-label"),
    (1011, "victory-label"),
    (1012, "loss-label"),
    (1013, "allies"),
    (386, "enemies"),
    (LIST, "list"),
    (ADVANCED, "advanced"),
    (RANDOM, "random"),
];

/// The labels whose word takes a colon (the executable's "%s:": the player's difficulty, the
/// rating, the allies, the enemies).
pub const COLONED: [u16; 4] = [132, 133, 1013, 386];

/// The items with a right press's popup in this task, by id, with their names in the key
/// catalogue (`shell.scenario.help.<name>`: `Help.txt`'s record id − 83, its right-click text).
pub const HELPED: [(u16, &str); 11] = [
    (LIST, "list"),
    (ADVANCED, "advanced"),
    (RANDOM, "random"),
    (BEGIN, "begin"),
    (BACK, "back"),
    (107, "easy"),
    (108, "normal"),
    (109, "hard"),
    (110, "expert"),
    (111, "impossible"),
    (SIZE, "size"),
];

/// The ratings by the chosen difficulty (`scenario_ratings` 0x683440), in percent.
pub const RATINGS: [u32; 5] = [80, 100, 130, 160, 200];

/// The map kept between visits at the start (`scenario_last_map` 0x683238).
pub const KEPT_MAP: &str = "Arrogance.h3m";

/// The difficulty kept between visits at the start (`scenario_last_difficulty` 0x683454:
/// Normal, recorded at M1 T2: `scn-new`).
pub const KEPT_DIFFICULTY: u8 = 1;

/// A map's format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Format {
    /// The Restoration of Erathia's (0x0E).
    RoE,
    /// Armageddon's Blade's (0x15).
    Ab,
    /// The Shadow of Death's (0x1C).
    SoD,
}

impl Format {
    /// Its frame of the list's version mark (`ScSelC.def`).
    #[must_use]
    pub const fn mark(self) -> u32 {
        match self {
            Self::RoE => 0,
            Self::Ab => 1,
            Self::SoD => 2,
        }
    }
}

/// Who may play a colour (the map's player +0 and +1).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Seat {
    /// A human may.
    pub human: bool,
    /// The computer may.
    pub computer: bool,
}

impl Seat {
    /// Whether anyone plays the colour.
    #[must_use]
    pub const fn plays(self) -> bool {
        self.human || self.computer
    }
}

/// A map of the list (`scenario_info`, `scenario_header_read` 0x583170: what the screen shows
/// of it), read by the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The file's name in the maps' folder.
    pub file: String,
    /// The format (none for another: an old format's map, T1b).
    pub format: Option<Format>,
    /// The name.
    pub name: String,
    /// The description.
    pub description: String,
    /// The side's length in tiles.
    pub size: u32,
    /// The map's difficulty (0 easy to 4 impossible).
    pub difficulty: u8,
    /// The special victory's code (0 to 10), none for the standard victory alone.
    pub victory: Option<u8>,
    /// With a special victory, whether the standard victory counts too.
    pub standard: bool,
    /// The special loss's code (0 to 2), none for the standard loss alone.
    pub loss: Option<u8>,
    /// Who may play each colour, red first.
    pub seats: [Seat; 8],
    /// Each colour's team (a map without teams: each its own).
    pub teams: [u8; 8],
}

impl Entry {
    /// The colours anyone plays (the record's +6, *inferred*: the list's players column).
    #[must_use]
    pub fn players(&self) -> u8 {
        count(self.seats.iter().filter(|s| s.plays()))
    }

    /// The colours a human may play (the record's +8, *inferred*: the list's players column).
    #[must_use]
    pub fn humans(&self) -> u8 {
        count(self.seats.iter().filter(|s| s.human))
    }

    /// The special victory's code as the record keeps it (+0x30): −1 for the standard alone.
    fn victory_code(&self) -> i16 {
        self.victory.map_or(-1, i16::from)
    }

    /// The special loss's code as the record keeps it (+0x7C): −1 for the standard alone.
    fn loss_code(&self) -> i16 {
        self.loss.map_or(-1, i16::from)
    }

    /// The colour a single player takes (`scenario_players_init` 0x584C40, §3.12): the first only
    /// a human may play, else the first a human may play, else the first anyone plays.
    #[must_use]
    pub fn local(&self) -> Option<usize> {
        let first = |f: fn(Seat) -> bool| self.seats.iter().position(|&s| f(s));
        first(|s| s.human && !s.computer)
            .or_else(|| first(|s| s.human))
            .or_else(|| first(Seat::plays))
    }
}

/// The list's order as built (`scenario_list_finish` 0x582E90: key 2, the format then the name
/// case-insensitively, `scenario_compare_version` 0x591190), the file's name breaking ties
/// (CLT-Q90: case-insensitively, then as it is).
pub fn sort(entries: &mut [Entry]) {
    sort_by(entries, BUILT_KEY, false);
}

/// The list sorted by `key` (`scenario_list_sort` 0x585050), descending when `descending`: the
/// key's own comparison reversed, then a stable sort's tie-breakers (CLT-Q90, decision 12): the
/// name case-insensitively, the file's name case-insensitively, then as it is.
pub fn sort_by(entries: &mut [Entry], key: u8, descending: bool) {
    entries.sort_by(|a, b| {
        let own = compare(key, a, b);
        let own = if descending { own.reverse() } else { own };
        own.then_with(|| caseless(&a.name).cmp(caseless(&b.name)))
            .then_with(|| caseless(&a.file).cmp(caseless(&b.file)))
            .then_with(|| a.file.cmp(&b.file))
    });
}

/// A sort key's comparison (design §3.4): 0 the names case-insensitively
/// (`scenario_compare_name` 0x5903B0), 1 the size (0x591310), 2 the format then the name
/// (`scenario_compare_version` 0x591190), 3 ten times the players plus the humans
/// (`scenario_compare_players` 0x590E00), 4 the victory's code (`scenario_compare_victory`
/// 0x591CB0), 5 the loss's (`scenario_compare_loss` 0x592030); another key none.
fn compare(key: u8, a: &Entry, b: &Entry) -> Ordering {
    let players = |e: &Entry| 10 * u16::from(e.players()) + u16::from(e.humans());
    match key {
        0 => caseless(&a.name).cmp(caseless(&b.name)),
        1 => a.size.cmp(&b.size),
        2 => a
            .format
            .cmp(&b.format)
            .then_with(|| caseless(&a.name).cmp(caseless(&b.name))),
        3 => players(a).cmp(&players(b)),
        4 => a.victory_code().cmp(&b.victory_code()),
        5 => a.loss_code().cmp(&b.loss_code()),
        _ => Ordering::Equal,
    }
}

/// The count of an iterator's items (at most 8 here).
fn count<T>(items: impl Iterator<Item = T>) -> u8 {
    u8::try_from(items.count()).unwrap_or(u8::MAX)
}

/// `text`'s characters lower-cased, for a case-insensitive comparison.
fn caseless(text: &str) -> impl Iterator<Item = char> + '_ {
    text.chars().flat_map(char::to_lowercase)
}

/// The screen's texts, laid out by the caller: the user's tables'.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Texts {
    /// The labels' and captions' texts by id ([`LABELS`]).
    pub labels: BTreeMap<u16, String>,
    /// A map's name when it has none (GENRLTXT 509).
    pub unnamed: String,
    /// The difficulties' names, easy to impossible (`difficulty_names` 0x6A77EC: ArrayTxt.txt 142
    /// to 146).
    pub difficulties: Vec<String>,
    /// The ratings' texts by the chosen difficulty ("%d%%" of [`RATINGS`]).
    pub ratings: Vec<String>,
    /// The victories' texts (`victory_texts` 0x69FB70: VcDesc.txt's records, the standard's
    /// first, then each special goal's by its code + 1).
    pub victories: Vec<String>,
    /// A special goal's text with the standard victory's (`"%s %s %s"`: the goal's, GENRLTXT 5,
    /// the standard's), by the goal's code.
    pub with_standard: Vec<String>,
    /// The losses' texts (`loss_texts` 0x69FC1C: LcDesc.txt's records, the standard's first).
    pub losses: Vec<String>,
    /// The right press's popups by item ([`HELPED`]; style 4).
    pub help: BTreeMap<u16, MessageBox>,
    /// The list's players column (`"%d/%d"`, 0x679DC8) by the players and the humans, 0 to 8
    /// each: the text of `p` players and `h` humans at `9 × p + h`.
    pub players: Vec<String>,
    /// The list's size column (GENRLTXT 512 to 515): 36, 72 and 108 tiles, another.
    pub sizes: Vec<String>,
}

/// The list's rows as the redraw draws them (`scenario_dialog_redraw` 0x584550; design §3.4):
/// each cell's item at the first row, the rows `step` apart, the selection's texts in the colour
/// `selected` (the others in their item's), and the list's scroll bar.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListLook {
    /// The rows (`scenario_rows` 0x69FDC8: 18).
    pub rows: u16,
    /// A row's height.
    pub step: i32,
    /// The selection's colour.
    pub selected: u32,
    /// The players column's text.
    pub players: Label,
    /// The size column's text.
    pub size: Label,
    /// The version's mark (`ScSelC.def`: frame 0 RoE, 1 AB, 2 SoD; none for another format).
    pub version: Def,
    /// The name's text.
    pub name: Label,
    /// The victory's mark (`scnrvict.def`: the special victory's frame, 11 for none).
    pub victory: Def,
    /// The loss's mark (`scnrloss.def`: the special loss's frame, 3 for none).
    pub loss: Def,
    /// The scroll bar (337).
    pub bar: scrollbar::Look,
}

/// Every asset `look` names (the adapter's batch at the shell's start): its marks' DEFs, its texts'
/// fonts and palette, the bar's DEF and track.
#[must_use]
pub fn list_names(look: &ListLook) -> Vec<String> {
    let mut names = Vec::new();
    for label in [&look.players, &look.size, &look.name] {
        names.push(label.font.clone());
        names.push(label.font_palette.clone());
    }
    for def in [&look.version, &look.victory, &look.loss] {
        names.extend(def.def.clone());
    }
    names.push(look.bar.def.clone());
    names.push(look.bar.track.clone());
    names
}

/// What is open over the screen.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Over {
    Nothing,
    /// An item's help popup, while the right button is held.
    Help(u16),
}

/// The scenario screen as it shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario {
    dialog: Dialog,
    shown: Shown,
    input: Input,
    texts: Texts,
    /// The maps in the list's order (the full list, +0x1030).
    entries: Vec<Entry>,
    /// The shown list: the entries of the filter's size, in order (+0x1050).
    listed: Vec<usize>,
    /// The selection in the shown list (+0x374).
    selected: Option<usize>,
    /// The shown list's row at the top (+0x370).
    top: usize,
    look: ListLook,
    bar: Scrollbar,
    /// The list shows (+0x37D).
    showing: bool,
    /// The list was built (+0x1864): its first showing.
    built: bool,
    /// The sort key recorded (+0x185C) and the order (+0x36C).
    key: u8,
    descending: bool,
    /// The filter's size, 0 for every one (+0x1860).
    filter: u32,
    /// The row the mouse pressed, until its release.
    row: Option<u16>,
    /// The mouse, as last moved or pressed.
    pointer: [i32; 2],
    /// The original's profile: the last redraw was the partial one (`scenario_select` 0x585500
    /// with the selection unchanged, 0x585A06), and the size mark as the last whole redraw drew
    /// it (its state and frame): CLT-Q138.
    partial: bool,
    drawn_size: (State, u32),
    /// The shell's clock (ms) and the last row press's time with its entry in the shown list
    /// (+0x60; none for the screen's opening).
    now: u32,
    last_row: Option<(u32, Option<usize>)>,
    kept: String,
    difficulty: u8,
    background: u8,
    over: Over,
    hovers: Hovers,
    bugs: Bugs,
}

impl Scenario {
    /// The screen from the layout's `dialog` and the list's look `look`, its texts `texts` and
    /// the maps `entries` (in the list's order, [`sort`]), the kept map [`KEPT_MAP`] and
    /// difficulty [`KEPT_DIFFICULTY`], in the profile `bugs`; the list hidden.
    #[must_use]
    pub fn new(
        dialog: Dialog,
        look: ListLook,
        texts: Texts,
        entries: Vec<Entry>,
        bugs: Bugs,
    ) -> Self {
        let mut shown = Shown::default();
        for (id, text) in &texts.labels {
            shown.texts.insert(*id, text.clone());
        }
        let bar = Scrollbar::new(look.bar.clone());
        let listed = (0..entries.len()).collect();
        let mut scenario = Self {
            dialog,
            shown,
            input: Input::new(bugs),
            texts,
            entries,
            listed,
            selected: None,
            top: 0,
            look,
            bar,
            showing: false,
            built: false,
            key: BUILT_KEY,
            descending: false,
            filter: 0,
            row: None,
            pointer: [0, 0],
            partial: false,
            drawn_size: (State::default(), 0),
            now: 0,
            last_row: None,
            kept: String::from(KEPT_MAP),
            difficulty: KEPT_DIFFICULTY,
            background: 0,
            over: Over::Nothing,
            hovers: Hovers::default(),
            bugs,
        };
        scenario.list_items(false);
        scenario.selecting()
    }

    /// The screen opened anew (`scenario_new` 0x4F0490), the menus' last items `hovers` carried
    /// for the menu after it, its background `gamselb0.pcx` until the caller draws
    /// ([`Scenario::with_background`]). Its busy cursor leaves the adventure map's arrow in the
    /// original's profile (CLT-Q137 (b): the client's keeps the default one).
    #[must_use]
    pub fn opened(&self, hovers: Hovers) -> Self {
        let hovers = match self.bugs {
            Bugs::Original => Hovers {
                arrow: Arrow::Adventure,
                ..hovers
            },
            Bugs::Fixed => hovers,
        };
        let mut opened = Self::new(
            self.dialog.clone(),
            self.look.clone(),
            self.texts.clone(),
            self.entries.clone(),
            self.bugs,
        );
        opened.kept.clone_from(&self.kept);
        opened.difficulty = self.difficulty;
        opened.hovers = hovers;
        opened.selecting()
    }

    /// The screen with its background `gamselb<n>.pcx` (`n` 0 or 1; another is taken as 0): the
    /// draw `random_between(1, 100) < 51` gives 1.
    #[must_use]
    pub const fn with_background(mut self, n: u8) -> Self {
        self.background = if n == 1 { 1 } else { 0 };
        self
    }

    /// Its background's index (0 or 1).
    #[must_use]
    pub const fn background(&self) -> u8 {
        self.background
    }

    /// The menus' last items, carried from the menu before it.
    #[must_use]
    pub const fn hovers(&self) -> Hovers {
        self.hovers
    }

    /// The entry selected, if any.
    #[must_use]
    pub fn entry(&self) -> Option<&Entry> {
        self.selected
            .and_then(|i| self.listed.get(i))
            .and_then(|&e| self.entries.get(e))
    }

    /// The maps, in the list's order (the full list).
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The shown list's maps, in order (the filter's size).
    pub fn listed(&self) -> impl Iterator<Item = &Entry> {
        self.listed.iter().filter_map(|&e| self.entries.get(e))
    }

    /// The selection's row in the shown list and the row at the top.
    #[must_use]
    pub const fn rows(&self) -> (Option<usize>, usize) {
        (self.selected, self.top)
    }

    /// Whether the list shows.
    #[must_use]
    pub const fn showing(&self) -> bool {
        self.showing
    }

    /// The chosen difficulty (0 easy to 4 impossible).
    #[must_use]
    pub const fn difficulty(&self) -> u8 {
        self.difficulty
    }

    /// Where it takes its keys.
    #[must_use]
    pub const fn context(&self) -> Context {
        Context::Scenario
    }

    /// Whether the cursor shows (a help popup hides it).
    #[must_use]
    pub const fn cursor(&self) -> bool {
        matches!(self.over, Over::Nothing)
    }

    /// The shell's clock is `ms`: a row's double press is timed on it, from the screen's opening
    /// (its first time here) on.
    pub fn clock(&mut self, ms: u32) {
        self.now = ms;
        if self.last_row.is_none() {
            self.last_row = Some((ms, None));
        }
    }

    /// The kept map's entry selected (else the first, none in an empty list) and shown.
    fn selecting(mut self) -> Self {
        let kept = self.kept_row();
        self.selected = kept.or_else(|| (!self.listed.is_empty()).then_some(0));
        self.show();
        self.whole();
        self
    }

    /// The kept map's row in the shown list (`scenario_find_by_name` 0x58D3E0: its file's name).
    fn kept_row(&self) -> Option<usize> {
        self.listed()
            .position(|e| e.file.eq_ignore_ascii_case(&self.kept))
    }

    /// The list's items shown or hidden: its panel, title, sizes label, rows, filters and
    /// columns (the bar is drawn by the screen while the list shows).
    fn list_items(&mut self, show: bool) {
        let rows = (0..self.look.rows).map(|r| ROW.saturating_add(r));
        let ids = [PANEL, TITLE, SIZES]
            .into_iter()
            .chain(rows)
            .chain(FILTERS.map(|(id, _)| id))
            .chain(COLUMNS.map(|(id, _)| id));
        for id in ids {
            let state = self.shown.state(id);
            self.shown.states.insert(
                id,
                State {
                    hidden: !show,
                    ..state
                },
            );
        }
    }

    /// The shown list's count of rows less the rows plus one: the bar's ticks.
    fn ticks(&self) -> i32 {
        let count = i32::try_from(self.listed.len()).unwrap_or(i32::MAX);
        count - i32::from(self.look.rows) + 1
    }

    /// Show Available Scenarios (128, `scenario_list_toggle` 0x580A70): a shown list hidden, else
    /// shown, the bar's ticks the list's and its value the top, the first time the list built.
    fn toggle_list(&mut self) {
        if self.showing {
            self.showing = false;
            self.list_items(false);
            self.whole();
            return;
        }
        self.list_items(true);
        self.showing = true;
        if !self.built {
            self.built = true;
            self.finish();
        }
        self.bar.reset(self.ticks());
        self.set_bar();
        self.show();
        self.whole();
    }

    /// The list as built (`scenario_list_finish` 0x582E90): sorted by the version, the recorded
    /// key the size's in the original's profile (CLT-Q91 (c): the client's records the key it
    /// sorted by); the kept map's row selected on the top row, the top no further than a page
    /// from the end.
    fn finish(&mut self) {
        self.sort_list(BUILT_KEY, false);
        self.key = match self.bugs {
            Bugs::Original => 1,
            Bugs::Fixed => BUILT_KEY,
        };
        self.descending = false;
        if let Some(row) = self.kept_row() {
            self.selected = Some(row);
            let rows = usize::from(self.look.rows);
            self.top = if row + rows > self.listed.len() {
                self.listed.len().saturating_sub(rows)
            } else {
                row
            };
        }
    }

    /// The full list sorted by `key` (`scenario_list_sort` 0x585050), the shown list rebuilt in
    /// its order, the top and the selection its first row.
    fn sort_list(&mut self, key: u8, descending: bool) {
        sort_by(&mut self.entries, key, descending);
        self.relist();
        self.top = 0;
        self.selected = (!self.listed.is_empty()).then_some(0);
        self.bar.set_value(0);
    }

    /// The shown list rebuilt from the full one: the filter's size, every one for 0.
    fn relist(&mut self) {
        let filter = self.filter;
        self.listed = (0..self.entries.len())
            .filter(|&e| filter == 0 || self.entries[e].size == filter)
            .collect();
    }

    /// A column's press with its key (`scenario_sort_click` 0x5870FA): on the recorded key's
    /// column the order flipped, another key recorded ascending; then sorted.
    fn sort_click(&mut self, key: u8) {
        if self.key == key {
            self.descending = !self.descending;
        } else {
            self.descending = false;
            self.key = key;
        }
        self.sort_list(key, self.descending);
        // The sort redraws whole before the click selects its first row (0x5852BF), which it
        // already is: the select's partial redraw (CLT-Q138).
        self.whole();
        self.show();
        self.partial = true;
    }

    /// A filter's press (`scenario_list_filter` 0x585D90): the shown list of the maps of `size`
    /// (every one for 0), the top and the selection its first row, the bar's ticks.
    fn filter_by(&mut self, size: u32) {
        self.filter = size;
        self.relist();
        self.top = 0;
        self.selected = (!self.listed.is_empty()).then_some(0);
        self.bar.reset(self.ticks());
        self.show();
        self.whole();
    }

    /// The bar moved the top to `value` (`scenario_list_scrolled` 0x57C990).
    fn scrolled(&mut self, value: Option<i32>) {
        if let Some(value) = value {
            self.top = usize::try_from(value).unwrap_or(0);
            self.show();
            self.partial = true;
        }
    }

    /// A list key (`scenario_dialog_proc` 0x587D00, the list showing): Up and Down the selection
    /// by a row (Up nothing on the first, Down nothing on the last), the top following it; Page
    /// Up the top up a page less one (to 0 at most), Page Down down one (to the last page at
    /// most; nothing in a list of one page), the selection keeping its distance from the top
    /// (none past the list; before it, *provisional*: the original selects a negative index).
    fn scroll(&mut self, scroll: Scroll) {
        let count = self.listed.len();
        let rows = usize::from(self.look.rows);
        let page = rows.saturating_sub(1);
        let Some(selected) = self.selected else {
            return;
        };
        // The selection's row from the top, negative above it (its distance kept by a page).
        let signed = |n: usize| i64::try_from(n).unwrap_or(i64::MAX);
        let row = signed(selected) - signed(self.top);
        let before = self.selected;
        match scroll {
            Scroll::Up => {
                let Some(new) = selected.checked_sub(1) else {
                    return;
                };
                if new < self.top {
                    self.top = new;
                    self.set_bar();
                }
                self.selected = Some(new);
            }
            Scroll::Down => {
                let new = selected + 1;
                if new >= count {
                    return;
                }
                // The top follows (the original's clamp to count − rows never binds here: the
                // next row is within the list, so the top stays at most count − rows).
                if new > self.top + page {
                    self.top += 1;
                    self.set_bar();
                }
                self.selected = Some(new);
            }
            Scroll::PageUp | Scroll::PageDown => {
                if scroll == Scroll::PageUp {
                    self.top = self.top.saturating_sub(page);
                } else if count > rows {
                    self.top = (self.top + page).min(count - rows);
                } else {
                    return;
                }
                self.set_bar();
                let target = usize::try_from(signed(self.top) + row).ok();
                if let Some(target) = target.filter(|&t| t < count) {
                    self.selected = Some(target);
                }
            }
        }
        // The key redraws whole (0x587ED5), then selects: a selection that stays redraws part
        // (CLT-Q138).
        self.whole();
        self.show();
        if self.selected == before {
            self.partial = true;
        } else {
            self.whole();
        }
    }

    /// The bar's value the top.
    fn set_bar(&mut self) {
        self.bar
            .set_value(i32::try_from(self.top).unwrap_or(i32::MAX));
    }

    /// A row's click (`scenario_row_click` 0x5872E4): within the list, a second within
    /// [`DOUBLE_MS`] of the last (in the original's profile any row's or the opening; in the
    /// client's the same row's, CLT-Q91 (c)) begins (Begin: T2, nothing yet); else it is timed
    /// and selected.
    fn row_click(&mut self, row: u16) {
        let index = self.top + usize::from(row - ROW);
        if index >= self.listed.len() {
            return;
        }
        let double = self.last_row.is_some_and(|(at, last)| {
            let soon = self.now.saturating_sub(at) < DOUBLE_MS;
            soon && (self.bugs == Bugs::Original || last == Some(index))
        });
        if double {
            return;
        }
        self.last_row = Some((self.now, Some(index)));
        let same = self.selected == Some(index);
        self.selected = Some(index);
        self.show();
        if same {
            self.partial = true;
        } else {
            self.whole();
        }
    }

    /// The screen redrawn whole: the size mark as it is now is the one drawn.
    fn whole(&mut self) {
        self.partial = false;
        let frame = self.shown.frames.get(&SIZE).copied().unwrap_or(0);
        self.drawn_size = (self.shown.state(SIZE), frame);
    }

    /// Whether the screen shows the partial redraw (the original's profile, the list showing).
    fn stale(&self) -> bool {
        self.partial && self.showing && self.bugs == Bugs::Original
    }

    /// What the information panel shows of the entry selected (`scenario_select` 0x585500, its
    /// draw `scenario_info_draw` 0x5840F0, the flags `scenario_team_flags` 0x585300), and the
    /// chosen difficulty's button selected.
    fn show(&mut self) {
        for (k, &id) in DIFFICULTIES.iter().enumerate() {
            let state = self.shown.state(id);
            let selected = usize::from(self.difficulty) == k;
            self.shown.states.insert(id, State { selected, ..state });
        }
        let rating = text_at(&self.texts.ratings, usize::from(self.difficulty));
        self.shown.texts.insert(RATING, rating);
        for id in ALLIES..ENEMIES + 8 {
            self.shown.states.insert(id, hidden());
        }
        let entry = self
            .selected
            .and_then(|i| self.listed.get(i))
            .and_then(|&e| self.entries.get(e));
        let shown = &mut self.shown;
        let Some(entry) = entry else {
            for id in [NAME, DESCRIPTION, VICTORY, LOSS, MAP_DIFFICULTY] {
                shown.texts.insert(id, String::new());
            }
            for id in [SIZE, VICTORY_MARK, LOSS_MARK] {
                shown.states.insert(id, hidden());
            }
            return;
        };
        let t = &self.texts;
        let name = if entry.name.is_empty() {
            t.unnamed.clone()
        } else {
            entry.name.clone()
        };
        shown.texts.insert(NAME, name);
        shown.texts.insert(DESCRIPTION, entry.description.clone());
        let victory = match entry.victory {
            Some(goal) if entry.standard => text_at(&t.with_standard, usize::from(goal)),
            Some(goal) => text_at(&t.victories, usize::from(goal) + 1),
            None => text_at(&t.victories, 0),
        };
        shown.texts.insert(VICTORY, victory);
        let loss = text_at(&t.losses, entry.loss.map_or(0, |l| usize::from(l) + 1));
        shown.texts.insert(LOSS, loss);
        let difficulty = text_at(&t.difficulties, usize::from(entry.difficulty));
        shown.texts.insert(MAP_DIFFICULTY, difficulty);
        for id in [SIZE, VICTORY_MARK, LOSS_MARK] {
            shown.states.insert(id, State::default());
        }
        shown.frames.insert(SIZE, size_frame(entry.size));
        // The marks: the goal's frame (0 to 10), 11 for none; the loss's (0 to 2), 3 for none.
        let victory_mark = entry.victory.map_or(11, u32::from);
        shown.frames.insert(VICTORY_MARK, victory_mark);
        shown
            .frames
            .insert(LOSS_MARK, entry.loss.map_or(3, u32::from));
        // Each colour that plays, in colour order: of the local colour's team the next allies'
        // flag (the local one's own first), any other the next enemies'.
        let team = entry.local().map(|c| entry.teams[c]);
        let (mut allies, mut enemies) = (ALLIES, ENEMIES);
        for colour in (0..8u8).filter(|&c| entry.seats[usize::from(c)].plays()) {
            let slot = if Some(entry.teams[usize::from(colour)]) == team {
                &mut allies
            } else {
                &mut enemies
            };
            shown.states.insert(*slot, State::default());
            shown.frames.insert(*slot, u32::from(colour));
            *slot += 1;
        }
    }

    /// The screen's draw list as it is (the background is the caller's: [`Scenario::background`]):
    /// its items, then with the list its bar and its rows.
    #[must_use]
    pub fn draws(&self, players: &str) -> Vec<Draw> {
        let mut draws = if self.stale() {
            // The partial redraw (CLT-Q138, the original's profile): the items it leaves as the
            // last whole redraw drew them (the size mark's frame then), then those it draws again
            // over them (0x585A06: the panel, the title, the filters, the columns; the redraw's
            // own).
            let (mut rest, mut again) = (self.shown.clone(), self.shown.clone());
            for item in &self.dialog.items {
                let id = item.id();
                hide(if redrawn(id) { &mut rest } else { &mut again }, id);
            }
            let (state, frame) = self.drawn_size;
            rest.states.insert(SIZE, state);
            rest.frames.insert(SIZE, frame);
            let mut draws = self.dialog.draws(&rest, players);
            draws.extend(self.dialog.draws(&again, players));
            draws
        } else {
            self.dialog.draws(&self.shown, players)
        };
        if self.showing {
            draws.extend(self.bar.draws(self.dialog.place));
            draws.extend(self.row_draws());
        }
        draws
    }

    /// What the partial redraw leaves as the screen kept it (CLT-Q138, the original's profile):
    /// the flags' strip ([`STRIP`]), restored from its copy of the background (the right-hand
    /// panel) before the flags and the enemies' label are drawn again (`scenario_team_flags`
    /// 0x585300, its copy kept at its first call), nothing drawn over it after: the allies'
    /// label's colon loses its shadow there. Its rectangle and its draws over the background.
    #[must_use]
    pub fn stale_draws(&self, players: &str) -> Option<([i32; 4], Vec<Draw>)> {
        if !self.stale() {
            return None;
        }
        let mut strip = self.shown.clone();
        for item in &self.dialog.items {
            let id = item.id();
            if id != PANEL_BACK && id != ENEMIES_LABEL && !(ALLIES..ENEMIES + 8).contains(&id) {
                hide(&mut strip, id);
            }
        }
        Some((STRIP, self.dialog.draws(&strip, players)))
    }

    /// The rows the redraw draws (`scenario_dialog_redraw` 0x584550): from the top, as many as
    /// the list has up to the rows.
    fn row_draws(&self) -> Vec<Draw> {
        let look = &self.look;
        let [px, py] = self.dialog.place;
        let mut draws = Vec::new();
        let rows = (self.listed.iter().enumerate().skip(self.top))
            .filter_map(|(index, &e)| self.entries.get(e).map(|entry| (index, entry)));
        for (r, (index, entry)) in (0i32..i32::from(look.rows)).zip(rows) {
            let dy = py + look.step * r;
            let colour = if self.selected == Some(index) {
                look.selected
            } else {
                look.players.colour
            };
            let text = |label: &Label, text: String| Draw::Label {
                item: Label {
                    colour,
                    ..label.clone()
                },
                x: px + label.at[0],
                y: dy + label.at[1],
                text,
                disabled: false,
            };
            let mark = |def: &Def, frame: Option<u32>| {
                let name = def.def.clone()?;
                Some(Draw::Button {
                    def: name,
                    frame: frame?,
                    x: px + def.at[0],
                    y: dy + def.at[1],
                    mirrored: false,
                    player: None,
                })
            };
            let players = 9 * usize::from(entry.players()) + usize::from(entry.humans());
            draws.push(text(&look.players, text_at(&self.texts.players, players)));
            let size = match entry.size {
                36 => 0,
                72 => 1,
                108 => 2,
                _ => 3,
            };
            draws.push(text(&look.size, text_at(&self.texts.sizes, size)));
            draws.extend(mark(&look.version, entry.format.map(Format::mark)));
            let name = if entry.name.is_empty() {
                self.texts.unnamed.clone()
            } else {
                entry.name.clone()
            };
            draws.push(text(&look.name, name));
            draws.extend(mark(
                &look.victory,
                Some(entry.victory.map_or(11, u32::from)),
            ));
            draws.extend(mark(&look.loss, Some(entry.loss.map_or(3, u32::from))));
        }
        draws
    }

    /// The popup open over the screen, if any.
    #[must_use]
    pub fn box_draws(&self, players: &str) -> Vec<Draw> {
        match &self.over {
            Over::Help(id) => self
                .texts
                .help
                .get(id)
                .map(|m| m.dialog.draws(&m.shown, players))
                .unwrap_or_default(),
            Over::Nothing => Vec::new(),
        }
    }

    /// The point `point` from the dialog's corner.
    const fn inside(&self, point: [i32; 2]) -> [i32; 2] {
        [
            point[0] - self.dialog.place[0],
            point[1] - self.dialog.place[1],
        ]
    }

    /// The mouse moved to `point`: the bar it holds follows it (`h3_ui::scrollbar`), a button it
    /// holds follows it (its sample as it comes back); nothing else shows a hover.
    pub fn moved(&mut self, point: [i32; 2]) -> Did {
        self.pointer = point;
        if self.bar.held() {
            let value = self.bar.moved(self.inside(point));
            self.scrolled(value);
            return Did::CHANGED;
        }
        match self.over {
            Over::Nothing => Did::held(self.input.moved(&self.dialog, &mut self.shown, point)),
            Over::Help(_) => Did::default(),
        }
    }

    /// The left button pressed at `point`: the list's bar or a row while the list shows, else the
    /// shared input's buttons (a hidden item takes no press).
    pub fn press(&mut self, point: [i32; 2]) -> Did {
        self.pointer = point;
        if self.over != Over::Nothing {
            return self.close_help();
        }
        let inside = self.inside(point);
        if self.showing && self.row.is_none() && self.bar.contains(inside) {
            let value = self.bar.press(inside);
            self.scrolled(value);
            return Did::CHANGED;
        }
        let rows = ROW..ROW.saturating_add(self.look.rows);
        match self.item_at(point) {
            Some(id) if rows.contains(&id) => {
                self.row = Some(id);
                return Did::default();
            }
            _ => {}
        }
        let hidden = self
            .dialog
            .item_at(point)
            .is_some_and(|item| self.shown.state(item.id()).hidden);
        if hidden {
            return Did::default();
        }
        Did::held(self.input.press(&self.dialog, &mut self.shown, point))
    }

    /// The left button released: the bar let go, a row clicked (in the original's profile
    /// wherever the mouse is, in the client's over the row), a button clicked.
    pub fn release(&mut self) -> Did {
        if self.bar.held() {
            let value = self.bar.release();
            self.scrolled(value);
            return Did::CHANGED;
        }
        if let Some(row) = self.row.take() {
            let over = self.item_at(self.pointer) == Some(row);
            if self.bugs == Bugs::Original || over {
                self.row_click(row);
            }
            return Did::CHANGED;
        }
        match self.over {
            Over::Nothing => {
                let did = self.input.release(&mut self.shown);
                self.clicked(did.changed, did.clicked)
            }
            Over::Help(_) => self.close_help(),
        }
    }

    /// The right button pressed at `point`: the help of the item there, unless the mouse holds a
    /// button or the bar.
    pub fn right(&mut self, point: [i32; 2]) -> Did {
        if self.over != Over::Nothing || self.input.holds_button() || self.bar.held() {
            return Did::default();
        }
        match self
            .item_at(point)
            .filter(|id| self.texts.help.contains_key(id))
        {
            Some(id) => {
                self.over = Over::Help(id);
                Did::CHANGED
            }
            None => Did::default(),
        }
    }

    /// The right button released: the popup closed; the bar or a button the mouse holds released
    /// as by the left button (the bar's loop ends at either release, 0x596607).
    pub fn right_release(&mut self) -> Did {
        if self.bar.held() {
            return self.release();
        }
        match self.over {
            Over::Help(_) => self.close_help(),
            Over::Nothing => {
                let did = self.input.right_release(&mut self.shown);
                self.clicked(did.changed, did.clicked)
            }
        }
    }

    /// A key down: the key table's `command` for it, `by` its name (whose release clicks); the
    /// list's keys while it shows. None while the bar is held (its loop takes no key).
    pub fn key(&mut self, command: Option<Command>, by: &str) -> Did {
        if self.bar.held() {
            return Did::default();
        }
        match (&self.over, command) {
            (Over::Nothing, Some(Command::Dialog(id))) if self.has(id) => {
                Did::held(self.input.key(&mut self.shown, id, by))
            }
            (Over::Nothing, Some(Command::List(scroll))) if self.showing => {
                self.scroll(scroll);
                Did::CHANGED
            }
            _ => Did::default(),
        }
    }

    /// The key `by` released.
    pub fn key_up(&mut self, by: &str) -> Did {
        match self.over {
            Over::Nothing => {
                let did = self.input.key_up(&mut self.shown, by);
                self.clicked(did.changed, did.clicked)
            }
            Over::Help(_) => {
                let did = self.input.let_go(&mut self.shown, Some(by));
                Did {
                    changed: did.changed,
                    ..Did::default()
                }
            }
        }
    }

    /// Whether the layout has a button `id`.
    fn has(&self, id: u16) -> bool {
        self.dialog
            .items
            .iter()
            .any(|item| matches!(item, h3_ui::ui::Item::Button(b) if b.id == id))
    }

    /// The item under the mouse at `point` (`dialog_item_at` 0x5FF3E0: the last in order whose
    /// box holds it, not hidden), the client's own items ([`OWN`]) passed over.
    fn item_at(&self, point: [i32; 2]) -> Option<u16> {
        let [x, y] = self.inside(point);
        self.dialog
            .items
            .iter()
            .rev()
            .filter(|item| item.id() < OWN && !self.shown.state(item.id()).hidden)
            .find(|item| {
                item.area().is_some_and(|([left, top], [w, h])| {
                    (left..left.saturating_add(w)).contains(&x)
                        && (top..top.saturating_add(h)).contains(&y)
                })
            })
            .map(h3_ui::ui::Item::id)
    }

    /// The help popup closed, the screen redrawn whole (§3.8).
    fn close_help(&mut self) -> Did {
        self.over = Over::Nothing;
        self.whole();
        Did::CHANGED
    }

    /// A hold ended (`changed`: a button went up) with the click `clicked`, if any: its command
    /// (`scenario_dialog_click` 0x5865B0): Back ends the screen (0x7801), Show Available
    /// Scenarios toggles the list, a filter and a column act on it; the others' commands are later
    /// tasks' (the module's notes).
    fn clicked(&mut self, changed: bool, clicked: Option<u16>) -> Did {
        let mut did = Did {
            changed,
            sample: false,
            ended: (clicked == Some(BACK)).then_some(Ended::Scenario(false)),
        };
        let Some(id) = clicked else {
            return did;
        };
        if id == LIST {
            self.toggle_list();
            did.changed = true;
        } else if let Some(&(_, size)) = FILTERS.iter().find(|&&(f, _)| f == id) {
            self.filter_by(size);
            did.changed = true;
        } else if let Some(&(_, key)) = COLUMNS.iter().find(|&&(c, _)| c == id) {
            self.sort_click(key);
            did.changed = true;
        }
        did
    }
}

/// Whether the partial redraw (`scenario_select` 0x585500 with the selection unchanged, from
/// 0x585A06) draws item `id` again: the list's panel, its title, the filters and the columns, and
/// what the redraw draws itself (the client's ids but the right-hand panel and the constructor's
/// labels).
fn redrawn(id: u16) -> bool {
    let own = (NAME..=RATING).contains(&id) || id == SIZES;
    own || id == PANEL
        || id == TITLE
        || FILTERS.iter().any(|&(f, _)| f == id)
        || COLUMNS.iter().any(|&(c, _)| c == id)
}

/// Item `id` hidden in `shown`, its other state bits kept.
fn hide(shown: &mut Shown, id: u16) {
    let state = shown.state(id);
    shown.states.insert(
        id,
        State {
            hidden: true,
            ..state
        },
    );
}

/// The size mark's frame (`scnrmpsz.def`): 0 to 3 for 36, 72, 108 and 144 tiles, else 4.
#[must_use]
pub const fn size_frame(size: u32) -> u32 {
    match size {
        36 => 0,
        72 => 1,
        108 => 2,
        144 => 3,
        _ => 4,
    }
}

/// A hidden item's state.
const fn hidden() -> State {
    State {
        hidden: true,
        pressed: false,
        disabled: false,
        selected: false,
    }
}

/// The text `k` of `texts`, empty past them.
fn text_at(texts: &[String], k: usize) -> String {
    texts.get(k).cloned().unwrap_or_default()
}

#[cfg(test)]
pub(crate) mod tests;
