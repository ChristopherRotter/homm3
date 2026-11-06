//! The town harvest on synthetic inputs (E09 T5b2b2): the crate's fixture with three factions'
//! town tables, Alpha and Beta in `base`, Gamma in `ab`, each with seven levels of two creatures
//! (Alpha ids 100 to 113, Beta 114 to 127, Gamma 128 to 141, Gamma's in `ab`).

use h3_formats::exe::towns::{BuildingEntry, FactionTowns, GarrisonLevel, HordeRecord, TownData};

use super::*;
use crate::gapfill::GapFill;
use crate::harvest;
use crate::tests::{GAP, inputs};

const FACTIONS: [&str; 3] = ["Alpha", "Beta", "Gamma"];

/// The first creature id of a faction's fourteen.
fn first(f: usize) -> usize {
    100 + 14 * f
}

/// The gap-fill's `[towns]` for the fixture, and Gamma's creatures in `ab`.
const TOWNS: &str = r#"
[towns]
role = [
  { building = 10, roles = { hall = 1 }, sources = ["s"] },
  { building = 11, roles = { hall = 2 }, sources = ["s"] },
  { building = 12, roles = { hall = 3 }, sources = ["s"] },
  { building = 13, roles = { hall = 4 }, sources = ["s"] },
  { building = 7, roles = { fort = 1 }, sources = ["s"] },
  { building = 8, roles = { fort = 2 }, sources = ["s"] },
  { building = 9, roles = { fort = 3 }, sources = ["s"] },
  { building = 5, roles = { tavern = true, morale = 1 }, sources = ["s"] },
  { building = 15, roles = { silo = true }, sources = ["s"] },
  { building = 26, roles = { grail = true }, sources = ["s"] },
  { faction = "Alpha", building = 21, roles = { luck = 2 }, sources = ["s"] },
  { faction = "Beta", building = 26, roles = { reveal = "all", defender = { attack = 20, mana = 5 } }, sources = ["s"] },
]

[towns.params]
sources = ["s"]
gold = "Gold"
hall_gold = [500, 1000, 2000, 4000]
grail_gold = 5000
reveal_radius = 5
capitol_demoted_to = 12

[towns.visions]
sources = ["s"]
factor = 3
min_radius = 2
creatures = [129, 100]

[towns.moat]
sources = ["s"]
roe_ab = { moat = "harmless" }
sod = { moat = "damaging" }
faction = [
  { faction = "Beta", roe_ab = { moat = "absent", siege_obstacles = true }, sod = { moat = "damaging" } },
]
"#;

fn town_gap(towns: &str) -> String {
    let ab: Vec<String> = (first(2)..first(3))
        .map(|id| format!("\"Unit {id}\""))
        .collect();
    let gap = GAP.replace(
        "creatures = [\"Unit 20\", \"Unit 21\", \"Unit 22\", \"Unit 30\"]",
        &format!(
            "creatures = [\"Unit 20\", \"Unit 21\", \"Unit 22\", \"Unit 30\", {}]",
            ab.join(", ")
        ),
    );
    format!("{gap}{towns}")
}

fn entry(building: u8, listed: &[u8]) -> BuildingEntry {
    BuildingEntry {
        building,
        listed: listed.to_vec(),
    }
}

