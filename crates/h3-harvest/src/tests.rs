//! Harvester tests on synthetic inputs (the real installation is checked by
//! `cargo xtask verify --with-original-data`).

use super::*;

fn table(rows: &[Vec<String>]) -> TextTable {
    TextTable {
        records: rows.to_vec(),
    }
}

fn single(names: &[&str]) -> TextTable {
    table(
        &names
            .iter()
            .map(|n| vec![(*n).to_owned()])
            .collect::<Vec<_>>(),
    )
}

/// A creature row with the given name, growth and attack.
fn row(name: &str, growth: u16, attack: u16) -> Vec<String> {
    let mut r: Vec<String> = vec![name.into(), format!("{name}s")];
    r.extend(["0", "0", "0", "0", "0", "0", "100"].map(String::from)); // cost: 100 gold
    r.extend([50, 60, growth, 0, 10, 5, attack, 3, 1, 2, 0, 0, 5, 9].map(|n| n.to_string()));
    r.push("flavor".into());
    r.push("DOUBLE_WIDE | const_jousting".into());
    r
}

/// 150 creatures: ids 0-9 town 0, 10-19 town 1, 20-24 town 2 (an AB town), the rest neutral;
/// id 3 is an unused slot.
fn inputs() -> Inputs {
    let header = vec![vec!["Name".to_owned()], vec!["Singular".to_owned()]];
    let mut sod = header.clone();
    let mut exe = Vec::new();
    for id in 0..CREATURE_COUNT {
        if id % 14 == 0 {
            sod.push(vec![String::new(); 3]); // separator row
        }
        let unused = id == 3;
        let name = if unused {
            "NOT USED (1)".to_owned()
        } else {
            format!("Unit {id}")
        };
        sod.push(row(&name, 7, 4));
        let town = match id {
            0..=9 => 0,
            10..=19 => 1,
            20..=24 => 2,
            _ => -1,
        };
        exe.push(CreatureRecord {
            id: u16::try_from(id).unwrap(),
            town,
            level: if unused { -1 } else { 1 },
            sound: String::new(),
            def_name: String::new(),
            flags: match id {
                7 => 0x11 | UNHARVESTED_BITS,
                149 => WAR_MACHINE_BIT,
                _ => 0x11,
            },
        });
    }
    let mut ab = sod.clone();
    // Unit 5 changed in SoD only, Unit 6 already in the latest RoE patch.
    let pos = |t: &[Vec<String>], n: &str| t.iter().position(|r| r[0] == n).unwrap();
    let i5 = pos(&ab, "Unit 5");
    ab[i5] = row("Unit 5", 5, 4);
    let i6 = pos(&ab, "Unit 6");
    ab[i6] = row("Unit 6", 7, 2);
    let i8 = pos(&ab, "Unit 8");
    ab[i8][23] = "older flavor".into();
    Inputs {
        exe,
        sod_creatures: table(&sod),
        ab_creatures: table(&ab),
        towns: single(&["Alpha", "Beta", "Gamma"]),
        resources: single(&[
            "Wood",
            "Mercury",
            "Ore",
            "Sulfur",
            "Crystal",
            "Gems",
            "Gold",
            "Abandoned",
        ]),
    }
}

const GAP: &str = r#"
[[source]]
id = "s"
kind = "wiki"
ref = "test"
url = "https://example.org"
retrieved = "2026-10-01"

[ab]
sources = ["s"]
factions = ["Gamma"]
creatures = ["Unit 20", "Unit 21", "Unit 22", "Unit 30"]

[[roe_creature]]
name = "Unit 23"
level = 4
sources = ["s"]

[[roe_creature]]
name = "Unit 24"
level = 3
sources = ["s"]

[[table_change]]
name = "Unit 5"
since = "sod"
sources = ["s"]

[[table_change]]
name = "Unit 6"
since = "base"
sources = ["s"]

[battle]
field_width = 17
field_height = 11
side_columns = 1
army_slots = 2
damage_roll_samples = 10
attack_bonus_per_point_percent = 5
attack_bonus_cap_percent = 300
defense_reduction_per_point_permille = 25
defense_reduction_cap_permille = 700
defend_bonus_percent = 20
defend_bonus_min = 1
ranged_full_damage_distance = 10
ranged_penalty_percent = 50
melee_penalty_percent = 50
morale_dice = 24
good_morale_chances = [1, 2, 3]
bad_morale_chances = [2, 4, 6]
luck_dice = 24
good_luck_chances = [1, 2, 3]
formation = [[5], [2, 8]]
formation_sources = ["s"]

[[creature_abilities]]
creature = "Unit 1"
abilities = [{ kind = "jousting", percent_per_hex = 5 }]
sources = ["s"]

[[creature_abilities]]
creature = "Unit 2"
abilities = [
    { kind = "hate", creature = "Unit 1", percent = 50 },
    { kind = "hate", creature = "Unit 20", percent = 50 },
]
sources = ["s"]

[[creature_abilities]]
creature = "Unit 20"
abilities = [{ kind = "hate", creature = "Unit 2", percent = 100 }]
sources = ["s"]
"#;

