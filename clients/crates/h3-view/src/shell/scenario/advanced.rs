//! The scenario screen's advanced options of a single player (E27 M3 T3a;
//! `docs/epics/E27-design.md` §3.12 to §3.15, §23) and of the hot seat (T4a, §28): Show Advanced
//! Options (129, `scenario_advanced_toggle` 0x581100) shows `AdvOptBk.pcx` (102), the headers,
//! the turn's duration and a row of each colour that plays, a second press hides them
//! (`scenario_advanced_hide` 0x581B20); the list's showing hides them too.
//!
//! - **The records** (§3.12, `H3ScenarioPlayer`): the single player's (the local slot's) holds its
//!   colour; every other colour plays by its computer record. At the showing after a select that
//!   changed the selection (+0x1854) every record's colour, town and hero are cleared and the
//!   local colour set again (`scenario_players_init` 0x584C40); the bonus and the handicap stay.
//!   The showing gives the map to the game again (`scenario_to_game` 0x583580: the description
//!   from its first line).
//! - **A row** (`scenario_player_row_draw` 0x58D510, after the dialog's items): the flag, the
//!   colour's bar `adop%cpnl.pcx`, who may play and the handicap, then the town, the hero and the
//!   bonus, each a picture and a tiny text with their arrows, and the name. The arrows show by the
//!   record: the town's when it may change it (`scenario_town_choosable` 0x58CEB0), the hero's when
//!   its hero may (`scenario_hero_choosable` 0x58CFB0), the bonus's for the local colour and a
//!   computer's whose town shows a town or may change. A bonus of a resource under a shown Random
//!   town is made Random (the draw's own write, 0x58DDAB).
//! - **The town's arrows** (215 to 230; `scenario_dialog_click` 0x586710, 0x586894): from the shown
//!   town to the next (or the previous) allowed faction, past either end to Random and from Random
//!   to the far end; the hero Random; the row drawn again.
//! - **The bonus's arrows** (247 to 262; 0x5873C7, 0x58753C): left one back, right one on, modulo
//!   4 (artifact, gold, resource, Random); the artifact passed over for a record with no hero to
//!   take it (its hero Random and the colour with neither the random-hero flag, the generating main
//!   town nor a main hero), the resource under a shown Random town (left to gold, right to Random).
//! - **The hero** (T3b1; design §24): with neither the generating main town nor the random-hero
//!   flag the map's main hero (its portrait, CLT-Q141), else the record's pick through the map's
//!   customised heroes (`scenario_shown_hero` 0x58D0E0); its small portrait (`hero_table`
//!   0x679DD0's +0x30) and its name (`scenario_shown_hero_name` 0x58D1F0: the main hero's own
//!   name of the map, a customised hero's, the type's; CLT-Q142).
//! - **The hero's arrows** (231 to 246, a human record's colour since T4a; `scenario_hero_step`
//!   0x58AF20): through the list its town offers (`GameSetup::offer`, the original's
//!   `scenario_hero_lists` 0x583890): right from Random to the first, on to the last, past it to
//!   Random; left the reverse; the row drawn again.
//! - **A right press** (`scenario_dialog_right_click` 0x5822D0, §3.15): on a town's picture
//!   (370 + c) its alignment (`scenario_town_popup` 0x575E60: the town's picture and name, its
//!   seven creatures in two rows) or a Random town's (`scenario_info_popup` 0x5757E0, GENRLTXT 104,
//!   523, 105); on a hero's (362 + c) the hero's own (T3b2: `scenario_hero_popup` 0x575A90, its
//!   portrait, name and class, its specialty's picture and name), a Random hero's (GENRLTXT 102,
//!   523, 103), none where the row shows None (CLT-Q112 (b)); on a bonus's (378 + c) its popup
//!   (`scenario_bonus_popup` 0x5754F0); each while the right button is held, the cursor showing,
//!   then the screen redrawn whole.
//!
//! - **The flags** (T3c1; design §26): the hover (`scenario_dialog_proc` 0x5881B2) highlights the
//!   flag under the mouse (its state 0x10: `AOFLGB%c.DEF`'s frame 3) and clears the one it left,
//!   each one's row drawn again; a press (`scenario_flag_click` 0x58B790, single) first takes the
//!   focus from the name's edit, then moves the single player to a colour a human may play that is
//!   not its own: the old flag's highlight cleared, its town and hero Random; the screen's flags
//!   (`scenario_team_flags` 0x585300) and its redraw follow (the screen's).
//! - **The handicap** (207 + c): a single player's is drawn up and takes no press; the record's
//!   handicap reaches no game (`CLT-STUB-10`).
//! - **The name's edit** (353 + c, T3c1): a click on the single player's name
//!   (`scenario_name_click` 0x58B620, at the text item's release) shows the edit over it with the
//!   record's name and the focus, the name hidden; the edit (its own vtable 0x641C14) takes every
//!   key but Escape and Tab: Enter (0x57CDC0) and a loss of the focus (0x57CEE0: a flag's press,
//!   the options' hiding) close it, its text the record's name, the row drawn again. A row's draw
//!   shows the name again (0x58DEBA); in the original's profile under the open edit too, and the
//!   opening pastes the colour's bar's piece at the bar's corner, not under the edit (CLT-Q143:
//!   the original's for the comparisons only).
//!
//! - **The duration** (T3c2; design §27): its bar (338, `h3_ui::scrollbar` on the x axis: 11
//!   ticks, no page, an arrow's step at the release, the track's drag) reports its value to
//!   `scenario_duration_scrolled` 0x57C7F0: the duration (`turndur.txt`'s record) and its text, the
//!   screen redrawn whole; no timer runs it (`CLT-STUB-11`). A right press on the headers, the bar
//!   and each colour's items shows their `Help.txt` hints (the screen's `HELPED` and
//!   `COLOUR_HELPED`); the names, the areas and a single player's handicap (not enabled) none.
//!
//! - **The hot seat** (T4a; design §28): a human record each of the hot seat's names, in order
//!   (`scenario_slots_new` 0x57F740), each colour a human may play to the next record without one
//!   (`scenario_players_init` 0x584C40), the colours past the last name the computer's; each row
//!   the name of the record holding its colour; the handicap a caption button (its first name,
//!   enabled, taking the mouse and its hint), no name's edit; the hero's arrows a human record's,
//!   the bonus's every colour's (0x58DE46); the local record the first name's.
//! - **The hot seat's presses** (T4b; design §29): the handicap one on, modulo 3, its caption the
//!   record's (the original's the colour's item's, set at the press alone; the client's follows
//!   the record holding the colour: CLT-Q144 (b)); a flag passes its colour on
//!   (`hot_seat_colour_cycle` 0x577AE0 as recorded, the original's; the client's to the next
//!   player, the two exchanging: CLT-Q94); the hero's arrows pass over a hero another colour's
//!   human record holds, a town's arrow and the showing step on a hero held twice
//!   (`scenario_heroes_unclash` 0x58BBC0); Begin refused with GENRLTXT 531 (a player without a
//!   colour: the original's the first player alone) or 83 (fewer than two placed).
//!
//! - **A colour only a human may play** (E27 M3 T7; §3.12, CLT-Q93, decision 13: no shipped map
//!   has one): where the single player takes one, the original's makes it and every later colour
//!   a human may play the computer's alone, the client's only the other colours only a human may
//!   play (the screen's `seats_of`).
//! - **Begin with the name's edit open** (E27 M3 T7, the review's B2): the client's profile takes
//!   the edit's text into the record first, as Enter does ([`Advanced::begun`]); the original's
//!   path is not read (design §26.5), its record's name as it was.
//!
//! *Not built yet:* the name written to
//! the setting `Network Default Name` (0x698817 and `settings_save` 0x50C1B0 at the edit's close:
//! the client has no settings, M4); a town a human-only colour draws at the list's read (no
//! shipped map has one; CLT-Q95); the edition's version (+0x1898: a single player's is SoD, so
//! Conflux is left out only by the map's mask); a portrait past the hero types' (the campaign
//! heroes' seven: no shipped map gives one).

use alloc::string::String;
use alloc::vec::Vec;

use h3_render::Bugs;
use h3_ui::edit::{self, Caret, Edit, Scan};
use h3_ui::scrollbar::{self, Scrollbar};
use h3_ui::ui::{Area, Button, Caption, Def, Dialog, Draw, Item, Label, Shown, State};
use serde::Deserialize;

use super::{Heroes, Saved, Seat};
use crate::message::MessageBox;

/// What stands for a hero's name in [`AdvancedTexts::hero_line`].
pub const HERO_NAME: char = '\u{1}';
/// What stands for a hero's class in [`AdvancedTexts::hero_line`].
pub const HERO_CLASS: char = '\u{2}';

/// The advanced options' panel (102, `AdvOptBk.pcx`).
pub const PANEL: u16 = 102;
/// The headers (GENRLTXT 518 to 521: the name's, the town's (342: AB and Complete), the hero's,
/// the bonus's) and the duration's label (340, GENRLTXT 522).
pub const HEADERS: [u16; 5] = [339, 342, 343, 344, 340];
/// The client's ids of what the redraw draws itself: the title (GENRLTXT 516), the line under it
/// (517) and the duration's text.
pub const TITLE: u16 = 1030;
/// The line under the title.
pub const SUBTITLE: u16 = 1031;
/// The duration's text (`turn_duration_texts` 0x69FB44 by the duration).
pub const DURATION: u16 = 1032;
/// The duration of a new game (10: unlimited; `scenario_slots_new` 0x57F740).
pub const UNLIMITED: u8 = 10;
/// The duration's bar (a scroll bar of the dialog: drawn by the options, E27 M3 T3c2).
pub const BAR: u16 = 338;

/// A colour's items, colour 0's ids (`+ c` for colour `c`): the flag, who may play, the handicap,
/// the town's, the hero's and the bonus's arrows, the name, the hero's, the town's and the
/// bonus's areas (the constructor's order, 0x57AF82 to 0x57B51B).
pub const FLAG: u16 = 263;
/// Who may play (`player_kind_texts` 0x6A7E18).
pub const KIND: u16 = 199;
/// The handicap (`adopb2%c.def`; a single player's disabled).
pub const HANDICAP: u16 = 207;
/// The town's left arrow.
pub const TOWN_LEFT: u16 = 215;
/// The town's right arrow.
pub const TOWN_RIGHT: u16 = 223;
/// The hero's left arrow.
pub const HERO_LEFT: u16 = 231;
/// The hero's right arrow.
pub const HERO_RIGHT: u16 = 239;
/// The bonus's left arrow.
pub const BONUS_LEFT: u16 = 247;
/// The bonus's right arrow.
pub const BONUS_RIGHT: u16 = 255;
/// The player's name (the human record's, else GENRLTXT 469).
pub const PLAYER: u16 = 345;
/// The single player's name's edit (E27 M3 T3c1; the constructor's last item of a colour).
pub const EDIT: u16 = 353;
/// The hero's area (its right press).
pub const HERO_AREA: u16 = 362;
/// The town's area.
pub const TOWN_AREA: u16 = 370;
/// The bonus's area.
pub const BONUS_AREA: u16 = 378;

