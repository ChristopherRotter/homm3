use super::*;
use h3_ui::ui::Item;

const LAYOUT: &str = r#"
frame = [800, 600]
picture = "AdvMap.pcx"
player_palette = "PLAYERS.PAL"

[view]
origin = [0, 8]
tile = 32
columns = [-1, 19]
rows = [-1, 17]
clip = [8, 8, 600, 552]
gems = [
    { def = "agemul.def", at = [6, 6] },
    { def = "agemur.def", at = [556, 6] },
]

[scroll]
middle = [9, 8]
edge = 16

[cursor]
def = "cradvntr.def"
arrow = 0
edges = 2
hot = [[0, 0], [1, 2], [3, 4], [5, 6], [7, 8], [9, 10], [11, 12], [13, 14], [15, 16], [17, 18]]

[dialog]
place = [0, 0]

[[dialog.items]]
kind = "area"
id = 0
at = [8, 8]
size = [592, 544]

[[dialog.items]]
kind = "button"
id = 3
def = "iam002.def"
at = [679, 196]
size = [32, 32]

[[dialog.items]]
kind = "def"
id = 32
def = "itpa.def"
at = [747, 212]
size = [48, 32]
repeat = { count = [1, 2], step = [0, 32] }

[[dialog.items]]
kind = "text"
id = 200
picture = "AdRollvr.pcx"
at = [8, 556]
size = [592, 18]
font = "smalfont.fnt"
font_palette = "game.pal"
colour = 1
align = 1

[[dialog.items]]
kind = "label"
id = 37
at = [54, 100]
size = [520, 440]
font = "medfont.fnt"
font_palette = "game.pal"
colour = 13
align = 8

[resource_bar]
place = [3, 575]

[[resource_bar.items]]
kind = "picture"
id = 1000
picture = "AResBar.pcx"
at = [0, 0]
size = [794, 22]

[panels]
coloured = [3]
underground = ["iam010.def", "iam003.def"]
sleep = ["iam005.def", "iam011.def"]
"#;

#[test]
fn loads_a_layout_and_places_its_tiles() {
    let layout = AdventureLayout::parse(LAYOUT).unwrap();
    assert_eq!(layout.frame, [800, 600]);
    let view = &layout.view;
    assert_eq!(view.place(0, 0), [0, 8]);
    assert_eq!(view.place(-1, 17), [-32, 552]);
    assert_eq!(view.place(18, 16), [576, 520]);
    assert_eq!(view.gems[1].at, [556, 6]);
    assert_eq!((layout.scroll.middle, layout.scroll.edge), ([9, 8], 16));
    assert_eq!(
        (
            layout.cursor.arrow,
            layout.cursor.edges,
            layout.cursor.hot[9]
        ),
        (0, 2, [17, 18])
    );
    // The dialog's entries made into their copies (E24 M3 T2a).
    let ids: Vec<u16> = layout.dialog.items.iter().map(Item::id).collect();
    assert_eq!(ids, [0, 3, 32, 33, 200, 37]);
    assert_eq!(layout.resource_bar.place, [3, 575]);
    assert_eq!(layout.panels.sleep[1], "iam011.def");
    let names: Vec<String> = layout.names().into_iter().collect();
    assert_eq!(
        names,
        [
            "ADROLLVR.PCX",
            "ADVMAP.PCX",
            "AGEMUL.DEF",
            "AGEMUR.DEF",
            "ARESBAR.PCX",
            "CRADVNTR.DEF",
            "GAME.PAL",
            "IAM002.DEF",
            "IAM003.DEF",
            "IAM005.DEF",
            "IAM010.DEF",
            "IAM011.DEF",
            "ITPA.DEF",
            "MEDFONT.FNT",
            "PLAYERS.PAL",
            "SMALFONT.FNT",
        ]
    );
}

#[test]
fn refuses_a_layout_that_breaks_its_rules() {
    for (from, to, message) in [
        (
            "AdvMap.pcx",
            "AdvMap.def",
            "`AdvMap.def` is not a .PCX name",
        ),
        ("PLAYERS.PAL", ".pal", "`.pal` is not a .PAL name"),
        ("agemur.def", "agemur", "`agemur` is not a .DEF name"),
        (
            "clip = [8, 8, 600, 552]",
            "clip = [8, 8, 801, 552]",
            "the view's clip [8, 8, 801, 552] is not inside the frame",
        ),
        (
            "clip = [8, 8, 600, 552]",
            "clip = [-1, 8, 600, 552]",
            "the view's clip [-1, 8, 600, 552] is not inside the frame",
        ),
        (
            "clip = [8, 8, 600, 552]",
            "clip = [8, 8, 8, 552]",
            "the view's clip [8, 8, 8, 552] is not inside the frame",
        ),
        (
            "clip = [8, 8, 600, 552]",
            "clip = [8, -8, 600, 552]",
            "the view's clip [8, -8, 600, 552] is not inside the frame",
        ),
        (
            "clip = [8, 8, 600, 552]",
            "clip = [8, 8, 600, 8]",
            "the view's clip [8, 8, 600, 8] is not inside the frame",
        ),
        (
            "clip = [8, 8, 600, 552]",
            "clip = [8, 8, 600, 601]",
            "the view's clip [8, 8, 600, 601] is not inside the frame",
        ),
        (
            "columns = [-1, 19]",
            "columns = [19, -1]",
            "the view's columns or rows run backwards",
        ),
        (
            "rows = [-1, 17]",
            "rows = [17, -1]",
            "the view's columns or rows run backwards",
        ),
        ("tile = 32", "tile = 0", "a tile of 0 pixels"),
        (
            "\"cradvntr.def\"",
            "\"cradvntr.pcx\"",
            "`cradvntr.pcx` is not a .DEF name",
        ),
        (
            "edges = 2",
            "edges = 3",
            "the cursor's hot spots end at frame 10: the arrow 0 or the edges 3 to 10 lie past \
             them",
        ),
        (
            "arrow = 0",
            "arrow = 10",
            "the cursor's hot spots end at frame 10: the arrow 10 or the edges 2 to 9 lie past \
             them",
        ),
        (
            "edge = 16",
            "edge = 0",
            "edge zones of 0 pixels in a frame of [800, 600]",
        ),
        (
            "edge = 16",
            "edge = 301",
            "edge zones of 301 pixels in a frame of [800, 600]",
        ),
        // The dialog's names (E24 M3 T2a).
        ("\"iam002.def\"", "\"iam002\"", "`iam002` is not a .DEF name"),
        ("\"itpa.def\"", "\"itpa.pcx\"", "`itpa.pcx` is not a .DEF name"),
        ("\"AdRollvr.pcx\"", "\"AdRollvr\"", "`AdRollvr` is not a .PCX name"),
        ("\"smalfont.fnt\"", "\"smalfont\"", "`smalfont` is not a .FNT name"),
        ("\"medfont.fnt\"", "\"medfont\"", "`medfont` is not a .FNT name"),
        ("\"game.pal\"", "\"game\"", "`game` is not a .PAL name"),
        ("\"AResBar.pcx\"", "\"AResBar\"", "`AResBar` is not a .PCX name"),
        ("\"iam011.def\"", "\"iam011\"", "`iam011` is not a .DEF name"),
    ] {
        let text = LAYOUT.replacen(from, to, 1);
        assert_eq!(AdventureLayout::parse(&text).unwrap_err(), message, "{to}");
    }
    let unknown = LAYOUT.replace("tile = 32", "tile = 32\nscale = 2");
    assert!(
        AdventureLayout::parse(&unknown)
            .unwrap_err()
            .contains("scale")
    );
}
