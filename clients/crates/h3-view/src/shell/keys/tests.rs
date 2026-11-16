//! The shell's keys' tests (E27 M2 T6a).

use alloc::string::String;
use alloc::vec::Vec;

use h3_ui::keys::{Command as _, Keys};

use super::{Command, Context};
use crate::message;
use crate::shell::Choice;

#[test]
fn the_main_menu_s_hotkeys_and_the_box_s_keys_are_the_original_s() {
    let keys = Keys::<Command>::original();
    let on = |context, key| keys.command(context, false, key);
    assert_eq!(on(Context::MainMenu, "N"), Some(Command::Button(101)));
    assert_eq!(on(Context::MainMenu, "L"), Some(Command::Button(102)));
    assert_eq!(on(Context::MainMenu, "H"), Some(Command::Button(103)));
    assert_eq!(on(Context::MainMenu, "C"), Some(Command::Button(104)));
    // Escape is Quit's over the main menu, the box's Cancel over a box.
    assert_eq!(on(Context::MainMenu, "Escape"), Some(Command::Button(105)));
    assert_eq!(
        on(Context::Question, "Escape"),
        Some(Command::Message(message::Key::Cancel))
    );
    assert_eq!(
        on(Context::Question, "Return"),
        Some(Command::Message(message::Key::Ok))
    );
    // The menu's keys do nothing over the box, nor the box's over the menu.
    assert_eq!(on(Context::Question, "N"), None);
    assert_eq!(on(Context::MainMenu, "Return"), None);
    assert_eq!(keys.command(Context::MainMenu, true, "N"), None);
}

#[test]
fn the_game_type_menus_hotkeys_are_the_original_s() {
    // E27 M2 T6b1: S, M, C, T and Escape; C and Escape are other buttons' over the main menu.
    let keys = Keys::<Command>::original();
    let on = |context, key| keys.command(context, false, key);
    for (key, choice) in [
        ("S", Choice::Single),
        ("M", Choice::Multiplayer),
        ("C", Choice::Campaign),
        ("T", Choice::Tutorial),
        ("Escape", Choice::Back),
    ] {
        assert_eq!(on(Context::GameType, key), Some(Command::Choice(choice)));
    }
    assert_eq!(on(Context::MainMenu, "C"), Some(Command::Button(104)));
    assert_eq!(on(Context::GameType, "N"), None);
    assert_eq!(on(Context::MainMenu, "S"), None);
}

#[test]
fn the_commands_have_names_in_the_shell_s_table() {
    assert_eq!(Command::TABLE, "shell");
    let names: Vec<String> = Command::all().into_iter().map(Command::name).collect();
    assert_eq!(
        names,
        [
            "new-game",
            "load-game",
            "high-scores",
            "credits",
            "quit",
            "single",
            "multiplayer",
            "campaign",
            "tutorial",
            "back",
            "box-ok",
            "box-cancel",
            "multiplayer-cancel",
            "hot-seat-ok",
            "hot-seat-cancel"
        ]
    );
    assert_eq!(Command::from_name("quit"), Some(Command::Button(105)));
    assert_eq!(
        Command::from_name("back"),
        Some(Command::Choice(Choice::Back))
    );
    // A button without a hotkey has no name.
    assert_eq!(Command::Button(1).name(), "");
    // A user's file rebinds them.
    let keys = Keys::<Command>::with_file("[shell]\nquit = [\"Q\"]\n").unwrap();
    assert_eq!(
        keys.command(Context::MainMenu, false, "Q"),
        Some(Command::Button(105))
    );
    assert_eq!(keys.command(Context::MainMenu, false, "Escape"), None);
}

#[test]
fn a_keys_name_reads_as_its_scan_code() {
    // E27 M2 T7: the harness's (`xdotool`'s) names and the adapter's.
    let at = |name| super::scan(name).map(|s| (s.code, s.shift));
    assert_eq!(at("e"), Some((0x12, false)));
    assert_eq!(at("E"), Some((0x12, false)));
    assert_eq!(at("1"), Some((0x02, false)));
    assert_eq!(at("shift+Tab"), Some((0x0F, true)));
    assert_eq!(at("SHIFT+a"), Some((0x1E, true)));
    assert_eq!(at("BackSpace"), Some((0x0E, false)));
    assert_eq!(at("Next"), Some((0x51, false)));
    assert_eq!(at("F1"), Some((0x3B, false)));
    assert_eq!(at("F10"), Some((0x44, false)));
    assert_eq!(at("F11"), Some((0x57, false)));
    assert_eq!(at("F12"), Some((0x58, false)));
    assert_eq!(at("F13"), None);
    assert_eq!(at("Fx"), None);
    assert_eq!(at("F"), Some((0x21, false)));
    assert_eq!(at("~"), None);
    assert_eq!(at("Menu"), None);
}

#[test]
fn the_dialogs_hotkeys_are_taken_over_their_dialogs() {
    let keys = Keys::<Command>::original();
    assert_eq!(
        keys.command(Context::Multiplayer, false, "Escape"),
        Some(Command::Dialog(124))
    );
    assert_eq!(
        keys.command(Context::HotSeat, false, "Return"),
        Some(Command::Dialog(519))
    );
    assert_eq!(
        keys.command(Context::HotSeat, false, "Escape"),
        Some(Command::Dialog(520))
    );
    assert_eq!(keys.command(Context::Multiplayer, false, "Return"), None);
    // An id without a hotkey: no name, no context.
    assert_eq!(Command::Dialog(1).name(), "");
    assert_eq!(Command::Dialog(1).context(), None);
}
