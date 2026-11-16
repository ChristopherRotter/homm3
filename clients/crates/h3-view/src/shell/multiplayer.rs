//! The Multiplayer dialog's first stage and the hot seat's names taking their inputs (E27 M2 T7;
//! `docs/epics/E27-design.md` §2.6, §2.7, §16): one dialog over New Game's or Load Game's menu as
//! that menu left the screen (`multiplayer_dialog_run` 0x4F0590), the hot seat's names a dialog
//! over it (`hot_seat_run` 0x511D40).
//!
//! - **The Multiplayer dialog** (`multiplayer_dialog_init` 0x50E050; stage 1,
//!   `multiplayer_stage_types` 0x50EFC0): its picture, the six connection types, Cancel, the
//!   player's name (an edit with the focus, `h3_ui::edit`, the setting `Network Default Name`:
//!   none in the client, so empty; a typed name is not kept, `edit-keys`' `reopened`), the status
//!   line and the map's picture over the list. Hot Seat opens the hot seat's names; IPX, TCP/IP,
//!   Modem, Direct Connect and Online show the network's notice (`CLT-STUB-6`, decision 10: a box
//!   of style 1 in the client's own words, the dialog then on its first stage; Online never ends
//!   the client); Cancel and Escape end the dialog (0x7801).
//! - **The hot seat's names** (`hot_seat_dialog_init` 0x511E20, a shadow of its own): eight edits
//!   in a ring, the first with the Multiplayer dialog's name and the focus; OK, disabled until two
//!   edits hold a name (0x20: its frame 2, its hint and help kept), ends the dialog with the
//!   names (0x7802: the scenario screen, M3, takes them); Cancel and Escape close it, the
//!   Multiplayer dialog on its first stage. An edit's keys (`hot_seat_edit_key` 0x50DF60): Shift
//!   and Tab, and Up, to the previous; Tab, Enter and Down to the next; any other to the edit;
//!   then OK enabled while more than one holds a name. **Enter presses an enabled OK**
//!   (CLT-Q107, decision 22: fixed; the original's to the next edit for the comparisons only,
//!   [`Bugs::Original`]); with OK disabled it moves on, as the original's.
//! - **Typing** (E27 M2 T7b, decision 39; CLT-Q127 (b)): in the client's profile the edit with
//!   the focus takes the window's text input ([`Multiplayer::typed`]: what the keyboard prints)
//!   and its own keys and the ring as the shell's key table's commands (`Command::Edit`,
//!   `Command::Ring`, asked in [`Multiplayer::contexts`]' order); hot-seat-ok with OK disabled
//!   moves on as Enter does. In the original's profile the edits take each key's scan code
//!   (`h3_ui::edit::Edit::key`) and no text.
//! - **The hover** (`dialog_mouse_move` 0x5FF6E0): a move puts the hint of the item under the
//!   mouse (`dialog_item_at` 0x5FF3E0: the last in order whose box holds it; a button disabled by
//!   0x20 kept) in the status line when that item changes,
//!   none when it has none (the picture, the edits' outside the hot seat, the map). A button the
//!   mouse holds runs its own loop: its moves change no hint (`mp-new`'s `hot-left`).
//! - **A right press** (`dialog_right_click` 0x5FF790): the item's right-click text, else its
//!   hint, in a popup of style 4 while the right button is held; none for an item with neither.
//! - **A press** on a DEF button: its own loop (`h3_ui::input`), its sample; the command at the
//!   release over it. A press on an edit gives it the focus (0x5BB287).
//!
//! The texts (the hints, the popups, the notice, the hot seat's title) come laid out from the
//! caller ([`Texts`]); the dialogs from the shell's layout. *Not built* (design §16): stage 2
//! and the network (decision 10, E28), the CD notice (Hot Seat then absent), the deletion of a
//! transfer's file, the session list's refresh, the dialog's name kept as the setting.

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use alloc::vec;

use h3_render::Bugs;
use h3_ui::edit::{self, Edit, Scan};
use h3_ui::input::Input;
use h3_ui::ui::{Dialog, Draw, Item, Shown, State};

use super::Ended;
use super::keys::Command;
use super::keys::Context;
use super::menu::{Did, Menu};
use crate::message::{MessageBox, Open, Outcome};

