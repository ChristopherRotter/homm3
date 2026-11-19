//! The scenario screen's random map tab (E27 M3 T5a; `docs/epics/E27-design.md` §3.16, §30):
//! Random Map (130, `random_map_toggle` 0x57FEB0) hides the list and the advanced options and
//! shows `RanMapBk.pcx` (103) and the tab's items (280 to 335, `random_map_items_create`
//! 0x57D170), a second press hides them (`random_map_hide` 0x581D80); the list's showing and the
//! advanced options' hide them too.
//!
//! - **The choices** (the screen's +0x18A0 to +0x18BC, set by the constructor's call of
//!   `random_map_items_create`: each opening of the screen starts from them): the size (36, 72,
//!   108, 144; M first), two levels (285 toggles them, on first), the human or computer players
//!   (1 to 8, Random first), their teams (0 to 7, Random first), the computer only players (0 to
//!   7, 0 first), their teams (0 to 6, Random first), the water (none, normal, islands, Random
//!   first) and the monsters (weak, normal, strong, Random first). A press sets its choice
//!   (`scenario_dialog_click` 0x586DCC to 0x586F7A); a disabled button takes none.
//! - **The tab's state** (`random_map_tab_draw` 0x57EF70, after each press and in each redraw):
//!   each group's chosen button selected (state 0x10); the players' teams disabled (0x1000) from
//!   the players' count on (Random players as 8), the computer only players above 8 less the
//!   players (Random players as 1), their teams from the computer only players' count on (Random
//!   computer players as their cap); a choice above its cap shows the cap selected, the choice
//!   kept.
//! - **The own record** (`random_map_info_build` 0x580430; the screen's +0x38C): built at the
//!   tab's first opening (+0x37F, a random map chosen; the list's showing clears it) and again at
//!   each press of the players' count, of one of their teams' counts (not Random) and of the
//!   computer only players' count: the name GENRLTXT 741, the description 742, the size and the
//!   levels chosen, the map's difficulty Normal, the standard victory and loss, a human may play
//!   the colours below the players' count (8 for Random), anyone those below the players and
//!   the computer only players together (8 if either is Random), each colour its own team, every
//!   faction allowed, a random town. While chosen the screen shows it as the game's map
//!   (`scenario_to_game` 0x583580 and `scenario_entry_name` 0x584010 read it before the list's
//!   selection).
//!
//! Built at T5b: the items' right press and hints, Show Random Maps (335: the random maps' list,
//! kind 1, the client's empty: CLT-Q92), Begin with a random map chosen (CLT-Q92: the client's
//! stub's notice). *Not built yet:* the generation itself (decision 13). *Provisional* (the
//! agent's): what the record leaves of a header unread: each colour's main town generating its
//! hero (the player's +0xB set, design §30.2; *inferred* a main town: E27 M3 T7, the review's
//! C2), a random town and no main hero (T6 begins none). Each colour offered each faction's free
//! heroes (E27 M3 T7: `scenario_hero_lists` 0x583890 over the game the build resets: the faction's
//! first 16 hero types of its two classes; recorded, `adv-random-options`).

use alloc::string::String;
use alloc::vec::Vec;

use h3_render::Bugs;
use h3_ui::ui::{Dialog, Item, Shown, State};

use super::{Entry, Format, Heroes, Seat};