/// A name's room with its terminator (`edit_create`'s 21: 20 characters).
const NAME_ROOM: usize = 21;

/// A colour's item ids of colour 0, in the constructor's order.
const ROW_IDS: [u16; 14] = [
    FLAG,
    KIND,
    HANDICAP,
    TOWN_LEFT,
    TOWN_RIGHT,
    HERO_LEFT,
    HERO_RIGHT,
    BONUS_LEFT,
    BONUS_RIGHT,
    PLAYER,
    HERO_AREA,
    TOWN_AREA,
    BONUS_AREA,
    EDIT,
];

/// The factions (Castle 0 to Conflux 8).
const FACTIONS: u8 = 9;

/// The advanced options' look (the layout's `[scenario.advanced]`): the colours' letters of the
/// per-colour files (`colour_letters` 0x641AE0), a row's height, the duration's bar, a colour's
/// items at row 0 (their DEFs' `{}` the colour's letter) and what its row draws itself, and the
/// popups.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdvancedLook {
    /// The colours' letters, red first (`rbygopts`).
    pub letters: String,
    /// A row's height (50).
    pub step: i32,
    /// The duration's bar (338, horizontal: `SlideBuH.def`, `slider.pcx`).
    pub bar: scrollbar::Look,
    /// The flag (`AOFLGB%c.DEF`, 42 × 50).
    pub flag: Button,
    /// Who may play (tiny, colour 4, centred).
    pub kind: Label,
    /// The handicap (`adopb2%c.def`).
    pub handicap: Button,
    /// The hot seat's handicap's caption (E27 M3 T4a: `tiny.fnt`, colour 4, centred).
    pub handicap_caption: Caption,
    /// The arrows (`adoplfa.def`, `adoprta.def`, 11 × 24): the town's, the hero's, the bonus's.
    pub town_left: Button,
    /// The town's right arrow.
    pub town_right: Button,
    /// The hero's left arrow.
    pub hero_left: Button,
    /// The hero's right arrow.
    pub hero_right: Button,
    /// The bonus's left arrow.
    pub bonus_left: Button,
    /// The bonus's right arrow.
    pub bonus_right: Button,
    /// The name (small, colour 1).
    pub name: Label,
    /// The single player's name's edit (small, colour 1, left; E27 M3 T3c1).
    pub edit: Label,
    /// The pictures' areas (48 × 32): the hero's, the town's, the bonus's.
    pub hero_area: Area,
    /// The town's area.
    pub town_area: Area,
    /// The bonus's area.
    pub bonus_area: Area,
    /// The colour's bar (`adop%cpnl.pcx`) and its corner.
    pub panel: String,
    /// The bar's corner.
    pub panel_at: [i32; 2],
    /// The town's picture: a town's `itpa.def` frame 2 t + 2, Random's `hpsrand0.pcx`.
    pub town: Pictured,
    /// The hero's picture: Random's `hpsrand1.pcx`, none's `hpsrand6.pcx`.
    pub hero: Pictured,
    /// The bonus's picture (`ScnrStar.def`).
    pub bonus: Pictured,
    /// The texts under the pictures (tiny, colour 4, centred): the town's, the hero's, the
    /// bonus's.
    pub town_name: Label,
    /// The hero's text.
    pub hero_name: Label,
    /// The bonus's text.
    pub bonus_name: Label,
    /// The small popup (Random's and the bonus's).
    pub popup: PopupLook,
    /// The town's popup.
    pub town_popup: TownPopupLook,
    /// A hero's own popup (E27 M3 T3b2).
    pub hero_popup: HeroPopupLook,
}

/// A picture a row draws itself: a DEF's frame or a picture by name, at its corner.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pictured {
    /// Its corner at row 0.
    pub at: [i32; 2],
    /// Its DEF, if it has one.
    #[serde(default)]
    pub def: Option<String>,
    /// Random's picture.
    #[serde(default)]
    pub random: Option<String>,
    /// None's picture.
    #[serde(default)]
    pub none: Option<String>,
}

/// The small popup (`scenario_info_popup` 0x5757E0, `scenario_bonus_popup` 0x5754F0): its place
/// and size, its frame, the title, the picture's corner, its caption and its text.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PopupLook {
    /// Its corner on the screen.
    pub place: [i32; 2],
    /// Its size (the frame's: multiples of 64).
    pub size: [i32; 2],
    /// The background's tile (`diboxbck.pcx`).
    pub background: String,
    /// The frame's pieces (`dialgbox.def`, in its own colours).
    pub frame: String,
    /// The title (medfont, colour 1).
    pub title: Label,
    /// The picture's corner (its 48 pixels centred).
    pub picture: [i32; 2],
    /// The caption under it.
    pub caption: Label,
    /// The text.
    pub text: Label,
}

/// The town's popup (`scenario_town_popup` 0x575E60): its place and size, its frame, the title,
/// the town's picture and name, the creatures' heading, and the faction's seven creatures
/// (`town_creatures` 0x6747B4's first seven: `cprsmall.def` frame + 2) in two rows, each with its
/// name under it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TownPopupLook {
    /// Its corner on the screen.
    pub place: [i32; 2],
    /// Its size.
    pub size: [i32; 2],
    /// The background's tile.
    pub background: String,
    /// The frame's pieces.
    pub frame: String,
    /// The title (GENRLTXT 81).
    pub title: Label,
    /// The town's picture's corner (`itpa.def`).
    pub picture: [i32; 2],
    /// The town's name.
    pub name: Label,
    /// The creatures' heading (GENRLTXT 80).
    pub heading: Label,
    /// A creature's picture (`cprsmall.def`, 32 × 32).
    pub creature: Def,
    /// A creature's name (tiny, colour 1, 52 × 32).
    pub creature_name: Label,
    /// Each row's first creature's corner, its name's and its count of creatures.
    pub rows: Vec<CreatureRow>,
    /// A creature's step to the next (52).
    pub step: i32,
    /// Each faction's creatures by level (their original ids).
    pub creatures: Vec<[u16; 7]>,
}

/// A hero's popup (`scenario_hero_popup` 0x575A90; E27 M3 T3b2, design §25): its place and size,
/// its frame, the title (GENRLTXT 78), the hero's small portrait, its name and class, the
/// specialty's heading (GENRLTXT 79), its picture (`un44.def`, the hero type's frame) and its
/// name.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeroPopupLook {
    /// Its corner on the screen.
    pub place: [i32; 2],
    /// Its size (the frame's).
    pub size: [i32; 2],
    /// The background's tile.
    pub background: String,
    /// The frame's pieces.
    pub frame: String,
    /// The title (medfont, colour 1).
    pub title: Label,
    /// The portrait's corner (its 48 pixels centred).
    pub portrait: [i32; 2],
    /// The name and the class (smalfont).
    pub name: Label,
    /// The specialty's heading (medfont).
    pub heading: Label,
    /// The specialty's picture (`un44.def`, 44 × 44, centred).
    pub specialty: Def,
    /// The specialty's name (smalfont, under its picture).
    pub specialty_name: Label,
}

/// A row of the town's popup's creatures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureRow {
    /// The first creature's corner.
    pub at: [i32; 2],
    /// Its name's corner.
    pub name: [i32; 2],
    /// The creatures in the row.
    pub count: usize,
}

/// The advanced options' look and texts (the shell's catalogue).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancedTexts {
    /// The look.
    pub look: AdvancedLook,
    /// Who may play: both, a human only, the computer only (ArrayTxt 206 to 208).
    pub kinds: [String; 3],
    /// A computer's name (GENRLTXT 469).
    pub computer: String,
    /// The handicaps' names: none, mild, severe (`handicap_names` 0x6A7800: ArrayTxt 210 to 212;
    /// the hot seat's captions, E27 M3 T4a).
    pub handicaps: [String; 3],
    /// Random (GENRLTXT 523) and none (524).
    pub random: String,
    /// None.
    pub none: String,
    /// The factions' names (ArrayTxt 219 to 227).
    pub towns: Vec<String>,
    /// The bonuses' names: artifact, gold, resource (ArrayTxt 214 to 216).
    pub bonuses: Vec<String>,
    /// The durations' texts (`turndur.txt` 0 to 10).
    pub durations: Vec<String>,
    /// The hero types' names by hero type (the catalogue's `hero` family: HOTRAITS).
    pub hero_names: Vec<String>,
    /// The small portraits by portrait (`hero_table` 0x679DD0's +0x30: the hero types', in type
    /// order; the asset tables' `small_portrait`).
    pub portraits: Vec<String>,
    /// A Random hero's popup: its title (GENRLTXT 102) and text (103).
    pub random_hero: [String; 2],
    /// A hero's own popup's title (GENRLTXT 78) and its specialty's heading (79; E27 M3 T3b2).
    pub hero_popup: [String; 2],
    /// A hero's popup's line of its name and class (`name_dash_format` 0x66033C, "%s - %s"):
    /// [`HERO_NAME`] stands for the name, [`HERO_CLASS`] for the class.
    pub hero_line: String,
    /// The hero types' classes' names, by hero type (HCTRAITS by the type's class).
    pub hero_classes: Vec<String>,
    /// The hero types' specialties' names, by hero type (HeroSpec.txt's first column:
    /// `hero_specialty_text` 0x4D7220).
    pub specialties: Vec<String>,
    /// The town's popup's title (GENRLTXT 81) and its creatures' heading (80).
    pub alignment: String,
    /// The creatures' heading.
    pub creatures: String,
    /// The creatures' names by original id.
    pub creature_names: Vec<(u16, String)>,
    /// A Random town's popup: its title (GENRLTXT 104) and text (105).
    pub random_town: [String; 2],
    /// The bonuses' popups by bonus (artifact, gold, resource, Random): the title, the caption
    /// and the text; a resource's caption and text by the faction ([`AdvancedTexts::resources`]).
    pub bonus_popups: [[String; 3]; 4],
    /// A resource's caption and text by faction (GENRLTXT 90 and 94, 693 to 696 and 689 to 692).
    pub resources: Vec<[String; 2]>,
}

/// A starting bonus (the record's +0x6C): chosen, no effect yet (E10's gap, decision 6).
// stub: CLT-STUB-9
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bonus {
    /// An artifact (0).
    Artifact,
    /// Gold (1).
    Gold,
    /// A resource of the town's (2).
    Resource,
    /// Random (3).
    Random,
}

impl Bonus {
    /// The bonus of the record's code, modulo 4.
    const fn of(code: u8) -> Self {
        match code % 4 {
            0 => Self::Artifact,
            1 => Self::Gold,
            2 => Self::Resource,
            _ => Self::Random,
        }
    }

