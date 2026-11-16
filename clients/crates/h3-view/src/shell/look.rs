//! What the shell's built screens show (E27 M2 T4; `docs/epics/E27-design.md` §2.2, §2.3): a
//! screen is its background, a picture copied whole onto the screen (the menu loop's
//! `GamSelBk.pcx`, 0x4EE842), and its dialog's items over it (`h3_ui::ui`), from the shell's
//! layout (`clients/layouts/shell.toml`). The cursor is the adapter's (`h3_ui::cursor`, the
//! arrow at the pointer). The main menu (E27 M2 T6a) and New Game's and Load Game's menus (T6b1)
//! are drawn as they are ([`super::menu`]): their buttons in their states, a game type menu's title
//! by its game, the rows its redraw does not show as last shown in the original's profile
//! (CLT-Q118), the box open over them; the help popup hides the cursor. A stub's screen (T6b2) is
//! the menu it was chosen from with the stub's notice over it. The Multiplayer dialog (T7) shows
//! over the game type menu as that menu left the screen, the hot seat's names over it, a popup or
//! the network's notice over them ([`super::multiplayer`]).

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use h3_ui::ui::{Dialog, Draw, Shown};
use serde::Deserialize;

use super::menu::{Front, Kind, Stale, TITLE};
use super::{Game, Screen};
use crate::message::MessageLayout;

/// The shell's layout: each built screen's background and dialog.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShellLayout {
    /// The cursor the menu loop sets (0x4EE842: the arrow, the default cursor's frame 0): its DEF
    /// and frame, its hot spot at the canvas's corner (`h3_ui::cursor`).
    pub cursor: CursorLook,
    /// The players' palette (`PLAYERS.PAL`, 32 entries a player): a box's frame in a player's
    /// colours (E27 M2 T6a).
    pub player_palette: String,
    /// The message boxes' assets and fonts (E27 M2 T6a: Quit's question, the buttons' help; the
    /// battle's `[message]`, `message_box` 0x4F6570).
    pub message: MessageLayout,
    /// The main menu (§2.3: `main_menu_create` 0x4FB2A0).
    pub main_menu: MenuLayout,
    /// New Game's and Load Game's menu (§2.4: `game_type_menu_create` 0x4D54C0; E27 M2 T6b1).
    pub game_type: MenuLayout,
    /// The Multiplayer dialog's first stage (§2.6: `multiplayer_dialog_init` 0x50E050; E27 M2
    /// T7): its items in the order it adds them, its edit a label.
    pub multiplayer: Dialog,
    /// The hot seat's names (§2.7: `hot_seat_dialog_init` 0x511E20), its edits labels.
    pub hot_seat: Dialog,
}

/// A cursor's DEF and frame.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CursorLook {
    /// The DEF.
    pub def: String,
    /// The frame of its group 0.
    pub frame: u32,
}

/// A menu: the background under it, its dialog and what its hover's redraw shows.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MenuLayout {
    /// The picture copied whole onto the screen first (opaque, from the screen's corner).
    pub background: String,
    /// The screen's rectangles (left, top, width, height) the hover's redraw shows (E27 M2 T6b1:
    /// the rows outside them keep a button as last shown, CLT-Q118's, the original's profile).
    pub redrawn: Vec<[i32; 4]>,
    /// A game type menu's title pictures by its game, New Game's then Load Game's
    /// (`game_type_menu_titles` 0x6780E8): its picture [`TITLE`]'s; none for the main menu.
    #[serde(default)]
    pub titles: Vec<String>,
    /// The dialog over it.
    pub dialog: Dialog,
}

impl MenuLayout {
    /// The dialog for `game`: its title picture the game's (the layout's own when it has none).
    #[must_use]
    pub fn dialog_for(&self, game: Game) -> Dialog {
        let mut dialog = self.dialog.clone();
        if let Some(title) = self.titles.get(game as usize) {
            for item in &mut dialog.items {
                if let h3_ui::ui::Item::Picture(p) = item
                    && p.id == TITLE
                {
                    p.picture.clone_from(title);
                }
            }
        }
        dialog
    }
}