/// The tab's panel (`RanMapBk.pcx`).
pub const PANEL: u16 = 103;
/// The client's ids of what the redraw draws itself while the tab shows
/// (`scenario_dialog_redraw` 0x584B24): the title (GENRLTXT 739) and the line under it (740).
pub const TITLE: u16 = 1040;
/// The line under the title.
pub const SUBTITLE: u16 = 1041;
/// The labels (GENRLTXT 753 to 759): the size, the human or computer players, their teams, the
/// computer only players, their teams, the water, the monsters.
pub const LABELS: [u16; 7] = [280, 286, 296, 306, 316, 325, 330];
/// The sizes' buttons (`RanSizS`, `RanSizM`, `RanSizL`, `RanSizX.def`) by the side's length.
pub const SIZES: [(u16, u32); 4] = [(281, 36), (282, 72), (283, 108), (284, 144)];
/// The two levels' switch (`RanUndr.def`).
pub const LEVELS: u16 = 285;
/// The human or computer players' buttons: 1 to 8 (`RanNum1` to `RanNum8.def`) from here, then
/// Random (`RanRand.def`, [`PLAYERS`] + 8).
pub const PLAYERS: u16 = 287;
/// Their teams' buttons: 0 to 7 from here, then Random (+ 8).
pub const TEAMS: u16 = 297;
/// The computer only players' buttons: 0 to 7 from here, then Random (+ 8).
pub const COMPUTERS: u16 = 307;
/// Their teams' buttons: 0 to 6 from here, then Random (+ 7).
pub const COMPUTER_TEAMS: u16 = 317;
/// The water's buttons: none, normal, islands (`RanNone`, `RanNorm`, `RanIsld.def`), Random.
pub const WATER: u16 = 326;
/// The monsters' buttons: weak, normal, strong (`RanWeak`, `RanNorm`, `RanStrg.def`), Random.
pub const MONSTERS: u16 = 331;
/// Show Random Maps (`RanShow.def`).
pub const SHOW: u16 = 335;
/// The human or computer players' teams 1 and 2 (`RanNum1`, `RanNum2.def`), made 30 × 24 over
/// their DEFs' 30 × 32 frames (0x57D85B, 0x57D8A9): their lowest 8 rows take no press in the
/// original's (CLT-Q146); the client's profile makes them 32 high like every other count (E27 M3
/// T5b: their frames are buttons drawn to their last row).
pub const SHORT: [u16; 2] = [TEAMS + 1, TEAMS + 2];
/// The height every count's button takes the mouse over in the client's profile (its frames').
pub const COUNT_HEIGHT: i32 = 32;

/// The tab's items with a right press's popup (E27 M3 T5b), by id, with their names in the key
/// catalogue (`shell.scenario.help.<name>`: `Help.txt`'s record id − 83, 197 to 252, its
/// right-click text; `dialog_items_set_hints` gives every item of 104 to 344 its record): the
/// labels, the sizes, the levels, each count, the water's, the monsters' and Show Random Maps.
pub const HELPED: [(u16, &str); 56] = [
    (280, "random-size-label"),
    (281, "random-size-s"),
    (282, "random-size-m"),
    (283, "random-size-l"),
    (284, "random-size-xl"),
    (LEVELS, "random-levels"),
    (286, "random-players-label"),
    (287, "random-players-1"),
    (288, "random-players-2"),
    (289, "random-players-3"),
    (290, "random-players-4"),
    (291, "random-players-5"),
    (292, "random-players-6"),
    (293, "random-players-7"),
    (294, "random-players-8"),
    (295, "random-players-random"),
    (296, "random-teams-label"),
    (297, "random-teams-0"),
    (298, "random-teams-1"),
    (299, "random-teams-2"),
    (300, "random-teams-3"),
    (301, "random-teams-4"),
    (302, "random-teams-5"),
    (303, "random-teams-6"),
    (304, "random-teams-7"),
    (305, "random-teams-random"),
    (306, "random-computers-label"),
    (307, "random-computers-0"),
    (308, "random-computers-1"),
    (309, "random-computers-2"),
    (310, "random-computers-3"),
    (311, "random-computers-4"),
    (312, "random-computers-5"),
    (313, "random-computers-6"),
    (314, "random-computers-7"),
    (315, "random-computers-random"),
    (316, "random-computer-teams-label"),
    (317, "random-computer-teams-0"),
    (318, "random-computer-teams-1"),
    (319, "random-computer-teams-2"),
    (320, "random-computer-teams-3"),
    (321, "random-computer-teams-4"),
    (322, "random-computer-teams-5"),
    (323, "random-computer-teams-6"),
    (324, "random-computer-teams-random"),
    (325, "random-water-label"),
    (326, "random-water-none"),
    (327, "random-water-normal"),
    (328, "random-water-islands"),
    (329, "random-water-random"),
    (330, "random-monsters-label"),
    (331, "random-monsters-weak"),
    (332, "random-monsters-normal"),
    (333, "random-monsters-strong"),
    (334, "random-monsters-random"),
    (SHOW, "random-show"),
];