/// A faction's tables: guild levels 1 and 2, the Tavern, the forts, the halls, the silo, the
/// Blacksmith, the horde 18 of level 2 and (Alpha and Gamma) its upgrade 19, the special 21, the
/// Grail, the fourteen dwellings.
fn faction(f: usize) -> FactionTowns {
    let mut ids: Vec<u8> = vec![0, 1, 5, 7, 8, 9, 11, 12, 13, 15, 16, 18, 21, 26];
    if f != 1 {
        ids.push(19);
    }
    ids.extend(30..44);
    let mut map: Vec<Option<u8>> = vec![None; 41];
    for (slot, id) in [
        (0, 11),
        (1, 12),
        (2, 13),
        (3, 7),
        (4, 8),
        (5, 9),
        (6, 5),
        (7, 16),
        (9, 15),
        (11, 0),
        (12, 1),
        (13, 2), // a guild level the faction lacks: no slot
        (17, 26),
        (18, 21),
        (21, 18),
    ] {
        map[slot] = Some(id);
    }
    for k in 0..14 {
        map[22 + k] = Some(30 + u8::try_from(k).unwrap());
    }
    let base = |l: usize| u16::try_from(first(f) + 2 * l).unwrap();
    let mut creatures: Vec<u16> = (0..7).map(base).collect();
    creatures.extend((0..7).map(|l| base(l) + 1));
    FactionTowns {
        buildings: ids.into_iter().map(|id| entry(id, &[])).collect(),
        supersedes: if f == 1 {
            vec![entry(18, &[31])]
        } else {
            vec![entry(18, &[31]), entry(19, &[38, 18])]
        },
        map_building_ids: map,
        creatures,
        hordes: vec![
            HordeRecord {
                creature: Some(base(1)),
                growth: 3,
            },
            HordeRecord {
                creature: None,
                growth: 0,
            },
            HordeRecord {
                creature: None,
                growth: 0,
            },
            HordeRecord {
                creature: None,
                growth: 0,
            },
        ],
        silo: vec![1, 0, 1, 0, 0, 0, 0],
    }
}

fn names(rows: Vec<String>) -> TextTable {
    TextTable {
        records: rows.into_iter().map(|r| vec![r]).collect(),
    }
}

fn town_tables() -> Tables {
    let mut neutral: Vec<String> = [
        "Guild 1",
        "Guild 2",
        "Guild 3",
        "Guild 4",
        "Guild 5",
        "Tavern",
        "Shipyard",
        "Fort",
        "Citadel",
        "Castle",
        "Village Hall",
        "Town Hall",
        "City Hall",
        "Capitol",
        "Marketplace",
        "",
        "Blacksmith",
    ]
    .map(String::from)
    .to_vec();
    neutral.push("Moat".into());
    let mut special = Vec::new();
    let mut dwellings = Vec::new();
    for name in FACTIONS {
        for id in SPECIAL_ORDER {
            special.push(match id {
                15 => "Resource Silo".to_owned(),
                18 | 19 => format!("{name} Horde"),
                26 => format!("{name} Grail"),
                _ => format!("{name} Special {id}"),
            });
        }
        for upgraded in [false, true] {
            for l in 1..=7 {
                let up = if upgraded { "Upg. " } else { "" };
                dwellings.push(format!("{up}{name} Den {l}"));
            }
        }
    }
    Tables {
        neutral: names(neutral),
        special: names(special),
        dwellings: names(dwellings),
    }
}

fn town_inputs() -> Inputs {
    let mut i = inputs();
    for f in 0..3 {
        for k in 0..14 {
            let rec = &mut i.exe[first(f) + k];
            rec.town = i32::try_from(f).unwrap();
            rec.level = i32::try_from(k / 2).unwrap();
        }
    }
    i.town_tables = TownData {
        factions: (0..3).map(faction).collect(),
        common_supersedes: vec![
            entry(1, &[0]),
            entry(2, &[1]),
            entry(8, &[7]),
            entry(9, &[8]),
            entry(11, &[10]),
            entry(12, &[11]),
            entry(13, &[12]),
            entry(19, &[18]),
            entry(25, &[24]),
            entry(37, &[30]),
            entry(38, &[31]),
            entry(39, &[32]),
            entry(40, &[33]),
            entry(41, &[34]),
            entry(42, &[35]),
            entry(43, &[36]),
        ],
        horde_buildings: vec![18, 19, 24, 25],
        garrison: [(33, 8, 15), (33, 5, 7), (20, 3, 5), (13, 1, 3)]
            .map(|(chance, least, most)| GarrisonLevel {
                chance,
                least,
                most,
            })
            .to_vec(),
        dwelling_2_chance: 30,
    };
    i.town_names = town_tables();
    i
}

