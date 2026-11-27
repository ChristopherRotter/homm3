use super::*;
use crate::gapfill::DataQuirks;

/// A row of `len` cells, `cells` set at their columns.
fn row(len: usize, cells: &[(usize, &str)]) -> Vec<String> {
    let mut r = vec![String::new(); len];
    for &(c, v) in cells {
        r[c] = v.to_owned();
    }
    r
}

fn table(rows: Vec<Vec<String>>) -> TextTable {
    let mut records = vec![vec![String::from("header")]; HEADER_ROWS];
    records.extend(rows);
    TextTable { records }
}

/// A template row (also a zone and a connection row) and a zone row with a connection.
fn rmg(name: &str) -> TextTable {
    table(vec![
        row(
            CONNECTION_CELLS,
            &[
                (0, name),
                (1, "1"),
                (2, " 8"),
                (3, "1"),
                (4, "x"),
                (6, "x"),
                (8, "20"),
                (9, "1"),
                (10, "2"),
                (13, "1"),
                (23, "x"),
                (31, "x"),
                (47, " "),
                (54, "x"),
                (55, "Strong"),
                (67, "500"),
                (75, "9"),
                (76, "1"),
                (77, "2"),
                (78, "3000"),
                (79, "x"),
                (83, "-1"),
                (84, "+4"),
            ],
        ),
        vec![String::new()],
        row(
            CONNECTION_CELLS,
            &[
                (3, "2"),
                (7, "x"),
                (55, ""),
                (76, "2"),
                (77, " 1"),
                (80, "x"),
            ],
        ),
    ])
}

fn objects_table(lines: &[&str]) -> TextTable {
    let mut records = vec![vec![lines.len().to_string()]];
    records.extend(lines.iter().map(|l| vec![(*l).to_owned()]));
    TextTable { records }
}

fn object(picture: &str, editor: &str) -> String {
    format!(
        "{picture} {} {} 111111111 {editor} 5 1 2 0",
        "1".repeat(48),
        "0".repeat(48)
    )
}

/// The sprite archives' masks: `a.def`'s 2 × 1, its anchor and the cell left of it coloured;
/// `default.msk`, 1 × 1, its anchor.
fn masks() -> BTreeMap<String, Msk> {
    let msk = |width, height, last| Msk {
        width,
        height,
        colour: [0, 0, 0, 0, 0, last],
    };
    BTreeMap::from([
        ("a.msk".to_owned(), msk(2, 1, 0xC0)),
        ("default.msk".to_owned(), msk(1, 1, 0x80)),
    ])
}

/// The object settings of three classes: 0 not entered, 1 filed among class 0, 2 entered from
/// above and removable.
fn settings() -> Vec<h3_formats::exe::ObjectSettingsData> {
    let class =
        |class_group, not_entered, from_above, removable| h3_formats::exe::ObjectSettingsData {
            not_entered,
            from_above,
            removable,
            class_group,
        };
    vec![
        class(0, true, false, false),
        class(0, false, false, false),
        class(2, false, true, true),
    ]
}

fn rand_trn() -> TextTable {
    let mut first = row(
        18,
        &[
            (0, "a"),
            (3, "135"),
            (4, "0"),
            (6, "0"),
            (7, "10"),
            (15, "-5000"),
        ],
    );
    first[16] = "100".into();
    first[17] = "5".into();
    let mut second = row(20, &[(0, "b"), (3, "118"), (6, "1")]);
    second[18] = "30".into();
    second[19] = "-10".into();
    table(vec![
        first,
        second,
        row(3, &[(0, " stop")]),
        row(4, &[(0, "after")]),
    ])
}

fn files(rmg_text: TextTable, objects: &[&str]) -> Files {
    Files {
        rmg: rmg_text,
        objects: objects_table(objects),
        rand_trn: rand_trn(),
    }
}

fn gap(fixes: Vec<RmgZoneIdsFix>) -> GapFill {
    let mut gap = GapFill::parse(crate::gapfill::GAPFILL).unwrap();
    gap.data_quirks = DataQuirks {
        rmg_zone_ids: fixes,
        ..DataQuirks::default()
    };
    gap
}

fn harvest(tables: &Tables, fixes: Vec<RmgZoneIdsFix>) -> Result<(String, String), String> {
    let mut report = Vec::new();
    let packs = harvest_rmg(tables, &gap(fixes), &mut report)?;
    assert_eq!(report.len(), 2);
    let [("ab", ab), ("sod", sod)] = packs.as_slice() else {
        panic!("{packs:?}");
    };
    for text in [ab, sod] {
        toml::from_str::<toml::Value>(text).unwrap();
    }
    Ok((ab.clone(), sod.clone()))
}