/// Online (101).
pub const ONLINE: u16 = 101;
/// Hot Seat (102).
pub const HOT_SEAT: u16 = 102;
/// The network modes but Online: IPX, TCP/IP, Modem, Direct Connect (103 to 106).
pub const NETWORK: [u16; 4] = [103, 104, 105, 106];
/// Cancel (124: Escape).
pub const CANCEL: u16 = 124;
/// The player's name, an edit (125).
pub const NAME: u16 = 125;
/// The status line (123).
pub const STATUS: u16 = 123;
/// The hot seat's title (517: GENRLTXT 447).
pub const TITLE: u16 = 517;
/// The hot seat's eight name edits, in their ring's order (509 to 516).
pub const NAMES: [u16; 8] = [509, 510, 511, 512, 513, 514, 515, 516];
/// The hot seat's OK (519: Enter).
pub const OK: u16 = 519;
/// The hot seat's Cancel (520: Escape).
pub const HOT_SEAT_CANCEL: u16 = 520;
/// The hot seat's status line (518).
pub const HOT_SEAT_STATUS: u16 = 518;
/// The Multiplayer dialog's items with a hint and a right press's help, by id, with their names
/// in the key catalogue (`shell.multiplayer.hint.<name>`, `shell.multiplayer.help.<name>`:
/// `Help.txt`'s record 264 + id − 100; the picture's and the status line's records are empty).
pub const TEXTED: [(u16, &str); 7] = [
    (ONLINE, "online"),
    (HOT_SEAT, "hot-seat"),
    (103, "ipx"),
    (104, "tcp-ip"),
    (105, "modem"),
    (106, "direct-connect"),
    (CANCEL, "cancel"),
];

/// The hot seat's items with a hint and help, by id, with their names in the key catalogue
/// (`shell.hot-seat.hint.<name>`, `shell.hot-seat.help.<name>`: the edits `Help.txt` 559, OK 560
/// and Cancel 561, the last two's help their hint, their right-click texts empty).
pub const HOT_SEAT_TEXTED: [(u16, &str); 10] = [
    (509, "name"),
    (510, "name"),
    (511, "name"),
    (512, "name"),
    (513, "name"),
    (514, "name"),
    (515, "name"),
    (516, "name"),
    (OK, "ok"),
    (HOT_SEAT_CANCEL, "cancel"),
];

/// The status lines' background colour (`text_item_create`'s 0x20: an entry of `game.pal`,
/// black), filling their boxes before their text (`text_item_draw` 0x5BC5F0).
pub const STATUS_BACKGROUND: u32 = 0x20;

/// An edit's room with its terminator (`H3DlgEdit::Create`'s 21 in both dialogs: 20 letters).
pub const LIMIT: usize = 21;

/// The dialogs' texts, laid out by the caller: the user's tables' (the hints, the right-click
/// help, the hot seat's title) and the client's (the network's notice).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Texts {
    /// Each item's hint by id, both dialogs' (`Help.txt`'s first column: the Multiplayer dialog's
    /// record 264 + id − 100 for 100 to 124, the hot seat's 559 to 561); none for an item without
    /// one.
    pub hints: BTreeMap<u16, String>,
    /// Each item's right press's popup by id (style 4: its right-click text, else its hint).
    pub help: BTreeMap<u16, MessageBox>,
    /// The network modes' notice (`CLT-STUB-6`, style 1).
    pub network: MessageBox,
    /// The hot seat's title (GENRLTXT 447).
    pub title: String,
}

/// What is open over the Multiplayer dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Over {
    Nothing,
    /// An item's help popup, while the right button is held.
    Help(u16),
    /// The network's notice.
    Notice(Box<Open>),
    /// The hot seat's names.
    HotSeat(Box<HotSeat>),
}

/// A dialog's items, state, input and hover: what both dialogs share.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Part {
    dialog: Dialog,
    shown: Shown,
    input: Input,
    /// The item under the mouse at the last move (`+0x4C`): its hint in the status line.
    hovered: Option<u16>,
    /// The status line's id.
    status: u16,
}

impl Part {
    fn new(dialog: Dialog, status: u16, bugs: Bugs) -> Self {
        let mut shown = Shown::default();
        shown.texts.insert(status, String::new());
        Self {
            dialog,
            shown,
            input: Input::new(bugs),
            hovered: None,
            status,
        }
    }