/// Runs both harvests; the town files by pack.
fn run(i: &Inputs, gap: &str) -> Result<BTreeMap<&'static str, String>, String> {
    let gap = GapFill::parse(gap)?;
    let mut h = harvest(i, &gap)?;
    let files = harvest_towns(i, &gap, &h.packs, &mut h.report)?;
    assert!(h.report.iter().any(|l| l.contains("towns: requirements")));
    Ok(files.into_iter().collect())
}

fn files() -> BTreeMap<&'static str, String> {
    run(&town_inputs(), &town_gap(TOWNS)).unwrap()
}

#[test]
fn harvests_the_town_files_that_compile() {
    let f = files();
    assert_eq!(f, files());
    let (base, ab, sod) = (&f["base"], &f["ab"], &f["sod"]);
    // The parameters in base, from the gap-fill and the executable's tables.
    assert!(base.contains(
        "[town_params]\ngold = \"gold\"\nhall_gold = [500, 1000, 2000, 4000]\n\
         grail_gold = 5000\nreveal_radius = 5\nlevel_2_dwelling_chance = 30\n\
         neutral_garrison = [\n  { chance = 33, count = [8, 15] },\n"
    ));
    assert!(base.contains(
        "capitol_demoted_to = \"city_hall\"\nvisions_factor = 3\nvisions_min_radius = 2\n"
    ));
    assert!(!ab.contains("[town_params]") && !sod.contains("[town_params]"));
    // Visions' creatures, each in its own pack's file (E09 T8c2).
    assert!(base.contains("\n[town_visions]\ncreatures = [\"unit_100\"]\n"));
    assert!(ab.contains("\n[town_visions]\ncreatures = [\"unit_129\"]\n"));
    assert!(!sod.contains("[town_visions]"));
    // Alpha and Beta in base, Gamma in ab with its creatures' pack.
    assert!(
        base.contains("faction = \"alpha\"\nmoat = \"harmless\"\nsilo = { wood = 1, ore = 1 }\n")
    );
    assert!(base.contains("faction = \"beta\"\nmoat = \"absent\"\nsiege_obstacles = true\n"));
    assert!(ab.contains("faction = \"gamma\"\nmoat = \"harmless\"\nsilo = { \"base:wood\" = 1"));
    assert!(base.contains(
        "creatures = [\n  [\"unit_100\", \"unit_101\"],\n  [\"unit_102\", \"unit_103\"],"
    ));
    assert!(ab.contains("[\"unit_128\", \"unit_129\"]"));
    // The Village Hall, the slots (none for a building the faction lacks), the names and keys.
    assert!(base.contains("key = \"village_hall\"\n  name = \"Village Hall\"\n  hall = 1\n"));
    assert!(base.contains(
        "key = \"guild_2\"\n  name = \"Guild 2\"\n  slot = 12\n  supersedes = [\"guild_1\"]\n"
    ));
    assert!(!base.contains("guild_3"));
    assert!(base.contains("key = \"upg_alpha_den_2\"\n  name = \"Upg. Alpha Den 2\"\n  slot = 30\n  supersedes = [\"alpha_den_2\"]\n  dwelling = [2, true]\n"));
    // The horde supersedes its level's dwelling; its upgrade is forbidden with it and built with
    // it and the level's upgraded dwelling.
    assert!(base.contains("key = \"alpha_horde\"\n  name = \"Alpha Horde\"\n  slot = 21\n  supersedes = [\"alpha_den_2\"]\n"));
    assert!(base.contains(
        "key = \"upg_alpha_horde\"\n  name = \"Alpha Horde\"\n  supersedes = [\"alpha_horde\", \"upg_alpha_den_2\"]\n  \
         forbidden_with = \"alpha_horde\"\n  auto_with = [\"alpha_horde\", \"upg_alpha_den_2\"]\n"
    ));
    assert!(!base.contains("upg_beta_horde"));
    // The roles, merged and in the format's order.
    assert!(base.contains(
        "key = \"tavern\"\n  name = \"Tavern\"\n  slot = 6\n  tavern = true\n  morale = 1\n"
    ));
    assert!(base.contains(
        "key = \"alpha_special_21\"\n  name = \"Alpha Special 21\"\n  slot = 18\n  luck = 2\n"
    ));
    assert!(
        base.contains("grail = true\n  defender = { attack = 20, mana = 5 }\n  reveal = \"all\"\n")
    );
    // SoD's overrides: the moat everywhere, Beta's obstacles too.
    assert!(sod.contains("[[town_override]]\nfaction = \"base:alpha\"\nmoat = \"damaging\"\n"));
    assert!(
        sod.contains("faction = \"base:beta\"\nmoat = \"damaging\"\nsiege_obstacles = false\n")
    );
    assert!(sod.contains("faction = \"ab:gamma\"\nmoat = \"damaging\"\n"));
    assert!(base.starts_with("# GENERATED by h3-harvest (E09 T5b2b2)"));
}

