//! Harvester tests on synthetic inputs (the real installation is checked by
//! `cargo xtask verify --with-original-data`).

use super::*;
use h3_formats::exe::{HeroRecord, SkillValue, SpecialtyRecord};

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
        let mut r = row(&name, 7, 4);
        if matches!(id, 148 | 149) {
            r[14] = "0".into(); // war machines have speed 0 (BAT-WAR-001)
        }
        sod.push(r);
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
                148 | 149 => WAR_MACHINE_BIT,
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
        terrains: single(&["Mud", "Ice"]),
        hero_data: hero_data(),
        hero_tables: hero_tables(),
        artifact_tables: artifact_tables(),
    }
}

/// Artifact positions of the AB-era and the SoD `ArtSlots.txt`.
const AB_POSITIONS: [&str; 6] = [
    "Head",
    "Right Ring",
    "Left Ring",
    "Misc 1",
    "Misc 2",
    "War Machine 1",
];

/// An `ARTRAITS.TXT` row: name, cost, the flagged positions, class.
fn artifact_row(
    header: &[String],
    name: &str,
    cost: &str,
    slots: &[&str],
    class: &str,
) -> Vec<String> {
    header
        .iter()
        .map(|h| match h.as_str() {
            "Name" => name.to_owned(),
            "Cost" => cost.to_owned(),
            "Class (S,T,N,J,R)" => class.to_owned(),
            "Description" => "text".to_owned(),
            p if slots.contains(&p) => "x".to_owned(),
            _ => " ".to_owned(),
        })
        .collect()
}

/// Ten SoD artifacts (eight in the AB-era table; `Vial` is an AB addition, `Circle` a
/// combination, `Trace` unused) over six AB-era positions plus SoD's `Misc 5` and `Spell Book`.
fn artifact_tables() -> artifacts::Tables {
    let sod_positions: Vec<&str> = AB_POSITIONS
        .iter()
        .copied()
        .chain(["Misc 5", "Spell Book"])
        .collect();
    let header = |positions: &[&str]| {
        let mut h = strings(&["Name", "Cost"]);
        h.extend(positions.iter().rev().map(|p| (*p).to_owned()));
        h.extend(strings(&["Class (S,T,N,J,R)", "Description"]));
        h
    };
    let rows = |positions: &[&str], sod: bool| {
        let h = header(positions);
        let misc: &[&str] = if sod {
            &["Misc 1", "Misc 2", "Misc 5"]
        } else {
            &["Misc 1", "Misc 2"]
        };
        let ring = ["Right Ring", "Left Ring"];
        let mut t = vec![
            vec![String::new(), String::new(), "Hero Slots".to_owned()],
            h.clone(),
        ];
        t.push(artifact_row(&h, "Helm", "1000", &["Head"], "T"));
        t.push(artifact_row(&h, "Band", "500", &ring, "N"));
        t.push(artifact_row(&h, "Loop", "600", &ring, "J"));
        t.push(artifact_row(&h, "Charm", "100", misc, "T"));
        t.push(artifact_row(&h, "Kit", "2500", &["War Machine 1"], "S"));
        t.push(artifact_row(&h, "Relic", "0", &[], "S"));
        t.push(artifact_row(&h, "Bow", "2000", misc, "R"));
        t.push(artifact_row(&h, "Vial", "20000", misc, "R"));
        if sod {
            t.push(artifact_row(&h, "Circle", "", &ring, "R"));
            t.push(artifact_row(&h, "Trace", "", &["Head"], "R"));
        }
        t.push(vec![String::new(); 4]); // padding row
        table(&t)
    };
    artifacts::Tables {
        sod: rows(&sod_positions, true),
        ab: rows(&AB_POSITIONS, false),
        sod_slots: single(&sod_positions),
        ab_slots: single(&AB_POSITIONS),
    }
}

fn strings(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).to_owned()).collect()
}

/// A hero record: class, two starting skills, starting spell, three creatures; RoE or not.
fn hero(class: i32, spell: i32, creatures: [i32; 3], roe: bool) -> HeroRecord {
    HeroRecord {
        class,
        skills: [(6, 1), (-1, 0)],
        spell_book: spell >= 0,
        spell,
        creatures,
        roe,
        expansions: true,
        campaign: false,
    }
}

