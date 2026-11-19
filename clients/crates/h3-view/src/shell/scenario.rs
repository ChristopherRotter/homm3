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
//! - **A right press** (`scenario_dialog_right_click` 0x5822D0, §3.8, §20): the item's `Help.txt`
//!   right-click text (record id − 83) in a popup of style 4 while the right button is held, the
//!   cursor hidden; the list's bar's too (337: record 254), the advanced options' headers, bar and
//!   each colour's items (M3 T3c2: [`HELPED`], [`COLOUR_HELPED`]); none for an item without one,
//!   a list row, an item not enabled (`dialog_item_at` 0x5FF3E0: a single player's handicap), nor
//!   while the mouse holds a button or a bar. On the flags' strip (387) the teams' dialog
//!   (`scenario_teams_dialog` 0x576540, M3 T1b2) while the right button is held, the cursor
//!   showing: "Team %d" of each team of the colours that play, in the order of their first
//!   colour, each with its colours' flags. The screen is redrawn whole as either closes.
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
//! - **The hot seat** (E27 M3 T4a; §28): opened from the Multiplayer dialog's hot seat
//!   ([`Scenario::opened_hot_seat`]): its players the hot seat's names, its list the maps two
//!   humans may play, the flags' strip from the first player's colour; its advanced options
//!   (`advanced`'s hot seat).
//!
//! - **The difficulty** (E27 M3 T2a; §3.5, §21): a press of 107 to 111 chooses it
//!   (`scenario_difficulty_click` 0x586BF3: its button selected, the rating its own) and redraws
//!   the screen whole. Each opening takes Normal ([`OPENING_DIFFICULTY`]: the list's build at the
//!   opening sets it from the selected map's record), the list's showing keeps the choice.
//! - **The empty list** (E27 M3 T2b; §21): the panel shows nothing of a map, the rating neither
//!   (`scenario_info_draw` 0x5840F0 draws nothing there). In the original's profile the size mark
//!   and the flags keep what they showed (a filter's size, the last map's flags), the teams'
//!   dialog shows the last map's teams (with no map ever shown, one team of the eight colours),
//!   and with no map at all no difficulty shows chosen before a press; in the client's the size
//!   mark shows frame 4 (CLT-Q115 (b)), no flags, no teams' dialog, the choice always shown
//!   (CLT-Q139 (b)). **Begin** there shows GENRLTXT 530 in an OK box (`scenario_begin` 0x58BCE0);
//!   the original's only once a map was shown (with no map at all it does nothing: CLT-Q114 (b)).
//!
//! - **The random map** (E27 M3 T5a; §3.16, §30): Random Map (130) shows its tab over the list's
//!   place (`random`); a chosen random map's own record is what the panel, the game and the
//!   advanced options read first (`scenario_to_game` 0x583580, `scenario_entry_name` 0x584010).
//!
//! *Not built yet:* Begin with a map (T6: a double press begins nothing yet either), the random
//! map's right press, Show Random Maps and Begin with it (T5b), the load's modes (the client's
//! stub until M4), GENRLTXT 503 while the list is read again (a network game's message 1015
//! alone, `sub_58B120` 0x58B120: the network modes', `CLT-STUB-6`; design §20.3).

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;

use h3_assets::fnt::Font;
use h3_render::Bugs;
use h3_ui::edit::Scan;
use h3_ui::input::Input;
use h3_ui::scrollbar::{self, Scrollbar};
use h3_ui::ui::{Def, Dialog, Draw, Label, Shown, State};
use serde::Deserialize;

use super::Ended;
use super::keys::{Command, Context, Scroll};
use super::menu::{Arrow, Did, Hovers};
use crate::message::{MessageBox, Open, Outcome};

mod advanced;
mod description;
mod random;

use advanced::{Advanced, Map, Refusal};
pub use advanced::{
    AdvancedLook, AdvancedTexts, Bonus, CreatureRow, HERO_CLASS, HERO_NAME, HeroPopupLook,
    Pictured, PopupLook, Record, TownPopupLook,
};
use description::Description;
pub use description::{DESCRIPTION_BYTES, DescriptionLook, description_names};
pub use random::{Choices, HELPED as RANDOM_HELPED};
use random::{Pressed, RandomMap};

/// Every asset the advanced options' look names (the adapter's batch at the shell's start).
#[must_use]
pub fn advanced_names(look: &AdvancedLook) -> Vec<String> {
    Advanced::names(look)
}

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
/// The flags' strip as an item (387, over the flags and the enemies' label): its right press the
/// teams' dialog (`scenario_teams_dialog` 0x576540).
pub const TEAMS: u16 = 387;
/// The list's scroll bar (337, `scrollbar_create` 0x5960D0; drawn beside the dialog's items): its
/// right press the hint of `Help.txt`'s record 254, as an item's.
pub const BAR: u16 = 337;
/// The key the list is built in (`scenario_list_finish` 0x582E90: the version, then the name).
pub const BUILT_KEY: u8 = 2;
/// A second row press within this many milliseconds of the last begins (`scenario_row_click`
/// 0x5872E4: 0x190).
pub const DOUBLE_MS: u32 = 400;

/// The labels and captions whose text the screen sets as it opens, by id, with their names in
/// the key catalogue (`shell.scenario.<name>`): the constructor's texts (GENRLTXT's; the title
/// ArrayTxt.txt's) and the list's sizes label the redraw draws.
pub const LABELS: [(u16, &str); 30] = [
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
    // The advanced options' (E27 M3 T3a).
    (advanced::TITLE, "advanced-title"),
    (advanced::SUBTITLE, "advanced-subtitle"),
    (339, "name-header"),
    (342, "town-header"),
    (343, "hero-header"),
    (344, "bonus-header"),
    (340, "duration-label"),
    // The random map's tab (E27 M3 T5a: GENRLTXT 739, 740, 753 to 759).
    (random::TITLE, "random-title"),
    (random::SUBTITLE, "random-subtitle"),
    (random::LABELS[0], "random-size"),
    (random::LABELS[1], "random-players"),
    (random::LABELS[2], "random-teams"),
    (random::LABELS[3], "random-computers"),
    (random::LABELS[4], "random-computer-teams"),
    (random::LABELS[5], "random-water"),
    (random::LABELS[6], "random-monsters"),
];

/// The labels whose word takes a colon (the executable's "%s:": the player's difficulty, the
/// rating, the allies, the enemies).
pub const COLONED: [u16; 4] = [132, 133, 1013, 386];

/// The items with a right press's popup, by id, with their names in the key catalogue
/// (`shell.scenario.help.<name>`: `Help.txt`'s record id − 83, its right-click text): every item
/// of 104 to 344 the screen has but the rows (none) and the flags (under the strip, [`TEAMS`]),
/// the bar ([`BAR`]); the advanced options' headers, the duration's label and bar (E27 M3 T3c2)
/// and each colour's items ([`COLOUR_HELPED`]).
pub const HELPED: [(u16, &str); 33] = [
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
    (105, "description-label"),
    (132, "player-difficulty"),
    (133, "rating-label"),
    (134, "map-difficulty-label"),
    (137, "filter-small"),
    (138, "filter-medium"),
    (139, "filter-large"),
    (140, "filter-extra-large"),
    (141, "filter-all"),
    (190, "sort-players"),
    (191, "sort-size"),
    (192, "sort-version"),
    (193, "sort-name"),
    (194, "sort-victory"),
    (195, "sort-loss"),
    (BAR, "bar"),
    // The advanced options' (E27 M3 T3c2): the headers (the town's 342 on AB and Complete, record
    // 259 the one with Conflux), the duration's label and its bar (338, record 255).
    (339, "name-header"),
    (342, "town-header"),
    (343, "hero-header"),
    (344, "bonus-header"),
    (340, "duration-label"),
    (advanced::BAR, "duration-bar"),
];

/// The advanced options' items of every colour with a right press's popup (E27 M3 T3c2), by
/// colour 0's id (colour `c`'s `+ c`, its record `+ c`), with their names in the key catalogue
/// (`shell.scenario.help.<name>`): who may play (record 116), the handicap (124: a single player's
/// is not enabled, so the panel under it has none), the town's, the hero's and the bonus's arrows
/// (132 to 179) and the flag (180). Each colour's record is its own in `Help.txt`, the eight one
/// text in both LODs: one key, colour 0's record, for all eight (*provisional*). The names, the
/// edits and the areas (their popups or none) have no hint.
pub const COLOUR_HELPED: [(u16, &str); 9] = [
    (advanced::KIND, "kind"),
    (advanced::HANDICAP, "handicap"),
    (advanced::TOWN_LEFT, "town-left"),
    (advanced::TOWN_RIGHT, "town-right"),
    (advanced::HERO_LEFT, "hero-left"),
    (advanced::HERO_RIGHT, "hero-right"),
    (advanced::BONUS_LEFT, "bonus-left"),
    (advanced::BONUS_RIGHT, "bonus-right"),
    (advanced::FLAG, "flag"),
];

