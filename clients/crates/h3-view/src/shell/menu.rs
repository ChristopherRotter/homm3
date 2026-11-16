//! The shell's menus taking their inputs (`docs/epics/E27-design.md` §2.3, §2.4, §13, §14): the
//! main menu (E27 M2 T6a) and New Game's and Load Game's menus (one dialog, its argument the game:
//! `game_type_menu_create` 0x4D54C0; E27 M2 T6b1), each five DEF buttons' hover, press and release
//! (`h3_ui::input`, decision 20), their hotkeys ([`super::keys`]) and the right press's help popup;
//! the main menu's Quit asks its question.
//!
//! - **The hover** (`main_menu_proc` 0x4FB710, `game_type_menu_proc` 0x4D5940; a move, type 4):
//!   the item under the mouse (`dialog_item_id_at` 0x5FF3B0: a game type menu's title picture
//!   too) against the last (`main_menu_hovered` 0x67FA60, `game_type_menu_hovered` 0x6780F0); on
//!   a change the five buttons lose their state 0x10 and the new item gains it (a button drawn in
//!   its frame 3, `h3_ui::ui::Button::frame`; the picture's draws nothing new). A button the mouse
//!   holds runs its own loop (`def_button_message` 0x45641C): the moves are its own, the hover
//!   does not follow them. *Recorded, not read* (`menu-main`'s and `menu-new`'s `*-left`,
//!   `menu-quit`'s `cancelled`): once a hold ends (the release, a key's release) and once the
//!   question closes, the hover is the item under the pointer again, as at a move there. A menu
//!   opens with none hovered, the pointer where it is (`menu-new`'s `menu`: over Single). The
//!   last item is a static each procedure alone writes, never cleared as its menu opens again
//!   (E27 M2 T7: `edit-keys`' `reopened`, Multiplayer chosen again after the dialog's Escape stays
//!   in frame 0): an item the pointer left the menu over is no change when the menu shows again.
//!   **Fixed** (CLT-Q128, decision 40): a menu opening anew takes the item under the pointer at
//!   its first move; the original's statics ([`Hovers`]) for the comparisons only.
//! - **A press** (`button_press` 0x456860): the button drawn pressed with its sample
//!   (`button.wav`, [`Did::sample`]); the command at the release over it (0x200/0xD), none off it;
//!   a hotkey's key down presses it, its key up clicks it.
//! - **The commands:** the main menu's New Game, Load Game, High Scores and Credits end it with
//!   their id (0x4FB9BF); its Quit asks GENRLTXT 70 in a box of style 2 (the videos paused
//!   meanwhile: nothing shown, CLT-Q86): its OK ends the menu with Quit, its Cancel stays. A game
//!   type menu's five end it with their id (0x4D59DF), Back among them.
//! - **A right press** on a button (the message's +0xC bit 0x200, subtype 0xE, the button not
//!   going down): a popup of style 4 with the button's `Help.txt` right-click text
//!   (`main_menu_help` 0x6A6C20: records 3 to 7; `game_type_menu_help` 0x6A6BF8: records 10 to 14
//!   by id − 100, Load Game's the same), shown while the right button is held (`popup_run_held`
//!   0x602A40): the cursor hidden, the moves and keys dropped, a left press or release, or the
//!   right release, closing it. None while the mouse holds a button (its own loop drops it).
//!
//! - **The redraws** (recorded, `menu-main`'s `load-press`): a press, a hold's move off or on and
//!   a release redraw the button alone and show its box whole (`button_press`, 0x4563BB,
//!   0x456467), the hover's change all five in their order (0x4FBAEB, 0x4D5AC6: the dialog's
//!   items in the order it added them) and shows only the menu's rectangles ([`Menu::new`]'s
//!   `redrawn`: the main menu's (520, 4, 250, 567), a game type menu's (526, 7, 250, 560) and its
//!   title's (114, 312, 300, 48)). Load Game's frames reach two rows into High Scores', so a
//!   pressed Load Game shows over High Scores until the next hover change; Single's top rows 4
//!   to 6 and Back's bottom rows 567 to 569 lie outside a game type menu's rectangle, so they keep
//!   the frame the button's own loop last showed whole (`menu-new`, `menu-load`). The client draws
//!   the five in their order always and every button whole (CLT-Q126, decision 36; CLT-Q118,
//!   decision 22: fixed); the original's ([`Bugs::Original`]) draws the button redrawn alone last
//!   and the rows outside the rectangles as last shown ([`Menu::stale`]), for the comparisons
//!   only.
//!
//! - **The stubs** (E27 M2 T6b2, decision 37; [`STUBS`]): New Game's Campaign and Tutorial (until
//!   E20 and the engine's tutorial mode, decisions 7 and 9) and the main menu's High Scores and
//!   Credits (until M4) end their menu as the original's do, and their screen shows that menu
//!   opened anew with a notice over it, a box of style 1 in the client's own words (its language
//!   pack's `shell.stub.<screen>`, by the driver's names: `campaigns`, `tutorial`,
//!   `high-scores`, `credits`); the notice takes the box's inputs as Quit's question does, and
//!   its OK ends the screen (back to New Game's menu, or to the main menu), which opens anew.
//!
//! The boxes are laid out by the caller ([`Boxes`]: their texts are the user's tables', the
//! notices' the client's), framed in red (the viewing player at the menus, recorded at M1 T2:
//! `menu-quit`, `menu-main`).
//! *Not built* (design §13, §14): the first pass's disk check (GENRLTXT 708) and the CD notice
//! (a game type menu then without Single, Campaign and Tutorial), a DirectPlay lobby's start; the
//! menus' clip 33 draws nothing (CLT-Q86).

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use h3_render::Bugs;
use h3_ui::input::{Held, Input};
use h3_ui::ui::{Dialog, Draw, Item, Shown, State};

