use super::*;
use alloc::vec;

/// A shroud table: every mask piece 1 but the ones `pieces` set.
fn pieces(set: &[(u8, u8)]) -> String {
    let mut all = vec![1u8; MASKS];
    for &(mask, piece) in set {
        all[usize::from(mask)] = piece;
    }
    let list: Vec<String> = all.iter().map(ToString::to_string).collect();
    list.join(", ")
}

/// `n` DEF names `<stem>0.def` …
fn names(stem: &str, n: usize) -> String {
    let list: Vec<String> = (0..n).map(|i| format!("\"{stem}{i}.def\"")).collect();
    list.join(", ")
}

/// The `[map.heroes]` section: every list of its length.
fn heroes() -> String {
    format!(
        "[map.heroes]\nclasses = [{}]\nflags = [{}]\nboats = [{}]\nwaves = [{}]\nboat_flags = [{}]\ngroups = [0, 1, 2, 3, 4, 3, 2, 1]\n",
        names("ah", HERO_CLASSES),
        names("af", COLOURS),
        names("ab", BOAT_TYPES),
        names("abm", BOAT_TYPES),
        names("abf", BOAT_TYPES * COLOURS),
    )
}

/// The `[map.owners]` and `[map.towns]` sections: a list of every faction's town pictures each.
fn owners_and_towns() -> String {
    format!(
        "[map.owners]\npalette = \"game.pal\"\nneutral = 63\nclasses = [17, 53, 98]\n\n[map.towns]\nvillage = [{}]\nfort = [{}]\ncapitol = [{}]\n",
        names("tv", FACTIONS),
        names("tf", FACTIONS),
        names("tc", FACTIONS),
    )
}

/// A table with `[map]`.
fn base(extra: &str) -> String {
    format!(
        r#"
[table]
id = "base"
depends = []

[map]
edge = "edg.def"
path = "adag.def"
road_offset = 16
cycle_ms = 200
frame_ms = 180
scroll_steps = [1, 2, 3]
scroll_ms = 70
poll_ms = 33

[map.shroud]
full = "tshrc.def"
edge = "tshre.def"
pieces = [{}]

{}

{}

[[terrain]]
code = 0
def = "dirttl.def"

[[terrain]]
code = 8
def = "watrtl.def"
cycles = [[229, 240], [242, 253]]

[[river]]
code = 1
def = "clrrvr.def"

[[road]]
code = 3
def = "cobbrd.def"
{extra}"#,
        pieces(&[(0, 34), (1, 133), (2, 0)]),
        heroes(),
        owners_and_towns(),
    )
}

fn table(id: &str, depends: &[&str], extra: &str) -> String {
    let depends: Vec<String> = depends.iter().map(|d| format!("\"{d}\"")).collect();
    format!(
        "[table]\nid = \"{id}\"\ndepends = [{}]\n{extra}",
        depends.join(", ")
    )
}

fn files(texts: &[String]) -> Vec<TableFile> {
    texts.iter().map(|t| parse(t).unwrap()).collect()
}

#[test]
fn loads_a_selection_dependencies_first() {
    let ab = table(
        "ab",
        &["base"],
        "[[river]]\ncode = 4\ndef = \"lavrvr.def\"\ncycles = [[240, 248]]\n",
    );
    let files = files(&[ab, base("")]);
    let tables = load(&files, "ab").unwrap();
    assert_eq!(tables.order, ["base", "ab"]);
    assert_eq!(tables.map.edge, "edg.def");
    assert_eq!(tables.map.road_offset, 16);
    assert_eq!((tables.map.cycle_ms, tables.map.frame_ms), (200, 180));
    assert_eq!(
        (&tables.map.scroll_steps[..], tables.map.scroll_ms, tables.map.poll_ms),
        (&[1, 2, 3][..], 70, 33)
    );
    assert_eq!(
        tables
            .layer(Layer::Terrain)
            .keys()
            .copied()
            .collect::<Vec<_>>(),
        [0, 8]
    );
    assert_eq!(tables.terrains[&8].cycles, [[229, 240], [242, 253]]);
    assert_eq!(
        tables
            .layer(Layer::River)
            .keys()
            .copied()
            .collect::<Vec<_>>(),
        [1, 4]
    );
    assert_eq!(tables.layer(Layer::Road)[&3].def, "cobbrd.def");
    // The base alone leaves the expansion's river out.
    assert_eq!(load(&files, "base").unwrap().rivers.len(), 1);
    // A table two others build on loads once.
    let diamond = self::files(&[
        table("top", &["b", "c"], ""),
        table("b", &["base"], ""),
        table("c", &["base"], ""),
        base(""),
    ]);
    assert_eq!(
        load(&diamond, "top").unwrap().order,
        ["base", "b", "c", "top"]
    );
}

