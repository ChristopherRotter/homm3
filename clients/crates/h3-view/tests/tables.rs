//! The committed asset tables against the committed packs (E17 M1 T4; no original file needed):
//! each expansion selection loads, and every creature, terrain and obstacle key of a pack has an
//! entry in that pack's table, except the listed gaps; terrains and special battlefields share the
//! tables' battlefields (E17 M3 T2). No table has an entry no pack defines.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use h3_view::tables::{self, Tables};
use serde::Deserialize;

const PACKS: [&str; 3] = ["base", "ab", "sod"];
/// Pack keys without graphics: the Arrow Tower has no creature picture (its def name in the
/// creature table is empty: the original draws a town's turrets from the town's records,
/// 0x63CF88; E04c T7), and the original has no battlefield picture for these terrains (0x4646D0;
/// a battle on water shows the special battlefields `base:deck` or `base:boats`).
const GAPS: [&str; 3] = ["base:arrow_tower", "base:water", "base:rock"];

#[derive(Default, Deserialize)]
struct Pack {
    #[serde(default)]
    creature: Vec<Keyed>,
    #[serde(default)]
    obstacle: Vec<Keyed>,
    #[serde(default)]
    terrain: Vec<Keyed>,
    #[serde(default)]
    battlefield: Vec<Keyed>,
}

#[derive(Deserialize)]
struct Keyed {
    id: String,
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn load(top: &str) -> Tables {
    let files: Vec<_> = PACKS
        .iter()
        .map(|id| {
            let path = root().join(format!("clients/assets/{id}.toml"));
            tables::parse(&fs::read_to_string(&path).unwrap()).unwrap()
        })
        .collect();
    tables::load(&files, top).unwrap()
}

fn keys<T>(map: &std::collections::BTreeMap<String, T>) -> BTreeSet<String> {
    map.keys().cloned().collect()
}

#[test]
fn every_selection_loads() {
    for (top, order) in [
        ("base", &["base"][..]),
        ("ab", &["base", "ab"]),
        ("sod", &["base", "ab", "sod"]),
    ] {
        assert_eq!(load(top).order, order, "{top}");
    }
}

#[test]
fn every_pack_key_has_an_entry() {
    let t = load("sod");
    let (creatures, fields, obstacles) = (
        keys(&t.creatures),
        keys(&t.battlefields),
        keys(&t.obstacles),
    );
    let mut defined = BTreeSet::new();
    let mut missing = Vec::new();
    for id in PACKS {
        // The keys: creatures and terrains in core.toml, special battlefields and obstacles in
        // battle.toml; a terrain and a special battlefield never share a key.
        let read = |file: &str| -> Pack {
            let path = root().join(format!("packs/{id}/{file}"));
            fs::read_to_string(path).map_or(Pack::default(), |t| toml::from_str(&t).unwrap())
        };
        let (core, battle) = (read("core.toml"), read("battle.toml"));
        let lists = [
            (&core.creature, &creatures),
            (&core.terrain, &fields),
            (&battle.battlefield, &fields),
            (&battle.obstacle, &obstacles),
        ];
        for (list, have) in lists {
            for k in list {
                let key = format!("{id}:{}", k.id);
                if !have.contains(&key) {
                    missing.push(key.clone());
                }
                assert!(defined.insert(key.clone()), "{key} twice");
            }
        }
    }
    assert_eq!(missing, GAPS);
    for have in [&creatures, &fields, &obstacles] {
        assert_eq!(have.difference(&defined).count(), 0, "{have:?}");
    }
}