use super::keys::{Command, Context};
use super::multiplayer::Multiplayer;
use super::{Choice, Ended, Game, Kind as ScenarioKind, Request, Screen};
use crate::message::{self, MessageBox, Open, Outcome};

/// The player whose colours frame the menus' boxes: the viewing player there, red (0).
pub const BOX_PLAYER: u8 = 0;

/// The main menu's buttons, by id (101 to 105).
pub const BUTTONS: [u16; 5] = [101, 102, 103, 104, 105];
/// Quit's button.
pub const QUIT: u16 = 105;
/// The game type menus' buttons, by id (100 to 104: Single, Campaign, Multiplayer, Tutorial,
/// Back; `game_type_menu_proc` clears the hover of these).
pub const GAME_TYPE_BUTTONS: [u16; 5] = [100, 101, 102, 103, 104];
/// A game type menu's title picture (105: `newgame.pcx` or `loadgame.pcx`).
pub const TITLE: u16 = 105;

/// The client's stubs (E27 M2 T6b2, T7): each screen, the menu its notice shows over and the
/// notice's OK's ending. The scenario screen's (until M3) shows over the game type menu it was
/// reached from (by Single, Campaign, Tutorial or the Multiplayer dialog), its OK back there.
pub const STUBS: [(Screen, Kind, Ended); 10] = [
    (Screen::Campaigns, Kind::GameType(Game::New), Ended::Back),
    (Screen::Tutorial, Kind::GameType(Game::New), Ended::Back),
    (Screen::HighScores, Kind::Main, Ended::Done),
    (Screen::Credits, Kind::Main, Ended::Done),
    scenario(Game::New, ScenarioKind::Single),
    scenario(Game::New, ScenarioKind::Multiplayer),
    scenario(Game::Load, ScenarioKind::Single),
    scenario(Game::Load, ScenarioKind::Multiplayer),
    scenario(Game::Load, ScenarioKind::Campaign),
    scenario(Game::Load, ScenarioKind::Tutorial),
];

/// The scenario screen's stub of `game` and `kind`.
const fn scenario(game: Game, kind: ScenarioKind) -> (Screen, Kind, Ended) {
    (
        Screen::Scenario(game, kind),
        Kind::GameType(game),
        Ended::Scenario(false),
    )
}

/// A stub's name in its notice's key (`shell.stub.<name>`): the driver's name of its screen
/// ([`super::text::name`]), `scenario` for each of the scenario screen's.
#[must_use]
pub fn stub_name(screen: Screen) -> alloc::string::String {
    match screen {
        Screen::Scenario(..) => alloc::string::String::from("scenario"),
        _ => super::text::name(screen),
    }
}

/// Which menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The main menu (§2.3).
    Main,
    /// New Game's or Load Game's menu (§2.4).
    GameType(Game),
}

