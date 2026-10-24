//! The committed map tables and the adventure screen's layout load, and hold what the view draws
//! (E24 M1 T2): a DEF for every terrain, river and road code the `.h3m` format has, the shroud's
//! pieces within `tshre.def`'s 34 frames (E24 design §1.8), the native layout's numbers (§1.2).

use std::fs;
use std::path::Path;

use h3_map_view::art::Layer;
use h3_map_view::layout::AdventureLayout;
use h3_map_view::tables::{self, Tables};

fn map_dir() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../map"))
}

fn committed() -> Tables {
    let text = fs::read_to_string(map_dir().join("assets/base.toml")).unwrap();
    tables::load(&[tables::parse(&text).unwrap()], "base").unwrap()
}

#[test]
fn the_tables_name_every_code_of_the_format() {
    let tables = committed();
    for (layer, codes) in [
        (Layer::Terrain, 0..=9),
        (Layer::River, 1..=4),
        (Layer::Road, 1..=3),
    ] {
        let held: Vec<u8> = tables.layer(layer).keys().copied().collect();
        assert_eq!(held, codes.collect::<Vec<_>>(), "{layer:?}");
    }
    assert_eq!(tables.map.shroud.edge_frames(), 34);
    // Water's, lava's and three rivers' palettes cycle (design §1.6).
    let cycling: Vec<(Layer, u8)> = [Layer::Terrain, Layer::River]
        .into_iter()
        .flat_map(|l| {
            tables
                .layer(l)
                .iter()
                .filter(|(_, a)| !a.cycles.is_empty())
                .map(move |(c, _)| (l, *c))
        })
        .collect();
    assert_eq!(
        cycling,
        [
            (Layer::Terrain, 7),
            (Layer::Terrain, 8),
            (Layer::River, 1),
            (Layer::River, 3),
            (Layer::River, 4)
        ]
    );
}

#[test]
fn the_native_layout() {
    let text = fs::read_to_string(map_dir().join("layouts/adventure.toml")).unwrap();
    let layout = AdventureLayout::parse(&text).unwrap();
    assert_eq!(layout.frame, [800, 600]);
    assert_eq!(layout.view.clip, [8, 8, 600, 552]);
    // Columns 0 and 18 show 24 pixels each, rows 0 to 16 whole (design §1.2).
    assert_eq!(layout.view.place(18, 16), [576, 520]);
    assert_eq!(layout.view.gems.len(), 4);
}

#[test]
fn the_owners_towns_and_heroes() {
    // E24 M1 T6b (the reviewer's R2-14): what the captures alone checked. The owned classes as
    // the game loads them, 219 and 220 among them (0x41153C's jump table, design §2.9); the
    // neutral entry 63 of `game.pal`, the colours' 64 to 71; the towns' three lists (0x677A0C,
    // 0x677A30, 0x677A54); the heroes' groups by facing (§2.8).
    let tables = committed();
    let owners = &tables.map.owners;
    assert_eq!(owners.palette, "game.pal");
    assert_eq!(owners.neutral, 63);
    assert_eq!(owners.classes, [17, 20, 33, 42, 53, 77, 87, 98, 219, 220]);
    assert_eq!(owners.entry(219, None), Some(63));
    assert_eq!(owners.entry(33, Some(7)), Some(71));
    let towns = &tables.map.towns;
    assert_eq!(towns.village[0], "AVCcast0.def");
    assert_eq!(towns.fort[8], "AVChforx.def");
    assert_eq!(towns.capitol[7], "AVCforz0.def");
    assert!(
        [&towns.village, &towns.fort, &towns.capitol]
            .iter()
            .all(|list| list.len() == tables::FACTIONS)
    );
    let heroes = &tables.map.heroes;
    assert_eq!(heroes.groups, [0, 1, 2, 3, 4, 3, 2, 1]);
    assert_eq!(heroes.classes.len(), 18);
    assert_eq!(heroes.classes[0], "ah00_.def");
}