/// A need's name, what it is, its layer and frames.
type Need<'a> = (&'a str, &'a str, Option<(Layer, u8)>, usize);

#[test]
fn every_def_with_the_frames_the_view_draws() {
    let tables = load(&files(&[base("")]), "base").unwrap();
    let defs = tables.defs();
    let needs: Vec<Need<'_>> = defs
        .iter()
        .map(|n| (n.name.as_str(), n.what.as_str(), n.layer, n.frames))
        .collect();
    // The heroes' and boats' DEFs after the rest, each drawn in the facings' five groups.
    let heroes = HERO_CLASSES + COLOURS + 2 * BOAT_TYPES + BOAT_TYPES * COLOURS;
    // Then the towns' pictures, one frame of one group each (E24 M1 T5b1).
    assert_eq!(needs.len(), 8 + heroes + 3 * FACTIONS);
    assert!(defs[..8].iter().all(|n| n.groups == 1));
    assert!(
        defs[8..8 + heroes]
            .iter()
            .all(|n| n.groups == 5 && n.frames == 1)
    );
    assert!(
        defs[8 + heroes..]
            .iter()
            .all(|n| n.groups == 1 && n.frames == 1)
    );
    assert_eq!(
        [&needs[8 + heroes], needs.last().unwrap()],
        [
            &("tv0.def", "town village 0", None, 1),
            &("tc8.def", "town capitol 8", None, 1)
        ]
    );
    let ends = [&needs[8], &needs[7 + heroes]];
    assert_eq!(
        ends,
        [
            &("ah0.def", "hero 0", None, 1),
            &("abf23.def", "boat flag 23", None, 1)
        ]
    );
    assert_eq!(needs[8 + HERO_CLASSES].1, "hero flag 0");
    assert_eq!(
        needs[..8],
        [
            ("dirttl.def", "terrain 0", Some((Layer::Terrain, 0)), 1),
            ("watrtl.def", "terrain 8", Some((Layer::Terrain, 8)), 1),
            ("clrrvr.def", "river 1", Some((Layer::River, 1)), 1),
            ("cobbrd.def", "road 3", Some((Layer::Road, 3)), 1),
            ("edg.def", "edge", None, EDGE_FRAMES),
            ("tshrc.def", "full shroud", None, FULL_FRAMES),
            ("tshre.def", "edge shroud", None, 34),
            ("adag.def", "path", None, PATH_FRAMES),
        ]
    );
    let names: Vec<String> = tables
        .names()
        .into_iter()
        .filter(|n| !n.starts_with('A') && !n.starts_with('T'))
        .collect();
    assert_eq!(
        names,
        [
            "CLRRVR.DEF",
            "COBBRD.DEF",
            "DIRTTL.DEF",
            "EDG.DEF",
            "WATRTL.DEF"
        ]
    );
}

#[test]
fn the_shrouds_pieces() {
    let shroud = Shroud {
        full: "tshrc.def".to_string(),
        edge: "tshre.def".to_string(),
        pieces: (0..=255)
            .map(|m| match m {
                0 => 1,
                1 => 5,
                2 => 3,
                3 => 103,
                4 => 0,
                5 => 34,
                6 => 133,
                8 => 100,
                255 => 0,
                _ => 2,
            })
            .collect(),
    };
    let edge = |frame, mirror| ShroudPiece::Edge { frame, mirror };
    // v of 1 or 5 on an odd x takes v + 1; v of 3 on an odd y takes 4; the frame is v − 1.
    assert_eq!(shroud.piece(0, 2, 2), edge(0, false));
    assert_eq!(shroud.piece(0, 3, 2), edge(1, false));
    assert_eq!(shroud.piece(1, 1, 0), edge(5, false));
    assert_eq!(shroud.piece(1, 0, 1), edge(4, false));
    assert_eq!(shroud.piece(2, 1, 0), edge(2, false));
    assert_eq!(shroud.piece(2, 0, 1), edge(3, false));
    assert_eq!(shroud.piece(3, 0, 1), edge(3, true));
    // An entry 0 is the full piece (the original's table: the mask 255 only); 100 no frame.
    assert_eq!(shroud.piece(4, 1, 0), ShroudPiece::Full(1));
    assert_eq!(shroud.piece(8, 0, 0), ShroudPiece::None);
    assert_eq!(shroud.piece(5, 0, 0), edge(33, false));
    assert_eq!(shroud.piece(6, 1, 1), edge(32, true));
    assert_eq!(shroud.piece(7, 0, 0), edge(1, false));
    // The full piece: ((85 x xor 85 y) div 64) and 3.
    for (x, y, frame) in [
        (0, 0, 0),
        (1, 0, 1),
        (2, 0, 2),
        (3, 0, 3),
        (1, 1, 0),
        (4, 0, 1),
        (4, 1, 0),
    ] {
        assert_eq!(
            shroud.piece(ALL_HIDDEN, x, y),
            ShroudPiece::Full(frame),
            "({x}, {y})"
        );
    }
    assert_eq!(shroud.edge_frames(), 34);
    let none = Shroud {
        pieces: vec![0; MASKS],
        ..shroud
    };
    assert_eq!(none.edge_frames(), 0);
}