/// The tab's choices (the screen's +0x18A0 to +0x18BC); none is Random.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Choices {
    /// The side's length in tiles (+0x18A0).
    pub size: u32,
    /// The levels, 1 or 2 (+0x18A4).
    pub levels: u8,
    /// The human or computer players, 1 to 8 (+0x18A8).
    pub players: Option<u8>,
    /// Their teams, 0 to 7 (+0x18AC).
    pub teams: Option<u8>,
    /// The computer only players, 0 to 7 (+0x18B0).
    pub computers: Option<u8>,
    /// Their teams, 0 to 6 (+0x18B4).
    pub computer_teams: Option<u8>,
    /// The water: 0 none, 1 normal, 2 islands (+0x18B8, the original's 3 for Random).
    pub water: Option<u8>,
    /// The monsters: 0 weak, 1 normal, 2 strong (+0x18BC).
    pub monsters: Option<u8>,
}

impl Default for Choices {
    /// The constructor's (`random_map_items_create` 0x57D192 to 0x57D1D0).
    fn default() -> Self {
        Self {
            size: 72,
            levels: 2,
            players: None,
            teams: None,
            computers: Some(0),
            computer_teams: None,
            water: None,
            monsters: None,
        }
    }
}

impl Choices {
    /// The players' count the caps read: Random as `random` (8 for the teams' cap, 1 for the
    /// computer only players').
    fn players_or(self, random: u8) -> u8 {
        self.players.unwrap_or(random)
    }

    /// The computer only players' cap: 8 less the players (Random as 1).
    fn computers_cap(self) -> u8 {
        8u8.saturating_sub(self.players_or(1))
    }

    /// The computer only players' count their teams' cap reads: the choice up to its cap, Random
    /// as the cap.
    fn computers_shown(self) -> u8 {
        let cap = self.computers_cap();
        self.computers.map_or(cap, |c| c.min(cap))
    }

    /// The colours a human may play (the record's +8): below the players' count, 8 for Random.
    #[must_use]
    pub fn humans(self) -> u8 {
        self.players_or(8)
    }

    /// The colours anyone plays (the record's +6): below the players and the computer only
    /// players together, 8 if either is Random (`random_map_info_build` 0x5805DD).
    #[must_use]
    pub fn total(self) -> u8 {
        match (self.players, self.computers) {
            (Some(p), Some(c)) => p.saturating_add(c),
            _ => 8,
        }
    }
}

/// What a press of the tab did (`scenario_dialog_click`'s cases 281 to 334).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Pressed {
    /// A choice changed, the screen redrawn.
    Chosen,
    /// A count changed: the own record built again (`random_map_info_build` 0x580430).
    Counted,
}

/// The random map tab of the scenario screen.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct RandomMap {
    choices: Choices,
    /// The tab shows (+0x37E).
    showing: bool,
    /// A random map is chosen (+0x37F): the screen shows its own record.
    chosen: bool,
    /// The own record as last built (+0x38C).
    record: Option<Entry>,
}

impl RandomMap {
    /// Whether the tab shows.
    pub(super) const fn showing(&self) -> bool {
        self.showing
    }

    /// Whether a random map is chosen.
    pub(super) const fn chosen(&self) -> bool {
        self.chosen
    }

    /// The choices.
    pub(super) const fn choices(&self) -> Choices {
        self.choices
    }

    /// No random map chosen any longer (the list's showing, `scenario_list_toggle` 0x580AAA).
    pub(super) const fn unchoose(&mut self) {
        self.chosen = false;
    }

    /// Whether item `id` is the tab's.
    pub(super) const fn owns(id: u16) -> bool {
        matches!(id, PANEL | 280..=SHOW | TITLE | SUBTITLE)
    }