impl Kind {
    /// Its buttons (their hover cleared on a change).
    #[must_use]
    pub const fn buttons(self) -> [u16; 5] {
        match self {
            Self::Main => BUTTONS,
            Self::GameType(_) => GAME_TYPE_BUTTONS,
        }
    }

    /// Its screen.
    #[must_use]
    pub const fn screen(self) -> Screen {
        match self {
            Self::Main => Screen::MainMenu,
            Self::GameType(game) => Screen::GameType(game),
        }
    }
}

/// The boxes a menu shows, laid out ([`message::layout`]): the main menu's Quit's question
/// (GENRLTXT 70, style 2; none for a game type menu, and a main menu without it quits at once, as
/// a lobby's start does) and each button's help (its `Help.txt` right-click text, style 4), by id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Boxes {
    /// Quit's question.
    pub question: Option<MessageBox>,
    /// The buttons' help popups.
    pub help: BTreeMap<u16, MessageBox>,
}

/// Rows of the screen a menu's redraw does not show, in the original's profile (CLT-Q118): they
/// keep a button as it was last shown whole.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stale {
    /// The rectangles (left, top, width, height) on the screen.
    pub rects: Vec<[i32; 4]>,
    /// The button as last shown whole, to draw over the menu's background and keep inside them.
    pub draws: Vec<Draw>,
}

/// What is open over the menu.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Over {
    Nothing,
    /// A button's help popup, while the right button is held.
    Help(u16),
    /// Quit's question.
    Question(Box<Open>),
    /// A stub's notice (E27 M2 T6b2): its OK ends the stub's screen.
    Notice(Box<Open>),
}

/// What an input did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Did {
    /// What the menu shows changed: draw it again.
    pub changed: bool,
    /// A button went down: its press sample (`button.wav`).
    pub sample: bool,
    /// The menu ended so.
    pub ended: Option<Ended>,
}

impl Did {
    /// Drawn again, nothing else.
    pub(crate) const CHANGED: Self = Self {
        changed: true,
        sample: false,
        ended: None,
    };

    fn with(self, other: Self) -> Self {
        Self {
            changed: self.changed || other.changed,
            sample: self.sample || other.sample,
            ended: self.ended.or(other.ended),
        }
    }
}

/// A menu on the screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Menu {
    kind: Kind,
    /// The screen it shows on: its kind's, or a stub's under its notice.
    screen: Screen,
    dialog: Dialog,
    /// The rectangles the hover's redraw shows.
    redrawn: Vec<[i32; 4]>,
    shown: Shown,
    input: Input,
    hovered: Option<u16>,
    /// Both procedures' statics as they are (this one's `hovered`).
    hovers: Hovers,
    pointer: [i32; 2],
    over: Over,
    boxes: Boxes,
    bugs: Bugs,
    /// The button redrawn alone since the five were last (the original's profile draws it last).
    top: Option<u16>,
    /// Each button's state when its box was last shown whole (the opening's, else its own loop's
    /// draws): the rows outside `redrawn` keep it (the original's profile).
    whole: BTreeMap<u16, State>,
}

/// The menus' last items under the mouse (`main_menu_hovered` 0x67FA60, `game_type_menu_hovered`
/// 0x6780F0, New Game's and Load Game's one procedure): statics their procedures alone write,
/// carried from a menu to the next (E27 M2 T7; CLT-Q128: read by the original's profile alone).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Hovers {
    /// The main menu's.
    pub main: Option<u16>,
    /// The game type menus'.
    pub game_type: Option<u16>,
}

impl Hovers {
    /// The static of `kind`'s procedure.
    const fn of(self, kind: Kind) -> Option<u16> {
        match kind {
            Kind::Main => self.main,
            Kind::GameType(_) => self.game_type,
        }
    }
}

/// The button a hold names.
const fn held_id(held: &Held) -> u16 {
    match held {
        Held::Button { id, .. } | Held::Picture { id, .. } | Held::Key { id, .. } => *id,
    }
}