#[test]
fn refuses_what_breaks_the_format() {
    let check = |texts: &[String], top: &str, expected: TableError, message: &str| {
        let err = load(&files(texts), top).unwrap_err();
        assert_eq!(err, expected);
        assert_eq!(err.to_string(), message);
    };
    check(
        &[base(""), base("")],
        "base",
        TableError::DuplicateTable("base".to_string()),
        "two tables have the id `base`",
    );
    check(
        &[base("")],
        "sod",
        TableError::UnknownTable("sod".to_string()),
        "no table has the id `sod`",
    );
    check(
        &[table("a", &["b"], ""), table("b", &["a"], "")],
        "a",
        TableError::Cycle("a".to_string()),
        "the dependencies go round through `a`",
    );
    check(
        &[table("a", &[], "")],
        "a",
        TableError::Map(0),
        "0 tables of the selection hold [map], not one",
    );
    let both = base("").replace("id = \"base\"", "id = \"ab\"\n");
    check(
        &[
            base(""),
            both.replace("depends = []", "depends = [\"base\"]"),
        ],
        "ab",
        TableError::Map(2),
        "2 tables of the selection hold [map], not one",
    );
    check(
        &[base("[[river]]\ncode = 1\ndef = \"icyrvr.def\"\n")],
        "base",
        TableError::DuplicateCode("river 1".to_string()),
        "`river 1` is defined twice",
    );
    check(
        &[base("[[road]]\ncode = 0\ndef = \"dirtrd.def\"\n")],
        "base",
        entry("road 0", "code 0 is none"),
        "`road 0`: code 0 is none",
    );
    check(
        &[base("[[terrain]]\ncode = 1\ndef = \"sandtl.pcx\"\n")],
        "base",
        entry("terrain 1", "`sandtl.pcx` is not a .DEF name"),
        "`terrain 1`: `sandtl.pcx` is not a .DEF name",
    );
    check(
        &[base(
            "[[terrain]]\ncode = 7\ndef = \".def\"\ncycles = [[1, 2]]\n",
        )],
        "base",
        entry("terrain 7", "`.def` is not a .DEF name"),
        "`terrain 7`: `.def` is not a .DEF name",
    );
    check(
        &[base(
            "[[terrain]]\ncode = 7\ndef = \"lavatl.def\"\ncycles = [[254, 246]]\n",
        )],
        "base",
        entry("terrain 7", "a cycle's first entry is after its last"),
        "`terrain 7`: a cycle's first entry is after its last",
    );
    check(
        &[base(
            "[[road]]\ncode = 1\ndef = \"dirtrd.def\"\ncycles = [[1, 2]]\n",
        )],
        "base",
        entry("road 1", "a road's palette does not cycle"),
        "`road 1`: a road's palette does not cycle",
    );
    for (field, what) in [
        ("edge = \"edg.def\"", "edge"),
        ("full = \"tshrc.def\"", "full shroud"),
        ("edge = \"tshre.def\"", "edge shroud"),
        ("path = \"adag.def\"", "path"),
    ] {
        let bad = base("").replacen(field, &field.replace(".def", ".pcx"), 1);
        let err = load(&files(&[bad]), "base").unwrap_err();
        assert!(
            matches!(&err, TableError::Entry(w, _) if w == what),
            "{field}: {err}"
        );
    }
    let short = base("").replacen("pieces = [", "pieces = [1, ", 1);
    check(
        &[short],
        "base",
        entry("shroud", "257 pieces, not one per mask (256)"),
        "`shroud`: 257 pieces, not one per mask (256)",
    );
    // The heroes' and boats' lists: one entry per class, colour, type, type and colour, facing.
    for (list, what, want) in [
        ("classes = [", "heroes", HERO_CLASSES),
        ("flags = [", "hero flags", COLOURS),
        ("boats = [", "boats", BOAT_TYPES),
        ("waves = [", "waves", BOAT_TYPES),
        ("boat_flags = [", "boat flags", BOAT_TYPES * COLOURS),
        ("groups = [", "groups", FACINGS),
    ] {
        let one = if list.starts_with("groups") {
            "0, "
        } else {
            "\"x.def\", "
        };
        let long = base("").replacen(&format!("\n{list}"), &format!("\n{list}{one}"), 1);
        let rule = format!("{} entries, not {want}", want + 1);
        check(
            &[long],
            "base",
            entry(what, &rule),
            &format!("`{what}`: {rule}"),
        );
    }
    // The owners' palette and the towns' lists (E24 M1 T5b1): one picture per faction.
    let def = base("").replacen("\"game.pal\"", "\"game.def\"", 1);
    check(
        &[def],
        "base",
        entry("owners' palette", "`game.def` is not a .PAL name"),
        "`owners' palette`: `game.def` is not a .PAL name",
    );
    for (list, what) in [
        ("village = [", "villages"),
        ("fort = [", "forts"),
        ("capitol = [", "capitols"),
    ] {
        let long = base("").replacen(&format!("\n{list}"), &format!("\n{list}\"x.def\", "), 1);
        let rule = format!("{} entries, not {FACTIONS}", FACTIONS + 1);
        check(
            &[long],
            "base",
            entry(what, &rule),
            &format!("`{what}`: {rule}"),
        );
    }
    let neutral = base("").replacen("neutral = 63", "neutral = 248", 1);
    check(
        &[neutral],
        "base",
        entry(
            "owners",
            "the neutral entry 248 leaves no room for 8 colours",
        ),
        "`owners`: the neutral entry 248 leaves no room for 8 colours",
    );
    let town = base("").replacen("\"tf2.def\"", "\"tf2.pcx\"", 1);
    check(
        &[town],
        "base",
        entry("town fort 2", "`tf2.pcx` is not a .DEF name"),
        "`town fort 2`: `tf2.pcx` is not a .DEF name",
    );
    let pcx = base("").replacen("\"ab1.def\"", "\"ab1.pcx\"", 1);
    check(
        &[pcx],
        "base",
        entry("boat 1", "`ab1.pcx` is not a .DEF name"),
        "`boat 1`: `ab1.pcx` is not a .DEF name",
    );
    // The scroll's steps (E24 M2 T2): one per speed, each at least a tile; a poll's time.
    for steps in ["[1, 2]", "[1, 2, 3, 4]", "[0, 2, 3]"] {
        let bad = base("").replacen("scroll_steps = [1, 2, 3]", &format!("scroll_steps = {steps}"), 1);
        check(
            &[bad],
            "base",
            entry("scroll_steps", "one step of at least a tile per scroll speed (3)"),
            "`scroll_steps`: one step of at least a tile per scroll speed (3)",
        );
    }
    check(
        &[base("").replacen("poll_ms = 33", "poll_ms = 0", 1)],
        "base",
        entry("poll_ms", "a poll every 0 ms"),
        "`poll_ms`: a poll every 0 ms",
    );
}