/// The harvest's error for a fixture changed by `change`, with `towns` as the gap-fill's
/// `[towns]`.
fn error_with(change: impl FnOnce(&mut Inputs), towns: &str) -> String {
    let mut i = town_inputs();
    change(&mut i);
    run(&i, &town_gap(towns)).unwrap_err()
}

fn error(change: impl FnOnce(&mut Inputs)) -> String {
    error_with(change, TOWNS)
}

#[test]
fn inconsistent_tables_are_errors() {
    let e = run(&town_inputs(), &town_gap("")).unwrap_err();
    assert!(e.contains("no [towns]"), "{e}");
    let e = error(|i| {
        i.town_tables.factions.pop();
    });
    assert!(e.contains("2 factions' tables, TownType.txt 3 rows"), "{e}");
    let e = error(|i| {
        i.town_tables.horde_buildings.pop();
    });
    assert!(e.contains("not four"), "{e}");
    let e = error(|i| i.town_tables.factions[0].buildings.push(entry(20, &[])));
    assert!(e.contains("Alpha: its list holds building 20"), "{e}");
    let e = error(|i| i.town_tables.factions[0].buildings.push(entry(10, &[])));
    assert!(e.contains("holds building 10"), "{e}");
    let e = error(|i| i.town_names.neutral.records[5][0].clear());
    assert!(e.contains("BldgNeut.txt: no name in row 6"), "{e}");
    let e = error(|i| i.town_names.neutral.records[1][0] = "Guild 1".into());
    assert!(e.contains("the key `guild_1` twice"), "{e}");
    let e = error(|i| i.town_tables.factions[0].buildings.push(entry(44, &[])));
    assert!(e.contains("building id 44 has no name table"), "{e}");
    let e = error(|i| i.town_tables.factions[0].map_building_ids[40] = Some(11));
    assert!(e.contains("building town_hall in two slots"), "{e}");
    let e = error(|i| i.town_tables.factions[1].supersedes.push(entry(25, &[24])));
    assert!(
        e.contains("Beta: its supersede list names building 25"),
        "{e}"
    );
    let e = error(|i| i.town_tables.factions[1].supersedes.push(entry(18, &[2])));
    assert!(
        e.contains("building 18 supersedes building 2, not its"),
        "{e}"
    );
    let e = error(|i| {
        i.town_tables.factions[0].creatures.pop();
    });
    assert!(e.contains("the creatures or hordes do not fit"), "{e}");
    let e = error(|i| {
        i.town_tables.common_supersedes.retain(|e| e.building != 25);
        i.town_tables.factions[1].buildings.push(entry(25, &[]));
    });
    assert!(e.contains("building 25 without its horde 24"), "{e}");
    let e = error(|i| i.town_tables.factions[0].hordes[0].creature = None);
    assert!(e.contains("horde 18 without a creature"), "{e}");
    let e = error(|i| i.town_tables.factions[0].hordes[0].creature = Some(1));
    assert!(e.contains("horde 18's creature is no level's"), "{e}");
    let e = error(|i| i.town_tables.factions[0].supersedes[0] = entry(18, &[32]));
    assert!(
        e.contains("does not supersede its level's dwelling 31"),
        "{e}"
    );
    let e = error(|i| i.town_tables.factions[0].creatures[0] = 3);
    assert!(e.contains("creature 3 is not harvested"), "{e}");
    // A base faction's creature in ab: outside base's closure.
    let e = error(|i| i.town_tables.factions[0].creatures[0] = 20);
    assert!(e.contains("`ab:unit_20` is outside base's closure"), "{e}");
    let e = error(|i| {
        i.town_tables.garrison.pop();
    });
    assert!(e.contains("not four levels"), "{e}");
    let e = error(|i| i.town_tables.garrison[0].least = 70_000);
    assert!(e.contains("does not fit the format"), "{e}");
    // The town compiler's checks: no level-1 dwelling.
    let e = error(|i| {
        i.town_tables.factions[0]
            .buildings
            .retain(|b| !matches!(b.building, 30 | 37));
    });
    assert!(
        e.contains("generated town files do not compile (base"),
        "{e}"
    );
}