impl Menu {
    /// The menu `kind` as it opens (`main_menu_create`, `game_type_menu_create`: none hovered,
    /// pressed or disabled), its dialog the layout's, its hover's redraw showing the screen's
    /// rectangles `redrawn` (left, top, width, height), the pointer at `pointer` (taken at the
    /// first move), in the profile `bugs`.
    #[must_use]
    pub fn new(
        kind: Kind,
        dialog: Dialog,
        redrawn: Vec<[i32; 4]>,
        boxes: Boxes,
        pointer: [i32; 2],
        bugs: Bugs,
    ) -> Self {
        Self {
            kind,
            screen: kind.screen(),
            dialog,
            redrawn,
            shown: Shown::default(),
            input: Input::new(bugs),
            hovered: None,
            hovers: Hovers::default(),
            pointer,
            over: Over::Nothing,
            boxes,
            bugs,
            top: None,
            whole: BTreeMap::new(),
        }
    }

    /// The menu opened again with the pointer at `pointer`, nothing held or open, none drawn
    /// hovered; in the original's profile its procedure's last item `hovers`' (CLT-Q128: fixed,
    /// none in the client's).
    #[must_use]
    pub fn opened(&self, pointer: [i32; 2], hovers: Hovers) -> Self {
        let mut menu = Self::new(
            self.kind,
            self.dialog.clone(),
            self.redrawn.clone(),
            self.boxes.clone(),
            pointer,
            self.bugs,
        );
        if self.bugs == Bugs::Original {
            menu.hovered = hovers.of(self.kind);
        }
        menu.hovers = hovers;
        menu
    }

    /// The menus' last items under the mouse, as this one leaves them.
    #[must_use]
    pub const fn hovers(&self) -> Hovers {
        self.hovers
    }

    /// The menu as the stub `screen` shows it (E27 M2 T6b2): `notice` open over it, its OK ending
    /// the screen as [`STUBS`] says.
    fn stub(mut self, screen: Screen, notice: MessageBox) -> Self {
        self.screen = screen;
        self.over = Over::Notice(Box::new(Open::new(notice)));
        self
    }

    /// Which menu it is.
    #[must_use]
    pub const fn kind(&self) -> Kind {
        self.kind
    }

    /// The screen it shows on: its kind's ([`Kind::screen`]), or a stub's under the stub's notice
    /// ([`STUBS`]).
    #[must_use]
    pub const fn screen(&self) -> Screen {
        self.screen
    }

    /// Where the menu takes its keys: itself, or the box over it (the popup takes none).
    #[must_use]
    pub const fn context(&self) -> Context {
        match (&self.over, self.kind) {
            (Over::Question(_) | Over::Notice(_), _) => Context::Question,
            (Over::Nothing | Over::Help(_), Kind::Main) => Context::MainMenu,
            (Over::Nothing | Over::Help(_), Kind::GameType(_)) => Context::GameType,
        }
    }

    /// The item hovered (its state 0x10), if any.
    #[must_use]
    pub const fn hovered(&self) -> Option<u16> {
        self.hovered
    }

    /// Whether the cursor shows (the help popup hides it).
    #[must_use]
    pub const fn cursor(&self) -> bool {
        !matches!(self.over, Over::Help(_))
    }

    /// Whether Quit's question is open.
    #[must_use]
    pub const fn asking(&self) -> bool {
        matches!(self.over, Over::Question(_))
    }

    /// The dialog's draw list: the items as they are (in the original's profile the button
    /// redrawn alone last); a player's colours from the players' palette `players`.
    #[must_use]
    pub fn dialog_draws(&self, players: &str) -> Vec<Draw> {
        let mut dialog = self.dialog.clone();
        if self.bugs == Bugs::Original
            && let Some(at) =
                (self.top).and_then(|id| dialog.items.iter().position(|i| i.id() == id))
        {
            let item = dialog.items.remove(at);
            dialog.items.push(item);
        }
        dialog.draws(&self.shown, players)
    }

    /// The box open over the menu with its shadow, if any.
    #[must_use]
    pub fn box_draws(&self, players: &str) -> Vec<Draw> {
        let open = match &self.over {
            Over::Nothing => None,
            Over::Help(id) => self.boxes.help.get(id),
            Over::Question(open) | Over::Notice(open) => Some(&open.message),
        };
        open.map(|m| m.dialog.draws(&m.shown, players))
            .unwrap_or_default()
    }

    /// The draw list: the dialog's, then the box open over it ([`Menu::dialog_draws`],
    /// [`Menu::box_draws`]; the stale rows between them, [`Menu::stale`]).
    #[must_use]
    pub fn draws(&self, players: &str) -> Vec<Draw> {
        let mut list = self.dialog_draws(players);
        list.extend(self.box_draws(players));
        list
    }

