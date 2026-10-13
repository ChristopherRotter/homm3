//! The committed asset tables against the committed packs (E17 M1 T4; no original file needed):
//! each expansion selection loads, and every creature, terrain and obstacle key of a pack has an
//! entry in that pack's table, except the listed gaps. Extra entries are the special battlefields
//! and obstacle types no pack defines yet.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use h3_view::tables::{self, Tables};
use serde::Deserialize;

const PACKS: [&str; 3] = ["base", "ab", "sod"];
/// Pack keys without graphics: the original has no battlefield picture for these terrains
/// (0x4646D0; battles on water use the ship pictures).
const GAPS: [&str; 2] = ["base:water", "base:rock"];
/// The special battlefields' keys (VCMI's names) until the packs define them.
const SPECIAL: [&str; 12] = [
    "sand_shore",
    "magic_plains",
    "cursed_ground",
    "holy_ground",
    "evil_fog",
    "clover_field",
    "lucid_pools",
    "fiery_fields",
    "rocklands",
    "magic_clouds",
    "ship_to_ship",
    "ship",
];

#[derive(Default, Deserialize)]
struct Pack {
    #[serde(default)]
    creature: Vec<Keyed>,
    #[serde(default)]
    obstacle: Vec<Keyed>,
    #[serde(default)]
    terrain: Vec<Keyed>,
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
        // The keys: creatures and terrains in core.toml, obstacles in battle.toml.
        let read = |file: &str| -> Pack {
            let path = root().join(format!("packs/{id}/{file}"));
            fs::read_to_string(path).map_or(Pack::default(), |t| toml::from_str(&t).unwrap())
        };
        let (core, battle) = (read("core.toml"), read("battle.toml"));
        let pack = Pack {
            creature: core.creature,
            terrain: core.terrain,
            obstacle: battle.obstacle,
        };
        let lists = [
            (&pack.creature, &creatures),
            (&pack.terrain, &fields),
            (&pack.obstacle, &obstacles),
        ];
        for (list, have) in lists {
            for k in list {
                let key = format!("{id}:{}", k.id);
                if !have.contains(&key) {
                    missing.push(key.clone());
                }
                defined.insert(key);
            }
        }
    }
    assert_eq!(missing, GAPS);
    let special: BTreeSet<String> = SPECIAL.iter().map(|k| format!("base:{k}")).collect();
    for key in fields.difference(&defined) {
        assert!(special.contains(key), "{key}");
    }
    assert_eq!(creatures.difference(&defined).count(), 0);
    for key in obstacles.difference(&defined) {
        let n: usize = key.strip_prefix("base:obstacle_").unwrap().parse().unwrap();
        assert!(n < 91 || (100..134).contains(&n), "{key}");
    }
}