    /// Its code.
    const fn code(self) -> u8 {
        match self {
            Self::Artifact => 0,
            Self::Gold => 1,
            Self::Resource => 2,
            Self::Random => 3,
        }
    }
}

/// A player's record (`H3ScenarioPlayer`, 0x7C bytes; `scenario_player_init` 0x57C790): what a
/// colour's row chooses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Record {
    /// The colour it holds (+0x70), none.
    pub colour: Option<u8>,
    /// The town (+0x24), none for Random.
    pub town: Option<u8>,
    /// The hero type picked (the original keeps its index in the list, +0x20), none for Random.
    pub hero: Option<u8>,
    /// The bonus (+0x6C).
    pub bonus: Bonus,
    /// The handicap (+0x78: 0 none, 1 mild, 2 severe): shown, chosen in the hot seat (T4), no
    /// effect yet (decision 6).
    // stub: CLT-STUB-10
    pub handicap: u8,
}

impl Record {
    /// A record as `scenario_player_init` starts it: no colour, Random town, hero and bonus, no
    /// handicap.
    pub(super) const NEW: Self = Self {
        colour: None,
        town: None,
        hero: None,
        bonus: Bonus::Random,
        handicap: 0,
    };
}

/// What a colour's row shows of its hero (`scenario_shown_hero` 0x58D0E0 and the row's draw).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hero {
    /// Random (`hpsrand1.pcx`, GENRLTXT 523).
    Random,
    /// None (`hpsrand6.pcx`, GENRLTXT 524).
    Nobody,
    /// A hero: its portrait and its type (its name's).
    Portrait {
        /// The portrait.
        portrait: u8,
        /// The hero type.
        hero_type: u8,
    },
}

/// A human record (`H3ScenarioPlayer` of a slot, §3.12): its name (+4) and its choices.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Human {
    /// Its name (the hot seat's: its edit's; the single player's: `network_default_name` at the
    /// slot's making, none in the client, then its edit's; E27 M3 T3c1).
    name: String,
    /// Its record (its colour among them).
    record: Record,
}

/// What the options read of the map they show (the selected entry's, else the game's): its
/// seats, its heroes and the single player's colour.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(in crate::shell) struct Map {
    /// Who may play each colour and what it is offered.
    pub(in crate::shell) seats: [Seat; 8],
    /// The heroes its import offers and customises.
    pub(in crate::shell) heroes: Heroes,
    /// The single player's colour.
    pub(in crate::shell) local: Option<u8>,
    /// The save the hot seat's load shows (E27 M4 T5b1): its humans, its choices and its turn's
    /// duration; none in a new game.
    pub(in crate::shell) saved: Option<Saved>,
}

/// The advanced options' state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Advanced {
    texts: AdvancedTexts,
    /// The original's for the comparisons: a main hero of its type's own portrait not shown
    /// (CLT-Q141), Random's popup over a None (CLT-Q112), a pick named by the main hero (CLT-Q142).
    bugs: Bugs,
    /// The options show (+0x37C).
    showing: bool,
    /// The selection changed since the rows were made (+0x1854).
    changed: bool,
    /// The hot seat's (`game_type` 0x6989F0 3; E27 M3 T4a): every human record takes a colour a
    /// human may play, the handicap is a caption button, no name has an edit.
    hot_seat: bool,
    /// The hot seat's load (E27 M4 T5b1; design §41.1): the rows the save's (its choices and
    /// duration, the names placed by its humans first), no arrows, the handicap not enabled, the
    /// duration's bar disabled (dimmed, taking no press).
    load: bool,
    /// The human records in the slots' order (+0x1064: `scenario_slot_add` 0x577BE0, ids 1 on):
    /// the single player's alone (the local slot's), or the hot seat's names'; the first is the
    /// local one (`local_slot` 0x69D608: id 1).
    humans: Vec<Human>,
    /// The computers' records, by colour.
    computers: [Record; 8],
    /// Each colour's row while the options show.
    rows: [Option<i32>; 8],
    /// The turn's duration's index (+0x378 and the game's +0x1F843: `turndur.txt`'s record), set
    /// by its bar (E27 M3 T3c2); no timer runs it yet (decision 6: the engine has none, the hot
    /// seat's clock is E24 M5's).
    // stub: CLT-STUB-11
    duration: u8,
    /// The duration's bar (338: 11 ticks, enabled for a single player and in the hot seat).
    bar: Scrollbar,
    /// The name's edit while it shows, by colour (it has the focus then).
    editing: Option<(u8, Edit)>,
    /// The flag the mouse is over (`scenario_hovered` 0x683470), by colour.
    hovered: Option<u8>,
    /// The original's profile: the colour whose bar's piece the edit's opening pasted at the bar's
    /// corner, until the row is drawn again (CLT-Q143).
    pasted: Option<u8>,
    /// The hot seat's cycle in the original's profile (`hot_seat_colour_cycle` 0x577AE0, E27 M3
    /// T4b): the colour pressed last (+0x1824, none again at the options' hiding, 0x581D5F) and
    /// the colour its taker left (+0x1830), to which a second press on it returns its holder.
    pressed: Option<u8>,
    /// See [`Advanced::pressed`].
    kept: Option<u8>,
}

/// Why Begin is refused over a map (`scenario_begin_tail` 0x58BFC0, E27 M3 T4b).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Refusal {
    /// GENRLTXT 531: a player has no colour.
    Unplaced,
    /// GENRLTXT 83: fewer than two players hold a colour (the hot seat's).
    TooFew,
}

impl Advanced {
    /// The options of `texts` in the profile `bugs`, hidden, the duration unlimited.
    pub(super) fn new(texts: AdvancedTexts, bugs: Bugs) -> Self {
        let mut bar = Scrollbar::new(texts.look.bar.clone());
        bar.reset(i32::from(UNLIMITED) + 1);
        bar.set_value(i32::from(UNLIMITED));
        Self {
            texts,
            bugs,
            showing: false,
            changed: true,
            hot_seat: false,
            load: false,
            humans: alloc::vec![Human {
                name: String::new(),
                record: Record::NEW,
            }],
            computers: [Record::NEW; 8],
            rows: [None; 8],
            duration: UNLIMITED,
            bar,
            editing: None,
            hovered: None,
            pasted: None,
            pressed: None,
            kept: None,
        }
    }

    /// The options of the hot seat's `names` (E27 M3 T4a; `scenario_slots_new` 0x57F740): a human
    /// record a name in their order, at most eight (`scenario_slot_add` 0x577BE0: none past the
    /// eighth), each starting as `scenario_player_init` 0x57C790 starts one.
    pub(super) fn seated(mut self, names: &[String]) -> Self {
        self.hot_seat = true;
        self.humans = names
            .iter()
            .take(8)
            .map(|name| Human {
                name: name.clone(),
                record: Record::NEW,
            })
            .collect();
        self
    }

    /// The hot seat's options in its load mode (E27 M4 T5b1; `scenario_advanced_toggle` 0x581383,
    /// 0x581743, design §41.1): the rows made from the save the map holds ([`Map::saved`]).
    pub(super) const fn set_load(&mut self) {
        self.load = true;
        self.changed = true;
    }

    /// Whether the options show.
    pub(super) const fn showing(&self) -> bool {
        self.showing
    }

    /// Whether item `id` takes the mouse: not a single player's handicap (not enabled; the hot
    /// seat's is, 0x58170E), nor a load's (not enabled, 0x581743: no popup on its right press,
    /// recorded; E27 M4 T5b1).
    pub(super) fn takes(&self, id: u16) -> bool {
        (self.hot_seat && !self.load) || !(HANDICAP..HANDICAP + 8).contains(&id)
    }

    /// The human record holding colour `c`, if one does.
    fn human_of(&self, c: u8) -> Option<&Human> {
        self.humans.iter().find(|h| h.record.colour == Some(c))
    }

    /// Each colour's record, red first (what the begin hands the engine: T6).
    pub(super) fn records(&self) -> [Record; 8] {
        core::array::from_fn(|c| self.record(c as u8))
    }

    /// Colour `colour`'s record: the human record holding it, else the computer's (§3.12).
    pub(super) fn record(&self, colour: u8) -> Record {
        self.human_of(colour)
            .map_or_else(|| self.computers[usize::from(colour & 7)], |h| h.record)
    }

    /// Colour `colour`'s record to change.
    fn record_mut(&mut self, colour: u8) -> &mut Record {
        match self
            .humans
            .iter_mut()
            .find(|h| h.record.colour == Some(colour))
        {
            Some(human) => &mut human.record,
            None => &mut self.computers[usize::from(colour & 7)],
        }
    }

    /// The local record's (the first human record's): the single player's.
    fn local(&self) -> Option<&Human> {
        self.humans.first()
    }

    /// The local record's colour.
    fn local_held(&self) -> Option<u8> {
        self.local().and_then(|h| h.record.colour)
    }

    /// The selection changed (`scenario_select` 0x585500's +0x1854): the rows are made again at the
    /// next showing.
    pub(super) const fn selection_changed(&mut self) {
        self.changed = true;
    }

    /// Every colour's items of `look`, each colour's at its own row as the constructor places
    /// them, in its order.
    pub(super) fn items(look: &AdvancedLook) -> Vec<Item> {
        (0..8u8)
            .flat_map(|c| Self::colour_items(look, c, i32::from(c)))
            .collect()
    }

    /// Colour `c`'s items of `look` on row `row` (its ids its colour 0's + c, its DEFs' `{}` its
    /// letter), in the constructor's order.
    fn colour_items(look: &AdvancedLook, c: u8, row: i32) -> [Item; 14] {
        let letter = letter(look, c);
        let dy = look.step * row;
        let at = |at: [i32; 2]| [at[0], at[1] + dy];
        let id = |id: u16| id + u16::from(c);
        let button = |b: &Button| {
            Item::Button(Button {
                id: id(b.id),
                def: b.def.replace("{}", &letter),
                at: at(b.at),
                ..b.clone()
            })
        };
        let label = |l: &Label| {
            Item::Label(Label {
                id: id(l.id),
                at: at(l.at),
                ..l.clone()
            })
        };
        let area = |a: &Area| {
            Item::Area(Area {
                id: id(a.id),
                at: at(a.at),
                ..a.clone()
            })
        };
        [
            button(&look.flag),
            label(&look.kind),
            button(&look.handicap),
            button(&look.town_left),
            button(&look.town_right),
            button(&look.hero_left),
            button(&look.hero_right),
            button(&look.bonus_left),
            button(&look.bonus_right),
            label(&look.name),
            area(&look.hero_area),
            area(&look.town_area),
            area(&look.bonus_area),
            label(&look.edit),
        ]
    }

    /// Colour `c`'s items on row `row` as the mode makes them: the hot seat's handicap a caption
    /// button (`caption_button_create` 0x456A50 at 0x57B0D3: `tiny.fnt`, colour 4, centred by its
    /// draw), a single player's a plain DEF button (0x57B110).
    fn row_items(&self, c: u8, row: i32) -> [Item; 14] {
        let look = &self.texts.look;
        let mut items = Self::colour_items(look, c, row);
        if self.hot_seat {
            for item in &mut items {
                if let Item::Button(b) = item
                    && b.id == HANDICAP + u16::from(c)
                {
                    b.caption = Some(look.handicap_caption.clone());
                }
            }
        }
        items
    }