    /// The item under the mouse at `point` (`dialog_item_at` 0x5FF3E0): the last in order whose
    /// box holds it, not hidden (the texts' `flags` argument 8, their +0x14, is not the state
    /// the search tests: drawn in their own colour, `mp-new`, `hotseat-names`). Not
    /// `h3_ui::ui::Dialog::item_taking`, which skips a disabled item: the hot seat's OK disabled
    /// (`dialog_item_enable`'s 0x20) keeps its hint and popup here, as recorded (design §16.2;
    /// M2's review compared the two over both dialogs, the only difference that OK).
    fn item_at(&self, point: [i32; 2]) -> Option<u16> {
        let [x, y] = [
            point[0] - self.dialog.place[0],
            point[1] - self.dialog.place[1],
        ];
        self.dialog
            .items
            .iter()
            .rev()
            .find(|item| {
                let id = item.id();
                !self.shown.state(id).hidden
                    && item.area().is_some_and(|([left, top], [w, h])| {
                        (left..left.saturating_add(w)).contains(&x)
                            && (top..top.saturating_add(h)).contains(&y)
                    })
            })
            .map(Item::id)
    }

    /// A move to `point` (`dialog_mouse_move` 0x5FF6E0): on a change of the item under it, that
    /// item's hint, else none, in the status line. Whether anything changed.
    fn hover(&mut self, point: [i32; 2], hints: &BTreeMap<u16, String>) -> bool {
        let now = self.item_at(point);
        if now == self.hovered {
            return false;
        }
        self.hovered = now;
        let hint = now
            .and_then(|id| hints.get(&id))
            .cloned()
            .unwrap_or_default();
        let changed = self.shown.texts.get(&self.status) != Some(&hint);
        self.shown.texts.insert(self.status, hint);
        changed
    }

    /// The dialog's draw list as it is, its edits drawn as edits, its status line's box filled
    /// before its text.
    fn draws<'a>(
        &self,
        players: &str,
        edits: impl IntoIterator<Item = (u16, &'a Edit)>,
    ) -> Vec<Draw> {
        let mut draws = Vec::new();
        for draw in self.dialog.draws(&self.shown, players) {
            if let Draw::Label { item, x, y, .. } = &draw
                && item.id == self.status
            {
                draws.push(Draw::Fill {
                    rect: [*x, *y, item.size[0], item.size[1]],
                    palette: item.font_palette.clone(),
                    entry: STATUS_BACKGROUND,
                });
            }
            draws.push(draw);
        }
        for (id, edit) in edits {
            edit.place(id, &mut draws);
        }
        draws
    }

    /// The help a right press at `point` asks for: the item there, if it has a popup and the
    /// mouse holds no button.
    fn help_at(&self, point: [i32; 2], help: &BTreeMap<u16, MessageBox>) -> Option<u16> {
        if self.input.holds_button() {
            return None;
        }
        self.item_at(point).filter(|id| help.contains_key(id))
    }
}

/// The hot seat's names (§2.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotSeat {
    part: Part,
    edits: [Edit; 8],
    /// The edit with the focus.
    focus: usize,
    /// A right press's popup open, by item.
    help: Option<u16>,
    bugs: Bugs,
}

impl HotSeat {
    /// The dialog as it opens: the first edit holding `name` with the focus, OK disabled, its
    /// title `title`.
    fn new(dialog: Dialog, name: &str, title: &str, bugs: Bugs) -> Self {
        let mut part = Part::new(dialog, HOT_SEAT_STATUS, bugs);
        part.shown.texts.insert(TITLE, String::from(title));
        let mut edits: [Edit; 8] = core::array::from_fn(|_| Edit::new("", LIMIT));
        edits[0] = Edit::new(name, LIMIT);
        edits[0].set_focus(true);
        let mut hot_seat = Self {
            part,
            edits,
            focus: 0,
            help: None,
            bugs,
        };
        hot_seat.check();
        hot_seat
    }

    /// The names the edits hold, the empty ones left out, in order (`hot_seat_names_store`
    /// 0x512470).
    #[must_use]
    pub fn names(&self) -> Vec<String> {
        self.edits
            .iter()
            .map(Edit::text)
            .filter(|t| !t.is_empty())
            .map(String::from)
            .collect()
    }

    /// Whether OK is enabled.
    #[must_use]
    pub fn ok_enabled(&self) -> bool {
        !self.part.shown.state(OK).disabled
    }