/// What a screen shows: its background, then its dialog's draws over it, the rows a redraw does
/// not show, the box over them, and whether the cursor shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Look {
    /// The picture copied whole onto the screen first.
    pub background: String,
    /// The dialog's draw list.
    pub draws: Vec<Draw>,
    /// The rows a redraw did not show, each kept as last shown: drawn over the background alone
    /// and copied there (the original's profile only).
    pub stale: Vec<Stale>,
    /// The box open over the screen, drawn last.
    pub over: Vec<Draw>,
    /// The cursor drawn at the pointer.
    pub cursor: bool,
}

/// Reads the shell's layout.
///
/// # Errors
/// The text is not the layout's TOML.
pub fn parse_layout(text: &str) -> Result<ShellLayout, String> {
    let mut layout: ShellLayout = toml::from_str(text).map_err(|e| e.to_string())?;
    for menu in [&mut layout.main_menu, &mut layout.game_type] {
        menu.dialog = menu.dialog.clone().expanded();
    }
    for dialog in [&mut layout.multiplayer, &mut layout.hot_seat] {
        *dialog = dialog.clone().expanded();
    }
    Ok(layout)
}

/// What `screen` shows, none when it is not built yet ([`Screen::built`]) or without its state
/// `front` (the screen's own: a menu's, or a stub's menu under its notice, E27 M2 T6b2; the
/// Multiplayer dialog over its menu, T7): its buttons as they are, the stale rows and the box
/// over them ([`super::menu::Menu::dialog_draws`], [`super::menu::Menu::stale`],
/// [`super::menu::Menu::box_draws`]); the dialogs over the menu.
#[must_use]
pub fn look(layout: &ShellLayout, screen: Screen, front: Option<&Front>) -> Option<Look> {
    let players = &layout.player_palette;
    let (menu, dialog) = match front? {
        Front::Menu(menu) => (menu.as_ref(), None),
        Front::Multiplayer(dialog) => (dialog.under()?, Some(dialog)),
    };
    let screen_on = if dialog.is_some() {
        matches!(screen, Screen::Multiplayer(_))
    } else {
        menu.screen() == screen
    };
    if !screen_on {
        return None;
    }
    let background = match menu.kind() {
        Kind::GameType(_) => &layout.game_type.background,
        Kind::Main => &layout.main_menu.background,
    };
    let mut draws = menu.dialog_draws(players);
    let (over, cursor) = match dialog {
        Some(d) => {
            draws.extend(d.draws(players));
            (d.box_draws(players), d.cursor())
        }
        None => (menu.box_draws(players), menu.cursor()),
    };
    Some(Look {
        background: background.clone(),
        draws,
        stale: menu.stale(players),
        over,
        cursor,
    })
}

/// Every asset the built screens, their boxes and the cursor name, each once, in a fixed order
/// (the adapter's batch at the shell's start; a game type menu's titles both): the boxes' are the message layout's that a text
/// box draws (its background, frame, buttons' back, OK and Cancel, its font and the fonts'
/// palette) and the players' palette.
#[must_use]
pub fn names(layout: &ShellLayout) -> Vec<String> {
    let mut names = alloc::vec![layout.cursor.def.clone()];
    for menu in [&layout.main_menu, &layout.game_type] {
        names.push(menu.background.clone());
        for draw in menu.dialog.draws(&Shown::default(), "") {
            names.extend(h3_ui::paint::names(&draw).into_iter().map(String::from));
        }
        names.extend(menu.titles.iter().cloned());
    }
    for dialog in [&layout.multiplayer, &layout.hot_seat] {
        for draw in dialog.draws(&Shown::default(), "") {
            names.extend(h3_ui::paint::names(&draw).into_iter().map(String::from));
        }
    }
    let m = &layout.message;
    names.extend(
        [
            &m.background,
            &m.frame,
            &m.button_back,
            &m.ok,
            &m.cancel,
            &m.font,
            &m.caption_font,
            &m.font_palette,
            &layout.player_palette,
        ]
        .map(String::clone),
    );
    let mut unique: Vec<String> = Vec::new();
    for name in names {
        if !unique.iter().any(|n| n.eq_ignore_ascii_case(&name)) {
            unique.push(name);
        }
    }
    unique
}

#[cfg(test)]
mod tests;
