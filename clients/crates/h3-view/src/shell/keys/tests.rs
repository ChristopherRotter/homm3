//! The shell's keys' tests (E27 M2 T6a).

use alloc::string::String;
use alloc::vec::Vec;

use h3_ui::keys::{Command as _, Keys};

use super::{Command, Context};
use crate::message;

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
            "box-ok",
            "box-cancel"
        ]
    );
    assert_eq!(Command::from_name("quit"), Some(Command::Button(105)));
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