/// The ratings by the chosen difficulty (`scenario_ratings` 0x683440), in percent.
pub const RATINGS: [u32; 5] = [80, 100, 130, 160, 200];

/// The map kept between visits at the start (`scenario_last_map` 0x683238).
pub const KEPT_MAP: &str = "Arrogance.h3m";

/// The difficulty at each opening (E27 M3 T2a, design §21.1): the list's build at the opening
/// (`scenario_slots_new` 0x57FADC, `scenario_list_finish` 0x58314E) sets
/// `scenario_last_difficulty` 0x683454 from the selected record's +0x33C, which a new game's map
/// record holds at 1 (its constructors 0x579049, 0x579177), and the game from it
/// (`scenario_to_game` 0x583673): Normal, whatever was chosen before (recorded: `scn-difficulty`
/// reopened).
pub const OPENING_DIFFICULTY: u8 = 1;

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

/// Who may play a colour (the map's player +0 and +1) and what the advanced options offer it
/// (E27 M3 T3a: the player's +8, +0xA, +0xB, +0x18, +0x1C and +0x20; design §3.14).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Seat {
    /// A human may.
    pub human: bool,
    /// The computer may.
    pub computer: bool,
    /// The factions it may start with (bit per faction, Castle bit 0 to Conflux bit 8).
    pub factions: u16,
    /// The map's random-town flag (the dialog then offers a choice of town).
    pub random_town: bool,
    /// Its main town generates its starting hero.
    pub generates: bool,
    /// The random-hero flag.
    pub random_hero: bool,
    /// The main hero the map customises, if any.
    pub main_hero: Option<MainHero>,
}

/// A colour's main hero as the map's header customises it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MainHero {
    /// Its hero type.
    pub hero_type: u8,
    /// Its portrait, none where the map leaves it at its type's own (the file's 0xFF: the
    /// original keeps −1 and its row shows no hero; CLT-Q141 (b) shows the type's).
    pub portrait: Option<u8>,
}

/// What the map's import gives the advanced options of its heroes (E27 M3 T3b1; design §24): the
/// engine's offers, the main heroes' names and the map's customised heroes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Heroes {
    /// Each colour's offer by faction, red and Castle first (`h3_game::GameSetup::offer` with that
    /// faction chosen, TWN-SET-006: the hero types a human may pick, at most 16, in type order;
    /// the original's `scenario_hero_lists` 0x583890); empty where the engine offers none.
    pub offers: [[Vec<u8>; 9]; 8],
    /// Each colour's main hero's name of the map (the players block's +0x24), empty for none.
    pub main_names: [String; 8],
    /// The map's customised heroes (its disposed heroes, the original's game +0x1FB2C), in the
    /// map's order.
    pub custom: Vec<CustomHero>,
}

/// A hero type the map customises (its disposed heroes): its portrait and name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CustomHero {
    /// The hero type.
    pub hero_type: u8,
    /// Its portrait, none for its type's own.
    pub portrait: Option<u8>,
    /// Its name, empty for its type's own.
    pub name: String,
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
    /// The heroes its import offers and customises (E27 M3 T3b1).
    pub heroes: Heroes,
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

    /// The colour the hot seat's first player takes (`scenario_players_init` 0x584C40's hot-seat
    /// loop, E27 M3 T4a: the first a human may play).
    #[must_use]
    pub fn first_human(&self) -> Option<usize> {
        self.seats.iter().position(|s| s.human)
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
    /// The right press's popups by item ([`HELPED`], [`COLOUR_HELPED`] for each colour; style 4).
    pub help: BTreeMap<u16, MessageBox>,
    /// The list's players column (`"%d/%d"`, 0x679DC8) by the players and the humans, 0 to 8
    /// each: the text of `p` players and `h` humans at `9 × p + h`.
    pub players: Vec<String>,
    /// The list's size column (GENRLTXT 512 to 515): 36, 72 and 108 tiles, another.
    pub sizes: Vec<String>,
    /// The teams' dialog (none: the strip's right press opens nothing).
    pub teams: Option<Teams>,
    /// Begin's message over an empty list (GENRLTXT 530 in an OK box, E27 M3 T2b; none: Begin
    /// does nothing there).
    pub no_maps: Option<MessageBox>,
    /// Begin's message over a map where a player has no colour (GENRLTXT 531 in an OK box, E27 M3
    /// T4b; none: Begin does nothing there).
    pub unplaced: Option<MessageBox>,
    /// Begin's message over a map where fewer than two of the hot seat's players hold a colour
    /// (GENRLTXT 83 in an OK box, E27 M3 T4b; none: Begin does nothing there).
    pub too_few: Option<MessageBox>,
    /// The description's font (`smalfont.fnt`, its metrics: E27 M3 T2c, the split); none: the
    /// description one label, never scrolling.
    pub description_font: Option<Font>,
    /// The advanced options (E27 M3 T3a; none: Show Advanced Options does nothing).
    pub advanced: Option<AdvancedTexts>,
    /// A random map's name and description (E27 M3 T5a: GENRLTXT 741, 742).
    pub random: [String; 2],
    /// Begin's notice with a random map chosen (E27 M3 T5b; CLT-Q92, decision 13: the client's
    /// stub, `shell.stub.random-map`, where the original generates the map; none: Begin does
    /// nothing there).
    pub random_stub: Option<MessageBox>,
}

/// The teams' dialog's look (`scenario_teams_dialog` 0x576540; the layout's `[scenario.teams]`):
/// its frame's assets, the title and the first team's name and flag at their places in the
/// dialog, the next team's `step` lower, a team's next flag `flag_step` to the right.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TeamsLook {
    /// The background's tile (`diboxbck.pcx`).
    pub background: String,
    /// The frame's pieces (`dialgbox.def`, in its own colours).
    pub frame: String,
    /// The title (GENRLTXT 658: `medfont.fnt`, colour 1, alignment 1, at (10, 20, 236, 36)).
    pub title: Label,
    /// The first team's name ("Team %d", GENRLTXT 657: `smalfont.fnt`, at (10, 56, 236, 18)).
    pub team: Label,
    /// The first team's flags (`itgflags.def`, the colour's frame, at y 76, 15 × 20): their row
    /// centred on the dialog (its x unused).
    pub flag: Def,
    /// A team's height (50).
    pub step: i32,
    /// A flag's width with the gap to the next (18).
    pub flag_step: i32,
}

/// The teams' dialog's look and texts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Teams {
    /// The look.
    pub look: TeamsLook,
    /// The title.
    pub title: String,
    /// The teams' names, the first team's first ("Team %d" of 1 to 8).
    pub names: Vec<String>,
}

/// The teams' dialog's place and width (`scenario_teams_dialog` 0x576540: 272, 256; centred on
/// the screen's 600 rows).
const TEAMS_X: i32 = 272;
const TEAMS_WIDTH: i32 = 256;
/// Its title's id, its names' first and its flags' first (the original's are −1; the frame's
/// from 200).
const TEAMS_TITLE: u16 = 1;
const TEAMS_NAME: u16 = 2;
const TEAMS_FLAG: u16 = 10;

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

/// Every asset the teams' dialog names (the adapter's batch at the shell's start): its frame's,
/// its texts' fonts and palette, the flags' DEF.
#[must_use]
pub fn teams_names(look: &TeamsLook) -> Vec<String> {
    let mut names = alloc::vec![look.background.clone(), look.frame.clone()];
    for label in [&look.title, &look.team] {
        names.push(label.font.clone());
        names.push(label.font_palette.clone());
    }
    names.extend(look.flag.def.clone());
    names
}

