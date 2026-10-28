use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use super::*;

/// A viewer's commands for the tests: two of a kind, then two alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Test {
    Go(u8),
    Pause,
    Quit,
}

fn chords(texts: &[&str]) -> Vec<Chord> {
    texts.iter().map(|t| Chord::parse(t).unwrap()).collect()
}

impl Command for Test {
    fn all() -> Vec<Self> {
        vec![Self::Go(0), Self::Go(1), Self::Pause, Self::Quit]
    }

    fn name(self) -> String {
        match self {
            Self::Go(n) => format!("go-{n}"),
            Self::Pause => "pause".into(),
            Self::Quit => "quit".into(),
        }
    }

    fn defaults() -> Vec<(Self, Vec<Chord>)> {
        vec![
            (Self::Go(0), chords(&["Ctrl+Up", "Up"])),
            (Self::Go(1), chords(&["Down"])),
            (Self::Pause, chords(&["F5"])),
            (Self::Quit, chords(&["Escape"])),
        ]
    }
}

#[test]
fn the_defaults_are_the_viewers() {
    let keys = Keys::<Test>::original();
    assert_eq!(keys.command(true, "Up"), Some(Test::Go(0)));
    assert_eq!(keys.command(false, "Up"), Some(Test::Go(0)));
    assert_eq!(keys.command(false, "Down"), Some(Test::Go(1)));
    // A chord matches exactly: Ctrl held is another chord.
    assert_eq!(keys.command(true, "Down"), None);
    assert_eq!(keys.command(false, "F5"), Some(Test::Pause));
    assert_eq!(keys.command(false, "P"), None);
    // In the table's order, each command's chords in its own.
    let chords: Vec<(Test, String)> = keys
        .chords()
        .into_iter()
        .map(|(c, k)| (c, k.to_string()))
        .collect();
    assert_eq!(
        chords,
        [
            (Test::Go(0), "Ctrl+Up".to_string()),
            (Test::Go(0), "Up".to_string()),
            (Test::Go(1), "Down".to_string()),
            (Test::Pause, "F5".to_string()),
            (Test::Quit, "Escape".to_string()),
        ]
    );
    for command in Test::all() {
        assert_eq!(Test::from_name(&command.name()), Some(command));
    }
    assert_eq!(Test::from_name("go"), None);
}

#[test]
fn chords_read_and_write() {
    assert_eq!(
        Chord::parse("Ctrl+Up"),
        Ok(Chord {
            ctrl: true,
            key: "Up".into()
        })
    );
    assert_eq!(
        Chord::parse("F5"),
        Ok(Chord {
            ctrl: false,
            key: "F5".into()
        })
    );
    for text in ["Ctrl+Up", "F5", "Escape"] {
        assert_eq!(Chord::parse(text).unwrap().to_string(), text);
    }
    for bad in ["", "Ctrl+", "Shift+Up", "Page Up", "Ctrl+Ctrl+Up"] {
        assert_eq!(
            Chord::parse(bad),
            Err(format!("`{bad}` is not a key, or Ctrl+ and a key")),
            "{bad}"
        );
    }
}

#[test]
fn a_users_file_overrides_command_by_command() {
    let keys = Keys::<Test>::with_file(
        "# mine\npause = [\"Space\", \"P\", \"P\"]\nquit = []\ngo-1 = [\"Ctrl+Down\"]\n",
    )
    .unwrap();
    assert_eq!(keys.command(false, "Space"), Some(Test::Pause));
    assert_eq!(keys.command(false, "P"), Some(Test::Pause));
    assert_eq!(keys.command(false, "F5"), None);
    // Unbound.
    assert_eq!(keys.command(false, "Escape"), None);
    assert_eq!(keys.command(false, "Down"), None);
    assert_eq!(keys.command(true, "Down"), Some(Test::Go(1)));
    // The rest kept.
    assert_eq!(keys.command(false, "Up"), Some(Test::Go(0)));
    assert!(keys.chords().iter().all(|&(c, _)| c != Test::Quit));
    // An empty file is the defaults.
    assert_eq!(Keys::with_file(""), Ok(Keys::<Test>::original()));
}

#[test]
fn a_users_file_is_checked() {
    let cases = [
        ("pause = [", None),
        (
            "jump = [\"J\"]",
            Some("`jump` is no command; the commands: go-0, go-1, pause, quit".to_string()),
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
            "go-1 = [\"Ctrl+Up\"]",
            Some("Ctrl+Up is bound to both `go-0` and `go-1`".to_string()),
        ),
    ];
    for (text, message) in cases {
        let got = Keys::<Test>::with_file(text);
        match message {
            Some(m) => assert_eq!(got, Err(m), "{text}"),
            // The parser's own message.
            None => assert!(got.is_err(), "{text}"),
        }
    }
}
