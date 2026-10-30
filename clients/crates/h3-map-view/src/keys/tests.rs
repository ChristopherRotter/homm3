use alloc::string::ToString;
use alloc::vec::Vec;

use h3_ui::keys::{Command as _, Window};

use super::*;

#[test]
fn the_defaults_are_the_originals_keys_and_the_windows() {
    let keys = Keys::original();
    let directions = [
        ("Up", "Keypad8", Direction::North),
        ("PageUp", "Keypad9", Direction::NorthEast),
        ("Right", "Keypad6", Direction::East),
        ("PageDown", "Keypad3", Direction::SouthEast),
        ("Down", "Keypad2", Direction::South),
        ("End", "Keypad1", Direction::SouthWest),
        ("Left", "Keypad4", Direction::West),
        ("Home", "Keypad7", Direction::NorthWest),
    ];
    for (key, pad, d) in directions {
        for k in [key, pad] {
            assert_eq!(keys.command((), true, k), Some(Command::Scroll(d)), "{k}");
            assert_eq!(keys.command((), false, k), Some(Command::Hero(d)), "{k}");
        }
    }
    for (key, command) in [
        ("F2", Window::Music),
        ("F3", Window::Effects),
        ("F5", Window::Pause),
        ("F6", Window::Frame),
        ("F7", Window::Restart),
        ("F8", Window::Vsync),
        ("F9", Window::Cursor),
        ("F11", Window::Fullscreen),
        ("Escape", Window::Quit),
    ] {
        let command = Command::Window(command);
        assert_eq!(keys.command((), false, key), Some(command), "{key}");
        // A chord matches exactly: Ctrl held is another chord.
        assert_eq!(keys.command((), true, key), None, "{key}");
    }
    assert_eq!(keys.command((), false, "P"), None);
    // The panels' hotkeys (E24 M3 T6b1): the original's letters.
    for (key, hotkey) in [
        ("K", Hotkey::Kingdom),
        ("U", Hotkey::Underground),
        ("Q", Hotkey::Quests),
        ("Z", Hotkey::Sleep),
        ("W", Hotkey::Wake),
        ("M", Hotkey::MoveHero),
        ("C", Hotkey::Cast),
        ("A", Hotkey::Adventure),
        ("O", Hotkey::System),
        ("H", Hotkey::NextHero),
        ("E", Hotkey::EndTurn),
    ] {
        assert_eq!(keys.command((), false, key), Some(Command::Panel(hotkey)));
        assert_eq!(keys.command((), true, key), None);
    }
    // Every command bound, in the table's order: the map's twice (the key and the keypad's).
    let chords = keys.chords();
    assert_eq!(chords.len(), 53);
    let mut commands: Vec<Command> = chords.iter().map(|&(c, _)| c).collect();
    commands.dedup();
    assert_eq!(commands, Command::all());
    assert_eq!(chords[0].1.to_string(), "Ctrl+Up");
    assert_eq!(chords[1].1.to_string(), "Ctrl+Keypad8");
    assert_eq!(chords[16].1.to_string(), "Up");
    assert_eq!(chords[32].1.to_string(), "K");
    assert_eq!(chords[43].1.to_string(), "Space");
    assert_eq!(chords[52].1.to_string(), "Escape");
}