/// What is open over the screen.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Over {
    Nothing,
    /// An item's help popup, while the right button is held.
    Help(u16),
    /// The teams' dialog (`scenario_teams_dialog` 0x576540) or an advanced option's popup (E27 M3
    /// T3a: a town's, a bonus's), while the right button is held.
    Popup(MessageBox),
    /// Begin's message over an empty list (GENRLTXT 530), until its OK.
    Notice(Box<Open>),
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
    /// The description, its split and its bar (E27 M3 T2c).
    description: Description,
    /// The list shows (+0x37D).
    showing: bool,
    /// The list was built (+0x1864): its first showing.
    built: bool,
    /// The list's kind is the random maps' (+0x66 1, E27 M3 T5b: Show Random Maps): the client's
    /// own `random_maps`, which hold none (CLT-Q92), so the shown list is empty while it is.
    random_list: bool,
    /// The sort key recorded (+0x185C) and the order (+0x36C).
    key: u8,
    descending: bool,
    /// The filter's size, 0 for every one (+0x1860).
    filter: u32,
    /// The row the mouse pressed, until its release.
    row: Option<u16>,
    /// The advanced options' name the mouse pressed (a text item: its click at the release; E27 M3
    /// T3c1), until its release.
    name: Option<u16>,
    /// The mouse, as last moved or pressed.
    pointer: [i32; 2],
    /// The original's profile: the last redraw was the partial one (`scenario_select` 0x585500
    /// with the selection unchanged, 0x585A06), and the size mark as the last whole redraw drew
    /// it (its state and frame): CLT-Q138.
    partial: bool,
    drawn_size: (State, u32),
    /// The original's profile: a flag's press restored the flags' strip and no whole redraw
    /// followed (`scenario_team_flags` 0x585300, then `scenario_dialog_redraw` alone: CLT-Q138's
    /// colon; E27 M3 T3c1).
    strip: bool,
    /// The shell's clock (ms) and the last row press's time with its entry in the shown list
    /// (+0x60; none for the screen's opening).
    now: u32,
    last_row: Option<(u32, Option<usize>)>,
    kept: String,
    difficulty: u8,
    /// The original's profile: the difficulty's buttons show the choice (the opening selects it
    /// only over a list with a map, 0x57C556; a press always). The client's always (CLT-Q139 (b)).
    chosen: bool,
    /// The map the game holds (`scenario_to_game` 0x583580: the last one shown), whose players
    /// the original's flags and teams' dialog keep over an empty list; none before one showed.
    game: Option<Entry>,
    /// The advanced options (E27 M3 T3a), none without their texts.
    advanced: Option<Advanced>,
    /// The random map's tab (E27 M3 T5a).
    random: RandomMap,
    /// The selection the last select made (+0x374 as `scenario_select` 0x585500 compares it).
    last_select: Option<usize>,
    background: u8,
    over: Over,
    hovers: Hovers,
    /// The hot seat's (E27 M3 T4a): its players the hot seat's names.
    hot_seat: bool,
    bugs: Bugs,
}