    /// Every item of the tab.
    fn ids() -> impl Iterator<Item = u16> {
        [PANEL, TITLE, SUBTITLE].into_iter().chain(280..=SHOW)
    }

    /// The teams' 1 and 2 of `dialog` as the profile `bugs` takes the mouse over them
    /// (CLT-Q146, E27 M3 T5b): the original's 24 rows ([`SHORT`]), the client's
    /// [`COUNT_HEIGHT`].
    pub(super) fn heights(dialog: &mut Dialog, bugs: Bugs) {
        if bugs != Bugs::Fixed {
            return;
        }
        for item in &mut dialog.items {
            if let Item::Button(b) = item
                && SHORT.contains(&b.id)
            {
                b.size[1] = COUNT_HEIGHT;
            }
        }
    }

    /// Every item of the tab hidden in `shown` (the constructor's).
    pub(super) fn hide_items(shown: &mut Shown) {
        for id in Self::ids() {
            set(shown, id, |s| s.hidden = true);
        }
    }

    /// Shown (`random_map_toggle` 0x57FEE6: 103 and 280 to 335 shown, +0x37E set) and its state.
    pub(super) fn show(&mut self, shown: &mut Shown) {
        self.showing = true;
        for id in Self::ids() {
            set(shown, id, |s| s.hidden = false);
        }
        self.states(shown);
    }

    /// Hidden (`random_map_hide` 0x581D80: 103 and 280 to 335 hidden, +0x37E cleared).
    pub(super) fn hide(&mut self, shown: &mut Shown) {
        self.showing = false;
        Self::hide_items(shown);
    }

    /// The tab's state (`random_map_tab_draw` 0x57EF70; nothing while hidden): each group's choice
    /// selected, the counts above their caps disabled.
    pub(super) fn states(&self, shown: &mut Shown) {
        if !self.showing {
            return;
        }
        let c = self.choices;
        for &(id, size) in &SIZES {
            set(shown, id, |s| s.selected = size == c.size);
        }
        set(shown, LEVELS, |s| s.selected = c.levels != 1);
        // The players: 1 to 8, Random.
        group(
            shown,
            PLAYERS,
            9,
            c.players.map_or(8, |p| p.saturating_sub(1)),
        );
        // Their teams: 0 always enabled, 1 up to the players' count less one (Random as 8); the
        // choice shown up to the count (0x57F0C6: the count itself, a disabled button, past it).
        let players = c.players_or(8);
        enabled(shown, TEAMS, 8, players);
        group(shown, TEAMS, 9, c.teams.map_or(8, |t| t.min(players)));
        // The computer only players: 0 to 8 less the players (Random as 1).
        let cap = c.computers_cap();
        enabled(shown, COMPUTERS, 8, cap + 1);
        group(shown, COMPUTERS, 9, c.computers.map_or(8, |n| n.min(cap)));
        // Their teams: 0 always, 1 up to the computer players shown less one.
        let computers = c.computers_shown();
        enabled(shown, COMPUTER_TEAMS, 7, computers);
        let teams = c.computer_teams.map_or(7, |t| t.min(computers));
        group(shown, COMPUTER_TEAMS, 8, teams);
        group(shown, WATER, 4, c.water.unwrap_or(3));
        group(shown, MONSTERS, 4, c.monsters.unwrap_or(3));
    }