// covers: RMG-TPL-001, RMG-TPL-002, RMG-TPL-006
#[test]
fn the_tables_are_kept_as_read() {
    let a = object("a.def", "000000001");
    let tables = Tables {
        ab: files(rmg("One"), &[&a]),
        sod: files(rmg("One"), &[&a]),
        masks: masks(),
        constants: Constants {
            heroes: vec![[true, true, false], [false, true, true]],
            map_limits: vec![(26, 200), (6, 200)],
            zone_limits: vec![(113, 3)],
            layout_cos: vec![10_000, -9807],
            layout_sin: vec![0, 1951],
            terrain_towns: vec![[0, 1, 4, -1], [-1, 0, 0, 0]],
            native_terrains: vec![2, 7],
            picture_sets: vec![h3_formats::exe::PictureSetData {
                through_dirt: false,
                land: true,
                share: 70,
                pictures: vec![(0, false), (0, false), (0, true)],
            }],
            rock_pictures: vec![(0, [false, false]), (8, [true, false]), (8, [true, false])],
            object_settings: settings(),
        },
    };
    let (ab, sod) = harvest(&tables, Vec::new()).unwrap();
    // The executable's constants, in SoD's pack only.
    assert!(!ab.contains("[generator]"));
    assert!(sod.contains(
        "[generator]\nheroes_roe = \"10\"\nheroes_expansions = \"11\"\nheroes_campaign = \
         \"01\"\nmap_limits = [[26, 200], [6, 200]]\nzone_limits = [[113, 3]]\nlayout_cos = \
         [10000, -9807]\nlayout_sin = [0, 1951]\nterrain_towns = [[0, 1, 4, -1], [-1, 0, 0, 0]]\n\
         native_terrains = [2, 7]\n"
    ));
    // The picture sets and rock's pictures as runs.
    assert!(sod.contains(
        "picture_sets = [\n  { through_dirt = false, land = true, share = 70, pictures = [[0, 0, \
         2], [0, 1, 1]] },\n]\nrock_pictures = [[0, 0, 0, 1], [8, 1, 0, 2]]\n"
    ));
    // The object settings: a mark per class, the groups that are not their class.
    assert!(sod.contains(
        "object_not_entered = \"100\"\nobject_from_above = \"001\"\nobject_removable = \"001\"\n\
         object_class_groups = [[1, 0]]\n"
    ));
    // Column 6 set after 4: the last set wins; a space is not set; strength by its first letter.
    assert!(ab.contains(
        "{ id = 1, kind = \"treasure\", base_size = 20, humans = [1, 2], players = [0, 0], \
         ownership = 1, player_towns = [0, 0, 0, 0], neutral_towns = [0, 0, 0, 0], same_type = \
         false, town_types = \"100000001\", mines = [0, 0, 0, 0, 0, 0, 0], mine_density = [0, 0, \
         0, 0, 0, 0, 0], terrain_match = false, terrains = \"00000001\", strength = \"strong\", \
         monster_match = false, neutral_monsters = false, monster_towns = \"000000000\", treasure \
         = [[500, 0, 0], [0, 0, 0], [0, 0, 9]] },"
    ));
    assert!(ab.contains("kind = \"junction\""), "{ab}");
    assert!(ab.contains("strength = \"average\""));
    assert!(ab.contains(
        "{ zones = [1, 2], value = 3000, wide = true, border_guard = false, humans = [0, 0], \
         players = [-1, 4] },"
    ));
    assert!(ab.contains("{ zones = [2, 1], value = 0, wide = false, border_guard = true"));
    assert!(ab.contains("[[template]]\nname = \"One\"\nsizes = [1, 8]\n"));
    // The third column first, then the fourth; the picture's .msk after the line's fields.
    assert!(ab.contains(&format!(
        "{{ picture = \"a.def\", passability = \"{}\", actions = \"{}\", editor_terrains = \
         \"111111111\", terrains = \"000000001\", class = 5, subclass = 1, group = 2, overlay = 0, \
         width = 2, height = 1, colour = \"11{}\" }},",
        "1".repeat(48),
        "0".repeat(48),
        "0".repeat(46)
    )));
    assert!(ab.contains(
        "[[obstacle]]\ntype = 135\nsubtype = 0\nterrain = 0\nmodifiers = [10, 0, 0, 0, 0, 0, 0, \
         0, -5000]\nadjacent = [100, 5]\noverlap = [0, 0]\n"
    ));
    assert!(ab.contains("type = 118\nsubtype = 0\nterrain = 1\n"));
    assert!(ab.contains("adjacent = [0, 0]\noverlap = [30, -10]\n"));
    assert!(!ab.contains("connection_override"));
    // SoD's tables equal AB's: its pack adds nothing.
    assert!(!sod.contains("[[template]]") && !sod.contains("[[obstacle]]"));
    assert!(sod.contains("object_templates = [\n]\nobject_template_overrides = [\n]\n"));
}