    /// Every item id of the options: the panel, the headers, the redraw's own and the colours'.
    fn ids() -> impl Iterator<Item = u16> {
        let colours = ROW_IDS
            .into_iter()
            .flat_map(|id| (0..8u16).map(move |c| id + c));
        [PANEL, TITLE, SUBTITLE, DURATION]
            .into_iter()
            .chain(HEADERS)
            .chain(colours)
    }

    /// Whether item `id` is one of the options' ([`Advanced::ids`]: the colours' 199 to 270, 345 to
    /// 360 and 362 to 385, the headers 339 to 344).
    pub(super) const fn owns(id: u16) -> bool {
        matches!(id, PANEL | 199..=270 | 339..=360 | 362..=385 | TITLE..=DURATION)
    }

    /// Hidden (`scenario_advanced_hide` 0x581B20): the name's edit closed first (its focus
    /// taken), then every item of the options; the hot seat's colour pressed last forgotten
    /// (0x581D5F).
    pub(super) fn hide(&mut self, shown: &mut Shown) {
        self.close_edit(shown);
        self.showing = false;
        self.pressed = None;
        Self::hide_items(shown);
    }

    /// Every item of the options hidden in `shown` (the constructor's: also a screen without the
    /// options, whose layout has their panel and headers).
    pub(super) fn hide_items(shown: &mut Shown) {
        for id in Self::ids() {
            set(shown, id, |s| s.hidden = true);
        }
    }

    /// Shown over `map` (`scenario_advanced_toggle` 0x581100): the rows made again after a change
    /// of the selection; each colour that plays on its row (its items moved there), its flag where
    /// a human may play it, its handicap not enabled (a single player's; the hot seat's a caption
    /// button), who may play and its name set; the others' items hidden; a hero another human
    /// record holds stepped on (0x5817C1, as [`Advanced::unclash`]).
    pub(super) fn show(&mut self, map: &Map, dialog: &mut Dialog, shown: &mut Shown) {
        let seats = &map.seats;
        self.showing = true;
        if self.changed {
            self.remake(map);
            self.changed = false;
        }
        for id in [PANEL, TITLE, SUBTITLE, DURATION]
            .into_iter()
            .chain(HEADERS)
        {
            set(shown, id, |s| s.hidden = false);
        }
        let duration = text_at(&self.texts.durations, usize::from(self.duration));
        shown.texts.insert(DURATION, duration);
        let mut row = 0;
        for c in 0..8u8 {
            let seat = seats[usize::from(c)];
            let ids = ROW_IDS.map(|id| id + u16::from(c));
            if !seat.plays() {
                self.rows[usize::from(c)] = None;
                for id in ids {
                    set(shown, id, |s| s.hidden = true);
                }
                continue;
            }
            self.rows[usize::from(c)] = Some(row);
            for item in self.row_items(c, row) {
                if let Some(slot) = dialog.items.iter_mut().find(|i| i.id() == item.id()) {
                    *slot = item;
                }
            }
            // The flag shown and enabled where a human may play the colour (5, 6); its hover's
            // highlight (0x10) as the mouse left it.
            set(shown, FLAG + u16::from(c), |s| {
                *s = State {
                    hidden: !seat.human,
                    selected: s.selected,
                    ..State::default()
                };
            });
            for id in [KIND, PLAYER, HERO_AREA, TOWN_AREA, BONUS_AREA] {
                set(shown, id + u16::from(c), |s| s.hidden = false);
            }
            // A single player's handicap is not enabled (0x581713: its bit 2 cleared): drawn up,
            // taking no press ([`Advanced::takes`]); the name's edit hidden (0x58173B). The hot
            // seat's handicap is enabled, its caption the constructor's first name (0x57B0A4:
            // `handicap_names`' first) until a press; its edits stay hidden as made.
            // A load's not enabled either (E27 M4 T5b1: 0x581743, design §41.1): drawn up, taking
            // no press ([`Advanced::takes`]).
            set(shown, HANDICAP + u16::from(c), |s| *s = State::default());
            set(shown, EDIT + u16::from(c), |s| s.hidden = true);
            if self.hot_seat {
                let first = self.texts.handicaps[0].clone();
                shown.texts.entry(HANDICAP + u16::from(c)).or_insert(first);
            }
            let kind = match (seat.human, seat.computer) {
                (true, true) => 0,
                (true, false) => 1,
                _ => 2,
            };
            shown
                .texts
                .insert(KIND + u16::from(c), self.texts.kinds[kind].clone());
            row += 1;
        }
        self.unclash(map);
        self.set_names(shown);
        self.refresh(seats, shown);
    }

    /// The records made again over `map` (`scenario_players_init` 0x584C40): every record's
    /// colour, town and hero cleared, the human records seated. In a load (E27 M4 T5b1; design
    /// §41.1) every colour's record the save's choices and the turn's duration the save's: a
    /// human record takes the record of the colour it is seated on.
    fn remake(&mut self, map: &Map) {
        let humans = self.humans.iter_mut().map(|h| &mut h.record);
        for record in humans.chain(&mut self.computers) {
            record.colour = None;
            record.town = None;
            record.hero = None;
        }
        self.seat_humans(map);
        if let Some(saved) = map.saved.filter(|_| self.load) {
            self.computers = saved.records.map(|r| Record { colour: None, ..r });
            for human in &mut self.humans {
                if let Some(c) = human.record.colour {
                    human.record = Record {
                        colour: Some(c),
                        ..saved.records[usize::from(c & 7)]
                    };
                }
            }
            self.duration = saved.duration.min(UNLIMITED);
            self.bar.set_value(i32::from(self.duration));
        }
    }

    /// Each colour's record and the name of the human record holding it, red first, as Begin
    /// hands them on over `map` (E27 M3 T6): the rows' records, or while they are made again at
    /// the next showing (or `moved`: the client's, the map not the one the rows were made over,
    /// E27 M3 T7, CLT-Q152 (b)) those the showing will make ([`Advanced::remake`], as
    /// [`Advanced::refusal`] reads the colours). In the client's profile the name's open edit's
    /// text is the record's name first, as Enter's close makes it (E27 M3 T7, the review's B2;
    /// the original's path not read, design §26.5: its record's name as it was).
    pub(super) fn begun(&self, map: &Map, moved: bool) -> ([Record; 8], [Option<String>; 8]) {
        let mut made = self.clone();
        if let (Bugs::Fixed, Some((_, edit))) = (self.bugs, &self.editing) {
            let name = String::from(edit.text());
            for local in &mut made.humans {
                local.name.clone_from(&name);
            }
        }
        if made.changed || moved {
            made.remake(map);
        }
        let names = core::array::from_fn(|c| made.human_of(c as u8).map(|h| h.name.clone()));
        (made.records(), names)
    }

    /// The human records' colours from `map` (`scenario_players_init` 0x584C40, every record's
    /// colour cleared before): [`Advanced::seating`].
    fn seat_humans(&mut self, map: &Map) {
        let seating = self.seating(map);
        for (human, c) in self.humans.iter_mut().zip(seating) {
            human.record.colour = c;
        }
    }

    /// The human records' colours as `scenario_players_init` 0x584C40 makes them over `map`:
    /// single, the local record's the map's single player's colour (the only human record); the
    /// hot seat, each colour a human may play, in colour order, to the next human record (the loop
    /// 0x584DF0's hot-seat branch), the colours past the last name the computer's, the names past
    /// the last such colour none. In a load (E27 M4 T5b1; `scenario_players_init` 0x584C57 to
    /// 0x584D99, design §41.1, CLT-Q160 (a)): first each colour the save holds as human that a
    /// human may still play, in colour order, to the next record without one; then each colour a
    /// human may play that no record holds (a computer's in the save), until the names run out;
    /// a saved human past the names keeps no record (its row the computer's).
    fn seating(&self, map: &Map) -> Vec<Option<u8>> {
        if !self.hot_seat {
            return self.humans.iter().map(|_| map.local).collect();
        }
        let may = |c: u8| map.seats[usize::from(c)].human;
        let mut colours: Vec<u8> = match map.saved.filter(|_| self.load) {
            Some(saved) => {
                let saved_human = |c: u8| saved.humans[usize::from(c)];
                let first = (0..8u8).filter(|&c| may(c) && saved_human(c));
                first
                    .chain((0..8u8).filter(|&c| may(c) && !saved_human(c)))
                    .collect()
            }
            None => (0..8u8).filter(|&c| may(c)).collect(),
        };
        colours.reverse();
        self.humans.iter().map(|_| colours.pop()).collect()
    }

    /// Why Begin is refused over `map` (`scenario_begin_tail` 0x58C11F, a new game): GENRLTXT 531
    /// where the local record (the single player's, the hot seat's first) has no colour, in the
    /// client's profile where any human record has none (CLT-Q109 (b), decision 22: the
    /// original's drops a named player without a colour at the begin; CLT-Q145 (c): a map with
    /// fewer colours a human may play than names is listed but never begun); then the hot seat's
    /// 83 where fewer than two human records hold a colour. The colours the rows hold, or while
    /// they are made again at the next showing those that will make them.
    pub(super) fn refusal(&self, map: &Map) -> Option<Refusal> {
        let colours = if self.changed {
            self.seating(map)
        } else {
            self.humans.iter().map(|h| h.record.colour).collect()
        };
        let unplaced = match self.bugs {
            Bugs::Original => colours.first().is_some_and(Option::is_none),
            Bugs::Fixed => colours.contains(&None),
        };
        if unplaced {
            return Some(Refusal::Unplaced);
        }
        let placed = colours.iter().flatten().count();
        (self.hot_seat && placed < 2).then_some(Refusal::TooFew)
    }

    /// Each row's name (`scenario_dialog_redraw` 0x584550 sets them): the human record's that
    /// holds the colour, else a computer's (GENRLTXT 469).
    fn set_names(&self, shown: &mut Shown) {
        for c in (0..8u8).filter(|&c| self.rows[usize::from(c)].is_some()) {
            let name = self
                .human_of(c)
                .map_or_else(|| self.texts.computer.clone(), |h| h.name.clone());
            shown.texts.insert(PLAYER + u16::from(c), name);
        }
    }

    /// The colour the local record (the single player's, the hot seat's first) holds while the
    /// rows are as made (none: they are made again at the next showing, `+0x1854`), which the
    /// screen's flags read (`scenario_team_flags` 0x585300: the local record's colour).
    pub(super) fn local_colour(&self) -> Option<Option<u8>> {
        (!self.changed).then(|| self.local_held())
    }

