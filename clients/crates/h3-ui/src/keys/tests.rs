use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use super::*;

/// A viewer's commands for the tests: two of a kind, two alone, one taken only with a dialog open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Test {
    Go(u8),
    Pause,
    Quit,
    Close,
}

/// Where the test viewer takes its keys: its screen, or a dialog over it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Over {
    Nothing,
    Dialog,
}

fn chords(texts: &[&str]) -> Vec<Chord> {
    texts.iter().map(|t| Chord::parse(t).unwrap()).collect()
}

impl Command for Test {
    type Context = Over;
    const TABLE: &'static str = "battle";

    fn all() -> Vec<Self> {
        vec![
            Self::Go(0),
            Self::Go(1),
            Self::Pause,
            Self::Quit,
            Self::Close,
        ]
    }

    fn name(self) -> String {
        match self {
            Self::Go(n) => format!("go-{n}"),
            Self::Pause => "pause".into(),
            Self::Quit => "quit".into(),
            Self::Close => "close".into(),
        }
    }

    fn defaults() -> Vec<(Self, Vec<Chord>)> {
        vec![
            (Self::Go(0), chords(&["Ctrl+Up", "Up"])),
            (Self::Go(1), chords(&["Down"])),
            (Self::Pause, chords(&["F5"])),
            (Self::Quit, chords(&["Escape"])),
            (Self::Close, chords(&["Escape", "Return"])),
        ]
    }

    fn context(self) -> Option<Over> {
        match self {
            Self::Pause => None,
            Self::Close => Some(Over::Dialog),
            _ => Some(Over::Nothing),
        }
    }
}

#[test]
fn the_defaults_are_the_viewers() {
    let keys = Keys::<Test>::original();
    let on = |ctrl, key| keys.command(Over::Nothing, ctrl, key);
    assert_eq!(on(true, "Up"), Some(Test::Go(0)));
    assert_eq!(on(false, "Up"), Some(Test::Go(0)));
    assert_eq!(on(false, "Down"), Some(Test::Go(1)));
    // A chord matches exactly: Ctrl held is another chord.
    assert_eq!(on(true, "Down"), None);
    assert_eq!(on(false, "F5"), Some(Test::Pause));
    assert_eq!(on(false, "P"), None);
    // A key gives each context's command: Escape quits, or closes the dialog; the keys without a
    // context work in both, the screen's not over a dialog.
    assert_eq!(on(false, "Escape"), Some(Test::Quit));
    assert_eq!(on(false, "Return"), None);
    let over = |key| keys.command(Over::Dialog, false, key);
    assert_eq!(over("Escape"), Some(Test::Close));
    assert_eq!(over("Return"), Some(Test::Close));
    assert_eq!(over("F5"), Some(Test::Pause));
    assert_eq!(over("Up"), None);
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
            (Test::Close, "Escape".to_string()),
            (Test::Close, "Return".to_string()),
        ]
    );
    for command in Test::all() {
        assert_eq!(Test::from_name(&command.name()), Some(command));
    }
    assert_eq!(Test::from_name("go"), None);
    // The defaults bind no key twice in one context.
    assert_eq!(Keys::with_file(""), Ok(keys));
}