    /// In the original's profile, each button whose box reaches outside the redraw's rectangles
    /// and was last shown whole in another frame than it is drawn in now: those rows keep that
    /// frame (CLT-Q118); none in the client's.
    #[must_use]
    pub fn stale(&self, players: &str) -> Vec<Stale> {
        if self.bugs != Bugs::Original {
            return Vec::new();
        }
        let [px, py] = self.dialog.place;
        self.dialog
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
                let at = [px + b.at[0], py + b.at[1], b.size[0], b.size[1]];
                let rects = outside(at, &self.redrawn);
                if rects.is_empty() {
                    return None;
                }
                let mut shown = Shown::default();
                shown.states.insert(b.id, state);
                let alone = Dialog {
                    items: vec![item.clone()],
                    ..self.dialog.clone()
                };
                Some(Stale {
                    rects,
                    draws: alone.draws(&shown, players),
                })
            })
            .collect()
    }

    /// The mouse moved to `point`.
    pub fn moved(&mut self, point: [i32; 2]) -> Did {
        self.pointer = point;
        match &mut self.over {
            Over::Help(_) => Did::default(),
            Over::Question(open) | Over::Notice(open) => changed(open.moved(point)),
            Over::Nothing if self.input.holds_button() => {
                let held = self.held();
                let did = self.input.moved(&self.dialog, &mut self.shown, point);
                self.alone(did.changed, held);
                Did {
                    changed: did.changed,
                    sample: did.button_sound,
                    ended: None,
                }
            }
            Over::Nothing => self.hover(),
        }
    }

    /// The left button pressed at `point`.
    pub fn press(&mut self, point: [i32; 2]) -> Did {
        self.pointer = point;
        match &mut self.over {
            Over::Help(_) => self.close_help(),
            Over::Question(open) | Over::Notice(open) => pressed(open.press(point)),
            Over::Nothing => {
                let did = self.input.press(&self.dialog, &mut self.shown, point);
                let held = self.held();
                self.alone(did.changed, held);
                Did {
                    changed: did.changed,
                    sample: did.button_sound,
                    ended: None,
                }
            }
        }
    }

    /// The left button released.
    pub fn release(&mut self) -> Did {
        match &mut self.over {
            Over::Help(_) => self.close_help(),
            Over::Question(open) | Over::Notice(open) => {
                let outcome = open.release();
                self.answered(outcome)
            }
            Over::Nothing => {
                let held = self.held();
                let did = self.input.release(&mut self.shown);
                self.alone(did.changed, held);
                self.after_hold(did.changed, did.clicked)
            }
        }
    }

    /// The right button pressed at `point`: a button's help, unless the mouse holds a button.
    pub fn right(&mut self, point: [i32; 2]) -> Did {
        self.pointer = point;
        if self.over != Over::Nothing {
            return Did::default();
        }
        match self.input.help_at(&self.dialog, point) {
            Some(id) if self.boxes.help.contains_key(&id) => {
                self.over = Over::Help(id);
                Did::CHANGED
            }
            _ => Did::default(),
        }
    }

    /// The right button released: the help popup closed; a button the mouse holds released as by
    /// the left button (its own loop, CLT-Q72 (a)), in a box over the menu too.
    pub fn right_release(&mut self) -> Did {
        match &mut self.over {
            Over::Help(_) => self.close_help(),
            Over::Question(open) | Over::Notice(open) if open.holds_mouse() => {
                let outcome = open.release();
                self.answered(outcome)
            }
            Over::Question(_) | Over::Notice(_) => Did::default(),
            Over::Nothing => {
                let held = self.held();
                let did = self.input.right_release(&mut self.shown);
                self.alone(did.changed, held);
                self.after_hold(did.changed, did.clicked)
            }
        }
    }

    /// A key down giving `command` (the key `by`, whose release clicks).
    pub fn key(&mut self, command: Command, by: &str) -> Did {
        match (&mut self.over, command, self.kind) {
            (Over::Question(open) | Over::Notice(open), Command::Message(key), _) => {
                pressed(open.key(key, by))
            }
            (Over::Nothing, Command::Button(id), Kind::Main) if BUTTONS.contains(&id) => {
                self.hotkey(id, by)
            }
            (Over::Nothing, Command::Choice(choice), Kind::GameType(_)) => {
                self.hotkey(choice as u16, by)
            }
            _ => Did::default(),
        }
    }

    /// The key `by` released.
    pub fn key_up(&mut self, by: &str) -> Did {
        match &mut self.over {
            // The popup takes the release: the button the key held goes without its click.
            Over::Help(_) => {
                let held = self.held();
                let did = self.input.let_go(&mut self.shown, Some(by));
                self.alone(did.changed, held);
                Did {
                    changed: did.changed,
                    ..Did::default()
                }
            }
            Over::Question(open) | Over::Notice(open) => {
                let outcome = open.key_up(by);
                self.answered(outcome)
            }
            Over::Nothing => {
                let held = self.held();
                let did = self.input.key_up(&mut self.shown, by);
                self.alone(did.changed, held);
                self.after_hold(did.changed, did.clicked)
            }
        }
    }

    /// A hotkey's key `by` down: its button `id` pressed with its sample.
    fn hotkey(&mut self, id: u16, by: &str) -> Did {
        let did = self.input.key(&mut self.shown, id, by);
        self.alone(did.changed, Some(id));
        Did {
            changed: did.changed,
            sample: did.button_sound,
            ended: None,
        }
    }

    /// The button the mouse or a key holds.
    fn held(&self) -> Option<u16> {
        self.input.held().map(held_id)
    }

    /// The button `id` redrawn alone when `changed`, its box shown whole.
    fn alone(&mut self, changed: bool, id: Option<u16>) {
        if changed {
            self.top = id;
            self.whole.extend(id.map(|id| (id, self.shown.state(id))));
        }
    }

    /// The hover taken at the pointer (a move there); a change redraws the five in their order.
    fn hover(&mut self) -> Did {
        let now = self.dialog.item_at(self.pointer).map(Item::id);
        if now == self.hovered {
            return Did::default();
        }
        self.hovered = now;
        match self.kind {
            Kind::Main => self.hovers.main = now,
            Kind::GameType(_) => self.hovers.game_type = now,
        }
        self.top = None;
        for id in self.kind.buttons() {
            self.select(id, false);
        }
        if let Some(id) = now {
            self.select(id, true);
        }
        Did::CHANGED
    }

    /// Item `id`'s state 0x10 set or cleared.
    fn select(&mut self, id: u16, selected: bool) {
        let state = self.shown.state(id);
        self.shown.states.insert(id, State { selected, ..state });
    }

    /// A hold ended (`changed`: a button went up), with the click `clicked` if any: the hover
    /// taken again, then the command.
    fn after_hold(&mut self, changed: bool, clicked: Option<u16>) -> Did {
        let did = Did {
            changed,
            ..Did::default()
        }
        .with(self.hover());
        match clicked {
            Some(id) => did.with(self.command(id)),
            None => did,
        }
    }

    /// A button's command: the main menu's Quit asks its question (0x4FB9BF), the others end the
    /// menu, a game type menu's every one (0x4D59DF).
    fn command(&mut self, id: u16) -> Did {
        let ended = match (self.kind, id) {
            (Kind::Main, 101) => Ended::MainMenu(Request::NewGame),
            (Kind::Main, 102) => Ended::MainMenu(Request::LoadGame),
            (Kind::Main, 103) => Ended::MainMenu(Request::HighScores),
            (Kind::Main, 104) => Ended::MainMenu(Request::Credits),
            (Kind::Main, _) => match &self.boxes.question {
                Some(question) => {
                    self.over = Over::Question(Box::new(Open::new(question.clone())));
                    return Did::CHANGED;
                }
                None => Ended::MainMenu(Request::Quit),
            },
            (Kind::GameType(_), id) => match Choice::of(id) {
                Some(choice) => Ended::GameType(choice),
                None => return Did::default(),
            },
        };
        Did {
            ended: Some(ended),
            ..Did::default()
        }
    }

    /// The box over the menu after an input: the question's OK ends the menu with Quit, its
    /// Cancel closes it (the hover taken again); a stub's notice's OK ends the stub's screen.
    fn answered(&mut self, outcome: Outcome) -> Did {
        match outcome {
            Outcome::Nothing => Did::default(),
            Outcome::Changed => Did::CHANGED,
            Outcome::Closed(_) if matches!(self.over, Over::Notice(_)) => Did {
                ended: STUBS
                    .iter()
                    .find(|&&(screen, _, _)| screen == self.screen)
                    .map(|&(_, _, ended)| ended),
                ..Did::default()
            },
            Outcome::Closed(message::YES) => Did {
                ended: Some(Ended::MainMenu(Request::Quit)),
                ..Did::default()
            },
            Outcome::Closed(_) => {
                self.over = Over::Nothing;
                Did::CHANGED.with(self.hover())
            }
        }
    }

    /// The help popup closed.
    fn close_help(&mut self) -> Did {
        self.over = Over::Nothing;
        Did::CHANGED
    }
}