    /// Colour `c`'s row drawn again (`scenario_player_row_draw` 0x58D510: its bar whole, its name
    /// shown, 0x58DEBA): the original's piece gone; the name shown, in the client's profile not
    /// under its open edit (CLT-Q143).
    fn row_drawn(&mut self, c: u8, shown: &mut Shown) {
        if !self.showing || self.rows[usize::from(c)].is_none() {
            return;
        }
        if self.pasted == Some(c) {
            self.pasted = None;
        }
        let editing = matches!(self.editing, Some((e, _)) if e == c);
        if self.bugs == Bugs::Original || !editing {
            set(shown, PLAYER + u16::from(c), |s| s.hidden = false);
        }
    }

    /// The turn's duration (`turndur.txt`'s record: 0 to 9 one to thirty minutes, 10 unlimited).
    pub(super) const fn duration(&self) -> u8 {
        self.duration
    }

    /// Whether `point` (from the dialog's corner) is on the duration's bar while the options show
    /// and the bar is enabled (not in a load, E27 M4 T5b1: its vtable +0x24 skipped, 0x581383;
    /// drawn dimmed, its arrows taking no press: recorded).
    pub(super) fn on_bar(&self, point: [i32; 2]) -> bool {
        self.showing && !self.load && self.bar.contains(point)
    }

    /// Whether the mouse holds the duration's bar (its own loop runs: no hover, key or right
    /// press reaches the screen).
    pub(super) const fn bar_held(&self) -> bool {
        self.bar.held()
    }

    /// The duration's bar pressed at `point` (from the dialog's corner; `h3_ui::scrollbar`):
    /// whether the duration changed ([`Advanced::scrolled`]).
    pub(super) fn bar_press(&mut self, point: [i32; 2], shown: &mut Shown) -> bool {
        let value = self.bar.press(point);
        self.scrolled(value, shown)
    }

    /// The mouse moved to `point` while it holds the duration's bar: whether the duration changed.
    pub(super) fn bar_moved(&mut self, point: [i32; 2], shown: &mut Shown) -> bool {
        let value = self.bar.moved(point);
        self.scrolled(value, shown)
    }

    /// The duration's bar let go (an arrow steps it): whether the duration changed.
    pub(super) fn bar_release(&mut self, shown: &mut Shown) -> bool {
        let value = self.bar.release();
        self.scrolled(value, shown)
    }

    /// The bar's value reported (`scenario_duration_scrolled` 0x57C7F0, the bar's callback): the
    /// duration (+0x378, the game's +0x1F843; a network host would send the setup, message 0x402:
    /// no network play) and its text (`turn_duration_texts` 0x69FB44); the caller redraws the screen
    /// whole (the dialog's items, vtable +0x14 with 0, 0xFFFF0001, 0xFFFF, then
    /// `scenario_dialog_redraw`). Whether a value was reported.
    fn scrolled(&mut self, value: Option<i32>, shown: &mut Shown) -> bool {
        let Some(value) = value else {
            return false;
        };
        self.duration = u8::try_from(value).unwrap_or(UNLIMITED);
        let text = text_at(&self.texts.durations, usize::from(self.duration));
        shown.texts.insert(DURATION, text);
        true
    }

    /// The mouse over item `id` (none: no item; `scenario_dialog_proc` 0x5881B2): over another
    /// flag than the one hovered, that one's highlight (0x10) cleared and the new one's set, each
    /// one's row drawn again; off the flags the one hovered cleared. Whether anything changed.
    pub(super) fn hover(&mut self, id: Option<u16>, shown: &mut Shown) -> bool {
        let flag = id.and_then(flag_of);
        if flag == self.hovered {
            return false;
        }
        if let Some(old) = self.hovered.take() {
            set(shown, FLAG + u16::from(old), |s| s.selected = false);
            self.row_drawn(old, shown);
        }
        if let Some(c) = flag {
            set(shown, FLAG + u16::from(c), |s| s.selected = true);
            self.row_drawn(c, shown);
            self.hovered = Some(c);
        }
        true
    }

    /// Colour `c`'s flag clicked over `map` (`scenario_flag_click` 0x58B790): the name's edit
    /// closed first (`dialog_set_focus` −1, single), then to a colour a human may play: single,
    /// when it is not the single player's own, the old flag's highlight cleared, the single
    /// player's record to it, its town and hero Random (`scenario_hero_lists`); the hot seat
    /// (E27 M3 T4b), the colour passed on ([`Advanced::cycle`], the original's; the client's
    /// [`Advanced::pass`], CLT-Q94); then the names set and every row drawn again (the redraw).
    /// The screen's flags are the caller's.
    pub(super) fn flag_click(&mut self, c: u8, map: &Map, shown: &mut Shown) {
        self.close_edit(shown);
        let seat = map.seats[usize::from(c & 7)];
        let old = self.local_held();
        if self.hot_seat {
            if seat.human {
                match self.bugs {
                    Bugs::Original => self.cycle(c),
                    Bugs::Fixed => self.pass(c),
                }
            }
        } else if seat.human
            && old != Some(c)
            && let Some(local) = self.humans.first_mut()
        {
            if let Some(old) = old {
                set(shown, FLAG + u16::from(old), |s| s.selected = false);
            }
            local.record.colour = Some(c);
            local.record.town = None;
            local.record.hero = None;
        }
        self.set_names(shown);
        self.refresh(&map.seats, shown);
        for c in 0..8 {
            self.row_drawn(c, shown);
        }
    }

    /// The hot seat's press on colour `c`'s flag as the original passes it on
    /// (`hot_seat_colour_cycle` 0x577AE0; recorded: `adv-hotseat-rows`, `adv-hotseat-two`,
    /// `adv-hotseat-begin`): its holder loses it; the taker is the human record after the holder
    /// (the holder's +0x74 + 1: its index in the hot seat, as recorded), with no holder the first
    /// record without a colour, else the first; a second press on the colour pressed last first
    /// returns its holder to the colour the last taker left. The taker leaves its own colour
    /// (kept for that return) and takes `c`, its town and hero Random; past the last record
    /// nobody takes it (the computer's).
    fn cycle(&mut self, c: u8) {
        let holder = self.humans.iter().position(|h| h.record.colour == Some(c));
        let free = self.humans.iter().position(|h| h.record.colour.is_none());
        let taker = match holder {
            Some(h) => {
                self.humans[h].record.colour = None;
                h + 1
            }
            None => free.unwrap_or(0),
        };
        if self.pressed == Some(c) {
            if let (Some(k), Some(h)) = (self.kept, holder) {
                self.humans[h].record.colour = Some(k);
            }
        } else {
            self.pressed = Some(c);
        }
        self.kept = None;
        if let Some(taker) = self.humans.get_mut(taker) {
            self.kept = taker.record.colour;
            taker.record.colour = Some(c);
            taker.record.town = None;
            taker.record.hero = None;
        }
    }

    /// The hot seat's press on colour `c`'s flag as the client passes it on (CLT-Q94, decision
    /// 13; *provisional* in its detail): to the human record after its holder, the two exchanging
    /// colours (a taker without one leaves the holder with none); past the last record to the
    /// computer, the holder left with none; with no holder the first record takes it, its own the
    /// computer's. Each record that moves has its town and hero Random.
    fn pass(&mut self, c: u8) {
        let holder = self.humans.iter().position(|h| h.record.colour == Some(c));
        let taker = holder.map_or(0, |h| h + 1);
        let own = self.humans.get(taker).and_then(|t| t.record.colour);
        let moved = |record: &mut Record, colour: Option<u8>| {
            record.colour = colour;
            record.town = None;
            record.hero = None;
        };
        if let Some(h) = holder {
            moved(&mut self.humans[h].record, own);
        }
        if let Some(taker) = self.humans.get_mut(taker) {
            moved(&mut taker.record, Some(c));
        }
    }

    /// Colour `c`'s name clicked (`scenario_name_click` 0x58B620, single only: the hot seat's names
    /// take none): the focus taken from an open edit, then on the single player's own colour its
    /// edit shown over the name with the record's name (the caret at its end) and the focus, the
    /// name hidden; in the original's profile the bar's piece pasted at the bar's corner
    /// (CLT-Q143). Whether it opened.
    pub(super) fn name_click(&mut self, c: u8, shown: &mut Shown) -> bool {
        if !self.showing || self.hot_seat {
            return false;
        }
        self.close_edit(shown);
        if self.local_held() != Some(c) {
            return false;
        }
        let name = self.local().map(|h| h.name.clone()).unwrap_or_default();
        let mut edit = Edit::new(&name, NAME_ROOM);
        edit.set_focus(true);
        self.editing = Some((c, edit));
        set(shown, EDIT + u16::from(c), |s| *s = State::default());
        set(shown, PLAYER + u16::from(c), |s| s.hidden = true);
        shown.texts.insert(PLAYER + u16::from(c), name);
        if self.bugs == Bugs::Original {
            self.pasted = Some(c);
        }
        true
    }

    /// Whether the name's edit shows (with the focus).
    pub(super) const fn editing(&self) -> bool {
        self.editing.is_some()
    }

    /// The open edit's item in `draws` (where it is a label) drawn as the edit.
    pub(super) fn place_edit(&self, draws: &mut [Draw]) {
        if let Some((c, edit)) = &self.editing {
            edit.place(EDIT + u16::from(*c), draws);
        }
    }

    /// The name's edit closed (its Enter, 0x57CDC0, or its loss of the focus, 0x57CEE0): hidden,
    /// its text the record's name and the name's, the row drawn again. Whether one was open.
    pub(super) fn close_edit(&mut self, shown: &mut Shown) -> bool {
        let Some((c, edit)) = self.editing.take() else {
            return false;
        };
        let name = String::from(edit.text());
        // (The edit is a single player's, whose record is the only human one.)
        for local in &mut self.humans {
            local.name.clone_from(&name);
        }
        set(shown, EDIT + u16::from(c), |s| s.hidden = true);
        shown.texts.insert(PLAYER + u16::from(c), name);
        self.row_drawn(c, shown);
        true
    }

    /// A key down `scan` to the open edit as the original reads it (its key, 0x57CDC0): Enter
    /// closes it, any other but Escape and Tab is the edit's (`edit_key` 0x5BAC50). Whether it
    /// was taken (its dialog's hotkeys get the others).
    pub(super) fn edit_key(&mut self, scan: Scan, shown: &mut Shown) -> bool {
        let Some((_, edit)) = &mut self.editing else {
            return false;
        };
        if scan.code == edit::RETURN {
            return self.close_edit(shown);
        }
        edit.key(scan)
    }

    /// One of the open edit's own keys (the client's profile: the key table's): whether taken.
    pub(super) fn edit_caret(&mut self, caret: Caret) -> bool {
        self.editing
            .as_mut()
            .is_some_and(|(_, edit)| edit.caret_key(caret))
    }

    /// A character typed into the open edit (the client's profile): whether taken.
    pub(super) fn edit_typed(&mut self, c: char) -> bool {
        self.editing.as_mut().is_some_and(|(_, edit)| edit.typed(c))
    }