    /// The edit with the focus.
    #[must_use]
    pub const fn focus(&self) -> usize {
        self.focus
    }

    /// OK enabled while more than one edit holds a name (`hot_seat_edit_check` 0x50DEE0).
    fn check(&mut self) {
        let named = self.edits.iter().filter(|e| !e.text().is_empty()).count();
        let state = self.part.shown.state(OK);
        self.part.shown.states.insert(
            OK,
            State {
                disabled: named < 2,
                ..state
            },
        );
    }

    /// The focus to edit `to` (`edit_focus_next` 0x510850, `_previous` 0x510870; a press's
    /// `dialog_set_focus` 0x5FF490).
    fn focus_on(&mut self, to: usize) {
        self.edits[self.focus].set_focus(false);
        self.focus = to;
        self.edits[to].set_focus(true);
        self.check();
    }

    /// The edit after (`true`) or before the focused one in the ring.
    const fn ring(&self, next: bool) -> usize {
        if next {
            (self.focus + 1) % 8
        } else {
            (self.focus + 7) % 8
        }
    }

    /// A key down `scan` to the focused edit in the original's profile (`hot_seat_edit_key`
    /// 0x50DF60): whether it took it.
    fn edit_key(&mut self, scan: Scan) -> bool {
        match (scan.code, scan.shift) {
            (edit::TAB, true) | (edit::UP, _) => self.focus_on(self.ring(false)),
            (edit::TAB | edit::RETURN | edit::DOWN, _) => self.focus_on(self.ring(true)),
            _ => {
                if !self.edits[self.focus].key(scan) {
                    return false;
                }
                self.check();
            }
        }
        true
    }

    /// A key table's command to the focused edit and the ring in the client's profile (E27 M2
    /// T7b): whether it took it. OK's with OK disabled moves on (Enter's, as the original's);
    /// with OK enabled it is OK's hotkey (CLT-Q107).
    fn edit_command(&mut self, command: Option<Command>) -> bool {
        match command {
            Some(Command::Edit(caret)) => {
                self.edits[self.focus].caret_key(caret);
                self.check();
            }
            Some(Command::Ring(next)) => self.focus_on(self.ring(next)),
            Some(Command::Dialog(OK)) if !self.ok_enabled() => self.focus_on(self.ring(true)),
            _ => return false,
        }
        true
    }

    /// Its shadow and items as they are.
    fn draws(&self, players: &str) -> Vec<Draw> {
        self.part.draws(players, NAMES.into_iter().zip(&self.edits))
    }
}

/// The Multiplayer dialog on the screen (its first stage), and what is open over it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Multiplayer {
    part: Part,
    name: Edit,
    /// The game type menu as it left the screen.
    under: Option<Box<Menu>>,
    hot_seat: Dialog,
    texts: Texts,
    over: Over,
    pointer: [i32; 2],
    bugs: Bugs,
    /// The hot seat's names once its OK ended the dialog.
    names: Vec<String>,
}

impl Multiplayer {
    /// The dialog as it opens (stage 1, the name's edit empty with the focus), from the layout's
    /// `dialog` and `hot_seat`, its texts `texts`, over no menu yet, in the profile `bugs`.
    #[must_use]
    pub fn new(dialog: Dialog, hot_seat: Dialog, texts: Texts, bugs: Bugs) -> Self {
        let mut name = Edit::new("", LIMIT);
        name.set_focus(true);
        Self {
            part: Part::new(dialog, STATUS, bugs),
            name,
            under: None,
            hot_seat,
            texts,
            over: Over::Nothing,
            pointer: [0, 0],
            bugs,
            names: Vec::new(),
        }
    }

    /// The dialog opened anew over `under` (the game type menu as it ended), the pointer at
    /// `pointer`.
    #[must_use]
    pub fn opened(&self, under: Menu, pointer: [i32; 2]) -> Self {
        let mut opened = Self::new(
            self.part.dialog.clone(),
            self.hot_seat.clone(),
            self.texts.clone(),
            self.bugs,
        );
        opened.under = Some(Box::new(under));
        opened.pointer = pointer;
        opened
    }

    /// The menu under the dialog.
    #[must_use]
    pub fn under(&self) -> Option<&Menu> {
        self.under.as_deref()
    }