fn gap() -> GapFill {
    GapFill::parse(GAP).unwrap()
}

fn pack<'a>(h: &'a Harvest, id: &str) -> &'a str {
    &h.packs.iter().find(|(p, _)| *p == id).unwrap().1
}

#[test]
fn harvests_three_packs_that_compile_and_are_deterministic() {
    let h = harvest(&inputs(), &gap()).unwrap();
    assert_eq!(h, harvest(&inputs(), &gap()).unwrap());
    let (base, ab, sod) = (pack(&h, "base"), pack(&h, "ab"), pack(&h, "sod"));
    let r = h3_pack::compile(&[base, ab, sod], "base:faithful").unwrap();
    // 150 ids minus one unused slot and one war machine.
    assert_eq!(r.creature_count(), 148);
    assert!(r.creature_by_key("base:unit_149").is_none());
    assert_eq!(r.resources().len(), 7);
    let ids: Vec<&str> = r.factions().iter().map(|f| f.key.as_str()).collect();
    assert_eq!(ids, ["base:alpha", "base:beta", "ab:gamma"]);
    // AB creatures live in `ab`, RoE creatures in `base`.
    assert!(r.creature_by_key("ab:unit_20").is_some());
    assert!(r.creature_by_key("ab:unit_30").is_some());
    assert!(r.creature_by_key("base:unit_23").is_some());
    assert!(r.creature_by_key("base:unit_3").is_none());

    // RoE creature of an AB faction: neutral in base, moved by ab.
    let roe_only = h3_pack::compile(&[base], "base:faithful").unwrap();
    let u23 = roe_only.creature_by_key("base:unit_23").unwrap();
    let info = roe_only.creature_info(u23).unwrap();
    assert_eq!((info.faction, info.level), (None, 4));
    let info = r.creature_info(u23).unwrap();
    assert_eq!(
        r.factions()[info.faction.unwrap().index()].key.as_str(),
        "ab:gamma"
    );
    assert_eq!(info.level, 2);

    // `since = "sod"`: base has the AB-era value, sod overrides it.
    let u5 = r.creature_by_key("base:unit_5").unwrap();
    assert_eq!(roe_only.creature_info(u5).unwrap().growth, 5);
    assert_eq!(r.creature_info(u5).unwrap().growth, 7);
    assert!(sod.contains("target = \"base:unit_5\"\ngrowth = 7\n"));
    // `since = "base"`: base already has the SoD value.
    let u6 = r.creature_by_key("base:unit_6").unwrap();
    assert_eq!(roe_only.creature(u6).unwrap().attack, 4);
    // Costs and flags.
    let u0 = roe_only.creature_by_key("base:unit_0").unwrap();
    let i0 = roe_only.creature_info(u0).unwrap();
    assert_eq!(i0.cost.len(), 1);
    assert!(
        roe_only
            .creature(u0)
            .unwrap()
            .flags
            .contains(h3_content_flag("two_hex"))
    );
    assert!(base.contains("cost = { gold = 100 }"));
    assert!(ab.contains("cost = { \"base:gold\" = 100 }"));
    // Flavor text never reaches the packs.
    assert!(!base.contains("flavor") && !ab.contains("flavor") && !sod.contains("flavor"));
    // Abilities: a base creature's references to ab creatures go into an ab override.
    assert!(base.contains("abilities = [{ kind = \"jousting\", percent_per_hex = 5 }]"));
    assert!(
        base.contains("abilities = [{ kind = \"hate\", creature = \"unit_1\", percent = 50 }]")
    );
    assert!(ab.contains(
        "target = \"base:unit_2\"\nabilities = [{ kind = \"hate\", creature = \"base:unit_1\", \
         percent = 50 }, { kind = \"hate\", creature = \"unit_20\", percent = 50 }]"
    ));
    assert!(
        ab.contains("abilities = [{ kind = \"hate\", creature = \"base:unit_2\", percent = 100 }]")
    );
    let u2 = r.creature_by_key("base:unit_2").unwrap();
    assert_eq!(r.abilities(u2).len(), 2);
    assert_eq!(roe_only.abilities(u2).len(), 1);

    let report = h.report.join("\n");
    assert!(report.contains("unit_7`: unidentified flag bit 0x800"));
    assert!(report.contains("`unit_0`: attribute column says const_jousting, gap-fill lacks"));
    assert!(!report.contains("`unit_1`: attribute column says const_jousting"));
    assert!(report.contains("`unit_8`: ability text changed"));
    assert!(report.contains("row `Abandoned`"));
    assert!(report.contains("unused slot"));
    assert!(report.contains("`Unit 149`): war machine"));
}

fn h3_content_flag(name: &str) -> h3_content::CreatureFlag {
    h3_content::CreatureFlag::from_name(name).unwrap()
}