/// The menus as they open: the main menu, New Game's and Load Game's, and the stubs' notices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Menus {
    /// The main menu.
    pub main: Menu,
    /// New Game's menu.
    pub new_game: Menu,
    /// Load Game's menu.
    pub load_game: Menu,
    /// The stubs' notices by their screen ([`STUBS`]; E27 M2 T6b2), laid out (style 1).
    pub notices: Vec<(Screen, MessageBox)>,
    /// The Multiplayer dialog as it opens (E27 M2 T7), over no menu yet.
    pub multiplayer: Multiplayer,
}

impl Menus {
    /// The menu `screen` shows, opened with the pointer at `pointer` and the procedures' last
    /// items `hovers`: a menu's screen its menu, a stub's ([`STUBS`]) the menu it was chosen from
    /// with the stub's notice over it; none for another screen, or a stub without its notice.
    #[must_use]
    pub fn opened(&self, screen: Screen, pointer: [i32; 2], hovers: Hovers) -> Option<Menu> {
        let kind = match screen {
            Screen::MainMenu => Kind::Main,
            Screen::GameType(game) => Kind::GameType(game),
            _ => {
                let &(_, kind, _) = STUBS.iter().find(|&&(s, _, _)| s == screen)?;
                let (_, notice) = self.notices.iter().find(|(s, _)| *s == screen)?;
                let menu = self.menu(kind).opened(pointer, hovers);
                return Some(menu.stub(screen, notice.clone()));
            }
        };
        Some(self.menu(kind).opened(pointer, hovers))
    }

