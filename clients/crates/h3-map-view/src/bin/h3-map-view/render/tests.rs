//! `read_live`'s refusals (E24 M1 T6b, the reviewer's R2-14), on a writer's map with two
//! heroes (the capture set's 40 kept live parts read through it in `h3map.py drawn`).
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use h3_map_view::situation::Situation;

use super::*;

const PLAIN: &str = "size = 36\nterrain = \"grass\"\n\
    heroes = [{ at = [34, 34] }, { at = [34, 1], owner = 1 }]\nview = [0, 0, 0]\n";

fn map() -> (MapArt, Live) {
    let (map, extras) = Situation::parse(PLAIN)
        .unwrap()
        .build(&BTreeMap::new(), &[])
        .unwrap();
    let bytes = h3_map::writer::write_with(&map, &extras).unwrap();
    let raw = h3_map::parse(&bytes, &Limits::default()).unwrap();
    let art = MapArt::from_raw(&raw).unwrap();
    let tables = files::load_tables(&files::default_map()).unwrap();
    let live = Live::at_start(&raw, &art, 0, &tables.map.towns);
    (art, live)
}

/// `read_live` of `text` at the view (0, 0, 0): the live part, or the message after the path.
fn read(text: &str) -> Result<(MapArt, Live), String> {
    static FILES: AtomicUsize = AtomicUsize::new(0);
    let (mut art, mut live) = map();
    let n = FILES.fetch_add(1, Ordering::Relaxed);
    let path =
        std::env::temp_dir().join(format!("h3-map-view-live-{}-{n}.toml", std::process::id()));
    fs::write(&path, text).unwrap();
    let read = read_live(&path, ViewAt::default(), &mut art, &mut live, &[]);
    fs::remove_file(&path).unwrap();
    let prefix = format!("{}: ", path.display());
    read.map(|()| (art, live))
        .map_err(|e| e.strip_prefix(&prefix).unwrap_or(&e).to_string())
}

#[test]
fn the_view_must_be_the_one_drawn() {
    assert!(read("view = [0, 0, 0]\n").is_ok());
    assert_eq!(
        read("view = [1, 0, 0]\n").unwrap_err(),
        "the original showed the view at Some([1, 0, 0]), not --view [0, 0, 0]"
    );
}

#[test]
fn defs_and_classes_go_together() {
    assert_eq!(
        read("defs = [\"a.def\", \"b.def\"]\n").unwrap_err(),
        "`defs` and `classes` go together"
    );
    assert_eq!(
        read("classes = [[34, 0], [34, 0]]\n").unwrap_err(),
        "`defs` and `classes` go together"
    );
}

#[test]
fn owners_are_none_or_a_colour() {
    let (_, live) = read("owners = [-1, 3]\n").unwrap();
    assert_eq!(live.objects[0].owner, None);
    assert_eq!(live.objects[1].owner, Some(3));
    for wrong in [8, -2, 255] {
        assert_eq!(
            read(&format!("owners = [{wrong}, 0]\n")).unwrap_err(),
            format!("`owners`: object 0's {wrong} is not -1 or 0-7")
        );
    }
}

#[test]
fn the_objects_must_be_the_maps() {
    let (art, _) = map();
    let anchors: Vec<String> = art
        .objects
        .iter()
        .map(|o| format!("[{}, {}, {}]", o.anchor.x, o.anchor.y, o.anchor.z))
        .collect();
    assert!(read(&format!("anchors = [{}]\n", anchors.join(", "))).is_ok());
    let a = art.objects[0].anchor;
    let moved = format!("[{}, {}, {}]", a.x + 1, a.y, a.z);
    let text = format!("anchors = [{moved}, {}]\n", anchors[1]);
    assert_eq!(
        read(&text).unwrap_err(),
        "the original's objects are not the map's (`anchors`: object 0)"
    );
}