#[test]
fn every_command_has_a_name_and_the_maps_its_action() {
    let all = Command::all();
    assert_eq!(all.len(), 37);
    for &command in &all {
        assert_eq!(Command::from_name(&command.name()), Some(command));
    }
    let names: Vec<String> = all.iter().map(|c| c.name()).collect();
    assert_eq!(names[..2], ["scroll-north", "scroll-north-east"]);
    assert_eq!(names[13], "hero-south-west");
    assert_eq!(
        names[16..28],
        [
            "kingdom-overview",
            "underground",
            "quest-log",
            "sleep",
            "wake",
            "move-hero",
            "cast-spell",
            "adventure-options",
            "system-options",
            "next-hero",
            "end-turn",
            "visit"
        ]
    );
    assert_eq!(
        names[28..],
        [
            "music",
            "effects",
            "pause",
            "frame",
            "restart",
            "vsync",
            "cursor",
            "fullscreen",
            "quit"
        ]
    );
    assert_eq!(Command::from_name("scroll"), None);
    assert_eq!(
        Command::Scroll(Direction::West).action(),
        Some(Action::Scroll(Direction::West))
    );
    assert_eq!(
        Command::Hero(Direction::SouthEast).action(),
        Some(Action::Step(Direction::SouthEast))
    );
    assert_eq!(Command::Window(Window::Pause).action(), None);
    // A hotkey's key down presses its button, its release clicks it (E24 M3 T6b1); the rest's
    // release does nothing.
    let next = Command::Panel(Hotkey::NextHero);
    assert_eq!(next.action(), Some(Action::Key(Hotkey::NextHero)));
    assert_eq!(next.release(), Some(Action::KeyUp(Hotkey::NextHero)));
    // Space (E24 M3 T6b3); the map's other keys' release is a message that does nothing, the
    // window's own none.
    assert_eq!(Command::Visit.action(), Some(Action::Visit));
    assert_eq!(Command::Hero(Direction::North).release(), Some(Action::Up));
    assert_eq!(Command::Scroll(Direction::North).release(), Some(Action::Up));
    assert_eq!(Command::Visit.release(), Some(Action::Up));
    assert_eq!(Command::Window(Window::Pause).release(), None);
}

#[test]
fn a_users_file_overrides_command_by_command() {
    let keys = Keys::with_file(
        "# mine\n[map]\npause = [\"Space\", \"P\", \"P\"]\nquit = []\nhero-north = [\"W\"]\n\
         scroll-north = [\"Ctrl+W\", \"Ctrl+Up\"]\nwake = [\"J\"]\nvisit = []\n",
    )
    .unwrap();
    assert_eq!(
        keys.command((), false, "Space"),
        Some(Command::Window(Window::Pause))
    );
    assert_eq!(
        keys.command((), false, "P"),
        Some(Command::Window(Window::Pause))
    );
    assert_eq!(keys.command((), false, "F5"), None);
    // Unbound.
    assert_eq!(keys.command((), false, "Escape"), None);
    assert_eq!(keys.command((), false, "Up"), None);
    assert_eq!(keys.command((), false, "Keypad8"), None);
    assert_eq!(
        keys.command((), false, "W"),
        Some(Command::Hero(Direction::North))
    );
    assert_eq!(
        keys.command((), true, "W"),
        Some(Command::Scroll(Direction::North))
    );
    assert_eq!(
        keys.command((), true, "Up"),
        Some(Command::Scroll(Direction::North))
    );
    // The rest kept.
    assert_eq!(
        keys.command((), false, "F6"),
        Some(Command::Window(Window::Frame))
    );
    let chords = keys.chords();
    assert_eq!(chords.len(), 52);
    assert!(
        chords
            .iter()
            .all(|&(c, _)| c != Command::Window(Window::Quit))
    );
    assert_eq!(chords[0].1.to_string(), "Ctrl+W");
    // An empty file is the defaults.
    assert_eq!(Keys::with_file(""), Ok(Keys::original()));
    // The battle's table is the battle's.
    assert_eq!(Keys::with_file("[battle]\nquit = []"), Ok(Keys::original()));
}

#[test]
fn a_users_file_is_checked() {
    let all: Vec<String> = Command::all().into_iter().map(Command::name).collect();
    let cases = [
        ("[map]\npause = [", None),
        (
            "[map]\njump = [\"J\"]",
            Some(format!(
                "[map] `jump` is no command; the commands: {}",
                all.join(", ")
            )),
        ),
        (
            "[map]\npause = \"P\"",
            Some("[map] `pause` is not a list of keys".to_string()),
        ),
        (
            "[map]\npause = [1]",
            Some("[map] `pause` is not a list of keys".to_string()),
        ),
        (
            "[map]\npause = [\"\"]",
            Some("[map] `pause`: `` is not a key, or Ctrl+ and a key".to_string()),
        ),
        (
            "[map]\npause = [\"Escape\"]",
            Some("[map] Escape is bound to both `pause` and `quit`".to_string()),
        ),
        (
            "[map]\nhero-east = [\"Ctrl+Up\"]",
            Some("[map] Ctrl+Up is bound to both `scroll-north` and `hero-east`".to_string()),
        ),
    ];
    for (text, message) in cases {
        let got = Keys::with_file(text);
        match message {
            Some(m) => assert_eq!(got, Err(m), "{text}"),
            // The parser's own message.
            None => assert!(got.is_err(), "{text}"),
        }
    }
}
