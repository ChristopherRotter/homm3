use alloc::string::ToString;
use alloc::vec::Vec;

use h3_ui::keys::Command as _;

use super::*;

#[test]
fn the_defaults_are_the_windows_and_the_originals_by_context() {
    let keys = Keys::original();
    let on = |context, key| keys.command(context, false, key);
    // The window's own: the map's, the ones that work whatever is open everywhere.
    for w in Window::ALL {
        let over = if w.everywhere() {
            Some(Command::Window(w))
        } else {
            None
        };
        assert_eq!(on(Context::Battle, w.key()), Some(Command::Window(w)));
        for context in [Context::Book, Context::Message, Context::Aim] {
            let got = on(context, w.key());
            if w == Window::Quit {
                assert_ne!(got, Some(Command::Window(w)), "{context:?}");
            } else {
                assert_eq!(got, over, "{w:?} {context:?}");
            }
        }
    }
    // With nothing over the battle.
    let battle = [
        ("Up", Command::LogUp),
        ("Down", Command::LogDown),
        ("F", Command::DragonSpell),
        ("S", Command::Button(panel::SURRENDER)),
        ("R", Command::Button(panel::RETREAT)),
        ("O", Command::Button(panel::OPTIONS)),
        ("A", Command::Button(panel::AUTO_COMBAT)),
        ("C", Command::Button(panel::SPELL_BOOK)),
        ("W", Command::Button(panel::WAIT)),
        ("D", Command::Button(panel::DEFEND)),
        ("Space", Command::Button(panel::DEFEND)),
        ("Escape", Command::Window(Window::Quit)),
    ];
    for (key, command) in battle {
        assert_eq!(on(Context::Battle, key), Some(command), "{key}");
        // A chord matches exactly.
        assert_eq!(keys.command(Context::Battle, true, key), None, "{key}");
    }
    for key in ["Return", "Left", "Right", "RightButton", "P", "M", "Home"] {
        assert_eq!(on(Context::Battle, key), None, "{key}");
    }
    // The book's.
    let book = [
        ("Escape", book::Key::Close),
        ("Return", book::Key::Close),
        ("A", book::Key::Adventure),
        ("C", book::Key::Combat),
        ("Left", book::Key::Previous),
        ("Right", book::Key::Next),
        ("Up", book::Key::TabUp),
        ("Down", book::Key::TabDown),
    ];
    for (key, k) in book {
        assert_eq!(on(Context::Book, key), Some(Command::Book(k)), "{key}");
    }
    // The book is modal: the battle's keys do nothing while it is open.
    for key in ["S", "W", "F", "Space"] {
        assert_eq!(on(Context::Book, key), None, "{key}");
    }
    // A box's, and an aim's.
    assert_eq!(
        on(Context::Message, "Return"),
        Some(Command::Message(message::Key::Ok))
    );
    assert_eq!(
        on(Context::Message, "Escape"),
        Some(Command::Message(message::Key::Cancel))
    );
    assert_eq!(on(Context::Message, "A"), None);
    assert_eq!(on(Context::Aim, "Escape"), Some(Command::AimCancel));
    assert_eq!(on(Context::Aim, "RightButton"), Some(Command::AimCancel));
    assert_eq!(on(Context::Aim, "Up"), None);
    // The defaults bind no key twice in one context.
    assert_eq!(Keys::with_file(""), Ok(keys));
}

#[test]
fn every_command_has_a_name_in_the_tables_order() {
    let all = Command::all();
    assert_eq!(all.len(), 29);
    let mut sorted = all.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, all);
    for &command in &all {
        assert_eq!(Command::from_name(&command.name()), Some(command));
    }
    let names: Vec<String> = all.iter().map(|c| c.name()).collect();
    assert_eq!(
        names[9..],
        [
            "log-up",
            "log-down",
            "dragon-spell",
            "surrender",
            "retreat",
            "options",
            "auto-combat",
            "spell-book",
            "wait",
            "defend",
            "book-close",
            "book-adventure",
            "book-combat",
            "book-previous",
            "book-next",
            "book-tab-up",
            "book-tab-down",
            "box-ok",
            "box-cancel",
            "aim-cancel"
        ]
    );
    assert_eq!(names[..2], ["music", "effects"]);
    // A button without a hotkey has no name of its own and no key.
    assert_eq!(Command::Button(panel::LOG_UP).name(), "button-2006");
    assert_eq!(Command::from_name("button-2006"), None);
    // Every command bound by default, in the table's order.
    let keys = Keys::original();
    let chords = keys.chords();
    assert_eq!(chords.len(), 32);
    let mut commands: Vec<Command> = chords.iter().map(|&(c, _)| c).collect();
    commands.dedup();
    assert_eq!(commands, all);
    assert_eq!(chords[0].1.to_string(), "F2");
    assert_eq!(chords[31].1.to_string(), "RightButton");
}

#[test]
fn the_context_is_the_topmost_thing_open() {
    assert_eq!(Context::of(false, false, false), Context::Battle);
    assert_eq!(Context::of(false, false, true), Context::Aim);
    assert_eq!(Context::of(false, true, true), Context::Book);
    assert_eq!(Context::of(true, true, false), Context::Message);
}

#[test]
fn a_users_file_rebinds_the_battles_keys() {
    let keys = Keys::with_file(
        "[battle]\npause = [\"P\"]\nlog-up = [\"Ctrl+Up\"]\nbook-close = [\"B\"]\n\
         surrender = [\"Ctrl+S\"]\n[map]\npause = [\"Q\"]",
    )
    .unwrap();
    let on = |context, ctrl, key| keys.command(context, ctrl, key);
    // Pause works whatever is open.
    assert_eq!(
        on(Context::Book, false, "P"),
        Some(Command::Window(Window::Pause))
    );
    assert_eq!(on(Context::Battle, false, "F5"), None);
    assert_eq!(on(Context::Battle, false, "Q"), None);
    assert_eq!(on(Context::Battle, true, "Up"), Some(Command::LogUp));
    assert_eq!(on(Context::Battle, false, "Up"), None);
    assert_eq!(
        on(Context::Battle, true, "S"),
        Some(Command::Button(panel::SURRENDER))
    );
    assert_eq!(
        on(Context::Book, false, "B"),
        Some(Command::Book(book::Key::Close))
    );
    assert_eq!(on(Context::Book, false, "Escape"), None);
    // A key the book takes cannot work whatever is open, nor one the battle's buttons take twice.
    assert_eq!(
        Keys::with_file("[battle]\nmusic = [\"Left\"]"),
        Err("[battle] Left is bound to both `music` and `book-previous`".to_string())
    );
    assert_eq!(
        Keys::with_file("[battle]\nwait = [\"Space\"]"),
        Err("[battle] Space is bound to both `wait` and `defend`".to_string())
    );
    // A key of the battle's and the book's at once is fine.
    assert!(Keys::with_file("[battle]\nbook-next = [\"S\"]").is_ok());
}