#[test]
fn sods_pack_holds_what_its_tables_add() {
    let (a, b, c) = (
        object("a.def", "000000001"),
        object("b.def", "000000001"),
        object("c.def", "000000001"),
    );
    let (b2, new, late) = (
        object("b.def", "111111111"),
        object("new.def", "000000001"),
        object("late.def", "000000001"),
    );
    let mut sod_rmg = rmg("One");
    sod_rmg
        .records
        .extend(rmg("Two").records.into_iter().skip(HEADER_ROWS));
    let mut sod_trn = rand_trn();
    sod_trn.records[3][7] = "11".into();
    let tables = Tables {
        ab: files(rmg("One"), &[&a, &b, &c]),
        sod: Files {
            rmg: sod_rmg,
            objects: objects_table(&[&new, &a, &b2, &c, &late]),
            rand_trn: sod_trn,
        },
        masks: masks(),
        ..Tables::default()
    };
    let (_, sod) = harvest(&tables, Vec::new()).unwrap();
    assert!(sod.contains("  { before = 0, picture = \"new.def\""));
    assert!(sod.contains("  { before = 3, picture = \"late.def\""));
    assert!(sod.contains("  { line = 1, picture = \"b.def\""));
    assert!(sod.contains("terrains = \"111111111\", class"));
    // A picture without its own .msk takes default.msk's.
    assert!(sod.contains(&format!(
        "width = 1, height = 1, colour = \"1{}\" }},\n]\nobject_template_overrides",
        "0".repeat(47)
    )));
    assert!(sod.contains("name = \"Two\"") && !sod.contains("name = \"One\""));
    assert!(sod.contains("modifiers = [11,"));
}

#[test]
fn decimal_first_zones_get_their_gated_overrides() {
    let mut text = rmg("8XM8");
    text.records[5][76] = "5.9".into();
    text.records[3][76] = "1.5".into();
    let a = object("a.def", "000000001");
    let tables = Tables {
        ab: files(text.clone(), &[&a]),
        sod: files(text, &[&a]),
        masks: masks(),
        ..Tables::default()
    };
    let fix = |first_zones: Vec<i32>, template: &str| RmgZoneIdsFix {
        flag: "quirk.content.rmg_8xm8_zone_ids".into(),
        quirk: "RMG-QUIRK-22".into(),
        template: template.into(),
        first_zones,
    };
    let (ab, _) = harvest(&tables, vec![fix(vec![3, 6], "8XM8")]).unwrap();
    assert!(ab.contains("{ zones = [1, 2], value = 3000"));
    assert!(ab.contains("{ zones = [5, 1], value = 0"));
    assert!(ab.contains(
        "template = 0\nconnection = 0\nfixes = \"quirk.content.rmg_8xm8_zone_ids\"\nzones = [3, 2]\n"
    ));
    assert!(ab.contains("template = 0\nconnection = 1\nfixes = \"quirk.content.rmg_8xm8_zone_ids\"\nzones = [6, 1]\n"));
    for (f, message) in [
        (fix(vec![3], "8XM8"), "has 2 decimal first zones, the fix 1"),
        (fix(vec![3, 6], "9XM9"), "is not in the table once"),
    ] {
        let e = harvest(&tables, vec![f]).unwrap_err();
        assert!(e.contains(message), "{e}");
    }
    let mut twice = tables.clone();
    let again: Vec<_> = twice.ab.rmg.records[HEADER_ROWS..].to_vec();
    twice.ab.rmg.records.extend(again);
    let e = harvest(&twice, vec![fix(vec![3, 6], "8XM8")]).unwrap_err();
    assert!(
        e.contains("not in the table once") || e.contains("AB's templates"),
        "{e}"
    );
}

