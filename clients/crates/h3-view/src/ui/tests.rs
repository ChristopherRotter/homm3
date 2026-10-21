use super::*;
use alloc::format;
use alloc::string::ToString;
use alloc::vec;

/// A dialog at (10, 20): a picture, a button with the default frames, one with its own, a text.
const DIALOG: &str = r#"
place = [10, 20]

[[items]]
kind = "picture"
id = 1
picture = "bar.pcx"
at = [0, 0]

[[items]]
kind = "button"
id = 2
def = "b.def"
at = [3, 4]
size = [48, 36]
hint = "k.two"
keys = ["W", "Space"]

[[items]]
kind = "button"
id = 3
def = "s.def"
at = [5, 6]
size = [18, 17]
frames = { normal = 2, pressed = 3, disabled = 3 }

[[items]]
kind = "text"
id = 4
picture = "t.pcx"
at = [7, 8]
size = [40, 32]
font = "f.fnt"
font_palette = "game.pal"
colour = 1
align = 1
"#;

fn dialog() -> Dialog {
    toml::from_str(DIALOG).unwrap()
}

#[test]
fn a_dialog_reads_its_items_in_order() {
    let d = dialog();
    let ids: Vec<u16> = d.items.iter().map(Item::id).collect();
    assert_eq!(ids, [1, 2, 3, 4]);
    let Some(Item::Button(b)) = d.item(2) else {
        panic!("item 2 is a button");
    };
    // The constructor's frames by default (0x455F5D: disabled 2, selected 3).
    assert_eq!(b.frames, Frames::default());
    assert_eq!(
        (
            b.frames.normal,
            b.frames.pressed,
            b.frames.disabled,
            b.frames.selected
        ),
        (0, 1, 2, 3)
    );
    let Some(Item::Button(s)) = d.item(3) else {
        panic!("item 3 is a button");
    };
    assert_eq!(
        (
            s.frames.normal,
            s.frames.pressed,
            s.frames.disabled,
            s.frames.selected
        ),
        (2, 3, 3, 3)
    );
    assert_eq!(d.text(4).map(|t| t.size), Some([40, 32]));
    // Not a text, not there.
    assert_eq!(d.text(1), None);
    assert_eq!(d.text(9), None);
    assert_eq!(d.item(9), None);
}

#[test]
fn the_item_under_the_mouse_gives_its_hint() {
    // E17 M3 T6c-1: the last item in order whose area holds the point; a picture takes none.
    let d = dialog();
    let at = |x, y| d.item_at([x, y]).map(Item::id);
    assert_eq!(at(13, 24), Some(2));
    assert_eq!(at(20, 30), Some(4));
    assert_eq!(at(15, 26), Some(3));
    assert_eq!(at(40, 50), Some(4));
    assert_eq!(at(60, 59), Some(2));
    assert_eq!(at(61, 59), None);
    assert_eq!(at(10, 20), None);
    let hints: Vec<Option<&str>> = d.items.iter().map(Item::hint).collect();
    assert_eq!(hints, [None, Some("k.two"), None, None]);
    assert_eq!(d.items[0].area(), None);
    // E17 M3 T6c-2: the button a click presses, the one a hotkey does.
    assert_eq!(d.button_at([13, 24]), Some(2));
    assert_eq!(d.button_at([20, 30]), None);
    assert_eq!(d.button_at([61, 59]), None);
    assert_eq!(d.button_for_key("Space"), Some(2));
    assert_eq!(d.button_for_key("X"), None);
}

#[test]
fn a_bad_item_is_an_error() {
    let bad = |item: &str| toml::from_str::<Dialog>(&format!("place = [0, 0]\n[[items]]\n{item}"));
    assert!(bad("kind = \"slider\"\nid = 1").is_err());
    assert!(bad("kind = \"picture\"\nid = 1\npicture = \"a\"\nat = [0, 0]\nwhat = 1").is_err());
    assert!(bad("kind = \"button\"\nid = 1\ndef = \"a\"\nat = [0, 0]\nsize = [1, 1]\nframes = { up = 1 }").is_err());
    assert!(bad("id = 1").is_err());
}

#[test]
fn a_button_shows_the_frame_of_its_state() {
    let Some(Item::Button(b)) = dialog().item(2).cloned() else {
        panic!("item 2 is a button");
    };
    let state = |hidden, pressed, disabled, selected| State {
        hidden,
        pressed,
        disabled,
        selected,
    };
    // 0x456940: selected unless pressed, then disabled, then pressed, then normal.
    assert_eq!(b.frame(State::default()), Some(0));
    assert_eq!(b.frame(state(false, true, false, false)), Some(1));
    assert_eq!(b.frame(state(false, false, true, false)), Some(2));
    assert_eq!(b.frame(state(false, true, true, false)), Some(2));
    assert_eq!(b.frame(state(false, false, false, true)), Some(3));
    assert_eq!(b.frame(state(false, false, true, true)), Some(3));
    assert_eq!(b.frame(state(false, true, false, true)), Some(1));
    assert_eq!(b.frame(state(true, false, false, false)), None);
}

#[test]
fn a_dialog_draws_its_items_as_shown() {
    let d = dialog();
    // By default: the picture in its own colours, the buttons' normal frames, the text empty.
    let text = |text: &str| Draw::Text {
        item: d.text(4).unwrap().clone(),
        x: 17,
        y: 28,
        text: text.to_string(),
    };
    let button = |def: &str, frame, x, y| Draw::Button {
        def: def.to_string(),
        frame,
        x,
        y,
    };
    let picture = |player: Option<u8>| Draw::Picture {
        picture: "bar.pcx".to_string(),
        x: 10,
        y: 20,
        player: player.map(|player| PlayerColours {
            palette: "P.pal".to_string(),
            player,
        }),
    };
    assert_eq!(
        d.draws(&Shown::default(), "P.pal"),
        [
            picture(None),
            button("b.def", 0, 13, 24),
            button("s.def", 2, 15, 26),
            text("")
        ]
    );
    // Shown: the picture in player 3's colours, button 2 disabled, button 3 hidden, a text.
    let mut shown = Shown::default();
    shown.players.insert(1, 3);
    shown.states.insert(
        2,
        State {
            disabled: true,
            ..State::default()
        },
    );
    shown.states.insert(
        3,
        State {
            hidden: true,
            ..State::default()
        },
    );
    shown.texts.insert(4, "a\nb".to_string());
    assert_eq!(
        d.draws(&shown, "P.pal"),
        vec![picture(Some(3)), button("b.def", 2, 13, 24), text("a\nb")]
    );
}