fn specialty(kind: i32, id: i32, attack: i32, defense: i32, damage: i32) -> SpecialtyRecord {
    SpecialtyRecord {
        kind,
        id,
        attack,
        defense,
        damage,
        upgrades: [0, 0],
    }
}

/// Seven heroes, one per specialty kind 0, 1, 3, 4, 5, 7, 2; hero 3 is an expansion hero.
fn hero_data() -> HeroData {
    let int = |v: [i32; 4]| v.map(SkillValue::Integer);
    let float = |v: [f32; 4]| v.map(SkillValue::Float);
    HeroData {
        heroes: vec![
            hero(0, -1, [0, 2, 148], true),
            hero(1, 1, [4, 4, 10], true),
            hero(2, -1, [10, 11, 12], true),
            hero(4, 1, [20, 21, 0], false),
            hero(0, -1, [0, 0, 0], true),
            hero(1, -1, [1, 2, 4], true),
            hero(3, -1, [12, 13, 14], true),
        ],
        specialties: vec![
            specialty(0, 1, 0, 0, 0),
            specialty(1, 148, 0, 0, 0),
            specialty(3, 1, 0, 0, 0),
            specialty(4, 20, 2, 1, 3),
            specialty(5, 2, 0, 0, 0),
            specialty(7, 0, 5, 5, 0),
            specialty(2, 6, 0, 0, 0),
        ],
        spell_types: vec![0, -1, 1],
        skill_values: vec![
            (9, int([0, 1, 2, 3])),
            (6, int([0, 1, 2, 3])),
            (1, float([0.0, 0.1, 0.25, 0.5])),
            (22, float([0.0, 0.1, 0.2, 0.3])),
            (23, float([0.0, 0.05, 0.1, 0.15])),
            (4, float([0.0, 0.2, 0.4, 0.6])),
            (27, float([0.0, 1.0, 2.0, 3.0])),
        ],
        specialty_growth: 0.05,
        artifact_primary: [
            [1, 0, 0, 0],
            [0; 4],
            [0; 4],
            [0; 4],
            [0; 4],
            [0; 4],
            [0; 4],
            [0; 4],
            [0, 0, 6, 6],
            [0; 4],
        ]
        .to_vec(),
    }
}