// covers: RMG-TPL-001, RMG-TPL-006
#[test]
fn tables_that_do_not_read_are_refused() {
    let a = object("a.def", "000000001");
    let ok = || Tables {
        ab: files(rmg("One"), &[&a]),
        sod: files(rmg("One"), &[&a]),
        masks: masks(),
        ..Tables::default()
    };
    type Change = Box<dyn Fn(&mut Tables)>;
    let cases: Vec<(Change, &str)> = vec![
        (
            Box::new(|t| t.ab.rmg.records[3][0] = String::new()),
            "before any template",
        ),
        (
            Box::new(|t| t.ab.rmg.records[5][77] = "1.5".into()),
            "a decimal second zone id",
        ),
        (
            Box::new(|t| t.ab.rmg.records[3][8] = "99999999999".into()),
            "outside i32",
        ),
        (
            Box::new(|t| t.ab.rmg.records[3][1] = "-2147483649".into()),
            "outside i32",
        ),
        (
            Box::new(|t| t.sod.rmg.records[3][0] = "Other".into()),
            "does not begin with AB's templates",
        ),
        (
            Box::new(|t| t.ab.objects.records[0][0] = "two".into()),
            "no count line",
        ),
        (
            Box::new(|t| t.ab.objects.records[0][0] = "2".into()),
            "2 lines expected",
        ),
        (
            Box::new(|t| t.ab.objects.records[1][0].push_str(" 9")),
            "Objects.txt line 2",
        ),
        (
            Box::new(|t| {
                t.ab.objects.records[1][0] = t.ab.objects.records[1][0].replace(" 5 ", " x ")
            }),
            "Objects.txt line 2: invalid digit",
        ),
        (
            Box::new(|t| t.sod.objects = objects_table(&[&object("z.def", "000000001")])),
            "drops AB's line 2",
        ),
        (
            Box::new(|t| t.ab.rand_trn.records[3][16] = "9999999999".into()),
            "Rand_trn.txt row 4",
        ),
        (
            Box::new(|t| t.masks = BTreeMap::new()),
            "no a.msk and no default.msk",
        ),
    ];
    for (change, message) in cases {
        let mut tables = ok();
        change(&mut tables);
        let e = harvest(&tables, Vec::new()).unwrap_err();
        assert!(e.contains(message), "{message}: {e}");
    }
    assert_eq!(atoi(&[String::from("-12x")], 0), Ok(-12));
    assert_eq!(atoi(&[String::from("\t7")], 0), Ok(7));
    assert_eq!(atoi(&[String::from("-2147483648")], 0), Ok(i32::MIN));
    assert_eq!(atoi(&[], 0), Ok(0));
}

/// `msk_load`'s name, bytes and marks; the report's counts of own and default masks and of lines
/// blocking a cell without a body pixel.
#[test]
fn the_masks_are_the_pictures_msk() {
    assert_eq!(msk_name("AVWmon1.def"), "avwmon1.msk");
    assert_eq!(msk_name("a.b.DEF"), "a.b.msk");
    assert_eq!(msk_name("plain"), "plain.msk");
    let file = [3, 2, 1, 0, 0, 0, 0, 0x80, 9, 9, 9, 9, 9, 9];
    let msk = Msk::parse(&file).unwrap();
    assert_eq!((msk.width, msk.height), (3, 2));
    // Mark 0 is the game's bit 47 (byte 5, bit 7), mark 47 its bit 0 (byte 0, bit 0).
    assert_eq!(msk.marks(), format!("1{}1", "0".repeat(46)));
    assert!(Msk::parse(&file[..13]).unwrap_err().contains("13 bytes"));
    assert!(Msk::parse(&file[..3]).is_err());
    // d.def (default.msk, its anchor alone) blocks its anchor and the cell left of it.
    let d = format!(
        "d.def 00{} {} 111111111 000000001 7 0 0 0",
        "1".repeat(46),
        "0".repeat(48)
    );
    let a = object("a.def", "000000001");
    let tables = Tables {
        ab: files(rmg("One"), &[&a, &d]),
        sod: files(rmg("One"), &[&a, &d]),
        masks: masks(),
        ..Tables::default()
    };
    let mut report = Vec::new();
    harvest_rmg(&tables, &gap(Vec::new()), &mut report).unwrap();
    assert_eq!(
        report[1],
        "rmg: ab's object templates 1 with their own .msk, 1 with default.msk, 1 blocking a cell \
         without a body pixel (1 of class 7); sod's 1 with their own .msk, 1 with default.msk, 1 \
         blocking a cell without a body pixel (1 of class 7)"
    );
    let tables = Tables {
        ab: files(rmg("One"), &[&a]),
        sod: files(rmg("One"), &[&a]),
        ..tables
    };
    report.clear();
    harvest_rmg(&tables, &gap(Vec::new()), &mut report).unwrap();
    assert_eq!(
        report[1],
        "rmg: ab's object templates 1 with their own .msk, 0 with default.msk, 0 blocking a cell \
         without a body pixel; sod's 1 with their own .msk, 0 with default.msk, 0 blocking a cell \
         without a body pixel"
    );
}