    /// Item `id` pressed (`scenario_dialog_click` 0x586DCC to 0x586F7A): its choice set and what
    /// the press did; none for an item that is not one of the choices (Show Random Maps is the
    /// screen's, T5b: its list).
    pub(super) fn click(&mut self, id: u16) -> Option<Pressed> {
        let c = &mut self.choices;
        let offset = |first: u16| u8::try_from(id - first).unwrap_or(0);
        let pressed = match id {
            281..=284 => {
                c.size = SIZES
                    .iter()
                    .find(|&&(b, _)| b == id)
                    .map_or(c.size, |&(_, s)| s);
                Pressed::Chosen
            }
            LEVELS => {
                c.levels = 3 - c.levels;
                Pressed::Chosen
            }
            287..=294 => {
                c.players = Some(offset(PLAYERS) + 1);
                Pressed::Counted
            }
            295 => {
                c.players = None;
                Pressed::Counted
            }
            297..=304 => {
                c.teams = Some(offset(TEAMS));
                Pressed::Counted
            }
            // Random teams build no record (0x586E94).
            305 => {
                c.teams = None;
                Pressed::Chosen
            }
            307..=314 => {
                c.computers = Some(offset(COMPUTERS));
                Pressed::Counted
            }
            315 => {
                c.computers = None;
                Pressed::Counted
            }
            317..=323 => {
                c.computer_teams = Some(offset(COMPUTER_TEAMS));
                Pressed::Chosen
            }
            324 => {
                c.computer_teams = None;
                Pressed::Chosen
            }
            326..=328 => {
                c.water = Some(offset(WATER));
                Pressed::Chosen
            }
            329 => {
                c.water = None;
                Pressed::Chosen
            }
            331..=333 => {
                c.monsters = Some(offset(MONSTERS));
                Pressed::Chosen
            }
            334 => {
                c.monsters = None;
                Pressed::Chosen
            }
            _ => return None,
        };
        Some(pressed)
    }

    /// The own record while a random map is chosen.
    pub(super) fn record(&self) -> Option<&Entry> {
        self.record.as_ref().filter(|_| self.chosen)
    }

    /// The own record built from the choices (`random_map_info_build` 0x580430), named `name`
    /// and described as `description`, each colour offered each faction's `offers`, and a random
    /// map chosen; the record.
    pub(super) fn build(&mut self, name: &str, description: &str, offers: &[Vec<u8>; 9]) -> &Entry {
        self.chosen = true;
        let made = self.made(name, description, offers);
        self.record.insert(made)
    }

    /// The own record of the choices.
    fn made(&self, name: &str, description: &str, offers: &[Vec<u8>; 9]) -> Entry {
        let c = self.choices;
        let (humans, total) = (c.humans(), c.total());
        let seats = core::array::from_fn(|k| {
            let k = u8::try_from(k).unwrap_or(u8::MAX);
            // The generate flag set (+0xB, design §30.2): the row shows the hero Random, its
            // arrows once a town is chosen (§3.14; E27 M3 T7, the review's C2); no main hero,
            // *provisional* (the module's notes).
            Seat {
                human: k < humans,
                computer: k < total,
                factions: 0x1FF,
                random_town: true,
                generates: true,
                ..Seat::default()
            }
        });
        Entry {
            file: String::new(),
            // The single player's version (+0x1898) is SoD's (0x5805AA).
            format: Some(Format::SoD),
            name: String::from(name),
            description: String::from(description),
            size: c.size,
            difficulty: 1,
            victory: None,
            standard: false,
            loss: None,
            seats,
            teams: [0, 1, 2, 3, 4, 5, 6, 7],
            // Each faction's free heroes (E27 M3 T7: `scenario_hero_lists` over the reset game,
            // recorded by `adv-random-options`: Castle's first hero Orrin).
            heroes: Heroes {
                offers: core::array::from_fn(|_| offers.clone()),
                ..Heroes::default()
            },
        }
    }
}

/// The group of `count` buttons from `first` with the one at `chosen` selected, the others not.
fn group(shown: &mut Shown, first: u16, count: u8, chosen: u8) {
    for k in 0..count {
        set(shown, first + u16::from(k), |s| s.selected = k == chosen);
    }
}

/// Of the `count` buttons from `first`, those from 1 below `enabled` enabled, the rest from it on
/// disabled (0 is never touched: always enabled).
fn enabled(shown: &mut Shown, first: u16, count: u8, enabled: u8) {
    for k in 1..count {
        set(shown, first + u16::from(k), |s| s.disabled = k >= enabled);
    }
}

/// Item `id`'s state in `shown` changed by `change`.
fn set(shown: &mut Shown, id: u16, change: impl FnOnce(&mut State)) {
    let mut state = shown.state(id);
    change(&mut state);
    shown.states.insert(id, state);
}

#[cfg(test)]
mod tests;
