use alloc::string::ToString;
use alloc::vec::Vec;

use h3_ui::keys::Command as _;

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
            assert_eq!(keys.command(true, k), Some(Command::Scroll(d)), "{k}");
            assert_eq!(keys.command(false, k), Some(Command::Hero(d)), "{k}");
        }
    }
    for (key, command) in [
        ("F2", Command::Music),
        ("F3", Command::Effects),
        ("F5", Command::Pause),
        ("F6", Command::Frame),
        ("F7", Command::Restart),
        ("F8", Command::Vsync),
        ("F9", Command::Cursor),
        ("F11", Command::Fullscreen),
        ("Escape", Command::Quit),
    ] {
        assert_eq!(keys.command(false, key), Some(command), "{key}");
        // A chord matches exactly: Ctrl held is another chord.
        assert_eq!(keys.command(true, key), None, "{key}");
    }
    assert_eq!(keys.command(false, "P"), None);
    // Every command bound, in the table's order: the map's twice (the key and the keypad's).
    let chords = keys.chords();
    assert_eq!(chords.len(), 41);
    let mut commands: Vec<Command> = chords.iter().map(|&(c, _)| c).collect();
    commands.dedup();
    assert_eq!(commands, Command::all());
    assert_eq!(chords[0].1.to_string(), "Ctrl+Up");
    assert_eq!(chords[1].1.to_string(), "Ctrl+Keypad8");
    assert_eq!(chords[16].1.to_string(), "Up");
    assert_eq!(chords[40].1.to_string(), "Escape");
}

#[test]
fn every_command_has_a_name_and_the_maps_its_action() {
    let all = Command::all();
    assert_eq!(all.len(), 25);
    for &command in &all {
        assert_eq!(Command::from_name(&command.name()), Some(command));
    }
    let names: Vec<String> = all.iter().map(|c| c.name()).collect();
    assert_eq!(names[..2], ["scroll-north", "scroll-north-east"]);
    assert_eq!(names[13], "hero-south-west");
    assert_eq!(
        names[16..],
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
    assert_eq!(Command::Pause.action(), None);
}

#[test]
fn a_users_file_overrides_command_by_command() {
    let keys = Keys::with_file(
        "# mine\npause = [\"Space\", \"P\", \"P\"]\nquit = []\nhero-north = [\"W\"]\n\
         scroll-north = [\"Ctrl+W\", \"Ctrl+Up\"]\n",
    )
    .unwrap();
    assert_eq!(keys.command(false, "Space"), Some(Command::Pause));
    assert_eq!(keys.command(false, "P"), Some(Command::Pause));
    assert_eq!(keys.command(false, "F5"), None);
    // Unbound.
    assert_eq!(keys.command(false, "Escape"), None);
    assert_eq!(keys.command(false, "Up"), None);
    assert_eq!(keys.command(false, "Keypad8"), None);
    assert_eq!(
        keys.command(false, "W"),
        Some(Command::Hero(Direction::North))
    );
    assert_eq!(
        keys.command(true, "W"),
        Some(Command::Scroll(Direction::North))
    );
    assert_eq!(
        keys.command(true, "Up"),
        Some(Command::Scroll(Direction::North))
    );
    // The rest kept.
    assert_eq!(keys.command(false, "F6"), Some(Command::Frame));
    let chords = keys.chords();
    assert_eq!(chords.len(), 41);
    assert!(chords.iter().all(|&(c, _)| c != Command::Quit));
    assert_eq!(chords[0].1.to_string(), "Ctrl+W");
    // An empty file is the defaults.
    assert_eq!(Keys::with_file(""), Ok(Keys::original()));
}

#[test]
fn a_users_file_is_checked() {
    let all: Vec<String> = Command::all().into_iter().map(Command::name).collect();
    let cases = [
        ("pause = [", None),
        (
            "jump = [\"J\"]",
            Some(format!(
                "`jump` is no command; the commands: {}",
                all.join(", ")
            )),
        ),
        (
            "pause = \"P\"",
            Some("`pause` is not a list of keys".to_string()),
        ),
        (
            "pause = [1]",
            Some("`pause` is not a list of keys".to_string()),
        ),
        (
            "pause = [\"\"]",
            Some("`pause`: `` is not a key, or Ctrl+ and a key".to_string()),
        ),
        (
            "pause = [\"Escape\"]",
            Some("Escape is bound to both `pause` and `quit`".to_string()),
        ),
        (
            "hero-east = [\"Ctrl+Up\"]",
            Some("Ctrl+Up is bound to both `scroll-north` and `hero-east`".to_string()),
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
