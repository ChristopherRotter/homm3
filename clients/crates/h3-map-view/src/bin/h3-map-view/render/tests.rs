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
    read.map(|_| (art, live))
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

#[test]
fn the_kingdom_and_the_pointer() {
    // E24 M3 T2a: the panels' kingdom (`Kingdom::read`; T5a: with the resources and the date)
    // and the parked mouse's point.
    let (_, live) = read(
        "kingdom = { heroes = [[0, 1, 0, 1500, 0]], towns = [], selected = [1, 0], tops = [0, \
             0], quests = 0, resources = [1, 2, 3, 4, 5, 6, 7], date = [2, 3, 4] }\n",
    )
    .unwrap();
    assert_eq!(live.kingdom.heroes.len(), 1);
    assert_eq!(live.kingdom.resources, [1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(live.kingdom.date.week, 3);
    assert!(
        read("kingdom = { heroes = [] }\n")
            .unwrap_err()
            .starts_with("`kingdom` is not")
    );
    let (mut art, mut live) = map();
    let path =
        std::env::temp_dir().join(format!("h3-map-view-pointer-{}.toml", std::process::id()));
    fs::write(&path, "pointer = [700, 300]\n").unwrap();
    let pointer = read_live(&path, ViewAt::default(), &mut art, &mut live, &[]);
    fs::remove_file(&path).unwrap();
    assert_eq!(pointer, Ok(Some([700, 300])));
    for wrong in [
        "pointer = [700]\n",
        "pointer = 1\n",
        "pointer = [1, \"a\"]\n",
    ] {
        assert_eq!(
            read(wrong).unwrap_err(),
            "`pointer` is not [x, y]",
            "{wrong}"
        );
    }
}

/// `render`'s options (E24 M4 T6, R4-2 and R4-7): `--check` needs `--record` and `--live`,
/// `--packs` needs `--record`.
#[test]
fn a_check_needs_a_record_and_a_capture() {
    let parse = |extra: &[&str]| {
        let mut args: Vec<String> = ["m.h3m", "--view", "0,0"].map(String::from).to_vec();
        args.extend(extra.iter().map(|s| (*s).to_string()));
        Options::parse(&args)
    };
    assert!(parse(&[]).is_ok());
    let no_record = "render: --check compares a record's start: it needs --record";
    assert_eq!(parse(&["--check"]).unwrap_err(), no_record);
    assert_eq!(
        parse(&["--live", "l.toml", "--check"]).unwrap_err(),
        no_record
    );
    assert_eq!(
        parse(&["--packs", "p"]).unwrap_err(),
        "render: --packs reads a record's rules: it needs --record"
    );
    assert_eq!(
        parse(&["--record", "r.jsonl", "--check"]).unwrap_err(),
        "render: --check compares the record's start with a capture's: it needs --live"
    );
    let options = parse(&[
        "--record", "r.jsonl", "--packs", "p", "--live", "l.toml", "--check",
    ])
    .unwrap();
    assert!(options.check);
    assert_eq!(options.record, Some(PathBuf::from("r.jsonl")));
    assert_eq!(options.packs, PathBuf::from("p"));
    assert_eq!(
        parse(&["--record", "r.jsonl"]).unwrap().packs,
        files::default_packs()
    );
}

/// `--check`'s verdict (E24 M4 T6, R4-6): what it compared, the choices only when the capture
/// made them; a capture without `[hover]` fails it.
#[test]
fn a_check_says_what_it_compared() {
    let mut theirs = toml::Table::new();
    let said = verdict(Vec::new(), Some(Vec::new()), &theirs).unwrap();
    assert!(said.contains("the mouse-over over every explored tile"));
    assert!(said.ends_with("; not compared: the choices (the capture holds no `[chosen]`)"));
    theirs.insert("chosen".into(), toml::Value::Table(toml::Table::new()));
    let said = verdict(Vec::new(), Some(Vec::new()), &theirs).unwrap();
    assert!(said.ends_with(" and the choices"));
    assert_eq!(
        verdict(Vec::new(), None, &theirs),
        Err(vec![
            "the capture has no `[hover]`: the mouse-over not compared".to_string()
        ])
    );
    let found = vec!["`viewer` differs".to_string()];
    let hover = Some(vec!["(1, 2, 0): the hint differs".to_string()]);
    assert_eq!(verdict(found, hover, &theirs).unwrap_err().len(), 2);
}

/// A record's start is built on the map file's own art, never on one a capture changed (E24 M4
/// T6, R4-3): a random dwelling the capture read as a dwelling keeps its random template in the
/// engine's start (MCL-STUB-1), so the comparison names it.
#[test]
fn a_record_s_start_is_built_on_the_file_s_art() {
    use h3_protocol::wire::ServerMessage;
    use h3_protocol::wire::game::{GameRecordHeader, MapIdentity};

    let masks = "001111110011111111111111111111111111111111111111 \
                 110000000000000000000000000000000000000000000000 111111111 011111111";
    let templates: Vec<ObjectTemplate> = [("avrcgen0.def", 216), ("dw17.def", 17)]
        .iter()
        .map(|(def, class)| {
            ObjectTemplate::from_objects_txt(&format!("{def} {masks} {class} 0 0 0")).unwrap()
        })
        .collect();
    let text = "size = 36\nterrain = \"grass\"\nview = [0, 0, 0]\n\
        heroes = [{ at = [10, 10], class = 2 }, { at = [34, 34], owner = 1 }]\n\
        [[objects]]\ndef = \"avrcgen0.def\"\nat = [8, 15]\n";
    let (map, extras) = Situation::parse(text)
        .unwrap()
        .build(&BTreeMap::new(), &templates)
        .unwrap();
    let bytes = h3_map::writer::gzip(&h3_map::writer::write_with(&map, &extras).unwrap());
    let raw = h3_map::read(&bytes, &Limits::default()).unwrap();
    // The engine's game on it, Red's record of its start.
    let packs = files::default_packs();
    let ids = ["base", "ab", "sod"].map(String::from);
    let rules = h3_files::adventure::load_rules(&packs, &ids, "base:faithful").unwrap();
    let setup = h3_map::import(&raw, &rules).unwrap();
    let state = h3_game::GameState::new(&rules, &setup, 7).unwrap();
    let red = h3_game::Viewer::Player(h3_contracts::PlayerId::new(0));
    let observation = h3_protocol::map::game::observe(&state, &rules, red);
    let identity = MapIdentity {
        content_hash: h3_protocol::wire::WireU64(engine::content_hash(&bytes)),
        import_version: h3_map::MAP_IMPORT_VERSION,
    };
    let header = GameRecordHeader {
        format: h3_protocol::RECORD_FORMAT_VERSION,
        game: 0,
        player: 0,
        manifest: h3_protocol::map::game::manifest(&rules, identity),
        first: 1,
    };
    let snapshot = ServerMessage::GameSnapshot { n: 1, observation };
    let record = [
        h3_protocol::codec::encode_text(&header),
        h3_protocol::codec::encode_text(&snapshot),
    ]
    .join("\n");
    let dir = std::env::temp_dir();
    let record_path = dir.join(format!("h3-map-view-record-{}.jsonl", std::process::id()));
    fs::write(&record_path, record).unwrap();
    // A capture that read the random dwelling as the dwelling it became: the art changed.
    let mut art = MapArt::from_raw(&raw).unwrap();
    let tables = files::load_tables(&files::default_map()).unwrap();
    let mut live = Live::at_start(&raw, &art, 0, &tables.map.towns);
    let dwelling = (0..art.objects.len())
        .find(|&i| art.templates[art.objects[i].template].def == "avrcgen0.def")
        .unwrap();
    let (mut defs, mut classes) = (Vec::new(), Vec::new());
    for i in 0..art.objects.len() {
        let t = &art.templates[art.objects[i].template];
        let (def, class) = if i == dwelling {
            ("dw17.def", 17)
        } else {
            (t.def.as_str(), t.class)
        };
        defs.push(format!("\"{def}\""));
        classes.push(format!("[{class}, {}]", t.subclass));
    }
    let capture = format!(
        "defs = [{}]\nclasses = [{}]\n",
        defs.join(", "),
        classes.join(", ")
    );
    let live_path = dir.join(format!("h3-map-view-capture-{}.toml", std::process::id()));
    fs::write(&live_path, &capture).unwrap();
    read_live(
        &live_path,
        ViewAt::default(),
        &mut art,
        &mut live,
        &templates,
    )
    .unwrap();
    assert_eq!(
        art.templates[art.objects[dwelling].template].def,
        "dw17.def"
    );
    let start = record_start(&record_path, &bytes, &templates, &tables, &packs).unwrap();
    let theirs = read_table(&live_path).unwrap();
    fs::remove_file(&record_path).unwrap();
    fs::remove_file(&live_path).unwrap();
    assert!(start.seen[dwelling]);
    let ours = start.table["defs"].as_array().unwrap();
    assert_eq!(ours[dwelling].as_str(), Some("avrcgen0.def"));
    assert!(
        engine::differences(&start, &theirs).contains(&format!("object {dwelling}: `defs` differ"))
    );
}