    /// What takes the inputs on `screen` as it shows, the pointer at `pointer`, `previous` what
    /// took them on the screen before: a menu's or a stub's menu ([`Menus::opened`]); the
    /// Multiplayer dialog over the game type menu `previous` left (E27 M2 T7); none for another
    /// screen.
    #[must_use]
    pub fn front(
        &self,
        screen: Screen,
        previous: Option<Front>,
        pointer: [i32; 2],
    ) -> Option<Front> {
        let hovers = previous.as_ref().map(Front::hovers).unwrap_or_default();
        match (screen, previous) {
            (Screen::Multiplayer(_), Some(Front::Menu(menu))) => Some(Front::Multiplayer(
                Box::new(self.multiplayer.opened(*menu, pointer)),
            )),
            (Screen::Multiplayer(_), _) => None,
            _ => self
                .opened(screen, pointer, hovers)
                .map(|m| Front::Menu(Box::new(m))),
        }
    }

    /// The menu of `kind`.
    const fn menu(&self, kind: Kind) -> &Menu {
        match kind {
            Kind::Main => &self.main,
            Kind::GameType(Game::New) => &self.new_game,
            Kind::GameType(Game::Load) => &self.load_game,
        }
    }
}

/// What takes the inputs on a built screen (E27 M2 T7): a menu, or the Multiplayer dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Front {
    /// A menu (a stub's menu under its notice among them).
    Menu(Box<Menu>),
    /// The Multiplayer dialog over its menu.
    Multiplayer(Box<Multiplayer>),
}

impl Front {
    /// The menus' last items under the mouse as it leaves them (the Multiplayer dialog's: its
    /// menu's).
    #[must_use]
    pub fn hovers(&self) -> Hovers {
        match self {
            Self::Menu(m) => m.hovers(),
            Self::Multiplayer(m) => m.under().map(Menu::hovers).unwrap_or_default(),
        }
    }

    /// Where it takes its keys.
    #[must_use]
    pub const fn context(&self) -> Context {
        match self {
            Self::Menu(m) => m.context(),
            Self::Multiplayer(m) => m.context(),
        }
    }

