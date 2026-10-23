use super::*;

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
    let names: Vec<String> = layout.names().into_iter().collect();
    assert_eq!(
        names,
        ["ADVMAP.PCX", "AGEMUL.DEF", "AGEMUR.DEF", "PLAYERS.PAL"]
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
