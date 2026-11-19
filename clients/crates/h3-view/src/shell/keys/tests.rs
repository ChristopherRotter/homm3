//! The shell's keys' tests (E27 M2 T6a).

use alloc::string::String;
use alloc::vec::Vec;

use h3_ui::edit::Caret;
use h3_ui::keys::{Chord, Command as _, Keys};

use super::{Command, Context, Scroll, command_for, shifted};
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
            "hot-seat-cancel",
            "scenario-list",
            "scenario-random",
            "scenario-advanced",
            "scenario-begin",
            "scenario-back",
            "scores-campaign",
            "scores-standard",
            "scores-exit",
            "edit-home",
            "edit-end",
            "edit-left",
            "edit-right",
            "edit-delete",
            "edit-backspace",
            "hot-seat-next",
            "hot-seat-previous",
            "scenario-up",
            "scenario-down",
            "scenario-page-up",
            "scenario-page-down",
            "scenario-left",
            "scenario-right"
        ]
    );
    assert_eq!(Command::from_name("quit"), Some(Command::Button(105)));
    // E27 M3 T1b1: the scenario list's keys, over the scenario screen.
    let keys = Keys::<Command>::original();
    assert_eq!(
        keys.command(Context::Scenario, false, "PageDown"),
        Some(Command::List(Scroll::PageDown))
    );
    assert_eq!(keys.command(Context::HotSeat, false, "PageDown"), None);
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

#[test]
fn an_edit_s_keys_and_the_ring_are_commands_with_shift_chords() {
    // E27 M2 T7b: the edits' own keys while one types, the ring over the hot seat's names.
    let keys = Keys::<Command>::original();
    let typing = [Context::Edit, Context::HotSeat];
    let on = |shift, key| command_for(&keys, &typing, shift, key);
    for (key, caret) in [
        ("Home", Caret::Home),
        ("End", Caret::End),
        ("Left", Caret::Left),
        ("Right", Caret::Right),
        ("Delete", Caret::Delete),
        ("BackSpace", Caret::BackSpace),
    ] {
        assert_eq!(on(false, key), Some(Command::Edit(caret)));
        // With Shift held and that chord unbound: the key alone.
        assert_eq!(on(true, key), Some(Command::Edit(caret)));
    }
    assert_eq!(on(false, "Tab"), Some(Command::Ring(true)));
    assert_eq!(on(false, "Down"), Some(Command::Ring(true)));
    assert_eq!(on(true, "Tab"), Some(Command::Ring(false)));
    assert_eq!(on(false, "Up"), Some(Command::Ring(false)));
    assert_eq!(on(false, "Return"), Some(Command::Dialog(519)));
    assert_eq!(on(true, "Escape"), Some(Command::Dialog(520)));
    assert_eq!(on(false, "A"), None);
    assert_eq!(on(true, "A"), None);
    // Not over the menus, nor the ring over the Multiplayer dialog.
    assert_eq!(
        command_for(&keys, &[Context::MainMenu], false, "Home"),
        None
    );
    assert_eq!(
        command_for(&keys, &[Context::Edit, Context::Multiplayer], false, "Tab"),
        None
    );
    assert_eq!(Command::Edit(Caret::Home).context(), Some(Context::Edit));
    assert_eq!(Command::Ring(false).context(), Some(Context::HotSeat));
    assert_eq!(
        keys.chords()
            .into_iter()
            .filter(|(c, _)| *c == Command::Ring(false))
            .map(|(_, k)| k.clone())
            .collect::<Vec<_>>(),
        [
            Chord::new(false, "Tab").with_shift(true),
            Chord::new(false, "Up")
        ]
    );
    // A user's file binds them by name, Shift chords too; an exact chord wins over the key alone.
    let keys = Keys::<Command>::with_file(
        "[shell]\nedit-home = [\"Shift+Left\"]\nhot-seat-previous = [\"PageUp\"]\n",
    )
    .unwrap();
    let on = |shift, key| command_for(&keys, &typing, shift, key);
    assert_eq!(on(true, "Left"), Some(Command::Edit(Caret::Home)));
    assert_eq!(on(false, "Left"), Some(Command::Edit(Caret::Left)));
    assert_eq!(on(false, "Home"), None);
    assert_eq!(on(true, "Tab"), Some(Command::Ring(true)));
    assert_eq!(on(false, "PageUp"), Some(Command::Ring(false)));
    // M2's review: an edit's keys are asked before its dialog's, so a chord bound in both is
    // refused; the menus' are never asked with them.
    let refused = [
        (
            "edit-home = [\"Tab\"]",
            "Tab is bound to both `edit-home` and `hot-seat-next`",
        ),
        (
            "edit-end = [\"Escape\"]",
            "Escape is bound to both `multiplayer-cancel` and `edit-end`",
        ),
        (
            "edit-left = [\"Return\"]",
            "Return is bound to both `hot-seat-ok` and `edit-left`",
        ),
    ];
    for (line, message) in refused {
        let got = Keys::<Command>::with_file(&alloc::format!("[shell]\n{line}\n"));
        assert_eq!(got, Err(alloc::format!("[shell] {message}")), "{line}");
    }
    let keys = Keys::<Command>::with_file("[shell]\nedit-home = [\"N\"]\n").unwrap();
    assert_eq!(
        command_for(&keys, &[Context::MainMenu], false, "N"),
        Some(Command::Button(101))
    );
    assert_eq!(on_keys(&keys, "N"), Some(Command::Edit(Caret::Home)));
}

/// The command `key` gives with an edit typing over the hot seat's names.
fn on_keys(keys: &Keys<Command>, key: &str) -> Option<Command> {
    command_for(keys, &[Context::Edit, Context::HotSeat], false, key)
}

#[test]
fn a_name_s_shift_is_read_off_it() {
    assert_eq!(shifted("shift+Tab"), (true, "Tab"));
    assert_eq!(shifted("Shift+a"), (true, "a"));
    assert_eq!(shifted("Tab"), (false, "Tab"));
    assert_eq!(shifted("shift"), (false, "shift"));
}

#[test]
fn the_scenario_screen_s_hotkeys_are_the_original_s() {
    // E27 M3 T1a: S, R, A, Enter and Escape on the scenario screen alone.
    let keys = Keys::<Command>::original();
    let on = |key| keys.command(Context::Scenario, false, key);
    let ids: Vec<Option<Command>> = ["S", "R", "A", "Return", "Escape"].map(on).into();
    assert_eq!(
        ids,
        [128, 130, 129, 186, 188].map(|id| Some(Command::Dialog(id)))
    );
    assert_eq!(keys.command(Context::GameType, false, "R"), None);
}
