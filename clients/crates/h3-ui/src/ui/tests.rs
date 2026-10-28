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
    // E17 M3 T6c-1: the last item in order whose area holds the point; a picture without its
    // `size` takes none.
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
fn a_picture_with_its_size_takes_the_mouse_under_the_controls() {
    // E17 M4 T3b: the battle's panel picture is the hovered item where no control is.
    let mut d = dialog();
    if let Item::Picture(p) = &mut d.items[0] {
        p.size = Some([80, 44]);
    }
    assert_eq!(d.items[0].area(), Some(([0, 0], [80, 44])));
    let at = |x, y| d.item_at([x, y]).map(Item::id);
    assert_eq!(at(10, 20), Some(1));
    assert_eq!(at(61, 59), Some(1));
    assert_eq!(at(13, 24), Some(2));
    assert_eq!(at(90, 20), None);
    assert_eq!(d.button_at([10, 20]), None);
    let sized: Dialog =
        toml::from_str(&DIALOG.replace("at = [0, 0]", "at = [0, 0]\nsize = [80, 44]")).unwrap();
    assert_eq!(sized, d);
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

/// A spell book's kinds (E17 M5 T3a): a picture with a hint, DEF items (one repeated, one with a
/// DEF of its own), texts alone (repeated), an area.
const BOOK: &str = r#"
place = [100, 10]

[[items]]
kind = "picture"
id = 1
picture = "back.pcx"
at = [0, 0]
size = [300, 200]
hint = "k.back"

[[items]]
kind = "def"
id = 10
at = [5, 6]
size = [20, 10]
repeat = { count = [2, 3], step = [30, 40] }

[[items]]
kind = "def"
id = 20
def = "tab.def"
at = [200, 0]
size = [50, 100]
frame = 4

[[items]]
kind = "label"
id = 30
at = [5, 12]
size = [20, 10]
font = "tiny.fnt"
font_palette = "game.pal"
colour = 1
align = 1
repeat = { count = [2, 1], step = [30, 0] }

[[items]]
kind = "label"
id = 40
at = [250, 150]
size = [30, 30]
font = "smalfont.fnt"
font_palette = "game.pal"
colour = 7
align = 5
hint = "k.mana"

[[items]]
kind = "area"
id = 50
at = [200, 0]
size = [50, 30]
hint = "k.tab"
"#;

fn book() -> Dialog {
    toml::from_str::<Dialog>(BOOK).unwrap().expanded()
}

#[test]
fn a_repeated_entry_stands_for_its_copies_row_by_row() {
    let d = book();
    let ids: Vec<u16> = d.items.iter().map(Item::id).collect();
    assert_eq!(ids, [1, 10, 11, 12, 13, 14, 15, 20, 30, 31, 40, 50]);
    let places: Vec<[i32; 2]> = d.items[1..7].iter().map(|i| i.area().unwrap().0).collect();
    assert_eq!(
        places,
        [[5, 6], [35, 6], [5, 46], [35, 46], [5, 86], [35, 86]]
    );
    assert_eq!(d.item(31).and_then(Item::area), Some(([35, 12], [20, 10])));
    let Some(Item::Def(def)) = d.item(15) else {
        panic!("item 15 is a DEF item");
    };
    assert_eq!(def.repeat, None);
    // An expanded dialog stays as it is.
    assert_eq!(d.clone().expanded(), d);
    let hints: Vec<Option<&str>> = [1, 10, 30, 40, 50]
        .iter()
        .map(|&id| d.item(id).and_then(Item::hint))
        .collect();
    assert_eq!(
        hints,
        [Some("k.back"), None, None, Some("k.mana"), Some("k.tab")]
    );
}

#[test]
fn the_item_taking_the_mouse_is_the_last_shown_and_enabled() {
    // ItemAtPosition 0x5FF3E0: the last whose area holds the point, neither hidden nor disabled.
    let d = book();
    let mut shown = Shown::default();
    let at = |shown: &Shown, x, y| d.item_taking([x, y], shown).map(Item::id);
    assert_eq!(at(&shown, 106, 23), Some(30));
    assert_eq!(at(&shown, 301, 11), Some(50));
    assert_eq!(at(&shown, 301, 50), Some(20));
    assert_eq!(at(&shown, 99, 50), None);
    let off = |hidden, disabled| State {
        hidden,
        disabled,
        ..State::default()
    };
    shown.states.insert(30, off(false, true));
    assert_eq!(at(&shown, 106, 23), Some(10));
    shown.states.insert(10, off(true, false));
    assert_eq!(at(&shown, 106, 23), Some(1));
}

#[test]
fn the_book_s_kinds_draw_as_shown() {
    let d = book();
    let mut shown = Shown::default();
    // DEF item 10 a DEF and frame given, the others without a DEF but 20, which draws its own; a
    // disabled text; the picture hidden; the area nothing.
    shown.defs.insert(10, "r.def".to_string());
    shown.frames.insert(10, 2);
    shown.texts.insert(30, "x".to_string());
    shown.states.insert(
        31,
        State {
            disabled: true,
            ..State::default()
        },
    );
    shown.states.insert(
        1,
        State {
            hidden: true,
            ..State::default()
        },
    );
    let draws = d.draws(&shown, "P.pal");
    let label = |id, x, y, text: &str, disabled| {
        let Some(Item::Label(item)) = d.item(id).cloned() else {
            panic!("item {id} is a label");
        };
        Draw::Label {
            item,
            x,
            y,
            text: text.to_string(),
            disabled,
        }
    };
    assert_eq!(
        draws,
        [
            Draw::Button {
                def: "r.def".to_string(),
                frame: 2,
                x: 105,
                y: 16
            },
            Draw::Button {
                def: "tab.def".to_string(),
                frame: 4,
                x: 300,
                y: 10
            },
            label(30, 105, 22, "x", false),
            label(31, 135, 22, "", true),
            label(40, 350, 160, "", false),
        ]
    );
}

#[test]
fn a_dialog_with_a_shadow_darkens_two_strips_twice_inside() {
    // vShow 0x5FEAE0 with the flag 0x10: four `screen_darken` calls (E17 M5 T3a).
    let mut d = book();
    assert_eq!(d.shadow_draws(), []);
    d.shadow = Some([300, 200]);
    let rects: Vec<[i32; 4]> = d
        .shadow_draws()
        .into_iter()
        .map(|draw| match draw {
            Draw::Darken { rect } => rect,
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        rects,
        [
            [400, 19, 7, 191],
            [400, 18, 8, 192],
            [109, 210, 298, 7],
            [108, 210, 300, 8]
        ]
    );
    let draws = d.draws(&Shown::default(), "P.pal");
    assert!(matches!(draws[4], Draw::Picture { .. }));
}