#[test]
fn inconsistencies_are_errors() {
    let g = gap();
    let mut i = inputs();
    i.exe[3].level = 0;
    assert!(
        harvest(&i, &g)
            .unwrap_err()
            .contains("disagree on whether it is used")
    );

    let mut i = inputs();
    i.exe[0].flags = 0x2000;
    assert!(
        harvest(&i, &g)
            .unwrap_err()
            .contains("unexpected flag bits 0x2000")
    );

    let mut i = inputs();
    i.exe[0].town = 7;
    assert!(harvest(&i, &g).unwrap_err().contains("has no TownType row"));

    let mut i = inputs();
    i.exe.pop();
    assert!(harvest(&i, &g).unwrap_err().contains("149 records"));

    // An unclassified difference.
    let mut i = inputs();
    let at = i
        .ab_creatures
        .records
        .iter()
        .position(|r| r[0] == "Unit 9")
        .unwrap();
    i.ab_creatures.records[at] = row("Unit 9", 1, 4);
    assert!(
        harvest(&i, &g)
            .unwrap_err()
            .contains("classify it in gapfill")
    );

    // A classified row that does not differ.
    let stale = GapFill::parse(&GAP.replace(
        "[battle]",
        "[[table_change]]\nname = \"Unit 9\"\nsince = \"base\"\nsources = [\"s\"]\n\n[battle]",
    ))
    .unwrap();
    assert!(
        harvest(&inputs(), &stale)
            .unwrap_err()
            .contains("does not differ")
    );

    // A RoE creature in an AB faction without a RoE entry.
    let no_roe = GapFill::parse(&GAP.replace("name = \"Unit 24\"", "name = \"Unit 40\"")).unwrap();
    assert!(
        harvest(&inputs(), &no_roe)
            .unwrap_err()
            .contains("add a roe_creature entry")
    );

    // Listed as both.
    let both = GapFill::parse(&GAP.replace("\"Unit 30\"]", "\"Unit 30\", \"Unit 23\"]")).unwrap();
    assert!(
        harvest(&inputs(), &both)
            .unwrap_err()
            .contains("AB addition and as RoE")
    );

    // Ability entries: listed twice, for a creature that is not harvested, referring to one
    // that is not harvested, and with parameters the compiler rejects.
    let twice = GapFill::parse(&format!(
        "{GAP}\n[[creature_abilities]]\ncreature = \"Unit 1\"\nabilities = []\nsources = [\"s\"]\n"
    ))
    .unwrap();
    assert!(
        harvest(&inputs(), &twice)
            .unwrap_err()
            .contains("listed twice")
    );
    let machine =
        GapFill::parse(&GAP.replace("creature = \"Unit 20\"\n", "creature = \"Unit 149\"\n"))
            .unwrap();
    assert!(
        harvest(&inputs(), &machine)
            .unwrap_err()
            .contains("abilities for `Unit 149`, which is not harvested")
    );
    let target = GapFill::parse(&GAP.replace(
        "creature = \"Unit 1\", percent",
        "creature = \"Unit 149\", percent",
    ))
    .unwrap();
    assert!(
        harvest(&inputs(), &target)
            .unwrap_err()
            .contains("refers to `Unit 149`, not harvested")
    );
    let unnamed = GapFill::parse(&GAP.replace(
        "creature = \"Unit 1\", percent",
        "creature = \"Nobody\", percent",
    ))
    .unwrap();
    assert!(
        harvest(&inputs(), &unnamed)
            .unwrap_err()
            .contains("unknown creature `Nobody`")
    );
    let invalid =
        GapFill::parse(&GAP.replace("percent_per_hex = 5", "percent_per_hex = 0")).unwrap();
    assert!(
        harvest(&inputs(), &invalid)
            .unwrap_err()
            .contains("percent_per_hex must be at least 1")
    );

    let unknown = GapFill::parse(&GAP.replace("\"Unit 30\"", "\"Unit 999\"")).unwrap();
    assert!(
        harvest(&inputs(), &unknown)
            .unwrap_err()
            .contains("unknown creature")
    );
    let town =
        GapFill::parse(&GAP.replace("factions = [\"Gamma\"]", "factions = [\"Delta\"]")).unwrap();
    assert!(
        harvest(&inputs(), &town)
            .unwrap_err()
            .contains("unknown town")
    );
}

#[test]
fn table_parsing_errors() {
    let mut i = inputs();
    i.sod_creatures.records[5].pop();
    assert!(harvest(&i, &gap()).unwrap_err().contains("24 fields"));
    let mut i = inputs();
    i.sod_creatures.records[5][13] = "lots".into();
    assert!(
        harvest(&i, &gap())
            .unwrap_err()
            .contains("not a valid number")
    );
    let mut i = inputs();
    i.sod_creatures.records.truncate(20);
    assert!(harvest(&i, &gap()).unwrap_err().contains("creature rows"));
    let mut i = inputs();
    i.resources.records.truncate(3);
    assert!(harvest(&i, &gap()).unwrap_err().contains("RESTYPES"));
}

#[test]
fn helpers() {
    assert_eq!(key_of("Royal Griffin"), "royal_griffin");
    assert_eq!(key_of(" Will-o-wisp's "), "will_o_wisps");
    assert_eq!(quote("a \"b\" \\ c\n"), "\"a \\\"b\\\" \\\\ c\\u000A\"");
    assert_eq!(flags(0x8000_0003).unwrap(), ["two_hex", "flying", "dragon"]);
    assert!(flags(0x4000_0000).is_err());
}