    /// The original's piece (CLT-Q143): `scenario_name_click` 0x58B724 draws the colour's bar
    /// `adop%cpnl.pcx` from the edit's offset in it (5, 2) with the edit's size at the bar's
    /// corner, not under the edit; recorded (`adv-single-rest` name-edit): the bar shifted 5 left
    /// and 2 up, seen from the edit's corner to the bar's corner plus the edit's size (92 × 15),
    /// until the row is drawn again. Its rectangle, the bar so drawn, and the edit's draw over
    /// it.
    pub(super) fn piece(
        &self,
        dialog: &Dialog,
        shown: &Shown,
        players: &str,
    ) -> Option<([i32; 4], Vec<Draw>)> {
        let c = self.pasted?;
        // (A pasted colour has its row: the hiding takes the paste back first.)
        let row = self.rows[usize::from(c)].unwrap_or_default();
        let look = &self.texts.look;
        let [px, py] = dialog.place;
        let [bx, by] = look.panel_at;
        let [ex, ey] = look.edit.at;
        let (sx, sy) = (ex - bx, ey - by);
        let top = py + by + look.step * row;
        let [w, h] = look.edit.size;
        let rect = [px + ex, top + sy, w - sx, h - sy];
        let mut draws = alloc::vec![Draw::Picture {
            picture: look.panel.replace("{}", &letter(look, c)),
            x: px + bx - sx,
            y: top - sy,
            player: None,
            size: None,
        }];
        draws.extend(self.edit_draws(c, dialog, shown, players));
        Some((rect, draws))
    }

    /// Colour `c`'s edit as its row draws it (none unless it shows on that colour).
    fn edit_draws(&self, c: u8, dialog: &Dialog, shown: &Shown, players: &str) -> Vec<Draw> {
        let id = EDIT + u16::from(c);
        let mut draws = Dialog {
            place: dialog.place,
            shadow: None,
            items: dialog
                .items
                .iter()
                .filter(|i| i.id() == id)
                .cloned()
                .collect(),
        }
        .draws(shown, players);
        self.place_edit(&mut draws);
        draws
    }

    /// Each row's arrows shown or hidden by its record (the row's draw), and a resource's bonus
    /// under a shown Random town made Random (0x58DDAB).
    pub(super) fn refresh(&mut self, seats: &[Seat; 8], shown: &mut Shown) {
        for c in 0..8u8 {
            if self.rows[usize::from(c)].is_none() {
                continue;
            }
            let seat = seats[usize::from(c)];
            let town = self.shown_town(seat, c);
            if town.is_none() && self.record(c).bonus == Bonus::Resource {
                self.record_mut(c).bonus = Bonus::Random;
            }
            // The hot seat's handicap in the client's profile: the caption of the record holding
            // the colour (CLT-Q144 (b), decision 60); the original's is the colour's item's, set
            // at its press alone, so a player moved by a flag shows the caption pressed there.
            if self.hot_seat && self.bugs == Bugs::Fixed {
                let caption = self.handicap_caption(c);
                shown.texts.insert(HANDICAP + u16::from(c), caption);
            }
            let arrows = [
                (TOWN_LEFT, TOWN_RIGHT, self.town_choosable(seat, c)),
                (HERO_LEFT, HERO_RIGHT, self.hero_choosable(seat, c)),
                (BONUS_LEFT, BONUS_RIGHT, self.bonus_choosable(seat, c)),
            ];
            for (left, right, on) in arrows {
                for id in [left, right] {
                    set(shown, id + u16::from(c), |s| {
                        s.hidden = !on;
                        s.disabled = !on;
                    });
                }
            }
        }
    }

    /// The town colour `c`'s row shows: its record's when the map offers a choice (its random-town
    /// flag, or more than one faction: `faction_choice_offered` 0x58CE70), else its first allowed
    /// faction (`random_allowed_faction` 0x5132D0 with its first flag; 0 when none).
    fn shown_town(&self, seat: Seat, c: u8) -> Option<u8> {
        if offered(seat) {
            self.record(c).town
        } else {
            Some(first_faction(seat.factions))
        }
    }

    /// Whether colour `c`'s town may change (`scenario_town_choosable` 0x58CEB0, a single player's
    /// new game): its record the single player's or a computer's, and the map offering a choice.
    fn town_choosable(&self, seat: Seat, _c: u8) -> bool {
        !self.load && offered(seat)
    }

    /// Whether colour `c`'s hero may change (`scenario_hero_choosable` 0x58CFB0): a human record's
    /// (outside the hot seat the single player's: the only one), its shown town not Random, and
    /// its main town generating its hero or the random-hero flag (TWN-SET-006).
    fn hero_choosable(&self, seat: Seat, c: u8) -> bool {
        !self.load
            && self.human_of(c).is_some()
            && self.shown_town(seat, c).is_some()
            && (seat.generates || seat.random_hero)
    }

    /// Whether colour `c`'s bonus may change (the row's draw, 0x58DD48): the local record's
    /// colour, or a computer's whose town shows a town or may change; the hot seat every colour's
    /// (0x58DE46).
    fn bonus_choosable(&self, seat: Seat, c: u8) -> bool {
        // (E27 M3 T7, the review's B6: in a new game every colour's town shows a town or may
        // change, so the hot seat's and the local colour's terms decide nothing before the load's
        // records, M4: a load's colour whose town neither shows nor changes.)
        !self.load
            && (self.hot_seat
                || self.local_held() == Some(c)
                || self.shown_town(seat, c).is_some()
                || self.town_choosable(seat, c))
    }

    /// Whether colour `c` has no hero to take an artifact: its record's hero Random and the
    /// colour with neither the random-hero flag, a generating main town nor a main hero (0x587412).
    fn heroless(&self, seat: Seat, c: u8) -> bool {
        self.record(c).hero.is_none()
            && !seat.random_hero
            && !seat.generates
            && seat.main_hero.is_none()
    }

    /// What colour `c`'s row shows of its hero over `map` (`scenario_shown_hero` 0x58D0E0, the
    /// row's draw 0x58DB0F): with neither the generating main town nor the random-hero flag the
    /// map's main hero (none where the map leaves its portrait at its type's own: the original's;
    /// CLT-Q141 (b) shows the type's), else the record's pick (its portrait the map's customised
    /// one, else its type's); no hero shown is Random while the arrows show or the colour has
    /// either flag (or, under a Random town, any hero at all), else none.
    fn shown_hero(&self, map: &Map, c: u8) -> Hero {
        let seat = map.seats[usize::from(c)];
        let flags = seat.generates || seat.random_hero;
        let hero = if flags {
            self.record(c).hero.map(|t| {
                let custom = map.heroes.custom.iter().find(|h| h.hero_type == t);
                (custom.and_then(|h| h.portrait).unwrap_or(t), t)
            })
        } else {
            seat.main_hero.and_then(|h| {
                let portrait = match self.bugs {
                    Bugs::Original => h.portrait,
                    Bugs::Fixed => Some(h.portrait.unwrap_or(h.hero_type)),
                };
                portrait.map(|p| (p, h.hero_type))
            })
        };
        if let Some((portrait, hero_type)) = hero {
            return Hero::Portrait {
                portrait,
                hero_type,
            };
        }
        let none_at_all = !flags && seat.main_hero.is_none();
        let random_town = self.shown_town(seat, c).is_none();
        if self.hero_choosable(seat, c) || flags || (random_town && !none_at_all) {
            Hero::Random
        } else {
            Hero::Nobody
        }
    }

    /// The name under colour `c`'s hero of type `hero_type` over `map` (`scenario_shown_hero_name`
    /// 0x58D1F0): the main hero's own name of the map under the main hero (the original's under a
    /// pick too, a colour with a flag: CLT-Q142 (b), for the comparisons only), else a customised
    /// hero's of the map, else the type's.
    fn hero_name(&self, map: &Map, c: u8, hero_type: u8) -> String {
        let seat = map.seats[usize::from(c)];
        let pick = seat.generates || seat.random_hero;
        let main = &map.heroes.main_names[usize::from(c)];
        if !main.is_empty() && (!pick || self.bugs == Bugs::Original) {
            return main.clone();
        }
        map.heroes
            .custom
            .iter()
            .find(|h| h.hero_type == hero_type && !h.name.is_empty())
            .map_or_else(
                || text_at(&self.texts.hero_names, usize::from(hero_type)),
                |h| h.name.clone(),
            )
    }

    /// A town's arrow of colour `c` (`right` the right one): the next or previous allowed faction
    /// from the shown town, Random past either end and the far end from Random; the hero Random.
    fn step_town(&mut self, seat: Seat, c: u8, right: bool) {
        let shown = self.shown_town(seat, c).map_or(-1, i16::from);
        let allowed = |t: i16| t >= 0 && seat.factions & (1 << t) != 0;
        let mut t = shown;
        loop {
            t = if right { t + 1 } else { t - 1 };
            if right && t >= i16::from(FACTIONS) {
                t = -1;
            }
            if !right && t < -1 {
                t = i16::from(FACTIONS) - 1;
            }
            if t == -1 || allowed(t) {
                break;
            }
        }
        let record = self.record_mut(c);
        record.town = u8::try_from(t).ok();
        record.hero = None;
    }