    /// Where a key down is asked for its command, in order (E27 M2 T7b: an edit's own keys before
    /// its dialog's, `super::keys::command_for`).
    #[must_use]
    pub fn contexts(&self) -> Vec<Context> {
        match self {
            Self::Menu(m) => alloc::vec![m.context()],
            Self::Multiplayer(m) => m.contexts(),
        }
    }

    /// Whether an edit takes the window's text input (E27 M2 T7b, the client's profile).
    #[must_use]
    pub const fn typing(&self) -> bool {
        match self {
            Self::Menu(_) => false,
            Self::Multiplayer(m) => m.typing(),
        }
    }

    /// A character the window's text input typed (E27 M2 T7b).
    pub fn typed(&mut self, c: char) -> Did {
        match self {
            Self::Menu(_) => Did::default(),
            Self::Multiplayer(m) => m.typed(c),
        }
    }

    /// The mouse moved to `point`.
    pub fn moved(&mut self, point: [i32; 2]) -> Did {
        match self {
            Self::Menu(m) => m.moved(point),
            Self::Multiplayer(m) => m.moved(point),
        }
    }

    /// The left button pressed at `point`.
    pub fn press(&mut self, point: [i32; 2]) -> Did {
        match self {
            Self::Menu(m) => m.press(point),
            Self::Multiplayer(m) => m.press(point),
        }
    }

    /// The left button released.
    pub fn release(&mut self) -> Did {
        match self {
            Self::Menu(m) => m.release(),
            Self::Multiplayer(m) => m.release(),
        }
    }

    /// The right button pressed at `point`.
    pub fn right(&mut self, point: [i32; 2]) -> Did {
        match self {
            Self::Menu(m) => m.right(point),
            Self::Multiplayer(m) => m.right(point),
        }
    }

    /// The right button released.
    pub fn right_release(&mut self) -> Did {
        match self {
            Self::Menu(m) => m.right_release(),
            Self::Multiplayer(m) => m.right_release(),
        }
    }

    /// A key down: as the original reads it (`scan`, the dialogs' edits' in the original's
    /// profile), the key table's command for it here (`command`, [`Front::contexts`]) and its
    /// name `by`.
    pub fn key(
        &mut self,
        scan: Option<h3_ui::edit::Scan>,
        command: Option<Command>,
        by: &str,
    ) -> Did {
        match self {
            Self::Menu(m) => command.map_or(Did::default(), |c| m.key(c, by)),
            Self::Multiplayer(m) => m.key(scan, command, by),
        }
    }

    /// The key `by` released.
    pub fn key_up(&mut self, by: &str) -> Did {
        match self {
            Self::Menu(m) => m.key_up(by),
            Self::Multiplayer(m) => m.key_up(by),
        }
    }
}

/// The parts of `rect` (left, top, width, height) outside every rectangle of `shown`.
#[must_use]
pub fn outside(rect: [i32; 4], shown: &[[i32; 4]]) -> Vec<[i32; 4]> {
    let mut parts = vec![rect];
    for &[sx, sy, sw, sh] in shown {
        let (sr, sb) = (sx.saturating_add(sw), sy.saturating_add(sh));
        parts = parts
            .into_iter()
            .flat_map(|[x, y, w, h]| {
                let (r, b) = (x.saturating_add(w), y.saturating_add(h));
                let (top, bottom) = (y.max(sy), b.min(sb));
                let (left, right) = (x.max(sx), r.min(sr));
                if left >= right || top >= bottom {
                    return vec![[x, y, w, h]];
                }
                // Above, below, then left and right of the overlap within its rows.
                [
                    [x, y, w, top - y],
                    [x, bottom, w, b - bottom],
                    [x, top, left - x, bottom - top],
                    [right, top, r - right, bottom - top],
                ]
                .into_iter()
                .filter(|&[_, _, w, h]| w > 0 && h > 0)
                .collect()
            })
            .collect();
    }
    parts
}

/// A box's outcome as the menu's: drawn again.
fn changed(outcome: Outcome) -> Did {
    Did {
        changed: outcome != Outcome::Nothing,
        ..Did::default()
    }
}

/// A box's button going down: drawn again with its press sample.
fn pressed(outcome: Outcome) -> Did {
    let down = outcome == Outcome::Changed;
    Did {
        changed: down,
        sample: down,
        ended: None,
    }
}

#[cfg(test)]
pub(crate) mod tests;