#[test]
fn gapfill_town_errors() {
    let bad = |from: &str, to: &str| {
        assert!(TOWNS.contains(from), "{from}");
        error_with(|_| {}, &TOWNS.replace(from, to))
    };
    let e = bad("faction = \"Alpha\"", "faction = \"Delta\"");
    assert!(e.contains("unknown town `Delta`"), "{e}");
    let e = bad(
        "{ faction = \"Beta\", roe_ab",
        "{ faction = \"Omega\", roe_ab",
    );
    assert!(e.contains("unknown town `Omega`"), "{e}");
    let e = bad("roles = { luck = 2 }", "roles = { dwelling = [1, false] }");
    assert!(e.contains("`dwelling` is not a role it may give"), "{e}");
    let e = bad("roles = { luck = 2 }", "roles = { speed = 2 }");
    assert!(e.contains("`speed` is not a role it may give"), "{e}");
    let e = bad(
        "building = 21, roles = { luck = 2 }",
        "building = 26, roles = { grail = true }",
    );
    assert!(e.contains("Alpha building 26: `grail` twice"), "{e}");
    let e = bad(
        "building = 21, roles = { luck = 2 }",
        "building = 2, roles = { luck = 2 }",
    );
    assert!(
        e.contains("Alpha: the gap-fill gives roles to building 2, not its"),
        "{e}"
    );
    let e = bad("gold = \"Gold\"", "gold = \"Mithril\"");
    assert!(e.contains("unknown resource `Mithril`"), "{e}");
    let e = bad("creatures = [129, 100]", "creatures = [129, 999]");
    assert!(e.contains("unknown Visions creature 999"), "{e}");
    let e = bad("creatures = [129, 100]", "creatures = [129, 129]");
    assert!(e.contains("do not compile"), "{e}");
    let e = bad("roles = { luck = 2 }", "roles = { luck = 9 }");
    assert!(e.contains("do not compile"), "{e}");
}

#[test]
fn helpers() {
    assert_eq!(toml_key("wood"), "wood");
    assert_eq!(toml_key("base:wood"), "\"base:wood\"");
    assert_eq!(toml_key(""), "\"\"");
    assert_eq!(reference("base", "x", "base").unwrap(), "x");
    assert_eq!(reference("base", "x", "ab").unwrap(), "base:x");
    let v: toml::Value = toml::from_str("a = { b = [1, \"c\"] }").unwrap();
    assert_eq!(inline(&v["a"]), "{ b = [1, \"c\"] }");
    assert!(check(&[], &[]).unwrap_err().contains("no core pack `base`"));
    assert_eq!(crate::gapfill::TownMoat::Absent.word(), "absent");
}