    /// Colour `c`'s list of heroes (`scenario_hero_lists` 0x583890: a human record's alone): the
    /// offer of its shown town where a human record holds it, else none.
    fn hero_list<'m>(&self, map: &'m Map, c: u8) -> &'m [u8] {
        match self.shown_town(map.seats[usize::from(c)], c) {
            Some(f) if self.human_of(c).is_some() => {
                &map.heroes.offers[usize::from(c)][usize::from(f)]
            }
            _ => &[],
        }
    }

    /// A hero's arrow of colour `c` (`right` the right one; `scenario_hero_step` 0x58AF20): right
    /// from Random to the list's first, on to its last, past it to Random; left the reverse,
    /// passing over a hero another colour's human record holds (E27 M3 T4b: the hot seat's; Random
    /// never passed over); nothing with an empty list.
    fn step_hero(&mut self, map: &Map, c: u8, right: bool) {
        let list = self.hero_list(map, c);
        let Some(last) = list.len().checked_sub(1) else {
            return;
        };
        let mut at = self
            .record(c)
            .hero
            .and_then(|t| list.iter().position(|&h| h == t));
        loop {
            at = match (at, right) {
                (None, true) => Some(0),
                (Some(k), true) => (k < last).then_some(k + 1),
                (None, false) => Some(last),
                (Some(k), false) => k.checked_sub(1),
            };
            if !at.is_some_and(|k| self.held_elsewhere(c, list[k])) {
                break;
            }
        }
        self.record_mut(c).hero = at.map(|k| list[k]);
    }

    /// Whether hero type `hero_type` is the pick of a human record holding a colour other than
    /// `c` (`scenario_hero_step`'s test, 0x58AF93: each other colour's human record's).
    fn held_elsewhere(&self, c: u8, hero_type: u8) -> bool {
        self.humans
            .iter()
            .any(|h| h.record.colour.is_some_and(|o| o != c) && h.record.hero == Some(hero_type))
    }

    /// Each hero another colour's human record holds too stepped on (`scenario_heroes_unclash`
    /// 0x58BBC0, after a town's arrow; the showing's own loop 0x5817C1): colours 7 down to 0, a
    /// human record's pick held elsewhere stepped right once ([`Advanced::step_hero`] passes over
    /// what is held). The steps never make a clash, so none is found in play (kept as the
    /// original's).
    fn unclash(&mut self, map: &Map) {
        for c in (0..8u8).rev() {
            let hero = self.human_of(c).and_then(|h| h.record.hero);
            if hero.is_some_and(|t| self.held_elsewhere(c, t)) {
                self.step_hero(map, c, true);
            }
        }
    }

    /// Colour `c`'s handicap's caption: its record's (`handicap_names` 0x6A7800).
    fn handicap_caption(&self, c: u8) -> String {
        self.texts.handicaps[usize::from(self.record(c).handicap % 3)].clone()
    }

    /// A bonus's arrow of colour `c` (`right` the right one): one back or on, modulo 4, the
    /// artifact passed over for a colour with no hero to take it, the resource under a shown
    /// Random town (left to gold, right to Random).
    fn step_bonus(&mut self, seat: Seat, c: u8, right: bool) {
        let heroless = self.heroless(seat, c);
        let random_town = self.shown_town(seat, c).is_none();
        let record = self.record_mut(c);
        let code = record.bonus.code();
        let mut bonus = if right {
            Bonus::of(code + 1)
        } else {
            Bonus::of(code + 3)
        };
        if bonus == Bonus::Artifact && heroless {
            bonus = if right { Bonus::Gold } else { Bonus::Random };
        }
        if bonus == Bonus::Resource && random_town {
            bonus = if right { Bonus::Random } else { Bonus::Gold };
        }
        record.bonus = bonus;
    }

    /// Colour `c`'s handicap pressed (207 + c, the hot seat's; `scenario_dialog_click` 0x58706A):
    /// its record's handicap one on, modulo 3, the item's caption its name (`handicap_names`
    /// 0x6A7800, `dialog_item_send` 3).
    fn step_handicap(&mut self, c: u8, shown: &mut Shown) {
        let record = self.record_mut(c);
        record.handicap = (record.handicap + 1) % 3;
        let caption = self.handicap_caption(c);
        shown.texts.insert(HANDICAP + u16::from(c), caption);
    }

    /// The click of item `id` over `map` (an arrow of the town, the hero or the bonus; the hot
    /// seat's handicap), its row drawn again; whether it was one. A town's arrow then steps on a
    /// hero held twice (`scenario_heroes_unclash` 0x586A36).
    pub(super) fn click(&mut self, id: u16, map: &Map, shown: &mut Shown) -> bool {
        if !self.showing {
            return false;
        }
        let of = |first: u16| (first..first + 8).contains(&id).then(|| (id - first) as u8);
        type Step = fn(&mut Advanced, &Map, u8, &mut Shown);
        let (c, step): (u8, Step) = if let Some(c) = of(HANDICAP) {
            (c, |a, _, c, shown| a.step_handicap(c, shown))
        } else if let Some(c) = of(TOWN_LEFT) {
            (c, |a, m, c, _| {
                a.step_town(m.seats[usize::from(c)], c, false);
                a.unclash(m);
            })
        } else if let Some(c) = of(TOWN_RIGHT) {
            (c, |a, m, c, _| {
                a.step_town(m.seats[usize::from(c)], c, true);
                a.unclash(m);
            })
        } else if let Some(c) = of(HERO_LEFT) {
            (c, |a, m, c, _| a.step_hero(m, c, false))
        } else if let Some(c) = of(HERO_RIGHT) {
            (c, |a, m, c, _| a.step_hero(m, c, true))
        } else if let Some(c) = of(BONUS_LEFT) {
            (c, |a, m, c, _| {
                a.step_bonus(m.seats[usize::from(c)], c, false)
            })
        } else if let Some(c) = of(BONUS_RIGHT) {
            (c, |a, m, c, _| {
                a.step_bonus(m.seats[usize::from(c)], c, true)
            })
        } else {
            return false;
        };
        step(self, map, c, shown);
        self.refresh(&map.seats, shown);
        self.row_drawn(c, shown);
        true
    }

    /// The popup of a right press on item `id` over `map`: a town's area's alignment (or a Random
    /// town's), a hero's area's hero's own (T3b2) or a Random hero's, a bonus's area's bonus; none
    /// for another item.
    pub(super) fn popup(&self, id: u16, map: &Map) -> Option<MessageBox> {
        if !self.showing {
            return None;
        }
        let seats = &map.seats;
        let of = |first: u16| (first..first + 8).contains(&id).then(|| (id - first) as u8);
        if let Some(c) = of(HERO_AREA) {
            return self.hero_popup(map, c);
        }
        if let Some(c) = of(TOWN_AREA) {
            let seat = seats[usize::from(c)];
            return Some(match self.shown_town(seat, c) {
                Some(t) => self.town_popup(t),
                None => {
                    let t = &self.texts;
                    let random = t.look.town.random.clone().unwrap_or_default();
                    self.small_popup(
                        &t.random_town[0],
                        Picture::Named(random),
                        &t.random,
                        &t.random_town[1],
                    )
                }
            });
        }
        let c = of(BONUS_AREA)?;
        let seat = seats[usize::from(c)];
        let bonus = self.record(c).bonus;
        let town = self.shown_town(seat, c);
        let t = &self.texts;
        let [title, caption, text] = t.bonus_popups[usize::from(bonus.code())].clone();
        let (caption, text) = match (bonus, town) {
            (Bonus::Resource, Some(f)) => {
                let [c, x] = t.resources.get(usize::from(f)).cloned().unwrap_or_default();
                (c, x)
            }
            _ => (caption, text),
        };
        let frame = bonus_frame(bonus, town);
        let def = t.look.bonus.def.clone().unwrap_or_default();
        Some(self.small_popup(&title, Picture::Frame(def, frame), &caption, &text))
    }

    /// Colour `c`'s hero's popup over `map` (`scenario_dialog_right_click` 0x5823FF): none for a
    /// colour with no hero at all (neither flag nor a main hero); a hero shown, its own
    /// (`scenario_hero_popup` 0x575A90, T3b2); a Random one, Random's (`scenario_info_popup`
    /// 0x5757E0: GENRLTXT 102, `hpsrand1.pcx`, 523, 103); a None, none (CLT-Q112 (b); the
    /// original's Random's for the comparisons).
    fn hero_popup(&self, map: &Map, c: u8) -> Option<MessageBox> {
        let seat = map.seats[usize::from(c)];
        let none_at_all = !(seat.generates || seat.random_hero) && seat.main_hero.is_none();
        let random = match self.shown_hero(map, c) {
            Hero::Random => true,
            Hero::Nobody => self.bugs == Bugs::Original && !none_at_all,
            Hero::Portrait {
                portrait,
                hero_type,
            } => {
                let name = self.hero_name(map, c, hero_type);
                return Some(self.own_hero_popup(portrait, hero_type, &name));
            }
        };
        let t = &self.texts;
        random.then(|| {
            let picture = t.look.hero.random.clone().unwrap_or_default();
            self.small_popup(
                &t.random_hero[0],
                Picture::Named(picture),
                &t.random,
                &t.random_hero[1],
            )
        })
    }

    /// A hero's own popup (`scenario_hero_popup` 0x575A90; design §25): GENRLTXT 78, the small
    /// portrait `portrait` the row shows (none past the table), "%s - %s" of `name` and the class
    /// of `hero_type`, GENRLTXT 79, `un44.def`'s frame of `hero_type` and its specialty's name.
    fn own_hero_popup(&self, portrait: u8, hero_type: u8, name: &str) -> MessageBox {
        let texts = &self.texts;
        let look = &texts.look.hero_popup;
        let [w, h] = look.size;
        let mut items =
            crate::message::frame((&look.background, &look.frame), [w.into(), h.into()]);
        let mut shown = Shown::default();
        let t = usize::from(hero_type);
        let line = texts
            .hero_line
            .replace(HERO_NAME, name)
            .replace(HERO_CLASS, &text_at(&texts.hero_classes, t));
        let mut label = |label: &Label, text: String, items: &mut Vec<Item>| {
            items.push(Item::Label(label.clone()));
            shown.texts.insert(label.id, text);
        };
        let [title, heading] = texts.hero_popup.clone();
        label(&look.title, title, &mut items);
        if let Some(picture) = texts.portraits.get(usize::from(portrait)) {
            items.push(Picture::Named(picture.clone()).item(POPUP_PICTURE, look.portrait));
        }
        label(&look.name, line, &mut items);
        label(&look.heading, heading, &mut items);
        items.push(Item::Def(Def {
            frame: u32::from(hero_type),
            ..look.specialty.clone()
        }));
        label(
            &look.specialty_name,
            text_at(&texts.specialties, t),
            &mut items,
        );
        MessageBox {
            dialog: Dialog {
                place: look.place,
                shadow: Some(look.size),
                items,
            },
            shown,
        }
    }

    /// The small popup (`scenario_info_popup` 0x5757E0, `scenario_bonus_popup` 0x5754F0): its
    /// title, picture, caption and text.
    fn small_popup(&self, title: &str, picture: Picture, caption: &str, text: &str) -> MessageBox {
        let look = &self.texts.look.popup;
        let [w, h] = look.size;
        let mut items =
            crate::message::frame((&look.background, &look.frame), [w.into(), h.into()]);
        let mut shown = Shown::default();
        for (label, text) in [
            (&look.title, title),
            (&look.caption, caption),
            (&look.text, text),
        ] {
            items.push(Item::Label(label.clone()));
            shown.texts.insert(label.id, String::from(text));
        }
        items.push(picture.item(POPUP_PICTURE, look.picture));
        MessageBox {
            dialog: Dialog {
                place: look.place,
                shadow: Some(look.size),
                items,
            },
            shown,
        }
    }

    /// Faction `t`'s popup (`scenario_town_popup` 0x575E60).
    fn town_popup(&self, t: u8) -> MessageBox {
        let texts = &self.texts;
        let look = &texts.look.town_popup;
        let [w, h] = look.size;
        let mut items =
            crate::message::frame((&look.background, &look.frame), [w.into(), h.into()]);
        let mut shown = Shown::default();
        let town = text_at(&texts.towns, usize::from(t));
        let labels = [
            (&look.title, texts.alignment.clone()),
            (&look.name, town),
            (&look.heading, texts.creatures.clone()),
        ];
        let def = texts.look.town.def.clone().unwrap_or_default();
        items.push(Item::Label(labels[0].0.clone()));
        shown.texts.insert(labels[0].0.id, labels[0].1.clone());
        items.push(Picture::Frame(def, 2 * u32::from(t) + 2).item(POPUP_PICTURE, look.picture));
        for (label, text) in &labels[1..] {
            items.push(Item::Label((*label).clone()));
            shown.texts.insert(label.id, text.clone());
        }
        let creatures = look
            .creatures
            .get(usize::from(t))
            .copied()
            .unwrap_or_default();
        let mut next = creatures.iter();
        let mut id = POPUP_CREATURES;
        for row in &look.rows {
            for (k, &creature) in (0i32..).zip(next.by_ref().take(row.count)) {
                let dx = look.step * k;
                items.push(Item::Def(Def {
                    id,
                    at: [row.at[0] + dx, row.at[1]],
                    frame: u32::from(creature) + 2,
                    ..look.creature.clone()
                }));
                items.push(Item::Label(Label {
                    id: id + 1,
                    at: [row.name[0] + dx, row.name[1]],
                    ..look.creature_name.clone()
                }));
                let name = texts
                    .creature_names
                    .iter()
                    .find(|(c, _)| *c == creature)
                    .map(|(_, n)| n.clone())
                    .unwrap_or_default();
                shown.texts.insert(id + 1, name);
                id += 2;
            }
        }
        MessageBox {
            dialog: Dialog {
                place: look.place,
                shadow: Some(look.size),
                items,
            },
            shown,
        }
    }

    /// The draws of the options after the dialog's items (`scenario_dialog_redraw` 0x584550): the
    /// duration's bar, then each row (`scenario_player_row_draw` 0x58D510). A load's bar stays
    /// disabled as made (E27 M4 T5b1: its enabling, vtable +0x24 0x597140, skipped at 0x581383),
    /// so its redraw darkens it after drawing it (`dialog_item_message` 0x5FE54B: a shown and
    /// disabled item's +0x20, 0x5FE800, `screen_darken` over its rectangle; recorded, `adv-load`
    /// advanced).
    pub(super) fn draws(
        &self,
        map: &Map,
        dialog: &Dialog,
        shown: &Shown,
        players: &str,
    ) -> Vec<Draw> {
        if !self.showing {
            return Vec::new();
        }
        let mut draws = self.bar.draws(dialog.place);
        if self.load {
            let look = self.bar.look();
            let [x, y] = [dialog.place[0] + look.at[0], dialog.place[1] + look.at[1]];
            draws.push(Draw::Darken {
                rect: [x, y, look.size[0], look.size[1]],
            });
        }
        for c in 0..8u8 {
            if let Some(row) = self.rows[usize::from(c)] {
                draws.extend(self.row_draws((map, c, row), dialog, shown, players));
            }
        }
        draws
    }

    /// Colour `c`'s row `row` (`scenario_player_row_draw` 0x58D510): its flag, its bar, who may
    /// play and the handicap; the town's arrows, picture and text; the hero's; the bonus's text,
    /// picture and arrows; the name.
    fn row_draws(
        &self,
        (map, c, row): (&Map, u8, i32),
        dialog: &Dialog,
        shown: &Shown,
        players: &str,
    ) -> Vec<Draw> {
        let seat = map.seats[usize::from(c)];
        let look = &self.texts.look;
        let record = self.record(c);
        let [px, py] = dialog.place;
        let dy = py + look.step * row;
        let items = |ids: &[u16]| {
            let ids: Vec<u16> = ids.iter().map(|id| id + u16::from(c)).collect();
            Dialog {
                place: dialog.place,
                shadow: None,
                items: dialog
                    .items
                    .iter()
                    .filter(|i| ids.contains(&i.id()))
                    .cloned()
                    .collect(),
            }
            .draws(shown, players)
        };
        let picture = |name: String, at: [i32; 2]| Draw::Picture {
            picture: name,
            x: px + at[0],
            y: dy + at[1],
            player: None,
            size: None,
        };
        let frame = |def: String, frame: u32, at: [i32; 2]| Draw::Button {
            def,
            frame,
            x: px + at[0],
            y: dy + at[1],
            mirrored: false,
            player: None,
        };
        let text = |label: &Label, text: String| Draw::Label {
            item: label.clone(),
            x: px + label.at[0],
            y: dy + label.at[1],
            text,
            disabled: false,
        };
        let t = &self.texts;
        let mut draws = items(&[FLAG]);
        draws.push(picture(
            look.panel.replace("{}", &letter(look, c)),
            look.panel_at,
        ));
        draws.extend(items(&[KIND, HANDICAP]));
        draws.extend(items(&[TOWN_LEFT, TOWN_RIGHT]));
        let town = self.shown_town(seat, c);
        match town {
            Some(f) => {
                let def = look.town.def.clone().unwrap_or_default();
                draws.push(frame(def, 2 * u32::from(f) + 2, look.town.at));
                draws.push(text(&look.town_name, text_at(&t.towns, usize::from(f))));
            }
            None => {
                let random = look.town.random.clone().unwrap_or_default();
                draws.push(picture(random, look.town.at));
                draws.push(text(&look.town_name, t.random.clone()));
            }
        }
        draws.extend(items(&[HERO_LEFT, HERO_RIGHT]));
        match self.shown_hero(map, c) {
            Hero::Random => {
                let random = look.hero.random.clone().unwrap_or_default();
                draws.push(picture(random, look.hero.at));
                draws.push(text(&look.hero_name, t.random.clone()));
            }
            Hero::Nobody => {
                let none = look.hero.none.clone().unwrap_or_default();
                draws.push(picture(none, look.hero.at));
                draws.push(text(&look.hero_name, t.none.clone()));
            }
            Hero::Portrait {
                portrait,
                hero_type,
            } => {
                let at = look.hero.at;
                let portrait = t.portraits.get(usize::from(portrait));
                draws.extend(portrait.map(|p| picture(p.clone(), at)));
                draws.push(text(&look.hero_name, self.hero_name(map, c, hero_type)));
            }
        }
        // (A resource under a Random town was made Random by `refresh`.)
        let bonus_name = match record.bonus {
            Bonus::Random => t.random.clone(),
            b => text_at(&t.bonuses, usize::from(b.code())),
        };
        draws.push(text(&look.bonus_name, bonus_name));
        let def = look.bonus.def.clone().unwrap_or_default();
        draws.push(frame(def, bonus_frame(record.bonus, town), look.bonus.at));
        draws.extend(items(&[BONUS_LEFT, BONUS_RIGHT]));
        draws.extend(items(&[PLAYER]));
        // The name's edit over it (0x58DEEE, single).
        draws.extend(self.edit_draws(c, dialog, shown, players));
        draws
    }

    /// Every asset `look` names (the adapter's batch at the shell's start).
    #[must_use]
    pub fn names(look: &AdvancedLook) -> Vec<String> {
        let mut names = Vec::new();
        for c in 0..8u8 {
            let letter = letter(look, c);
            for name in [&look.flag.def, &look.handicap.def, &look.panel] {
                names.push(name.replace("{}", &letter));
            }
        }
        for b in [
            &look.town_left,
            &look.town_right,
            &look.hero_left,
            &look.hero_right,
        ] {
            names.push(b.def.clone());
        }
        for p in [&look.town, &look.hero, &look.bonus] {
            names.extend(p.def.iter().chain(&p.random).chain(&p.none).cloned());
        }
        let labels = [
            &look.kind,
            &look.name,
            &look.edit,
            &look.town_name,
            &look.popup.title,
            &look.popup.caption,
            &look.town_popup.creature_name,
            &look.hero_popup.title,
            &look.hero_popup.name,
        ];
        for label in labels {
            names.push(label.font.clone());
            names.push(label.font_palette.clone());
        }
        names.push(look.bar.def.clone());
        names.push(look.bar.track.clone());
        for (background, frame) in [
            (&look.popup.background, &look.popup.frame),
            (&look.town_popup.background, &look.town_popup.frame),
            (&look.hero_popup.background, &look.hero_popup.frame),
        ] {
            names.push(background.clone());
            names.push(frame.clone());
        }
        names.extend(look.town_popup.creature.def.clone());
        names.extend(look.hero_popup.specialty.def.clone());
        names
    }
}