    /// The player's name as the edit holds it.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.text()
    }

    /// The hot seat's names, once its OK ended the dialog (empty before).
    #[must_use]
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// The hot seat's names, while they are open.
    #[must_use]
    pub fn hot_seat(&self) -> Option<&HotSeat> {
        match &self.over {
            Over::HotSeat(h) => Some(h),
            _ => None,
        }
    }

    /// The status line's text.
    #[must_use]
    pub fn status(&self) -> &str {
        self.part
            .shown
            .texts
            .get(&STATUS)
            .map_or("", String::as_str)
    }

    /// Whether the network's notice is open (the client's stub: not the original's screen).
    #[must_use]
    pub const fn noticing(&self) -> bool {
        matches!(self.over, Over::Notice(_))
    }

    /// Where the dialog takes its keys: itself, the hot seat's names, or the notice.
    #[must_use]
    pub const fn context(&self) -> Context {
        match self.over {
            Over::Notice(_) => Context::Question,
            Over::HotSeat(_) => Context::HotSeat,
            Over::Nothing | Over::Help(_) => Context::Multiplayer,
        }
    }

    /// Whether an edit with the focus takes the window's text input (E27 M2 T7b): in the client's
    /// profile, the name's with nothing over the dialog, the hot seat's focused one with no popup
    /// over it.
    #[must_use]
    pub const fn typing(&self) -> bool {
        matches!(self.bugs, Bugs::Fixed)
            && match &self.over {
                Over::Nothing => true,
                Over::HotSeat(h) => h.help.is_none(),
                Over::Help(_) | Over::Notice(_) => false,
            }
    }

    /// Where a key down is asked for its command, in order (E27 M2 T7b): an edit's own keys
    /// while one types ([`Multiplayer::typing`]), then [`Multiplayer::context`].
    #[must_use]
    pub fn contexts(&self) -> Vec<Context> {
        if self.typing() {
            vec![Context::Edit, self.context()]
        } else {
            vec![self.context()]
        }
    }

    /// A character the window's text input typed (E27 M2 T7b, the client's profile): to the edit
    /// with the focus while one types (the hot seat's OK then checked).
    pub fn typed(&mut self, c: char) -> Did {
        if !self.typing() {
            return Did::default();
        }
        let edit = match &mut self.over {
            Over::HotSeat(h) => &mut h.edits[h.focus],
            _ => &mut self.name,
        };
        let before = edit.clone();
        edit.typed(c);
        let changed = *edit != before;
        if let Over::HotSeat(h) = &mut self.over {
            h.check();
        }
        Did {
            changed,
            ..Did::default()
        }
    }

    /// Whether the cursor shows (a help popup hides it).
    #[must_use]
    pub fn cursor(&self) -> bool {
        match &self.over {
            Over::Help(_) => false,
            Over::HotSeat(h) => h.help.is_none(),
            _ => true,
        }
    }

    /// The dialogs' draw list: the Multiplayer dialog's, then the hot seat's names with their
    /// shadow when open (the menu under them is the caller's: [`Multiplayer::under`]).
    #[must_use]
    pub fn draws(&self, players: &str) -> Vec<Draw> {
        let mut draws = self.part.draws(players, [(NAME, &self.name)]);
        if let Over::HotSeat(h) = &self.over {
            draws.extend(h.draws(players));
        }
        draws
    }

    /// The box open over the dialogs, if any: a help popup or the notice.
    #[must_use]
    pub fn box_draws(&self, players: &str) -> Vec<Draw> {
        let message = match &self.over {
            Over::Help(id) => self.texts.help.get(id),
            Over::Notice(open) => Some(&open.message),
            Over::HotSeat(h) => h.help.and_then(|id| self.texts.help.get(&id)),
            Over::Nothing => None,
        };
        message
            .map(|m| m.dialog.draws(&m.shown, players))
            .unwrap_or_default()
    }

    /// The mouse moved to `point`.
    pub fn moved(&mut self, point: [i32; 2]) -> Did {
        self.pointer = point;
        let hints = &self.texts.hints;
        match &mut self.over {
            Over::Help(_) => Did::default(),
            Over::Notice(open) => Did::of_box(open.moved(point)),
            Over::HotSeat(h) if h.help.is_some() => Did::default(),
            Over::HotSeat(h) => moved(&mut h.part, point, hints),
            Over::Nothing => moved(&mut self.part, point, hints),
        }
    }

    /// The left button pressed at `point`.
    pub fn press(&mut self, point: [i32; 2]) -> Did {
        self.pointer = point;
        match &mut self.over {
            Over::Help(_) => self.close_help(),
            Over::Notice(open) => Did::box_press(open.press(point)),
            Over::HotSeat(h) if h.help.is_some() => {
                h.help = None;
                Did::CHANGED
            }
            Over::HotSeat(h) => {
                let at = h.part.item_at(point);
                if let Some(to) = NAMES.iter().position(|&n| Some(n) == at) {
                    if to != h.focus {
                        h.focus_on(to);
                    }
                    return Did::CHANGED;
                }
                Did::held(h.part.input.press(&h.part.dialog, &mut h.part.shown, point))
            }
            Over::Nothing => Did::held(self.part.input.press(
                &self.part.dialog,
                &mut self.part.shown,
                point,
            )),
        }
    }

    /// The left button released.
    pub fn release(&mut self) -> Did {
        match &mut self.over {
            Over::Help(_) => self.close_help(),
            Over::Notice(open) => {
                let outcome = open.release();
                self.answered(outcome)
            }
            Over::HotSeat(h) if h.help.is_some() => {
                h.help = None;
                Did::CHANGED
            }
            Over::HotSeat(h) => {
                let did = h.part.input.release(&mut h.part.shown);
                self.clicked(did.changed, did.clicked)
            }
            Over::Nothing => {
                let did = self.part.input.release(&mut self.part.shown);
                self.clicked(did.changed, did.clicked)
            }
        }
    }

    /// The right button pressed at `point`: an item's help.
    pub fn right(&mut self, point: [i32; 2]) -> Did {
        self.pointer = point;
        let help = &self.texts.help;
        match &mut self.over {
            Over::Nothing => match self.part.help_at(point, help) {
                Some(id) => {
                    self.over = Over::Help(id);
                    Did::CHANGED
                }
                None => Did::default(),
            },
            Over::HotSeat(h) if h.help.is_none() => match h.part.help_at(point, help) {
                Some(id) => {
                    h.help = Some(id);
                    Did::CHANGED
                }
                None => Did::default(),
            },
            _ => Did::default(),
        }
    }

    /// The right button released: the help popup closed; a button the mouse holds released as by
    /// the left button.
    pub fn right_release(&mut self) -> Did {
        match &mut self.over {
            Over::Help(_) => self.close_help(),
            Over::Notice(open) if open.holds_mouse() => {
                let outcome = open.release();
                self.answered(outcome)
            }
            Over::Notice(_) => Did::default(),
            Over::HotSeat(h) if h.help.is_some() => {
                h.help = None;
                Did::CHANGED
            }
            Over::HotSeat(h) => {
                let did = h.part.input.right_release(&mut h.part.shown);
                self.clicked(did.changed, did.clicked)
            }
            Over::Nothing => {
                let did = self.part.input.right_release(&mut self.part.shown);
                self.clicked(did.changed, did.clicked)
            }
        }
    }

    /// A key down: `scan` as the original reads it, `command` the key table's for it in
    /// [`Multiplayer::contexts`], `by` its name (whose release clicks). The focused edit takes it
    /// first (by `scan` in the original's profile, by `command` in the client's); a key it leaves
    /// presses its hotkey's button.
    pub fn key(&mut self, scan: Option<Scan>, command: Option<Command>, by: &str) -> Did {
        let original = self.bugs == Bugs::Original;
        match &mut self.over {
            Over::Help(_) => Did::default(),
            Over::Notice(open) => match command {
                Some(Command::Message(key)) => Did::box_press(open.key(key, by)),
                _ => Did::default(),
            },
            Over::HotSeat(h) if h.help.is_some() => Did::default(),
            Over::HotSeat(h) => {
                let taken = if original {
                    scan.is_some_and(|s| h.edit_key(s))
                } else {
                    h.edit_command(command)
                };
                if taken {
                    return Did::CHANGED;
                }
                match command {
                    Some(Command::Dialog(id @ (OK | HOT_SEAT_CANCEL))) => {
                        Did::held(h.part.input.key(&mut h.part.shown, id, by))
                    }
                    _ => Did::default(),
                }
            }
            Over::Nothing => {
                // The name's edit (`multiplayer_name_key` 0x50ED60: Enter is not its).
                let taken = match (original, command) {
                    (true, _) => scan.is_some_and(|s| self.name.key(s)),
                    (false, Some(Command::Edit(caret))) => self.name.caret_key(caret),
                    (false, _) => false,
                };
                if taken {
                    return Did::CHANGED;
                }
                match command {
                    Some(Command::Dialog(CANCEL)) => {
                        Did::held(self.part.input.key(&mut self.part.shown, CANCEL, by))
                    }
                    _ => Did::default(),
                }
            }
        }
    }

    /// The key `by` released.
    pub fn key_up(&mut self, by: &str) -> Did {
        match &mut self.over {
            Over::Help(_) => {
                let did = self.part.input.let_go(&mut self.part.shown, Some(by));
                Did {
                    changed: did.changed,
                    ..Did::default()
                }
            }
            Over::Notice(open) => {
                let outcome = open.key_up(by);
                self.answered(outcome)
            }
            Over::HotSeat(h) if h.help.is_some() => {
                let did = h.part.input.let_go(&mut h.part.shown, Some(by));
                Did {
                    changed: did.changed,
                    ..Did::default()
                }
            }
            Over::HotSeat(h) => {
                let did = h.part.input.key_up(&mut h.part.shown, by);
                self.clicked(did.changed, did.clicked)
            }
            Over::Nothing => {
                let did = self.part.input.key_up(&mut self.part.shown, by);
                self.clicked(did.changed, did.clicked)
            }
        }
    }

    /// A hold ended (`changed`: a button went up) with the click `clicked`, if any: its command
    /// (`multiplayer_dialog_click` 0x50F4E0, `hot_seat_dialog_click` 0x512420); then the dialog
    /// on top takes the pointer's place as a move (*recorded*: `mp-new`'s `*-left`, the status
    /// line cleared after a release off a button over the map; `hotseat-cancel`'s `cancelled`).
    fn clicked(&mut self, changed: bool, clicked: Option<u16>) -> Did {
        let mut did = Did {
            changed,
            ..Did::default()
        };
        if let Some(id) = clicked {
            let hot_seat = matches!(self.over, Over::HotSeat(_));
            self.over = match (hot_seat, id) {
                (false, HOT_SEAT) => Over::HotSeat(Box::new(HotSeat::new(
                    self.hot_seat.clone(),
                    self.name.text(),
                    &self.texts.title,
                    self.bugs,
                ))),
                (false, CANCEL) => {
                    did.ended = Some(Ended::Multiplayer(false));
                    return did;
                }
                // stub: CLT-STUB-6 (the network modes, E28): every other button of stage 1.
                (false, _) => Over::Notice(Box::new(Open::new(self.texts.network.clone()))),
                (true, OK) => {
                    self.names = self.hot_seat().map(HotSeat::names).unwrap_or_default();
                    did.ended = Some(Ended::Multiplayer(true));
                    return did;
                }
                // The hot seat's other button, Cancel.
                (true, _) => Over::Nothing,
            };
            did.changed = true;
        }
        let (pointer, hints) = (self.pointer, &self.texts.hints);
        did.changed |= match &mut self.over {
            Over::Nothing => self.part.hover(pointer, hints),
            Over::HotSeat(h) => h.part.hover(pointer, hints),
            Over::Help(_) | Over::Notice(_) => false,
        };
        did
    }

    /// The notice after an input: its OK closes it, the status line then the hint under the
    /// pointer (decision 20's redraw at a change, CLT-Q101).
    fn answered(&mut self, outcome: Outcome) -> Did {
        match outcome {
            Outcome::Nothing => Did::default(),
            Outcome::Changed => Did::CHANGED,
            Outcome::Closed(_) => {
                self.over = Over::Nothing;
                self.part.hovered = None;
                self.part.hover(self.pointer, &self.texts.hints);
                Did::CHANGED
            }
        }
    }

    /// The help popup closed.
    fn close_help(&mut self) -> Did {
        self.over = Over::Nothing;
        Did::CHANGED
    }
}

/// A move to `point` over `part`: a button the mouse holds follows it (its sample as it comes
/// back), else the hover.
fn moved(part: &mut Part, point: [i32; 2], hints: &BTreeMap<u16, String>) -> Did {
    if part.input.holds_button() {
        return Did::held(part.input.moved(&part.dialog, &mut part.shown, point));
    }
    Did {
        changed: part.hover(point, hints),
        ..Did::default()
    }
}

#[cfg(test)]
pub(crate) mod tests;
