//! What the shell's built screens show (E27 M2 T4; `docs/epics/E27-design.md` §2.2, §2.3): a
//! screen is its background, a picture copied whole onto the screen (the menu loop's
//! `GamSelBk.pcx`, 0x4EE842), and its dialog's items over it (`h3_ui::ui`), from the shell's
//! layout (`clients/layouts/shell.toml`). The cursor is the adapter's (`h3_ui::cursor`, the
//! arrow at the pointer). The main menu is drawn as it is (E27 M2 T6a, [`super::menu`]): its five
//! buttons in their states, the box open over them; the help popup hides the cursor.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use h3_ui::ui::{Dialog, Draw, Shown};
use serde::Deserialize;

use super::Screen;
use super::menu::MainMenu;
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
    pub main_menu: Menu,
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

/// A menu: the background under it and its dialog.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Menu {
    /// The picture copied whole onto the screen first (opaque, from the screen's corner).
    pub background: String,
    /// The dialog over it.
    pub dialog: Dialog,
}

/// What a screen shows: its background, then its dialog's draws over it, and whether the cursor
/// shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Look {
    /// The picture copied whole onto the screen first.
    pub background: String,
    /// The dialog's draw list.
    pub draws: Vec<Draw>,
    /// The cursor drawn at the pointer.
    pub cursor: bool,
}

/// Reads the shell's layout.
///
/// # Errors
/// The text is not the layout's TOML.
pub fn parse_layout(text: &str) -> Result<ShellLayout, String> {
    let mut layout: ShellLayout = toml::from_str(text).map_err(|e| e.to_string())?;
    layout.main_menu.dialog = layout.main_menu.dialog.clone().expanded();
    Ok(layout)
}

/// What `screen` shows, none when it is not built yet ([`Screen::built`]) or, for the main menu,
/// without its state `menu`: its buttons as they are and the box over them ([`MainMenu::draws`]).
#[must_use]
pub fn look(layout: &ShellLayout, screen: Screen, menu: Option<&MainMenu>) -> Option<Look> {
    match (screen, menu) {
        (Screen::MainMenu, Some(menu)) => Some(Look {
            background: layout.main_menu.background.clone(),
            draws: menu.draws(&layout.player_palette),
            cursor: menu.cursor(),
        }),
        _ => None,
    }
}

/// Every asset the built screens, their boxes and the cursor name, each once, in a fixed order
/// (the adapter's batch at the shell's start): the boxes' are the message layout's that a text
/// box draws (its background, frame, buttons' back, OK and Cancel, its font and the fonts'
/// palette) and the players' palette.
#[must_use]
pub fn names(layout: &ShellLayout) -> Vec<String> {
    let mut names = alloc::vec![layout.cursor.def.clone()];
    let menu = &layout.main_menu;
    names.push(menu.background.clone());
    for draw in menu.dialog.draws(&Shown::default(), "") {
        names.extend(h3_ui::paint::names(&draw).into_iter().map(String::from));
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