/// A popup's picture: a DEF's frame or a picture by name.
enum Picture {
    Frame(String, u32),
    Named(String),
}

impl Picture {
    /// Its item, id `id` at `at`.
    fn item(self, id: u16, at: [i32; 2]) -> Item {
        match self {
            Self::Frame(def, frame) => Item::Def(Def {
                id,
                def: Some(def),
                at,
                size: [48, 32],
                frame,
                mirrored: false,
                group: 0,
                creature: false,
                frames_of: None,
                repeat: None,
            }),
            Self::Named(picture) => Item::Picture(h3_ui::ui::Picture {
                id,
                picture,
                at,
                size: None,
                hint: None,
            }),
        }
    }
}

/// A popup's picture's id and its creatures' first (the frame's from 200).
const POPUP_PICTURE: u16 = 2;
const POPUP_CREATURES: u16 = 10;

/// The bonus's frame of `ScnrStar.def`: an artifact 9, gold 8, a resource the shown town's (Conflux
/// 3; Random 10 under a Random town), Random 10.
const fn bonus_frame(bonus: Bonus, town: Option<u8>) -> u32 {
    match (bonus, town) {
        (Bonus::Artifact, _) => 9,
        (Bonus::Gold, _) => 8,
        (Bonus::Resource, Some(8)) => 3,
        (Bonus::Resource, Some(t)) => t as u32,
        (Bonus::Resource | Bonus::Random, _) => 10,
    }
}

/// Whether the map offers colour `seat` a choice of town (its random-town flag, or more than one
/// faction).
const fn offered(seat: Seat) -> bool {
    seat.random_town || (seat.factions & 0x1FF).count_ones() > 1
}

/// The first faction of `factions` (0 when none).
const fn first_faction(factions: u16) -> u8 {
    let f = factions & 0x1FF;
    if f == 0 { 0 } else { f.trailing_zeros() as u8 }
}

/// The colour of flag `id` (263 to 270), none for another item.
fn flag_of(id: u16) -> Option<u8> {
    (FLAG..FLAG + 8)
        .contains(&id)
        .then(|| u8::try_from(id - FLAG).unwrap_or(0))
}

/// Colour `c`'s letter of `look`.
fn letter(look: &AdvancedLook, c: u8) -> String {
    look.letters
        .chars()
        .nth(usize::from(c))
        .map(String::from)
        .unwrap_or_default()
}

/// Item `id`'s state in `shown` changed by `change`.
fn set(shown: &mut Shown, id: u16, change: impl FnOnce(&mut State)) {
    let mut state = shown.state(id);
    change(&mut state);
    shown.states.insert(id, state);
}

/// The text `k` of `texts`, empty past them.
fn text_at(texts: &[String], k: usize) -> String {
    texts.get(k).cloned().unwrap_or_default()
}

#[cfg(test)]
pub(super) mod tests;