/// 28 skills, six classes (two per town), seven heroes and three spells (the last one differs
/// between the AB-era and the SoD table).
fn hero_tables() -> heroes::Tables {
    let header = vec![vec!["".to_owned()], vec!["Name".to_owned()]];
    let mut skills = header.clone();
    for i in 0..28 {
        skills.push(strings(&[&format!("Skill {i}"), "b", "a", "e"]));
    }
    let mut classes = header.clone();
    for i in 0..6 {
        let mut r = strings(&[&format!("Class {i}"), "1.00", "2", "2", "1", "1"]);
        r.extend(strings(&["35", "45", "10", "10", "30", "30", "20", "20"]));
        r.extend((0..28).map(|s| (s % 3).to_string()));
        r.extend((0..9).map(|_| "5".to_owned()));
        classes.push(r);
    }
    let mut heroes = header.clone();
    for i in 0..7 {
        let mut r = vec![format!("Hero {i}")];
        for _ in 0..3 {
            r.extend(strings(&["2", "4", "ref"]));
        }
        heroes.push(r);
    }
    heroes[2][7..9].clone_from_slice(&strings(&["1", "1"])); // hero 0's war machine
    let spell = |name: &str, schools: [&str; 4]| {
        let mut r = vec![name.to_owned(), name.to_owned(), "1".to_owned()];
        r.extend(strings(&schools));
        r.extend((7..29).map(|c| c.to_string()));
        r.extend((29..34).map(|_| "text".to_owned()));
        r
    };
    let mut spells_header = header;
    spells_header[1] = strings(&["Name", "Abbr", "Level", "Earth", "Water", "Fire", "Air"]);
    let group = |n: &str| vec![n.to_owned()];
    let mut sod = spells_header.clone();
    sod.extend([
        group("Adventure Spells"),
        spell("Look", ["x", " ", " ", " "]),
        vec![String::new()],
        group("Combat Spells"),
        spell("Zap", [" ", " ", "x", "x"]),
        spell("Bolt", ["", "", "", "x"]),
    ]);
    let mut ab = sod.clone();
    let last = ab.len() - 1;
    ab[last] = spell("Old (disabled)", [" ", " ", " ", " "]);
    heroes::Tables {
        classes: table(&classes),
        heroes: table(&heroes),
        skills: table(&skills),
        sod_spells: table(&sod),
        ab_spells: table(&ab),
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
army_slots = 3
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
war_machine_positions = [[1, 3], [1, 1], [1, 9]]
loose_formation = [[5], [2, 8], [2, 5, 8]]
tight_formation = [[5], [4, 6], [4, 5, 6]]
formation_sources = ["s"]
tactics_zone_columns = [3, 5, 7]
tactics_sources = ["s"]

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

[[native_terrain]]
faction = "Alpha"
terrain = "Mud"
sources = ["s"]

[[native_terrain]]
faction = "Gamma"
terrain = "Ice"
sources = ["s"]

[war_machines]
harvest = ["Unit 148"]
sources = ["s"]

[obstacles]
sources = ["s"]
list = [
    { id = 0, terrains = ["Mud"], width = 2, height = 1, blocked = [[0, 0], [1, -1]] },
    { id = 7, terrains = ["Mud", "Ice"], absolute = true, blocked = [[5, 4]] },
]

[heroes]
upgrades = [["Unit 26", "Unit 30"]]
upgrade_sources = ["s"]

[heroes.creature_specialty]
bp_per_step = 500
speed = 1
war_machine_speed = 0
sources = ["s"]

[heroes.params]
primary_min = [0, 0, 1, 1]
primary_max = 99
backpack = 64
sources = ["s"]

[[heroes.skill_effects]]
skill = "Skill 19"
basic = [{ kind = "tactics", level = 1 }]
advanced = [{ kind = "tactics", level = 2 }]
expert = [{ kind = "tactics", level = 3 }]
sources = ["s"]

[[heroes.specialty_extra]]
hero = "Hero 3"
creature = "Unit 21"
stat = "speed"
amount = 1
sources = ["s"]

[artifacts]
ab = ["Vial"]
ab_sources = ["s"]
unused = ["Trace"]
unused_sources = ["s"]
no_backpack = ["war_machine_1", "spell_book"]
no_backpack_sources = ["s"]

[[artifacts.war_machines]]
artifact = "Kit"
creature = "Unit 148"
sources = ["s"]

[[artifacts.combinations]]
artifact = "Circle"
parts = ["Band", "Loop", "Helm"]
sources = ["s"]

[[artifacts.effects]]
artifacts = ["Band"]
effects = [{ kind = "luck", amount = 1 }]
sources = ["s"]

[[artifacts.effects]]
artifacts = ["Loop"]
effects = [{ kind = "morale", amount = 1 }]
sources = ["s"]

[[artifacts.effects]]
artifacts = ["Bow"]
effects = [{ kind = "ranged_damage", bp = 500 }]
needs_skill = "Skill 1"
sources = ["s"]

[[artifacts.effects]]
artifacts = ["Vial"]
effects = [{ kind = "creature_stat", amount = 5, stat = "attack", creatures = { flag = "dragon" } }]
sources = ["s"]

[[artifacts.effects]]
artifacts = ["Circle"]
effects = [
    { kind = "alignment_group", factions = ["Alpha", "Gamma"] },
    { kind = "regeneration", creatures = { creature = "Unit 21" }, hit_points = 5 },
]
sources = ["s"]

[[artifacts.pending]]
epic = "E04b"
what = "spells"
artifacts = ["Charm", "Relic"]
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
    // 150 ids minus one unused slot and one siege machine; the listed war machine is harvested.
    assert_eq!(r.creature_count(), 148);
    let machine = r.creature_by_key("base:unit_148").unwrap();
    assert!(
        r.creature(machine)
            .unwrap()
            .flags
            .contains(h3_content::CreatureFlag::WarMachine)
    );
    assert!(r.creature_by_key("base:unit_149").is_none());
    assert_eq!(r.resources().len(), 7);
    let ids: Vec<&str> = r.factions().iter().map(|f| f.key.as_str()).collect();
    assert_eq!(ids, ["base:alpha", "base:beta", "ab:gamma"]);
    // Terrains from TERRNAME.txt, native terrains and obstacles from the gap-fill file.
    let mud = r.terrain_by_key("base:mud").unwrap();
    let ice = r.terrain_by_key("base:ice").unwrap();
    let natives: Vec<_> = r.factions().iter().map(|f| f.native_terrain).collect();
    assert_eq!(natives, [Some(mud), None, Some(ice)]);
    assert_eq!(r.obstacles().len(), 2);
    assert_eq!(r.obstacles()[0].blocked, vec![(0, 0), (1, -1)]);
    assert!(r.obstacles()[1].absolute);
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
    assert!(report.contains("`Unit 149`): siege machine"));
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
    // Native terrains and obstacles: unknown town or terrain, a town listed twice.
    for (from, to, error) in [
        (
            "faction = \"Alpha\"",
            "faction = \"Delta\"",
            "unknown town `Delta`",
        ),
        (
            "terrain = \"Mud\"",
            "terrain = \"Sand\"",
            "unknown terrain `Sand`",
        ),
        (
            "faction = \"Gamma\"\nterrain",
            "faction = \"Alpha\"\nterrain",
            "native terrain of `Alpha` twice",
        ),
        (
            "terrains = [\"Mud\"], width",
            "terrains = [\"Sand\"], width",
            "unknown terrain `Sand`",
        ),
    ] {
        let g = GapFill::parse(&GAP.replace(from, to)).unwrap();
        assert!(
            harvest(&inputs(), &g).unwrap_err().contains(error),
            "{error}"
        );
    }
}

#[test]
fn harvests_hero_content() {
    let h = harvest(&inputs(), &gap()).unwrap();
    let (base, ab, sod) = (pack(&h, "base"), pack(&h, "ab"), pack(&h, "sod"));
    let r = h3_pack::compile(&[base, ab, sod], "base:faithful").unwrap();
    assert_eq!(r.skills().len(), 28);
    assert_eq!(r.spell_schools().len(), 4);
    assert_eq!(r.spells().len(), 3);
    assert_eq!(r.hero_classes().len(), 6);
    assert_eq!(r.hero_types().len(), 7);
    // Skill effects from the value tables and the gap-fill file.
    assert!(base.contains(
        "id = \"skill_1\"\nname = \"Skill 1\"\nbasic = [{ kind = \"ranged_damage\", bp = 1000 }]"
    ));
    assert!(base.contains("expert = [{ kind = \"first_aid\", heal_bp = 30000 }]"));
    assert!(base.contains("advanced = [{ kind = \"morale\", amount = 2 }]"));
    assert!(base.contains("expert = [{ kind = \"tactics\", level = 3 }]"));
    // Spells: schools, polarity, the SoD-only row.
    assert!(base.contains("schools = [\"fire\", \"air\"]"));
    assert!(sod.contains("id = \"bolt\"  # original id 2"));
    assert!(sod.contains("schools = [\"base:air\"]\ncost = [7, 8, 9, 10]\npower = 11"));
    assert!(sod.contains("polarity = \"positive\""));
    assert!(
        h.report
            .iter()
            .any(|l| l.contains("`Old (disabled)` in the AB-era table"))
    );
    // Classes in their faction's pack; heroes by the executable's flags.
    assert!(ab.contains("[[hero_class]]\nid = \"class_4\""));
    assert!(ab.contains("skill_weights = { \"base:skill_1\" = 1"));
    assert!(ab.contains("[[hero]]\nid = \"hero_3\"  # original id 3"));
    // Specialties, one per kind.
    for line in [
        "specialty = [{ kind = \"skill_specialty\", skill = \"skill_1\", bp_per_level = 500 }]",
        "war_machines = [\"unit_148\"]",
        "specialty = [{ kind = \"creature_specialty\", creature = \"unit_148\", bp_per_step = 500, speed = 0 }]",
        "spells = [\"zap\"]",
        "specialty = [{ kind = \"spell_specialty\", spell = \"zap\" }]",
        "{ kind = \"creature_stat\", stat = \"speed\", amount = 2 }",
        "{ kind = \"creature_stat\", creatures = { flag = \"dragon\" }, stat = \"defense\", amount = 5 }",
    ] {
        assert!(base.contains(line), "{line}");
    }
    assert!(ab.contains(
        "specialty = [{ kind = \"creature_stat\", creatures = { creature = \"unit_20\" }, stat = \"attack\", amount = 2 }, \
         { kind = \"creature_stat\", creatures = { creature = \"unit_20\" }, stat = \"defense\", amount = 1 }, \
         { kind = \"creature_stat\", creatures = { creature = \"unit_20\" }, stat = \"min_damage\", amount = 3 }, \
         { kind = \"creature_stat\", creatures = { creature = \"unit_20\" }, stat = \"max_damage\", amount = 3 }, \
         { kind = \"creature_stat\", creatures = { creature = \"unit_21\" }, stat = \"speed\", amount = 1 }]"
    ));
    assert!(
        h.report
            .iter()
            .any(|l| l == "specialties not modelled: 1 resource (adventure)")
    );
    // Upgrades: the pairing rule within a pack, a base creature's ab upgrade as an ab override.
    assert!(base.contains("upgrades = [\"unit_1\"]"));
    assert!(ab.contains("target = \"base:unit_26\"\nupgrades = [\"unit_30\"]"));
    let u26 = r.creature_by_key("base:unit_26").unwrap();
    assert_eq!(r.upgrades(u26), &[r.creature_by_key("ab:unit_30").unwrap()]);
    let u22 = r.creature_by_key("ab:unit_22").unwrap();
    assert_eq!(
        r.upgrades(u22),
        &[r.creature_by_key("base:unit_23").unwrap()]
    );
}

#[test]
fn hero_content_errors() {
    let g = gap();
    let fails = |change: &dyn Fn(&mut Inputs), error: &str| {
        let mut i = inputs();
        change(&mut i);
        let e = harvest(&i, &g).unwrap_err();
        assert!(e.contains(error), "{error}: {e}");
    };
    fails(
        &|i| i.hero_data.skill_values.pop().map(drop).unwrap(),
        "no value table for skill 27",
    );
    fails(
        &|i| i.hero_data.skill_values[0].1[0] = SkillValue::Float(0.0),
        "wrong value type",
    );
    fails(
        &|i| i.hero_data.skill_values[2].1[0] = SkillValue::Float(0.1),
        "a value without the skill",
    );
    fails(
        &|i| i.hero_data.skill_values[2].1[1] = SkillValue::Float(0.123_45),
        "basis points",
    );
    fails(
        &|i| i.hero_data.spell_types.pop().map(drop).unwrap(),
        "the executable has 2",
    );
    fails(&|i| i.hero_data.heroes[1].class = 9, "class 9 out of range");
    fails(
        &|i| i.hero_data.heroes[0].creatures[0] = 149,
        "creature id 149 is not harvested",
    );
    fails(
        &|i| i.hero_data.heroes[0].creatures[0] = -1,
        "army entry 0 is empty",
    );
    fails(
        &|i| {
            i.hero_data.heroes[0].roe = false;
            i.hero_data.heroes[0].expansions = false;
        },
        "neither RoE nor an expansion hero",
    );
    fails(
        &|i| i.hero_data.heroes[0].spell = 2,
        "`sod:bolt` is not available to `base`",
    );
    fails(
        &|i| i.hero_data.heroes[0].skills[0].0 = 40,
        "skill id 40 out of range",
    );
    fails(
        &|i| i.hero_data.specialties[1].id = 3,
        "specialty creature 3",
    );
    fails(
        &|i| i.hero_tables.heroes.records[2][7] = "2".into(),
        "war machine with count 2-1",
    );
    fails(
        &|i| i.hero_tables.heroes.records.truncate(5),
        "HOTRAITS.TXT: 3 rows, expected 7",
    );
    fails(
        &|i| i.hero_tables.skills.records[3].truncate(2),
        "has 2 fields",
    );
    fails(
        &|i| i.hero_tables.classes.records[2][3] = "x".into(),
        "is not a number",
    );
    fails(
        &|i| i.hero_tables.sod_spells.records[1].truncate(5),
        "school columns without names",
    );
    fails(
        &|i| i.hero_tables.sod_spells.records.truncate(1),
        "no header row",
    );
    fails(
        &|i| {
            let spells = &mut i.hero_tables.sod_spells.records;
            spells.remove(2);
        },
        "before a group row",
    );
    fails(
        &|i| i.hero_tables.sod_spells.records[3].truncate(9),
        "has 9 fields",
    );
    fails(
        &|i| {
            let ab = &mut i.hero_tables.ab_spells.records;
            ab[6][9] = "99".into();
        },
        "differs between H3ab_bmp and H3bitmap",
    );
    fails(
        &|i| i.hero_tables.sod_spells.records[6][0] = "Look".into(),
        "duplicate spell key `look`",
    );
    // Classes beyond the towns, and duplicate hero names.
    fails(
        &|i| {
            let extra = i.hero_tables.classes.records[2].clone();
            i.hero_tables.classes.records.push(extra);
        },
        "no town 3",
    );
    let mut i = inputs();
    i.hero_tables.heroes.records[3][0] = "Hero 0".into();
    let h = harvest(&i, &g).unwrap();
    assert!(pack(&h, "base").contains("id = \"hero_0_class_1\""));
    // Gap-fill entries that do not match the tables.
    for (from, to, error) in [
        (
            "[\"Unit 26\", \"Unit 30\"]",
            "[\"Unit 0\", \"Unit 1\"]",
            "is found by the rule",
        ),
        (
            "[\"Unit 26\", \"Unit 30\"]",
            "[\"Unit 26\", \"Nobody\"]",
            "`Nobody`, not a harvested creature",
        ),
        (
            "skill = \"Skill 19\"",
            "skill = \"Skill 99\"",
            "unknown skill `Skill 99`",
        ),
        (
            "skill = \"Skill 19\"",
            "skill = \"Skill 1\"",
            "has a value table",
        ),
        (
            "{ kind = \"tactics\", level = 1 }",
            "{ kind = \"tactics\", creature = \"Unit 1\", level = 1 }",
            "take no creature",
        ),
        (
            "hero = \"Hero 3\"",
            "hero = \"Hero 9\"",
            "unknown hero `Hero 9`",
        ),
        (
            "creature = \"Unit 21\"\nstat",
            "creature = \"Unit 999\"\nstat",
            "specialty_extra creature",
        ),
    ] {
        let g = GapFill::parse(&GAP.replace(from, to)).unwrap();
        let e = harvest(&inputs(), &g).unwrap_err();
        assert!(e.contains(error), "{error}: {e}");
    }
}

#[test]
fn harvests_artifacts() {
    let h = harvest(&inputs(), &gap()).unwrap();
    let (base, ab, sod) = (pack(&h, "base"), pack(&h, "ab"), pack(&h, "sod"));
    let r = h3_pack::compile(&[base, ab, sod], "base:faithful").unwrap();
    assert_eq!(r.artifact_slots().len(), 5);
    let misc = r.artifact_slot_by_key("base:misc").unwrap();
    assert_eq!(r.artifact_slot(misc).unwrap().count, 3);
    assert_eq!(r.artifacts().len(), 9);
    let flag = |k: &str| {
        r.artifact_slot(r.artifact_slot_by_key(k).unwrap())
            .unwrap()
            .backpack
    };
    assert!(flag("base:misc") && !flag("base:war_machine_1") && !flag("sod:spell_book"));
    assert_eq!(
        r.hero_params()
            .map(|p| (p.primary_min, p.primary_max, p.backpack)),
        Some(([0, 0, 1, 1], 99, 64))
    );
    for line in [
        "[[artifact_slot]]\nid = \"ring\"\nname = \"Ring\"\ncount = 2",
        "[[artifact_slot]]\nid = \"misc\"\nname = \"Misc\"\ncount = 2",
        "id = \"helm\"  # original id 0\nname = \"Helm\"\nclass = \"treasure\"\ncost = { \"gold\" = 1000 }\nslot = \"head\"\neffects = [{ kind = \"primary_skills\", attack = 1 }]",
        "slot = \"war_machine_1\"\nwar_machine = \"unit_148\"",
        "id = \"relic\"  # original id 5; pending: E04b\nname = \"Relic\"\nclass = \"special\"\n",
        "needs_skill = \"skill_1\"\neffects = [{ kind = \"ranged_damage\", bp = 500 }]",
    ] {
        assert!(base.contains(line), "{line}");
    }
    assert!(ab.contains("cost = { \"base:gold\" = 20000 }\nslot = \"base:misc\"\neffects = [{ kind = \"creature_stat\", creatures = { flag = \"dragon\" }, stat = \"attack\", amount = 5 }]"));
    for line in [
        "[[artifact_slot_override]]\ntarget = \"base:misc\"\ncount = 3",
        "[[artifact_slot]]\nid = \"spell_book\"\nname = \"Spell Book\"\ncount = 1\nbackpack = false\n",
        "effects = [{ kind = \"primary_skills\", power = 6, knowledge = 6 }, { kind = \"alignment_group\", factions = [\"base:alpha\", \"ab:gamma\"] }, \
         { kind = \"regeneration\", creatures = { creature = \"ab:unit_21\" }, hit_points = 5 }]\nparts = [\"base:band\", \"base:loop\", \"base:helm\"]",
    ] {
        assert!(sod.contains(line), "{line}");
    }
    assert!(!sod.contains("Trace"));
    let circle = r
        .artifact(r.artifact_by_key("sod:circle").unwrap())
        .unwrap();
    assert_eq!(circle.locks.len(), 2);
    assert!(
        h.report
            .iter()
            .any(|l| l == "slot `misc`: 2 positions before SoD, 3 in SoD")
    );
    assert!(
        h.report
            .iter()
            .any(|l| l == "artifacts per pack: ab 1, base 7, sod 1")
    );
}

#[test]
fn artifact_errors() {
    let fails_with =
        |change: &dyn Fn(&mut Inputs), gap_change: &dyn Fn(&str) -> String, error: &str| {
            let mut i = inputs();
            change(&mut i);
            let g = GapFill::parse(&gap_change(GAP)).unwrap();
            let e = harvest(&i, &g).unwrap_err();
            assert!(e.contains(error), "{error}: {e}");
        };
    let same = |g: &str| g.to_owned();
    let fails = |change: &dyn Fn(&mut Inputs), error: &str| fails_with(change, &same, error);
    let gap_fails = |from: &str, to: &str, error: &str| {
        assert!(GAP.contains(from), "{from}");
        fails_with(&|_| {}, &|g: &str| g.replacen(from, to, 1), error);
    };
    gap_fails(
        "no_backpack = [\"war_machine_1\", \"spell_book\"]",
        "no_backpack = [\"tail\"]",
        "no_backpack names unknown slot kind `tail`",
    );
    let row_of = |i: &mut Inputs, name: &str| {
        i.artifact_tables
            .sod
            .records
            .iter()
            .position(|r| r[0] == name)
            .unwrap()
    };
    fails(
        &|i| {
            i.artifact_tables.ab_slots.records[0][0] = "Tail".into();
            i.artifact_tables.sod_slots.records[0][0] = "Tail".into();
        },
        "unknown position `Tail`",
    );
    fails(
        &|i| i.artifact_tables.ab_slots.records.swap(0, 1),
        "not a prefix",
    );
    fails(
        &|i| i.artifact_tables.sod_slots.records.clear(),
        "no positions",
    );
    fails(
        &|i| i.hero_data.artifact_primary.pop().map(drop).unwrap(),
        "the executable has 9",
    );
    fails(
        &|i| {
            let r = row_of(i, "Helm");
            i.artifact_tables.sod.records[r][1] = "lots".into();
        },
        "has cost `lots`",
    );
    fails(
        &|i| {
            let r = row_of(i, "Helm");
            let c = i.artifact_tables.sod.records[r].len() - 2;
            i.artifact_tables.sod.records[r][c] = "Q".into();
        },
        "has class `Q`",
    );
    fails(
        &|i| {
            let r = row_of(i, "Helm");
            i.artifact_tables.sod.records[r][1] = "999".into();
        },
        "artifact id 0 `Helm` differs",
    );
    fails(
        &|i| {
            let r = row_of(i, "Band");
            for t in [&mut i.artifact_tables.sod, &mut i.artifact_tables.ab] {
                let h = t.records[1].clone();
                t.records[r] = artifact_row(&h, "Band", "500", &["Right Ring"], "N");
            }
        },
        "not the positions of one slot",
    );
    fails(
        &|i| i.artifact_tables.sod.records[1][2] = "Hat".into(),
        "no column",
    );
    fails(
        &|i| i.artifact_tables.sod.records.truncate(1),
        "no header row",
    );
    fails(
        &|i| {
            let r = row_of(i, "Loop");
            i.artifact_tables.sod.records[r][0] = "Band".into();
            i.artifact_tables.ab.records[r][0] = "Band".into();
        },
        "duplicate artifact key `band`",
    );
    gap_fails(
        "ab = [\"Vial\"]",
        "ab = [\"Circle\"]",
        "`Circle` is not in the AB-era table",
    );
    gap_fails(
        "ab = [\"Vial\"]",
        "ab = [\"Flask\"]",
        "unknown artifact `Flask`",
    );
    gap_fails(
        "creature = \"Unit 148\"\nsources",
        "creature = \"Unit 1\"\nsources",
        "`Unit 1` is not a war machine",
    );
    gap_fails(
        "creature = \"Unit 148\"\nsources",
        "creature = \"Unit 999\"\nsources",
        "`Unit 999` is not harvested",
    );
    gap_fails(
        "parts = [\"Band\", \"Loop\", \"Helm\"]",
        "parts = [\"Band\", \"Trace\"]",
        "`Trace` is not harvested",
    );
    gap_fails(
        "needs_skill = \"Skill 1\"",
        "needs_skill = \"Flying\"",
        "unknown skill `Flying`",
    );
    gap_fails(
        "factions = [\"Alpha\", \"Gamma\"]",
        "factions = [\"Alpha\", \"Delta\"]",
        "unknown town `Delta`",
    );
    gap_fails(
        "creatures = { creature = \"Unit 21\" }",
        "creatures = { creature = \"Unit 999\" }",
        "unknown creature `Unit 999`",
    );
    gap_fails(
        "creatures = { creature = \"Unit 21\" }",
        "creatures = { kind = \"x\" }",
        "malformed `creatures`",
    );
    gap_fails(
        "{ kind = \"luck\", amount = 1 }",
        "{ kind = \"luck\", amount = 1, creature = \"Unit 1\" }",
        "take no `creature`",
    );
    gap_fails(
        "artifacts = [\"Charm\", \"Relic\"]",
        "artifacts = [\"Charm\"]",
        "not pending: Relic",
    );
    gap_fails(
        "artifacts = [\"Loop\"]",
        "artifacts = [\"Band\"]",
        "effects of `Band` are listed twice",
    );
    gap_fails(
        "artifacts = [\"Loop\"]",
        "artifacts = [\"Loop\", \"Trace\"]",
        "`Trace` is not harvested",
    );
    gap_fails(
        "artifacts = [\"Charm\", \"Relic\"]",
        "artifacts = [\"Charm\", \"Relic\", \"Ghost\"]",
        "unknown artifact `Ghost`",
    );
    gap_fails(
        "[[artifacts.combinations]]\nartifact = \"Circle\"",
        "[[artifacts.combinations]]\nartifact = \"Circle\"\nparts = [\"Band\", \"Loop\"]\nsources = [\"s\"]\n\n[[artifacts.combinations]]\nartifact = \"Circle\"",
        "`Circle` is listed twice",
    );
    gap_fails(
        "[[artifacts.war_machines]]\nartifact = \"Kit\"",
        "[[artifacts.war_machines]]\nartifact = \"Kit\"\ncreature = \"Unit 148\"\nsources = [\"s\"]\n\n[[artifacts.war_machines]]\nartifact = \"Kit\"",
        "`Kit` has two war machines",
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