impl Scenario {
    /// The screen from the layout's `dialog`, the list's look `look` and the description's
    /// `description`, its texts `texts` and the maps `entries` (in the list's order, [`sort`]),
    /// the kept map [`KEPT_MAP`] and the difficulty [`OPENING_DIFFICULTY`], in the profile
    /// `bugs`; the list hidden, the flags too until a map shows (the constructor's).
    #[must_use]
    pub fn new(
        mut dialog: Dialog,
        (look, description): (ListLook, DescriptionLook),
        texts: Texts,
        entries: Vec<Entry>,
        bugs: Bugs,
    ) -> Self {
        // The random map's teams 1 and 2 take the mouse 32 rows high in the client's profile
        // (CLT-Q146, E27 M3 T5b).
        RandomMap::heights(&mut dialog, bugs);
        // The advanced options' rows (E27 M3 T3a): each colour's items where the constructor adds
        // them, before the options' headers (or Show Available Scenarios).
        // (An opening anew passes the dialog with them.)
        let advanced = texts.advanced.clone().map(|a| {
            if !dialog.items.iter().any(|i| i.id() == advanced::FLAG) {
                let items = Advanced::items(&a.look);
                let at = dialog
                    .items
                    .iter()
                    .position(|i| i.id() == advanced::HEADERS[0] || i.id() == LIST)
                    .unwrap_or(dialog.items.len());
                dialog.items.splice(at..at, items);
            }
            Advanced::new(a, bugs)
        });
        let mut shown = Shown::default();
        for (id, text) in &texts.labels {
            shown.texts.insert(*id, text.clone());
        }
        for id in ALLIES..ENEMIES + 8 {
            shown.states.insert(id, hidden());
        }
        let bar = Scrollbar::new(look.bar.clone());
        let description = Description::new(description, texts.description_font.clone());
        let chosen = !entries.is_empty();
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
            description,
            showing: false,
            built: false,
            random_list: false,
            key: BUILT_KEY,
            descending: false,
            filter: 0,
            row: None,
            name: None,
            pointer: [0, 0],
            partial: false,
            drawn_size: (State::default(), 0),
            strip: false,
            now: 0,
            last_row: None,
            kept: String::from(KEPT_MAP),
            difficulty: OPENING_DIFFICULTY,
            chosen,
            game: None,
            advanced,
            random: RandomMap::default(),
            last_select: None,
            background: 0,
            over: Over::Nothing,
            hovers: Hovers::default(),
            hot_seat: false,
            bugs,
        };
        Advanced::hide_items(&mut scenario.shown);
        RandomMap::hide_items(&mut scenario.shown);
        scenario.list_items(false);
        scenario.selecting()
    }

    /// The map `left` kept, for the screen's next opening (E27 M3 T2c: `scenario_last_map`
    /// 0x683238 outlives the screen).
    pub fn keep(&mut self, left: &Self) {
        self.kept.clone_from(&left.kept);
    }

    /// The screen opened anew (`scenario_new` 0x4F0490), the menus' last items `hovers` carried
    /// for the menu after it, its background `gamselb0.pcx` until the caller draws
    /// ([`Scenario::with_background`]): the kept map selected, the difficulty
    /// [`OPENING_DIFFICULTY`] again. Its busy cursor leaves the adventure map's arrow in the
    /// original's profile (CLT-Q137 (b): the client's keeps the default one).
    #[must_use]
    pub fn opened(&self, hovers: Hovers) -> Self {
        self.open(hovers, None)
    }

    /// The screen opened anew from the Multiplayer dialog's hot seat (E27 M3 T4a; New Game's
    /// Multiplayer, 0x4EED13: `multiplayer_dialog_run` 0x4F0590, then `scenario_new` 0x4F0490 as
    /// Single's, `game_type` 0x6989F0 3) as [`Scenario::opened`] does, its players the hot seat's
    /// `names` (`hot_seat_names` 0x69CA50: a human record each, `scenario_slots_new` 0x57F740) and
    /// its list the maps two humans may play (`scenario_list_build` 0x582D53: the record's +8,
    /// the colours a human may play, at least 2).
    #[must_use]
    pub fn opened_hot_seat(&self, hovers: Hovers, names: &[String]) -> Self {
        self.open(hovers, Some(names))
    }

    /// The screen opened anew, single or with the hot seat's `names`.
    fn open(&self, hovers: Hovers, names: Option<&[String]>) -> Self {
        let hovers = match self.bugs {
            Bugs::Original => Hovers {
                arrow: Arrow::Adventure,
                ..hovers
            },
            Bugs::Fixed => hovers,
        };
        let mut entries = self.entries.clone();
        if names.is_some() {
            entries.retain(|e| e.humans() >= 2);
        }
        let mut opened = Self::new(
            self.dialog.clone(),
            (self.look.clone(), self.description.look().clone()),
            self.texts.clone(),
            entries,
            self.bugs,
        );
        if let Some(names) = names {
            opened.hot_seat = true;
            opened.advanced = opened.advanced.take().map(|a| a.seated(names));
        }
        opened.kept.clone_from(&self.kept);
        opened.hovers = hovers;
        opened.selecting()
    }

    /// Whether it is the hot seat's (E27 M3 T4a).
    #[must_use]
    pub const fn hot_seat(&self) -> bool {
        self.hot_seat
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

    /// The menus' last items, carried from the menu before it; while Begin's message is open the
    /// default arrow (`message_box_build` 0x4F74AC sets it for its loop, then puts the screen's
    /// back, 0x4F753A; recorded: `scn-begin-filter` begin).
    #[must_use]
    pub const fn hovers(&self) -> Hovers {
        match self.over {
            Over::Notice(_) => Hovers {
                arrow: Arrow::Default,
                ..self.hovers
            },
            _ => self.hovers,
        }
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

    /// Each colour's record of the advanced options, red first (E27 M3 T3a: the town, the hero, the
    /// bonus and the handicap chosen; the game's begin hands them on, T6); none without them.
    #[must_use]
    pub fn records(&self) -> Option<[Record; 8]> {
        self.advanced.as_ref().map(Advanced::records)
    }

    /// The turn's duration the advanced options chose (E27 M3 T3c2: `turndur.txt`'s record, 10
    /// unlimited; no timer runs it yet, `CLT-STUB-11`), none without the options.
    #[must_use]
    pub fn duration(&self) -> Option<u8> {
        self.advanced.as_ref().map(Advanced::duration)
    }

    /// Whether the advanced options show.
    #[must_use]
    pub fn advanced_showing(&self) -> bool {
        self.advanced.as_ref().is_some_and(Advanced::showing)
    }

    /// Whether the random map's tab shows (E27 M3 T5a).
    #[must_use]
    pub const fn random_showing(&self) -> bool {
        self.random.showing()
    }

    /// The random map's choices while a random map is chosen (E27 M3 T5a: what Begin would
    /// generate, T5b), none otherwise.
    #[must_use]
    pub fn random_map(&self) -> Option<Choices> {
        self.random.chosen().then(|| self.random.choices())
    }

    /// The chosen difficulty (0 easy to 4 impossible).
    #[must_use]
    pub const fn difficulty(&self) -> u8 {
        self.difficulty
    }

    /// Where it takes its keys: a box's while Begin's message is open.
    #[must_use]
    pub const fn context(&self) -> Context {
        match self.over {
            Over::Notice(_) => Context::Question,
            _ => Context::Scenario,
        }
    }

    /// Whether the client's stub's notice shows (E27 M3 T5b: Begin with a random map chosen; not
    /// the original's screen, which generates the map).
    #[must_use]
    pub fn stubbed(&self) -> bool {
        matches!(self.over, Over::Notice(_)) && self.random.chosen()
    }

    /// Whether the cursor shows (a help popup hides it; the teams' dialog and a message run with
    /// it).
    #[must_use]
    pub const fn cursor(&self) -> bool {
        !matches!(self.over, Over::Help(_))
    }

    /// The shell's clock is `ms`: a row's double press is timed on it, from the screen's opening
    /// (its first time here) on.
    pub fn clock(&mut self, ms: u32) {
        self.now = ms;
        if self.last_row.is_none() {
            self.last_row = Some((ms, None));
        }
    }

    /// Whether item `id` takes the mouse (a single player's handicap does not: [`Advanced`]).
    fn takes(&self, id: u16) -> bool {
        self.advanced.as_ref().is_none_or(|a| a.takes(id))
    }

    /// The kept map's entry selected (else the first, none in an empty list) and shown.
    fn selecting(mut self) -> Self {
        let kept = self.kept_row();
        self.selected = kept.or_else(|| (!self.listed.is_empty()).then_some(0));
        self.select();
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

    /// Show Available Scenarios (128, `scenario_list_toggle` 0x580A70 with 0) or the random map's
    /// Show Random Maps (335, with 1: `random`; E27 M3 T5b): a shown list hidden, else shown, the
    /// bar's ticks the list's and its value the top, the first time the list built. A change of
    /// the list's kind (+0x66) sorts the list of that kind by the version (`scenario_list_sort`
    /// with 2, 0x580B38: the random maps' built first while empty, the client's none), the top
    /// and the selection its first row, which goes to the game (none over an empty list), the
    /// screen redrawn whole; the same kind keeps its list as it was.
    fn toggle_list(&mut self, random: bool) {
        if self.showing {
            self.showing = false;
            self.list_items(false);
            self.whole();
            return;
        }
        // The random map's tab and the advanced options hidden first (`random_map_hide`
        // 0x581D80, E27 M3 T5a; `scenario_advanced_hide` 0x581B20, T3a), no random map chosen
        // any longer (0x580AAA).
        self.random.hide(&mut self.shown);
        self.random.unchoose();
        if let Some(advanced) = &mut self.advanced {
            advanced.hide(&mut self.shown);
        }
        if random != self.random_list {
            self.random_list = random;
            self.sort_list(BUILT_KEY, false);
            self.give_to_game();
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

    /// The map the screen gives the game (`scenario_to_game` 0x583580): a chosen random map's own
    /// record (E27 M3 T5a), else the entry selected.
    fn shown_entry(&self) -> Option<&Entry> {
        self.random.record().or_else(|| self.entry())
    }

    /// The map whose players the advanced options show: the one the screen gives the game, else
    /// the game's (the last shown; E27 M3 T3a), with its heroes (T3b1).
    fn players(&self) -> Map {
        self.shown_entry()
            .or(self.game.as_ref())
            .map_or_else(Map::default, |e| Map {
                seats: e.seats,
                heroes: e.heroes.clone(),
                local: e.local().and_then(|c| u8::try_from(c).ok()),
            })
    }

    /// Show Advanced Options (129, `scenario_advanced_toggle` 0x581100; E27 M3 T3a): shown, hidden;
    /// else the list hidden, the map given to the game again (the description from its first
    /// line) and the options shown over it; the screen redrawn whole. Whether there are options.
    fn toggle_advanced(&mut self) -> bool {
        let Some(showing) = self.advanced.as_ref().map(Advanced::showing) else {
            return false;
        };
        if showing {
            self.advanced
                .iter_mut()
                .for_each(|a| a.hide(&mut self.shown));
        } else {
            if self.showing {
                self.showing = false;
                self.list_items(false);
            }
            // The random map's tab hidden too (0x58136F; E27 M3 T5a), a random map still chosen.
            self.random.hide(&mut self.shown);
            let map = self.players();
            let (dialog, shown) = (&mut self.dialog, &mut self.shown);
            self.advanced
                .iter_mut()
                .for_each(|a| a.show(&map, dialog, shown));
            self.give_to_game();
        }
        self.whole();
        true
    }

    /// An advanced option's arrow `id` clicked (E27 M3 T3a): its row drawn again; a flag (T3c1:
    /// `scenario_flag_click` 0x58B790, then `scenario_team_flags` with its redraw flag and
    /// `scenario_dialog_redraw`, 0x5870EE: the items not drawn again, so in the original's profile
    /// the strip stays restored, CLT-Q138). Whether it was one.
    fn advanced_click(&mut self, id: u16) -> bool {
        let map = self.players();
        let Some(advanced) = self.advanced.as_mut().filter(|a| a.showing()) else {
            return false;
        };
        if (advanced::FLAG..advanced::FLAG + 8).contains(&id) {
            let c = u8::try_from(id - advanced::FLAG).unwrap_or(0);
            advanced.flag_click(c, &map, &mut self.shown);
            if let Some(entry) = self.shown_entry().or(self.game.as_ref()).cloned() {
                self.team_flags(&entry);
            }
            self.strip = self.bugs == Bugs::Original;
            return true;
        }
        advanced.click(id, &map, &mut self.shown)
    }

    /// Random Map (130, `random_map_toggle` 0x57FEB0, then `random_map_tab_draw` 0x57EF70 and the
    /// redraw; E27 M3 T5a): shown, hidden (`random_map_hide` 0x581D80), a random map still
    /// chosen; else the list and the advanced options hidden, the tab shown, the map given to the
    /// game (`scenario_to_game` 0x583580) and, with no random map chosen yet, the own record built
    /// (in the client's profile at each opening: CLT-Q111 (b)); the screen redrawn whole.
    fn toggle_random(&mut self) {
        if self.random.showing() {
            self.random.hide(&mut self.shown);
        } else {
            if self.showing {
                self.showing = false;
                self.list_items(false);
            }
            if let Some(advanced) = &mut self.advanced {
                advanced.hide(&mut self.shown);
            }
            self.random.show(&mut self.shown);
            self.give_to_game();
            if !self.random.chosen() || self.bugs == Bugs::Fixed {
                self.random_built();
            }
        }
        self.whole();
    }

    /// The random map's own record built (`random_map_info_build` 0x580430; E27 M3 T5a): a random
    /// map chosen, the advanced options' records made again at their next showing
    /// (`scenario_players_init` 0x584C40), the record into the game with its description from
    /// its first line and its flags (`scenario_team_flags` 0x585300) whatever the list (E27 M3
    /// T5b: over the random maps' empty list too), then the panel drawn.
    fn random_built(&mut self) {
        let [name, description] = &self.texts.random;
        self.random.build(name, description);
        if let Some(advanced) = &mut self.advanced {
            advanced.selection_changed();
        }
        if let Some(record) = self.random.record().cloned() {
            self.describe(&record.description);
            self.team_flags(&record);
            self.game = Some(record);
        }
        self.show();
    }

    /// A press `id` of the random map's tab while it shows (`scenario_dialog_click` 0x586DCC to
    /// 0x586F7A; E27 M3 T5a): its choice set, the own record built again after a count's, the
    /// tab's state and the screen redrawn whole. Whether it was one.
    fn random_click(&mut self, id: u16) -> bool {
        if !self.random.showing() {
            return false;
        }
        let Some(pressed) = self.random.click(id) else {
            return false;
        };
        if pressed == Pressed::Counted {
            self.random_built();
        }
        self.random_size_mark();
        self.random.states(&mut self.shown);
        self.whole();
        true
    }

    /// The size mark over a chosen random map (E27 M3 T5a): in the client's profile the chosen
    /// size's (CLT-Q115 (b)); the original's keeps what it showed (only a select sets it).
    fn random_size_mark(&mut self) {
        if self.bugs == Bugs::Fixed && self.random.chosen() {
            let frame = size_frame(self.random.choices().size);
            self.shown.states.insert(SIZE, State::default());
            self.shown.frames.insert(SIZE, frame);
        }
    }

    /// The full list sorted by `key` (`scenario_list_sort` 0x585050), the shown list rebuilt in
    /// its order, the top and the selection its first row.
    fn sort_list(&mut self, key: u8, descending: bool) {
        // The random maps' list (E27 M3 T5b) is the client's own, empty: nothing to sort.
        if !self.random_list {
            sort_by(&mut self.entries, key, descending);
        }
        self.relist();
        self.top = 0;
        self.selected = (!self.listed.is_empty()).then_some(0);
        self.bar.set_value(0);
    }

    /// The shown list rebuilt from the full one: the filter's size, every one for 0; none while the
    /// list's kind is the random maps' (E27 M3 T5b: the client's own, empty).
    fn relist(&mut self) {
        if self.random_list {
            self.listed.clear();
            return;
        }
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
        // The sort gives its first row's map to the game (`scenario_to_game` from 0x5852A1: the
        // description set, E27 M3 T2c; the size mark as it was) and redraws whole (0x5852BF)
        // before the click selects that row, which it already is: the select's partial redraw
        // (CLT-Q138).
        self.give_to_game();
        self.whole();
        self.select();
        self.partial = true;
    }

    /// A filter's press (`scenario_list_filter` 0x585D90): the shown list of the maps of `size`
    /// (every one for 0), the top and the selection its first row, the bar's ticks; the size mark
    /// set to the filter's size first (0x585EFB: frame 4 for every one), which a selection
    /// replaces (an empty list keeps it in the original's profile).
    fn filter_by(&mut self, size: u32) {
        self.filter = size;
        self.relist();
        self.top = 0;
        self.selected = (!self.listed.is_empty()).then_some(0);
        self.bar.reset(self.ticks());
        self.shown.frames.insert(SIZE, size_frame(size));
        self.select();
        self.whole();
    }

    /// The bar moved the top to `value` (`scenario_list_scrolled` 0x57C990).
    fn scrolled(&mut self, value: Option<i32>) {
        if let Some(value) = value {
            self.top = usize::try_from(value).unwrap_or(0);
            self.select();
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
            // The description's bar's keys alone (the dialog's procedure's table: nothing).
            Scroll::Left | Scroll::Right => return,
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
        self.select();
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
        self.select();
        if same {
            self.partial = true;
        } else {
            self.whole();
        }
    }

    /// The screen redrawn whole: the size mark as it is now is the one drawn.
    fn whole(&mut self) {
        self.partial = false;
        self.strip = false;
        let frame = self.shown.frames.get(&SIZE).copied().unwrap_or(0);
        self.drawn_size = (self.shown.state(SIZE), frame);
        self.description.drew();
    }

    /// Whether the mouse holds the advanced options' duration's bar (E27 M3 T3c2).
    fn duration_held(&self) -> bool {
        self.advanced.as_ref().is_some_and(Advanced::bar_held)
    }

    /// The duration's bar drawn after an input (pressed, moved or let go); when it reported a
    /// value (`scenario_duration_scrolled` 0x57C7F0) the screen redrawn whole (E27 M3 T3c2).
    fn duration_scrolled(&mut self, changed: bool) -> Did {
        if changed {
            self.whole();
        }
        Did::CHANGED
    }

    /// The description's label shows its lines as they are now (its bar moved: E27 M3 T2c).
    fn described(&mut self) -> Did {
        self.shown
            .texts
            .insert(DESCRIPTION, self.description.text());
        Did::CHANGED
    }

    /// Whether the screen shows the partial redraw (the original's profile, the list showing).
    fn stale(&self) -> bool {
        self.partial && self.showing && self.bugs == Bugs::Original
    }

    /// What the information panel shows of the entry selected (`scenario_select` 0x585500, its
    /// draw `scenario_info_draw` 0x5840F0, the flags `scenario_team_flags` 0x585300), and the
    /// chosen difficulty's button selected (not before a press in the original's profile when the
    /// screen opened with no map).
    fn show(&mut self) {
        let chosen = self.chosen || self.bugs == Bugs::Fixed;
        for (k, &id) in DIFFICULTIES.iter().enumerate() {
            let state = self.shown.state(id);
            let selected = chosen && usize::from(self.difficulty) == k;
            self.shown.states.insert(id, State { selected, ..state });
        }
        let Some(entry) = self.shown_entry().cloned() else {
            self.show_empty();
            return;
        };
        // Over an empty shown list the original's draws nothing of a map, a chosen random map's
        // neither (`scenario_info_draw` 0x584128 tests the list, not +0x37F; E27 M3 T5b: Show
        // Random Maps, then the tab again): the description and the flags keep what they showed.
        // The client's shows the random map's record (CLT-Q111 (b)).
        if self.listed.is_empty() && self.bugs == Bugs::Original {
            self.blank_panel();
            return;
        }
        let random = self.random.chosen();
        self.game = Some(entry.clone());
        let rating = text_at(&self.texts.ratings, usize::from(self.difficulty));
        let shown = &mut self.shown;
        shown.texts.insert(RATING, rating);
        for id in ALLIES..ENEMIES + 8 {
            shown.states.insert(id, hidden());
        }
        let t = &self.texts;
        let name = if entry.name.is_empty() {
            t.unnamed.clone()
        } else {
            entry.name.clone()
        };
        shown.texts.insert(NAME, name);
        shown.texts.insert(DESCRIPTION, self.description.text());
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
        for id in [VICTORY_MARK, LOSS_MARK] {
            shown.states.insert(id, State::default());
        }
        // The size mark is the select's (`scenario_select` 0x585500): a chosen random map's
        // draw leaves it (E27 M3 T5a, CLT-Q115).
        if !random {
            shown.states.insert(SIZE, State::default());
            shown.frames.insert(SIZE, size_frame(entry.size));
        }
        // The marks: the goal's frame (0 to 10), 11 for none; the loss's (0 to 2), 3 for none.
        let victory_mark = entry.victory.map_or(11, u32::from);
        shown.frames.insert(VICTORY_MARK, victory_mark);
        shown
            .frames
            .insert(LOSS_MARK, entry.loss.map_or(3, u32::from));
        self.team_flags(&entry);
        self.random_size_mark();
    }

    /// The flags of `entry`'s colours (`scenario_team_flags` 0x585300): each colour that plays, in
    /// colour order, of the local colour's team the next allies' flag (the local one's own first),
    /// any other the next enemies'. The local colour: the advanced options' local record's (the
    /// single player's, the hot seat's first) while its rows are as made (a flag's press moves it:
    /// E27 M3 T3c1), else the map's for it (the hot seat's first player's: E27 M3 T4a).
    fn team_flags(&mut self, entry: &Entry) {
        let local = self
            .advanced
            .as_ref()
            .and_then(Advanced::local_colour)
            .map_or_else(
                || {
                    if self.hot_seat {
                        entry.first_human()
                    } else {
                        entry.local()
                    }
                },
                |c| c.map(usize::from),
            );
        let team = local.map(|c| entry.teams[c & 7]);
        let (mut allies, mut enemies) = (ALLIES, ENEMIES);
        for id in ALLIES..ENEMIES + 8 {
            self.shown.states.insert(id, hidden());
        }
        for colour in (0..8u8).filter(|&c| entry.seats[usize::from(c)].plays()) {
            let slot = if Some(entry.teams[usize::from(colour)]) == team {
                &mut allies
            } else {
                &mut enemies
            };
            self.shown.states.insert(*slot, State::default());
            self.shown.frames.insert(*slot, u32::from(colour));
            *slot += 1;
        }
    }

    /// A select (`scenario_select` 0x585500, whatever the change: `scenario_to_game` 0x583580):
    /// the map kept for the next opening (`scenario_last_map` 0x683238), the panel shown, the
    /// description set anew from its first line (E27 M3 T2c, `scrollable_text_set` 0x5BA6E0;
    /// empty over an empty list, the kept map as it was). A difficulty's press and the list's
    /// showing select nothing.
    fn select(&mut self) {
        if let Some(file) = self.entry().map(|e| e.file.clone()) {
            self.kept = file;
        }
        // A changed selection makes the advanced options' rows again at their next showing
        // (+0x1854; E27 M3 T3a).
        if self.selected != self.last_select {
            self.last_select = self.selected;
            if let Some(advanced) = &mut self.advanced {
                advanced.selection_changed();
            }
        }
        // The select of none (-1, an emptied list) empties the description (0x585639; E27 M3 T5b:
        // the game's map stays as it was).
        if self.entry().is_none() && !self.random.chosen() {
            self.describe("");
        }
        self.give_to_game();
        self.show();
    }

    /// The selected map (a chosen random map's own record: E27 M3 T5a) given to the game
    /// (`scenario_to_game` 0x583580): the description set anew (`scrollable_text_set` 0x5BA6E0);
    /// nothing over an empty shown list (0x583593; E27 M3 T5b: a chosen random map's record
    /// neither: its build sets the description itself).
    fn give_to_game(&mut self) {
        if self.listed.is_empty() {
            return;
        }
        let text = self
            .shown_entry()
            .map(|e| e.description.clone())
            .unwrap_or_default();
        self.describe(&text);
    }

    /// The description set to `text` from its first line, its label showing it.
    fn describe(&mut self, text: &str) {
        self.description.set(text);
        self.shown
            .texts
            .insert(DESCRIPTION, self.description.text());
    }

    /// The panel over an empty list (E27 M3 T2b, design §21.2): the select does nothing there
    /// (0x58553C) and `scenario_info_draw` 0x5840F0 draws nothing of a map (0x584128), the rating
    /// neither. The original's description, size mark and flags keep what they showed (a select of
    /// none empties the description; a filter's size, set first, or the constructor's S; the last
    /// map's flags; E27 M3 T5b: Show Random Maps keeps a random map's description and flags); the
    /// client's shows nothing of a map: no description (T5b), the size mark's frame 4 (CLT-Q115
    /// (b)) and no flags (CLT-Q139 (b)).
    fn show_empty(&mut self) {
        self.blank_panel();
        if self.bugs == Bugs::Fixed {
            self.describe("");
            let shown = &mut self.shown;
            shown.states.insert(SIZE, State::default());
            shown.frames.insert(SIZE, size_frame(0));
            for id in ALLIES..ENEMIES + 8 {
                shown.states.insert(id, hidden());
            }
        }
    }

    /// What `scenario_info_draw` 0x5840F0 draws over an empty list: nothing (0x584128): no name,
    /// texts, marks, map's difficulty nor rating.
    fn blank_panel(&mut self) {
        let shown = &mut self.shown;
        for id in [NAME, VICTORY, LOSS, MAP_DIFFICULTY, RATING] {
            shown.texts.insert(id, String::new());
        }
        for id in [VICTORY_MARK, LOSS_MARK] {
            shown.states.insert(id, hidden());
        }
    }

    /// A difficulty's press (`scenario_difficulty_click` 0x586BF3): the difficulty `k` chosen
    /// (game +0x1F6D8, `scenario_last_difficulty`), its button selected, the screen redrawn whole
    /// (the dialog's redraw, then `scenario_dialog_redraw`).
    fn choose(&mut self, k: u8) {
        self.difficulty = k;
        self.chosen = true;
        self.show();
        self.whole();
    }

    /// Begin (186, `scenario_begin` 0x58BCE0): with a random map chosen the client's stub's
    /// notice, whatever the list (E27 M3 T5b; CLT-Q92, decision 13: the original generates the
    /// map first, 0x58BD00, `random_map_generate` 0x5860E0); over an empty list GENRLTXT 530 in
    /// an OK box, the original's only once a map of a known format was shown (its check of the
    /// game's map comes first: with no map at all Begin does nothing, CLT-Q114 (b)); over a map
    /// the advanced options' refusal (E27 M3 T4b, `scenario_begin_tail` 0x58C11F: GENRLTXT 531,
    /// the hot seat's 83; [`Advanced::refusal`]), else nothing yet (the game's begin: T6).
    fn begin(&mut self) -> bool {
        let message = if self.random.chosen() {
            self.texts.random_stub.clone()
        } else if self.entry().is_some() {
            let map = self.players();
            match self.advanced.as_ref().and_then(|a| a.refusal(&map)) {
                Some(Refusal::Unplaced) => self.texts.unplaced.clone(),
                Some(Refusal::TooFew) => self.texts.too_few.clone(),
                None => None,
            }
        } else {
            let checked =
                self.bugs == Bugs::Fixed || self.game.as_ref().is_some_and(|g| g.format.is_some());
            self.texts.no_maps.clone().filter(|_| checked)
        };
        match message {
            Some(message) => {
                self.over = Over::Notice(Box::new(Open::new(message)));
                true
            }
            None => false,
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
            // The description's lines as last drawn (E27 M3 T2c: the select reset them, undrawn).
            let drawn = String::from(self.description.drawn_text());
            rest.texts.insert(DESCRIPTION, drawn);
            let mut draws = self.dialog.draws(&rest, players);
            draws.extend(self.dialog.draws(&again, players));
            draws
        } else {
            self.dialog.draws(&self.shown, players)
        };
        let stale = self.stale();
        draws.extend(self.description.bar_draws(self.dialog.place, stale));
        if self.showing {
            draws.extend(self.bar.draws(self.dialog.place));
            draws.extend(self.row_draws());
        }
        if let Some(advanced) = &self.advanced {
            let map = self.players();
            draws.extend(advanced.draws(&map, &self.dialog, &self.shown, players));
            // The name's edit drawn as the edit where the dialog draws its items too (E27 M3 T3c1).
            advanced.place_edit(&mut draws);
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
        let strip = self.strip && self.bugs == Bugs::Original;
        if !self.stale() && !strip {
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

    /// What the original's opening of the name's edit pasted (CLT-Q143, E27 M3 T3c1): the bar's
    /// piece at the bar's corner, the edit over it, until the row is drawn again: its rectangle
    /// and the screen's draws with the piece and the edit after them.
    #[must_use]
    pub fn piece_draws(&self, players: &str) -> Option<([i32; 4], Vec<Draw>)> {
        let (rect, over) = self
            .advanced
            .as_ref()?
            .piece(&self.dialog, &self.shown, players)?;
        let mut draws = self.draws(players);
        draws.extend(over);
        Some((rect, draws))
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
            // A mark: its DEF's frame, none for a frame of none (or a look without its DEF).
            let mark = |def: &Def, frame: Option<u32>| {
                def.def
                    .clone()
                    .zip(frame)
                    .map(|(name, frame)| Draw::Button {
                        def: name,
                        frame,
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
            Over::Popup(m) => m.dialog.draws(&m.shown, players),
            Over::Notice(open) => open.message.dialog.draws(&open.message.shown, players),
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
        if self.description.held() {
            self.description.moved(self.inside(point));
            return self.described();
        }
        // The duration's bar's own loop (E27 M3 T3c2).
        if self.duration_held() {
            let inside = self.inside(point);
            let changed = self
                .advanced
                .as_mut()
                .is_some_and(|a| a.bar_moved(inside, &mut self.shown));
            return self.duration_scrolled(changed);
        }
        match &mut self.over {
            Over::Nothing => {
                // The flags' hover (E27 M3 T3c1), unless a button's own loop holds the mouse.
                let hovered = !self.input.holds_button() && self.hover(point);
                let dialog = mouse_dialog(&self.dialog, &self.shown);
                let mut did = Did::held(self.input.moved(&dialog, &mut self.shown, point));
                did.changed |= hovered;
                did
            }
            Over::Notice(open) => Did::of_box(open.moved(point)),
            Over::Help(_) | Over::Popup(_) => Did::default(),
        }
    }

    /// The mouse's move to `point` as the screen's procedure takes it (`scenario_dialog_proc`
    /// 0x5881B2: the item there, `dialog_item_id_at` 0x5FF3B0): the advanced options' flags'
    /// hover. Whether it changed anything.
    fn hover(&mut self, point: [i32; 2]) -> bool {
        let id = self.item_at(point);
        self.advanced
            .as_mut()
            .is_some_and(|a| a.hover(id, &mut self.shown))
    }

    /// The left button pressed at `point`: the list's bar or a row while the list shows, else the
    /// shared input's buttons (a hidden item takes no press).
    pub fn press(&mut self, point: [i32; 2]) -> Did {
        self.pointer = point;
        match &mut self.over {
            Over::Nothing => {}
            Over::Help(_) => return self.close_help(),
            // The dialog's procedure (0x575430) takes the right release alone.
            Over::Popup(_) => return Did::default(),
            Over::Notice(open) => return Did::box_press(open.press(point)),
        }
        let inside = self.inside(point);
        if self.showing && self.row.is_none() && self.bar.contains(inside) {
            let value = self.bar.press(inside);
            self.scrolled(value);
            return Did::CHANGED;
        }
        // The duration's bar (338, E27 M3 T3c2): its arrow drawn pressed, a drag or a page moved.
        if let Some(advanced) = self.advanced.as_mut().filter(|a| a.on_bar(inside)) {
            let changed = advanced.bar_press(inside, &mut self.shown);
            return self.duration_scrolled(changed);
        }
        if self.row.is_none() && self.description.press(inside) {
            return self.described();
        }
        let rows = ROW..ROW.saturating_add(self.look.rows);
        match self.item_at(point) {
            Some(id) if rows.contains(&id) => {
                self.row = Some(id);
                return Did::default();
            }
            // A name of the advanced options, a text item as a row (E27 M3 T3c1).
            Some(id) if (advanced::PLAYER..advanced::PLAYER + 8).contains(&id) => {
                self.name = Some(id);
                return Did::default();
            }
            _ => {}
        }
        let dialog = mouse_dialog(&self.dialog, &self.shown);
        let hidden = dialog
            .item_at(point)
            .is_some_and(|item| self.shown.state(item.id()).hidden || !self.takes(item.id()));
        if hidden {
            return Did::default();
        }
        Did::held(self.input.press(&dialog, &mut self.shown, point))
    }

    /// The left button released: the bar let go, a row clicked (in the original's profile
    /// wherever the mouse is, in the client's over the row), a button clicked.
    pub fn release(&mut self) -> Did {
        if self.bar.held() {
            let value = self.bar.release();
            self.scrolled(value);
            return Did::CHANGED;
        }
        if self.duration_held() {
            let changed = self
                .advanced
                .as_mut()
                .is_some_and(|a| a.bar_release(&mut self.shown));
            return self.duration_scrolled(changed);
        }
        if self.description.held() {
            self.description.release();
            return self.described();
        }
        if let Some(row) = self.row.take() {
            let over = self.item_at(self.pointer) == Some(row);
            if self.bugs == Bugs::Original || over {
                self.row_click(row);
            }
            return Did::CHANGED;
        }
        // A name's click at the release, wherever the mouse is in the original's profile, over it
        // in the client's (CLT-Q43 (2)'s rule; `scenario_name_click` 0x58B620).
        if let Some(name) = self.name.take() {
            let over = self.item_at(self.pointer) == Some(name);
            let c = u8::try_from(name - advanced::PLAYER).unwrap_or(0);
            let opened = (self.bugs == Bugs::Original || over)
                && self
                    .advanced
                    .as_mut()
                    .is_some_and(|a| a.name_click(c, &mut self.shown));
            return Did {
                changed: opened,
                ..Did::default()
            };
        }
        match &mut self.over {
            Over::Nothing => {
                let did = self.input.release(&mut self.shown);
                self.clicked(did.changed, did.clicked)
            }
            Over::Help(_) => self.close_help(),
            Over::Popup(_) => Did::default(),
            Over::Notice(open) => {
                let outcome = open.release();
                self.answered(outcome)
            }
        }
    }

    /// The right button pressed at `point` (`scenario_dialog_right_click` 0x5822D0): on the
    /// flags' strip the teams' dialog, on the list's bar or another item its help, unless the
    /// mouse holds a button or a bar; anywhere else but a row the screen redrawn whole.
    pub fn right(&mut self, point: [i32; 2]) -> Did {
        if self.over != Over::Nothing
            || self.input.holds_button()
            || self.bar.held()
            || self.description.held()
            || self.duration_held()
        {
            return Did::default();
        }
        let inside = self.inside(point);
        let on_bar = self.showing && self.bar.contains(inside);
        let on_duration = self.advanced.as_ref().is_some_and(|a| a.on_bar(inside));
        // The item there (`dialog_item_at` 0x5FF3E0: shown and enabled, not disabled; a single
        // player's handicap is not enabled, E27 M3 T3c2, nor is the random map's tab's count
        // above its cap, T5b: the panel under it has no hint), the bars' own.
        let id = if on_bar {
            Some(BAR)
        } else if on_duration {
            Some(advanced::BAR)
        } else {
            self.item_at(point)
                .filter(|&id| self.takes(id) && !self.shown.state(id).disabled)
        };
        if id == Some(TEAMS) {
            return self.teams_dialog().map_or_else(Did::default, |m| {
                self.over = Over::Popup(m);
                Did::CHANGED
            });
        }
        // A town's, a hero's or a bonus's popup of the advanced options (E27 M3 T3a, T3b1).
        let map = self.players();
        let popup = id.and_then(|id| self.advanced.as_ref()?.popup(id, &map));
        if let Some(m) = popup {
            self.over = Over::Popup(m);
            return Did::CHANGED;
        }
        match id.filter(|id| self.texts.help.contains_key(id)) {
            Some(id) => {
                self.over = Over::Help(id);
                Did::CHANGED
            }
            // No popup: the screen redrawn whole at once, but on a row (E27 M3 T2c: the
            // description's lines and its bar, design §20.1).
            None if id.is_some_and(|id| (ROW..ROW + self.look.rows).contains(&id)) => {
                Did::default()
            }
            None => {
                self.whole();
                Did::CHANGED
            }
        }
    }

    /// The teams' dialog of the entry selected (`scenario_teams_dialog` 0x576540, its teams
    /// 0x576930): the colours that play grouped by team, the teams in the order of their first
    /// colour, each team's name and its colours' flags in colour order; the dialog
    /// [`TEAMS_WIDTH`] wide at x [`TEAMS_X`], ((50 × teams + 56) / 64 + 1) × 64 high, centred on
    /// the screen's rows, casting a shadow (its flags 0x12), its frame in the DEF's own colours.
    /// None without the dialog's texts. Over an empty list (E27 M3 T2b, design §21.2) the
    /// original's shows the game's players, the last map's (with none shown yet the reset
    /// game's: every colour in one team), the client's none (CLT-Q139 (b)).
    fn teams_dialog(&self) -> Option<MessageBox> {
        let teams = self.texts.teams.as_ref()?;
        let (plays, of) = match (self.entry(), self.bugs, &self.game) {
            (Some(entry), _, _) | (None, Bugs::Original, Some(entry)) => {
                (entry.seats.map(Seat::plays), entry.teams)
            }
            (None, Bugs::Original, None) => ([true; 8], [0; 8]),
            (None, Bugs::Fixed, _) => return None,
        };
        let mut groups: Vec<(u8, Vec<u8>)> = Vec::new();
        for colour in (0..8u8).filter(|&c| plays[usize::from(c)]) {
            let team = of[usize::from(colour)];
            match groups.iter_mut().find(|(t, _)| *t == team) {
                Some((_, colours)) => colours.push(colour),
                None => groups.push((team, alloc::vec![colour])),
            }
        }
        let look = &teams.look;
        let count = i32::try_from(groups.len()).unwrap_or(0);
        let height = ((look.step * count + 56) / 64 + 1) * 64;
        let mut items = crate::message::frame(
            (&look.background, &look.frame),
            [i64::from(TEAMS_WIDTH), i64::from(height)],
        );
        let mut shown = Shown::default();
        items.push(h3_ui::ui::Item::Label(Label {
            id: TEAMS_TITLE,
            ..look.title.clone()
        }));
        shown.texts.insert(TEAMS_TITLE, teams.title.clone());
        let mut flag = TEAMS_FLAG;
        for ((k, (_, colours)), name) in (0i32..).zip(&groups).zip(TEAMS_NAME..) {
            let dy = look.step * k;
            let [x, y] = look.team.at;
            items.push(h3_ui::ui::Item::Label(Label {
                id: name,
                at: [x, y + dy],
                ..look.team.clone()
            }));
            shown
                .texts
                .insert(name, text_at(&teams.names, usize::from(name - TEAMS_NAME)));
            let n = i32::try_from(colours.len()).unwrap_or(0);
            let mut x = (TEAMS_WIDTH - (look.flag_step * n - 3)) / 2;
            for &colour in colours {
                items.push(h3_ui::ui::Item::Def(Def {
                    id: flag,
                    at: [x, look.flag.at[1] + dy],
                    ..look.flag.clone()
                }));
                shown.frames.insert(flag, u32::from(colour));
                flag += 1;
                x += look.flag_step;
            }
        }
        Some(MessageBox {
            dialog: Dialog {
                place: [TEAMS_X, (600 - height) / 2],
                shadow: Some([TEAMS_WIDTH, height]),
                items,
            },
            shown,
        })
    }

    /// The right button released: the popup closed; the bar or a button the mouse holds released
    /// as by the left button (the bar's loop ends at either release, 0x596607).
    pub fn right_release(&mut self) -> Did {
        if self.bar.held() || self.description.held() || self.duration_held() {
            return self.release();
        }
        match &mut self.over {
            Over::Help(_) | Over::Popup(_) => self.close_help(),
            // A box's button the mouse holds goes up as at a left release (CLT-Q72 (a)).
            Over::Notice(open) if open.holds_mouse() => {
                let outcome = open.release();
                self.answered(outcome)
            }
            Over::Notice(_) => Did::default(),
            Over::Nothing => {
                let did = self.input.right_release(&mut self.shown);
                self.clicked(did.changed, did.clicked)
            }
        }
    }

    /// A key down: as the original reads it (`scan`, the name's edit's in the original's profile),
    /// the key table's `command` for it ([`Scenario::contexts`]), `by` its name (whose release
    /// clicks); the list's keys while it shows. None while the bar is held (its loop takes no
    /// key). The name's edit while it shows first (E27 M3 T3c1).
    pub fn key(&mut self, scan: Option<Scan>, command: Option<Command>, by: &str) -> Did {
        if self.bar.held() || self.description.held() || self.duration_held() {
            return Did::default();
        }
        if let Over::Notice(open) = &mut self.over {
            return match command {
                Some(Command::Message(key)) => Did::box_press(open.key(key, by)),
                _ => Did::default(),
            };
        }
        if self.over == Over::Nothing
            && let Some(taken) = self.edit_key(scan, command)
        {
            return Did {
                changed: taken,
                ..Did::default()
            };
        }
        match (&self.over, command) {
            (Over::Nothing, Some(Command::Dialog(id))) if self.has(id) => {
                Did::held(self.input.key(&mut self.shown, id, by))
            }
            (Over::Nothing, Some(Command::List(scroll))) => {
                let list = !matches!(scroll, Scroll::Left | Scroll::Right);
                // The description's bar steps at the key's release (E27 M3 T2c); in the client's
                // profile it takes no list key while the list shows, Left and Right always
                // (CLT-Q140 (b)).
                if !(list && self.showing && self.bugs == Bugs::Fixed) {
                    self.description.key_down(scroll, by);
                }
                if self.showing {
                    self.scroll(scroll);
                }
                if self.showing && list {
                    Did::CHANGED
                } else {
                    Did::default()
                }
            }
            _ => Did::default(),
        }
    }

    /// A key down to the advanced options' name's edit while it shows (none: not the edit's, its
    /// dialog's then): in the original's profile by `scan` (0x57CDC0: Enter closes it, Escape and
    /// Tab are the dialog's, any other key the edit's); in the client's by `command`, the edit's
    /// own keys, Begin's closing it as the original's Enter, Back's the dialog's, any other key
    /// taken by the edit (its character typed through [`Scenario::typed`]). Some(whether it
    /// changed) when the edit took it.
    fn edit_key(&mut self, scan: Option<Scan>, command: Option<Command>) -> Option<bool> {
        let advanced = self.advanced.as_mut().filter(|a| a.editing())?;
        let shown = &mut self.shown;
        match self.bugs {
            Bugs::Original => {
                let scan = scan?;
                advanced.edit_key(scan, shown).then_some(true)
            }
            Bugs::Fixed => match command {
                Some(Command::Edit(caret)) => Some(advanced.edit_caret(caret)),
                Some(Command::Dialog(BEGIN)) => Some(advanced.close_edit(shown)),
                Some(Command::Dialog(BACK)) => None,
                _ => Some(false),
            },
        }
    }

    /// Whether the name's edit takes the window's text input (E27 M3 T3c1, the client's profile):
    /// while it shows, nothing over the screen.
    #[must_use]
    pub fn typing(&self) -> bool {
        self.bugs == Bugs::Fixed
            && self.over == Over::Nothing
            && self.advanced.as_ref().is_some_and(Advanced::editing)
    }

    /// Where a key down is asked for its command, in order: an edit's own keys while the name's
    /// edit types ([`Scenario::typing`]), then [`Scenario::context`].
    #[must_use]
    pub fn contexts(&self) -> Vec<Context> {
        if self.typing() {
            alloc::vec![Context::Edit, self.context()]
        } else {
            alloc::vec![self.context()]
        }
    }

    /// A character the window's text input typed (the client's profile): to the name's edit while
    /// it types.
    pub fn typed(&mut self, c: char) -> Did {
        if !self.typing() {
            return Did::default();
        }
        let changed = self.advanced.as_mut().is_some_and(|a| a.edit_typed(c));
        Did {
            changed,
            ..Did::default()
        }
    }

    /// The key `by` released.
    pub fn key_up(&mut self, by: &str) -> Did {
        // The description's bar first (an item of the dialog: the key's release reaches the items
        // before the screen's procedure), with nothing over the screen (E27 M3 T2c).
        let free = self.over == Over::Nothing;
        let stepped = self.description.key_up(by, free) && self.described().changed;
        match &mut self.over {
            Over::Nothing => {
                let did = self.input.key_up(&mut self.shown, by);
                let mut did = self.clicked(did.changed, did.clicked);
                did.changed |= stepped;
                did
            }
            Over::Help(_) | Over::Popup(_) => {
                let did = self.input.let_go(&mut self.shown, Some(by));
                Did {
                    changed: did.changed,
                    ..Did::default()
                }
            }
            Over::Notice(open) => {
                let outcome = open.key_up(by);
                self.answered(outcome)
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

    /// The help popup or the teams' dialog closed (0x58AE80, the dialog's redraw, then
    /// `scenario_dialog_redraw`), the screen redrawn whole (§3.8).
    fn close_help(&mut self) -> Did {
        self.over = Over::Nothing;
        self.whole();
        Did::CHANGED
    }

    /// Begin's message after an input: its OK closes it, the box taking its pixels back (the
    /// screen as it was: in the original's profile a flags' strip a flag's press left stale stays
    /// so, E27 M3 T4b, `adv-hotseat-two` closed-1).
    fn answered(&mut self, outcome: Outcome) -> Did {
        match outcome {
            Outcome::Nothing => Did::default(),
            Outcome::Changed => Did::CHANGED,
            Outcome::Closed(_) => {
                self.over = Over::Nothing;
                Did::CHANGED
            }
        }
    }

    /// A hold ended (`changed`: a button went up) with the click `clicked`, if any: its command
    /// (`scenario_dialog_click` 0x5865B0): Back ends the screen (0x7801), Show Available
    /// Scenarios toggles the list, a filter and a column act on it, a difficulty is chosen, Begin
    /// shows its message over an empty list; the others' commands are later tasks' (the module's
    /// notes).
    fn clicked(&mut self, changed: bool, clicked: Option<u16>) -> Did {
        let mut did = Did {
            changed,
            sample: false,
            ended: (clicked == Some(BACK)).then_some(Ended::Scenario(false)),
        };
        let Some(id) = clicked else {
            return did;
        };
        if id == LIST || (id == random::SHOW && self.random.showing()) {
            self.toggle_list(id != LIST);
            did.changed = true;
        } else if id == ADVANCED {
            did.changed |= self.toggle_advanced();
        } else if id == RANDOM {
            self.toggle_random();
            did.changed = true;
        } else if self.random_click(id) || self.advanced_click(id) {
            did.changed = true;
        } else if let Some(&(_, size)) = FILTERS.iter().find(|&&(f, _)| f == id) {
            self.filter_by(size);
            did.changed = true;
        } else if let Some(&(_, key)) = COLUMNS.iter().find(|&&(c, _)| c == id) {
            self.sort_click(key);
            did.changed = true;
        } else if let Some(k) = DIFFICULTIES.iter().position(|&d| d == id) {
            self.choose(u8::try_from(k).unwrap_or(OPENING_DIFFICULTY));
            did.changed = true;
        } else if id == BEGIN {
            did.changed |= self.begin();
        }
        did
    }
}

/// `dialog` as the mouse finds its items: without the advanced options' and the random map's
/// tab's hidden items (E27 M3 T3a, T5a: their panels, headers, rows and buttons lie over the
/// list's columns, filters and rows and over each other; the original's `dialog_item_at`
/// 0x5FF3E0 passes over a hidden item, the shared input does not).
fn mouse_dialog(dialog: &Dialog, shown: &Shown) -> Dialog {
    let mut dialog = dialog.clone();
    dialog.items.retain(|i| {
        let id = i.id();
        !((Advanced::owns(id) || RandomMap::owns(id)) && shown.state(id).hidden)
    });
    dialog
}

/// Whether the partial redraw (`scenario_select` 0x585500 with the selection unchanged, from
/// 0x585A06) draws item `id` again: the list's panel, its title, the filters and the columns, and
/// what the redraw draws itself (the client's ids but the right-hand panel, the constructor's
/// labels and the description's lines, items of the scrolling text: E27 M3 T2c).
fn redrawn(id: u16) -> bool {
    let own = ((NAME..=RATING).contains(&id) && id != DESCRIPTION) || id == SIZES;
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
