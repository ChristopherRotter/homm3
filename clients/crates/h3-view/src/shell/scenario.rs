//! The scenario screen in its new-game mode (E27 M3 T1a; `docs/epics/E27-design.md` §3.1, §3.2,
//! §3.5, §3.8, §19): `scenario_dialog_create` 0x579960 with its argument 0, a single scenario,
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
//! *Not built yet:* the list and its panel (128's command: M3 T1b, which also reads the flags'
//! strip's teams dialog, 387, and the other items' popups), the difficulty's presses and Begin
//! (T2), the advanced options (129: T3), the random map (130: T5), a description too long for its
//! box (its scroll bar: T2), the hot seat's and the load's modes (the client's stub until T4 and
//! M4).

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use h3_render::Bugs;
use h3_ui::input::Input;
use h3_ui::ui::{Dialog, Draw, Shown, State};

use super::Ended;
use super::keys::{Command, Context};
use super::menu::{Did, Hovers};
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

/// The labels and captions whose text the screen sets as it opens, by id, with their names in
/// the key catalogue (`shell.scenario.<name>`): the constructor's texts (GENRLTXT's).
pub const LABELS: [(u16, &str); 12] = [
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
/// (CLT-Q90).
pub fn sort(entries: &mut [Entry]) {
    entries.sort_by(|a, b| {
        a.format
            .cmp(&b.format)
            .then_with(|| caseless(&a.name).cmp(caseless(&b.name)))
            .then_with(|| caseless(&a.file).cmp(caseless(&b.file)))
    });
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
    entries: Vec<Entry>,
    selected: Option<usize>,
    kept: String,
    difficulty: u8,
    background: u8,
    over: Over,
    hovers: Hovers,
    bugs: Bugs,
}

impl Scenario {
    /// The screen from the layout's `dialog`, its texts `texts` and the maps `entries` (in the
    /// list's order, [`sort`]), the kept map [`KEPT_MAP`] and difficulty [`KEPT_DIFFICULTY`], in
    /// the profile `bugs`.
    #[must_use]
    pub fn new(dialog: Dialog, texts: Texts, entries: Vec<Entry>, bugs: Bugs) -> Self {
        let mut shown = Shown::default();
        for (id, text) in &texts.labels {
            shown.texts.insert(*id, text.clone());
        }
        Self {
            dialog,
            shown,
            input: Input::new(bugs),
            texts,
            entries,
            selected: None,
            kept: String::from(KEPT_MAP),
            difficulty: KEPT_DIFFICULTY,
            background: 0,
            over: Over::Nothing,
            hovers: Hovers::default(),
            bugs,
        }
        .selecting()
    }

    /// The screen opened anew (`scenario_new` 0x4F0490), the menus' last items `hovers` carried
    /// for the menu after it, its background `gamselb0.pcx` until the caller draws
    /// ([`Scenario::with_background`]).
    #[must_use]
    pub fn opened(&self, hovers: Hovers) -> Self {
        let mut opened = Self::new(
            self.dialog.clone(),
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
        self.selected.and_then(|i| self.entries.get(i))
    }

    /// The maps listed, in the list's order.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
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

    /// The kept map's entry selected (else the first, none in an empty list) and shown.
    fn selecting(mut self) -> Self {
        let kept = self
            .entries
            .iter()
            .position(|e| e.file.eq_ignore_ascii_case(&self.kept));
        self.selected = kept.or_else(|| (!self.entries.is_empty()).then_some(0));
        self.show();
        self
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
        let shown = &mut self.shown;
        let Some(entry) = self.selected.and_then(|i| self.entries.get(i)) else {
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
        shown
            .texts
            .insert(DESCRIPTION, entry.description.clone());
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

    /// The screen's draw list as it is (the background is the caller's: [`Scenario::background`]).
    #[must_use]
    pub fn draws(&self, players: &str) -> Vec<Draw> {
        self.dialog.draws(&self.shown, players)
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

    /// The mouse moved to `point`: a button the mouse holds follows it (its sample as it comes
    /// back); nothing else shows a hover.
    pub fn moved(&mut self, point: [i32; 2]) -> Did {
        match self.over {
            Over::Nothing => Did::held(self.input.moved(&self.dialog, &mut self.shown, point)),
            Over::Help(_) => Did::default(),
        }
    }

    /// The left button pressed at `point`.
    pub fn press(&mut self, point: [i32; 2]) -> Did {
        match self.over {
            Over::Nothing => Did::held(self.input.press(&self.dialog, &mut self.shown, point)),
            Over::Help(_) => self.close_help(),
        }
    }

    /// The left button released.
    pub fn release(&mut self) -> Did {
        match self.over {
            Over::Nothing => {
                let did = self.input.release(&mut self.shown);
                clicked(did.changed, did.clicked)
            }
            Over::Help(_) => self.close_help(),
        }
    }

    /// The right button pressed at `point`: the help of the item there, unless the mouse holds a
    /// button.
    pub fn right(&mut self, point: [i32; 2]) -> Did {
        if self.over != Over::Nothing || self.input.holds_button() {
            return Did::default();
        }
        match self.item_at(point).filter(|id| self.texts.help.contains_key(id)) {
            Some(id) => {
                self.over = Over::Help(id);
                Did::CHANGED
            }
            None => Did::default(),
        }
    }

    /// The right button released: the popup closed; a button the mouse holds released as by the
    /// left button.
    pub fn right_release(&mut self) -> Did {
        match self.over {
            Over::Help(_) => self.close_help(),
            Over::Nothing => {
                let did = self.input.right_release(&mut self.shown);
                clicked(did.changed, did.clicked)
            }
        }
    }

    /// A key down: the key table's `command` for it, `by` its name (whose release clicks).
    pub fn key(&mut self, command: Option<Command>, by: &str) -> Did {
        match (&self.over, command) {
            (Over::Nothing, Some(Command::Dialog(id))) if self.has(id) => {
                Did::held(self.input.key(&mut self.shown, id, by))
            }
            _ => Did::default(),
        }
    }

    /// The key `by` released.
    pub fn key_up(&mut self, by: &str) -> Did {
        match self.over {
            Over::Nothing => {
                let did = self.input.key_up(&mut self.shown, by);
                clicked(did.changed, did.clicked)
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
        self.dialog.items.iter().any(|item| item.id() == id)
    }

    /// The item under the mouse at `point` (`dialog_item_at` 0x5FF3E0: the last in order whose
    /// box holds it, not hidden), the client's own items ([`OWN`]) passed over.
    fn item_at(&self, point: [i32; 2]) -> Option<u16> {
        let [x, y] = [
            point[0] - self.dialog.place[0],
            point[1] - self.dialog.place[1],
        ];
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

    /// The help popup closed.
    fn close_help(&mut self) -> Did {
        self.over = Over::Nothing;
        Did::CHANGED
    }
}

/// A hold ended (`changed`: a button went up) with the click `clicked`, if any: its command
/// (`scenario_dialog_click` 0x5865B0): Back ends the screen (0x7801); the others' commands are
/// later tasks' (the module's notes).
fn clicked(changed: bool, clicked: Option<u16>) -> Did {
    Did {
        changed,
        sample: false,
        ended: (clicked == Some(BACK)).then_some(Ended::Scenario(false)),
    }
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
mod tests;