#[test]
fn the_facings_groups() {
    let tables = load(&files(&[base("")]), "base").unwrap();
    assert_eq!(tables.map.heroes.drawn_groups(), 5);
    let none = HeroArt {
        groups: vec![],
        ..tables.map.heroes
    };
    assert_eq!(none.drawn_groups(), 0);
}

#[test]
fn refuses_what_is_not_a_table() {
    for text in [
        "not toml",
        "[table]\nid = \"a\"\ndepends = []\n[battle]\n",
        "[table]\nid = \"a\"\n",
    ] {
        let err = parse(text).unwrap_err();
        assert!(matches!(err, TableError::Syntax(_)), "{text}");
        assert!(err.to_string().starts_with("not a table: "));
    }
}

#[test]
fn the_owners_entries_and_the_towns_pictures() {
    let tables = load(&files(&[base("")]), "base").unwrap();
    let owners = &tables.map.owners;
    // An owned class's flag pixels: entry 63 without an owner, 64 + the colour with one.
    assert_eq!(owners.entry(53, None), Some(63));
    assert_eq!(owners.entry(17, Some(0)), Some(64));
    assert_eq!(owners.entry(98, Some(7)), Some(71));
    // Every other class's are skipped.
    assert_eq!(owners.entry(54, Some(2)), None);
    let towns = &tables.map.towns;
    let bit = |id: u32| 1u64 << id;
    assert_eq!(towns.picture(0, 0), Some("tv0.def"));
    assert_eq!(towns.picture(8, bit(10) | bit(5)), Some("tv8.def"));
    for building in [FORT, CITADEL, CASTLE] {
        assert_eq!(towns.picture(3, bit(building)), Some("tf3.def"));
    }
    assert_eq!(towns.picture(4, bit(FORT) | bit(CAPITOL)), Some("tc4.def"));
    assert_eq!(towns.picture(4, bit(CAPITOL)), Some("tc4.def"));
    // A faction past the lists has none.
    assert_eq!(towns.picture(9, 0), None);
    assert_eq!(towns.picture(u32::MAX, bit(FORT)), None);
}