#[test]
fn the_windows_own_commands_are_both_viewers() {
    let names: Vec<&str> = Window::ALL.iter().map(|w| w.name()).collect();
    assert_eq!(
        names,
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
    let keys: Vec<&str> = Window::ALL.iter().map(|w| w.key()).collect();
    assert_eq!(
        keys,
        ["F2", "F3", "F5", "F6", "F7", "F8", "F9", "F11", "Escape"]
    );
    let everywhere: Vec<Window> = Window::ALL
        .into_iter()
        .filter(|w| !w.everywhere())
        .collect();
    assert_eq!(everywhere, [Window::Frame, Window::Restart, Window::Quit]);
    // In the table's order.
    let mut sorted = Window::ALL;
    sorted.sort_unstable();
    assert_eq!(sorted, Window::ALL);
}

#[test]
fn chords_read_and_write() {
    assert_eq!(Chord::parse("Ctrl+Up"), Ok(Chord::new(true, "Up")));
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
        "# mine\n[battle]\npause = [\"Space\", \"P\", \"P\"]\nquit = []\ngo-1 = [\"Ctrl+Down\"]\n\
         [map]\npause = [\"Q\"]\n",
    )
    .unwrap();
    let on = |ctrl, key| keys.command(Over::Nothing, ctrl, key);
    assert_eq!(on(false, "Space"), Some(Test::Pause));
    assert_eq!(on(false, "P"), Some(Test::Pause));
    assert_eq!(on(false, "F5"), None);
    // The other viewer's table is its own.
    assert_eq!(on(false, "Q"), None);
    // Unbound.
    assert_eq!(on(false, "Escape"), None);
    assert_eq!(on(false, "Down"), None);
    assert_eq!(on(true, "Down"), Some(Test::Go(1)));
    // The rest kept.
    assert_eq!(on(false, "Up"), Some(Test::Go(0)));
    assert_eq!(
        keys.command(Over::Dialog, false, "Escape"),
        Some(Test::Close)
    );
    assert!(keys.chords().iter().all(|&(c, _)| c != Test::Quit));
    // An empty file, or one with the other viewer's table alone, is the defaults.
    assert_eq!(Keys::with_file(""), Ok(Keys::<Test>::original()));
    assert_eq!(
        Keys::with_file("[map]\nquit = []"),
        Ok(Keys::<Test>::original())
    );
    // A key bound in two contexts, once each.
    let both = Keys::<Test>::with_file("[battle]\nclose = [\"Up\"]").unwrap();
    assert_eq!(both.command(Over::Dialog, false, "Up"), Some(Test::Close));
    assert_eq!(both.command(Over::Nothing, false, "Up"), Some(Test::Go(0)));
}

#[test]
fn a_users_file_is_checked() {
    let cases = [
        ("[battle]\npause = [", None),
        (
            "pause = [\"P\"]",
            Some("`pause` is no viewer's table; the tables: map, battle".to_string()),
        ),
        (
            "battle = 1",
            Some("`battle` is not a table of commands".to_string()),
        ),
        (
            "[battle]\njump = [\"J\"]",
            Some(
                "[battle] `jump` is no command; the commands: go-0, go-1, pause, quit, close"
                    .to_string(),
            ),
        ),
        (
            "[battle]\npause = \"P\"",
            Some("[battle] `pause` is not a list of keys".to_string()),
        ),
        (
            "[battle]\npause = [1]",
            Some("[battle] `pause` is not a list of keys".to_string()),
        ),
        (
            "[battle]\npause = [\"\"]",
            Some("[battle] `pause`: `` is not a key, or Ctrl+ and a key".to_string()),
        ),
        (
            "[battle]\npause = [\"Escape\"]",
            Some("[battle] Escape is bound to both `pause` and `quit`".to_string()),
        ),
        (
            "[battle]\ngo-1 = [\"Ctrl+Up\"]",
            Some("[battle] Ctrl+Up is bound to both `go-0` and `go-1`".to_string()),
        ),
        // The key without a context and the dialog's.
        (
            "[battle]\npause = [\"Return\"]",
            Some("[battle] Return is bound to both `pause` and `close`".to_string()),
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

#[test]
fn the_keys_bound_are_the_adapters() {
    let known = [
        "Up",
        "Down",
        "F5",
        "Escape",
        "Return",
        "LeftCtrl",
        "RightCtrl",
    ];
    assert_eq!(Keys::<Test>::original().check(&known), Ok(()));
    let unknown = Keys::<Test>::with_file("[battle]\npause = [\"Backspace\"]").unwrap();
    assert_eq!(
        unknown.check(&known),
        Err("`pause`: Backspace: no such key; the keys: Up, Down, F5, Escape, Return".to_string())
    );
    let alone = Keys::<Test>::with_file("[battle]\npause = [\"Ctrl+LeftCtrl\"]").unwrap();
    assert_eq!(
        alone.check(&known),
        Err(
            "`pause`: Ctrl+LeftCtrl: Control is held with a key (`Ctrl+`), not bound alone"
                .to_string()
        )
    );
}
