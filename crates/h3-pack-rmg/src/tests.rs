use h3_content::{PackManifest, Ruleset};
use h3_pack::{SYNTHETIC_CORE, SYNTHETIC_EXT_CORE};
use h3_rmg::{GeneratorTables, RmgCompileError, RmgPack, RmgRuleset, Strength, ZoneKind};

use super::*;

const PROFILE: &str = "synthetic:faithful";

/// A core pack that depends on nothing, with a profile turning 8XM8's flag on and one leaving it
/// off; it loads before `synthetic` (ties by id) and is not of its closure.
const LONE_CORE: &str = r#"
[pack]
id = "lone"
version = "0.1.0"
depends = []

[[profile]]
id = "on"
quirks = { "quirk.content.rmg_8xm8_zone_ids" = true, "quirk.rmg.roe_monster_town_byte" = true, "quirk.rmg.unowned_start_seventh" = true, "quirk.rmg.zone_memory_model" = true, "quirk.rmg.terrain_towns_count" = true, "quirk.rmg.partition_units" = true, "quirk.rmg.cosmetic" = true }

[[profile]]
id = "off"
quirks = { "quirk.content.rmg_8xm8_zone_ids" = false, "quirk.rmg.roe_monster_town_byte" = false, "quirk.rmg.unowned_start_seventh" = false, "quirk.rmg.zone_memory_model" = false, "quirk.rmg.terrain_towns_count" = false, "quirk.rmg.partition_units" = false, "quirk.rmg.cosmetic" = false }
"#;

/// An `objects.txt` line of `picture` with `extra` fields first (`before = 1, `).
fn line(extra: &str, picture: &str) -> String {
    format!(
        "{{ {extra}picture = \"{picture}\", passability = \"{p}\", actions = \"{a}\", terrains = \
         \"000000001\", editor_terrains = \"100000000\", class = 5, subclass = 1, group = 2, \
         overlay = 0 }}",
        p = "1".repeat(48),
        a = "0".repeat(48),
    )
}

/// A template with one zone and the connections `connections` (inline tables).
fn template(name: &str, connections: &[&str]) -> String {
    format!(
        "[[template]]\nname = \"{name}\"\nsizes = [1, 32]\nzones = [{{ id = 1, kind = \
         \"treasure\", base_size = 5, humans = [0, 8], players = [0, 8], ownership = 0, \
         player_towns = [0, 0, 0, 0], neutral_towns = [0, 0, 0, 0], same_type = false, \
         town_types = \"000000000\", mines = [0, 0, 0, 0, 0, 0, 0], mine_density = [0, 0, 0, 0, \
         0, 0, 0], terrain_match = false, terrains = \"00000000\", strength = \"average\", \
         monster_match = false, neutral_monsters = false, monster_towns = \"000000000\", treasure \
         = [[0, 0, 0], [0, 0, 0], [0, 0, 0]] }}]\nconnections = [{}]\n",
        connections.join(", ")
    )
}

fn connection(zones: [i32; 2]) -> String {
    format!(
        "{{ zones = [{}, {}], value = 0, wide = false, border_guard = false, humans = [0, 0], \
         players = [0, 0] }}",
        zones[0], zones[1]
    )
}

fn core(packs: &[&str], profile: &str) -> Ruleset {
    let packs: Vec<_> = packs
        .iter()
        .map(|text| h3_pack::parse_pack(text).unwrap())
        .collect();
    Ruleset::compile(&packs, profile).unwrap()
}

fn manifest(text: &str) -> PackManifest {
    h3_pack::parse_pack(text).unwrap().manifest
}

fn compile_err(packs: &[(&str, &str)], profile: &str) -> RmgCompileError {
    match compile(packs, profile) {
        Err(RmgPackError::Compile(e)) => e,
        other => panic!("expected a compile error, got {other:?}"),
    }
}

fn pictures(part: &RmgRuleset) -> Vec<&str> {
    part.object_templates()
        .iter()
        .map(|o| o.picture.as_str())
        .collect()
}

#[test]
fn the_synthetic_tables_compile_as_written() {
    let part = compile(&[(SYNTHETIC_CORE, SYNTHETIC_RMG)], PROFILE).unwrap();
    let core = core(&[SYNTHETIC_CORE], PROFILE);
    assert_eq!(part.core_hash(), core.content_hash());
    let [vales, ring] = part.templates() else {
        panic!("two templates");
    };
    assert_eq!(vales.name, "Twin Vales");
    assert_eq!(ring.name, "Ring\nof Three");
    assert_eq!((vales.sizes, ring.sizes), ([1, 8], [2, 32]));
    assert_eq!(
        vales.zones.iter().map(|z| z.kind).collect::<Vec<_>>(),
        [
            ZoneKind::HumanStart,
            ZoneKind::ComputerStart,
            ZoneKind::Treasure
        ]
    );
    assert_eq!(ring.zones[1].kind, ZoneKind::Junction);
    assert_eq!(
        [
            ZoneKind::HumanStart,
            ZoneKind::ComputerStart,
            ZoneKind::Treasure,
            ZoneKind::Junction
        ]
        .map(ZoneKind::value),
        [0, 1, 2, 3]
    );
    assert_eq!(
        [
            &vales.zones[0],
            &vales.zones[1],
            &vales.zones[2],
            &ring.zones[0]
        ]
        .map(|z| z.strength),
        [
            Strength::Average,
            Strength::Weak,
            Strength::Strong,
            Strength::None
        ]
    );
    assert_eq!(
        [
            Strength::None,
            Strength::Weak,
            Strength::Average,
            Strength::Strong
        ]
        .map(Strength::value),
        [0, 2, 3, 4]
    );
    let treasure = &vales.zones[2];
    assert_eq!(treasure.town_types, 1 << 4);
    assert_eq!(treasure.terrains, 0b110);
    assert_eq!(treasure.neutral_towns, [0, 1, 0, 2]);
    assert_eq!(treasure.mine_density, [0, 1, 0, 1, 1, 1, 2]);
    assert_eq!(treasure.treasure[2], [15000, 25000, 1]);
    assert!(treasure.same_type && treasure.monster_match && !treasure.neutral_monsters);
    assert_eq!(ring.zones[0].town_types, 0xFF);
    assert_eq!(ring.zones[1].monster_towns, 0x1FF);
    assert_eq!(ring.zones[1].terrains, 0x80);
    assert_eq!(vales.connections.len(), 3);
    assert!(vales.connections[1].wide && vales.connections[2].border_guard);
    assert_eq!(vales.connections[2].humans, [2, 2]);
    assert_eq!(ring.connections[1].zones, [1, 1]);
    assert_eq!(
        pictures(&part),
        ["synth_blank.def", "synth_mine.def", "synth_tree.def"]
    );
    let mine = &part.object_templates()[1];
    assert_eq!(mine.passability, (1u64 << 48) - 2);
    assert_eq!(
        (mine.actions, mine.terrains, mine.editor_terrains),
        (1, 0x1FF, 1 << 8)
    );
    assert_eq!((mine.class, mine.subclass, mine.group), (53, 2, 2));
    assert_eq!(part.object_templates()[2].overlay, 1);
    let [tree, crater] = part.obstacles() else {
        panic!("two obstacles");
    };
    assert_eq!((tree.object_type, tree.subtype, tree.terrain), (135, 0, 0));
    assert_eq!(tree.modifiers[1], -5000);
    assert_eq!(
        (crater.adjacent.as_slice(), crater.overlap.as_slice()),
        (&[5, 5][..], &[-5000, 30][..])
    );
    assert_eq!(
        part.generator(),
        Some(&GeneratorTables {
            heroes_roe: vec![true, true, false, true],
            heroes_expansions: vec![true, false, true, true],
            heroes_campaign: vec![false, false, false, true],
            map_limits: vec![(26, 200), (8, 64)],
            zone_limits: vec![(113, 3)],
            layout_cos: COS.to_vec(),
            layout_sin: (0..32).map(|k| COS[(k + 24) % 32]).collect(),
            terrain_towns: vec![
                [1, 4, -1, 0],
                [6, -1, 0, 0],
                [0, 1, 2, -1],
                [2, -1, 0, 0],
                [-1, 7, 4, 0],
                [6, 8, -1, 0],
                [5, 3, 4, -1],
                [3, -1, 0, 0],
                [-1, 0, 0, 0],
            ],
            native_terrains: vec![2, 2, 3, 7, 0, 6, 5, 4, 2],
            picture_sets: synthetic_picture_sets(),
            rock_pictures: synthetic_rock_pictures(),
        })
    );
}

/// The synthetic pack's picture sets, its runs expanded.
fn synthetic_picture_sets() -> Vec<PictureSet> {
    let expand = |runs: &[(u8, bool, usize)]| -> Vec<(u8, bool)> {
        runs.iter()
            .flat_map(|&(shape, flag, n)| std::iter::repeat_n((shape, flag), n))
            .collect()
    };
    let plain = |shapes: &[u8]| shapes.iter().map(|&s| (s, false, 1)).collect::<Vec<_>>();
    let set = |through_dirt, land, share, runs: Vec<(u8, bool, usize)>| PictureSet {
        through_dirt,
        land,
        share,
        pictures: expand(&runs),
    };
    let land = |share| {
        let mut runs = vec![(0, false, 2), (0, true, 2)];
        runs.extend(plain(&(2..=28).collect::<Vec<_>>()));
        set(true, true, share, runs)
    };
    let mut dirt = vec![(0, false, 2), (0, true, 1)];
    dirt.extend(plain(&[8, 9, 10, 11, 12, 13, 16, 24]));
    let mut water = vec![(0, false, 3)];
    water.extend(plain(&[8, 9, 10, 11, 12, 13, 16]));
    vec![
        set(true, true, 40, dirt),
        set(false, true, 60, vec![(0, false, 1), (0, true, 2)]),
        land(40),
        land(70),
        land(70),
        land(70),
        land(50),
        land(70),
        set(false, false, 0, water),
    ]
}

/// The synthetic pack's rock pictures.
fn synthetic_rock_pictures() -> Vec<RockPicture> {
    let mut rock = vec![(0, [false, false]); 2];
    for shape in 8..=13 {
        for a in [false, true] {
            for b in [false, true] {
                rock.push((shape, [a, b]));
            }
        }
    }
    rock
}

/// Step 1's cosines in ten-thousandths (the original's, the synthetic pack's too).
const COS: [i32; 32] = [
    10000, 9807, 9239, 8315, 7071, 5556, 3827, 1951, 0, -1951, -3827, -5556, -7071, -8315, -9239,
    -9807, -10000, -9807, -9239, -8315, -7071, -5556, -3827, -1951, 0, 1951, 3827, 5556, 7071,
    8315, 9239, 9807,
];

/// The generator's tables: one hero byte each per hero, limits of the 232 object types, 32
/// layout points, nine terrains' town types (-1 to 8) and a native terrain (0 to 8) per town
/// type; a later pack's replace an earlier's.
#[test]
fn the_generators_tables_are_checked_and_replaced() {
    for (from, to) in [
        ("heroes_roe = \"1101\"", "heroes_roe = \"110\""),
        ("heroes_campaign = \"0001\"", "heroes_campaign = \"00010\""),
        ("heroes_expansions = \"1011\"", "heroes_expansions = \"\""),
        ("[8, 64]", "[232, 64]"),
        ("[113, 3]", "[255, 3]"),
        ("layout_cos = [10000, ", "layout_cos = ["),
        ("9239, 9807]\nlayout_sin", "9239]\nlayout_sin"),
        ("terrain_towns = [[1, 4, -1, 0], ", "terrain_towns = ["),
        (
            "[3, -1, 0, 0], [-1, 0, 0, 0]]",
            "[3, -1, 0, 0], [-2, 0, 0, 0]]",
        ),
        ("[1, 4, -1, 0], [6", "[1, 9, -1, 0], [6"),
        ("native_terrains = [2, 2, ", "native_terrains = [2, "),
        ("native_terrains = [2, 2, ", "native_terrains = [9, 2, "),
        ("native_terrains = [2, 2, ", "native_terrains = [-1, 2, "),
        // Eight picture sets, a share past 100, a shape past 28, rock's too, too many of rock's.
        (
            "  { through_dirt = false, land = true, share = 60, pictures = [[0, 0, 1], [0, 1, 2]] },\n",
            "",
        ),
        ("share = 60", "share = 101"),
        (
            "[24, 0, 1]] },\n  { through_dirt = false",
            "[29, 0, 1]] },\n  { through_dirt = false",
        ),
        (
            "rock_pictures = [[0, 0, 0, 2]",
            "rock_pictures = [[29, 0, 0, 2]",
        ),
    ] {
        let text = SYNTHETIC_RMG.replacen(from, to, 1);
        assert_eq!(
            compile_err(&[(SYNTHETIC_CORE, &text)], PROFILE),
            RmgCompileError::GeneratorTables("synthetic".into()),
            "{to}"
        );
    }
    let ext = format!(
        "[generator]\nheroes_roe = \"1\"\nheroes_expansions = \"0\"\nheroes_campaign = \"0\"\n\
         map_limits = []\nzone_limits = [[0, 1]]\nlayout_cos = {COS:?}\nlayout_sin = {COS:?}\n\
         terrain_towns = {:?}\nnative_terrains = {:?}\npicture_sets = [{}]\n\
         rock_pictures = []\n",
        [[8, -1, 0, 0]; 9],
        [8; 9],
        "{ through_dirt = false, land = false, share = 0, pictures = [] }, ".repeat(9)
    );
    let part = compile(
        &[(SYNTHETIC_CORE, SYNTHETIC_RMG), (SYNTHETIC_EXT_CORE, &ext)],
        PROFILE,
    )
    .unwrap();
    assert_eq!(part.generator().unwrap().zone_limits, [(0, 1)]);
    assert_eq!(part.generator().unwrap().native_terrains, [8; 9]);
    assert!(part.generator().unwrap().rock_pictures.is_empty());
    assert!(
        compile(&[(SYNTHETIC_CORE, "")], PROFILE)
            .unwrap()
            .generator()
            .is_none()
    );
}

#[test]
fn packs_without_tables_compile_to_the_empty_part() {
    let part = compile(&[(SYNTHETIC_CORE, ""), (SYNTHETIC_EXT_CORE, "")], PROFILE).unwrap();
    let empty = RmgRuleset::empty(&core(&[SYNTHETIC_CORE, SYNTHETIC_EXT_CORE], PROFILE));
    assert_eq!(part, empty);
    assert!(part.templates().is_empty() && part.obstacles().is_empty());
    assert!(part.object_templates().is_empty());
    let manifest = manifest(SYNTHETIC_CORE);
    assert_eq!(parse_rmg(&manifest, "").unwrap(), RmgPack::empty(manifest));
}

#[test]
fn the_hash_covers_every_table() {
    let hash = |rmg: &str| {
        compile(&[(SYNTHETIC_CORE, rmg)], PROFILE)
            .unwrap()
            .content_hash()
    };
    let base = hash(SYNTHETIC_RMG);
    assert_eq!(base, hash(SYNTHETIC_RMG));
    for (from, to) in [
        ("base_size = 10", "base_size = 11"),
        ("value = 5000", "value = 5001"),
        ("class = 53", "class = 54"),
        ("overlap = [5, 0]", "overlap = [5, 1]"),
        ("name = \"Twin Vales\"", "name = \"Twin Vale\""),
        ("strength = \"weak\"", "strength = \"none\""),
        ("heroes_campaign = \"0001\"", "heroes_campaign = \"0011\""),
        ("zone_limits = [[113, 3]]", "zone_limits = [[113, 4]]"),
        (
            "map_limits = [[26, 200], [8, 64]]",
            "map_limits = [[8, 64], [26, 200]]",
        ),
        ("layout_cos = [10000, 9807", "layout_cos = [10000, 9808"),
        ("layout_sin = [0, 1951", "layout_sin = [0, 1952"),
        ("terrain_towns = [[1, 4", "terrain_towns = [[1, 5"),
        ("native_terrains = [2, 2", "native_terrains = [2, 3"),
        (
            "through_dirt = true, land = true, share = 40",
            "through_dirt = false, land = true, share = 40",
        ),
        ("land = true, share = 60", "land = false, share = 60"),
        ("share = 60", "share = 61"),
        (
            "pictures = [[0, 0, 1], [0, 1, 2]]",
            "pictures = [[0, 0, 1], [0, 1, 1]]",
        ),
        (
            "pictures = [[0, 0, 1], [0, 1, 2]]",
            "pictures = [[0, 0, 1], [1, 1, 2]]",
        ),
        (
            "pictures = [[0, 0, 1], [0, 1, 2]]",
            "pictures = [[0, 0, 1], [0, 0, 2]]",
        ),
        (
            "rock_pictures = [[0, 0, 0, 2]",
            "rock_pictures = [[0, 0, 0, 3]",
        ),
        (
            "rock_pictures = [[0, 0, 0, 2]",
            "rock_pictures = [[1, 0, 0, 2]",
        ),
        (
            "rock_pictures = [[0, 0, 0, 2]",
            "rock_pictures = [[0, 1, 0, 2]",
        ),
        (
            "rock_pictures = [[0, 0, 0, 2]",
            "rock_pictures = [[0, 0, 1, 2]",
        ),
    ] {
        assert!(SYNTHETIC_RMG.contains(from), "{from}");
        assert_ne!(base, hash(&SYNTHETIC_RMG.replacen(from, to, 1)), "{from}");
    }
    assert_ne!(base, hash(""));
}

#[test]
fn a_later_pack_inserts_replaces_appends_and_overrides() {
    let ext = format!(
        "object_templates = [{}, {}, {}, {}]\nobject_template_overrides = [{}]\n{}\n\
         [[connection_override]]\ntemplate = 0\nconnection = 2\nzones = [2, 1]\n\n\
         [[connection_override]]\ntemplate = 2\nconnection = 0\nzones = [1, 7]\n\n\
         [[obstacle]]\ntype = 1\nsubtype = 2\nterrain = 3\nmodifiers = [0, 0, 0, 0, 0, 0, 0, 0, \
         0]\nadjacent = [9]\noverlap = [8]\n",
        line("before = 1, ", "ext_a.def"),
        line("before = 0, ", "ext_b.def"),
        line("", "ext_c.def"),
        line("before = 3, ", "ext_d.def"),
        line("line = 2, ", "ext_tree.def"),
        template("Ext", &[&connection([1, 1])]),
    );
    let part = compile(
        &[(SYNTHETIC_CORE, SYNTHETIC_RMG), (SYNTHETIC_EXT_CORE, &ext)],
        PROFILE,
    )
    .unwrap();
    assert_eq!(
        pictures(&part),
        [
            "ext_b.def",
            "synth_blank.def",
            "ext_a.def",
            "synth_mine.def",
            "ext_tree.def",
            "ext_c.def",
            "ext_d.def"
        ]
    );
    let names: Vec<_> = part.templates().iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["Twin Vales", "Ring\nof Three", "Ext"]);
    assert_eq!(part.templates()[0].connections[2].zones, [2, 1]);
    assert_eq!(part.templates()[2].connections[0].zones, [1, 7]);
    assert_eq!(part.obstacles().len(), 1);
    assert_eq!(part.obstacles()[0].adjacent, [9]);
}

#[test]
fn a_gated_override_applies_only_with_its_flag_off() {
    let rmg = format!(
        "{}\n[[connection_override]]\ntemplate = 0\nconnection = 1\nfixes = \
         \"quirk.content.rmg_8xm8_zone_ids\"\nzones = [8, 9]\n",
        template("Gated", &[&connection([1, 2]), &connection([5, 6])])
    );
    let zones = |profile| {
        compile(&[(LONE_CORE, &rmg)], profile).unwrap().templates()[0]
            .connections
            .iter()
            .map(|c| c.zones)
            .collect::<Vec<_>>()
    };
    assert_eq!(zones("lone:on"), [[1, 2], [5, 6]]);
    assert_eq!(zones("lone:off"), [[1, 2], [8, 9]]);
    let on = compile(&[(LONE_CORE, &rmg)], "lone:on").unwrap();
    let off = compile(&[(LONE_CORE, &rmg)], "lone:off").unwrap();
    assert_ne!(on.content_hash(), off.content_hash());
}

#[test]
fn malformed_files_are_refused() {
    let manifest = manifest(SYNTHETIC_CORE);
    let parse = |text: &str| parse_rmg(&manifest, text);
    assert!(matches!(parse("bogus = 1"), Err(RmgPackError::Syntax(_))));
    assert!(matches!(
        parse(&SYNTHETIC_RMG.replacen("kind = \"treasure\"", "kind = \"castle\"", 1)),
        Err(RmgPackError::Syntax(_))
    ));
    for (from, to) in [
        ("terrains = \"01100000\"", "terrains = \"0110000\""),
        ("town_types = \"000010000\"", "town_types = \"00001000x\""),
        (
            "monster_towns = \"111111111\"",
            "monster_towns = \"1111111111\"",
        ),
        (
            "editor_terrains = \"000000001\"",
            "editor_terrains = \"00000001\"",
        ),
        ("terrains = \"100000000\"", "terrains = \"10000000\""),
        ("heroes_roe = \"1101\"", "heroes_roe = \"11x1\""),
        // A flag or a variant of 2, more than 127 pictures in a set or in rock's.
        (
            "pictures = [[0, 0, 1], [0, 1, 2]]",
            "pictures = [[0, 0, 1], [0, 2, 2]]",
        ),
        (
            "rock_pictures = [[0, 0, 0, 2]",
            "rock_pictures = [[0, 2, 0, 2]",
        ),
        (
            "rock_pictures = [[0, 0, 0, 2]",
            "rock_pictures = [[0, 0, 2, 2]",
        ),
        (
            "pictures = [[0, 0, 1], [0, 1, 2]]",
            "pictures = [[0, 0, 100], [0, 1, 28]]",
        ),
        (
            "rock_pictures = [[0, 0, 0, 2]",
            "rock_pictures = [[0, 0, 0, 127]",
        ),
        (
            "actions = \"100000000000000000000000000000000000000000000000\"",
            "actions = \"10000000000000000000000000000000000000000000000\"",
        ),
    ] {
        assert!(SYNTHETIC_RMG.contains(from), "{from}");
        assert!(
            matches!(
                parse(&SYNTHETIC_RMG.replacen(from, to, 1)),
                Err(RmgPackError::Invalid(_))
            ),
            "{to}"
        );
    }
    for text in [
        format!("object_templates = [{}]", line("line = 0, ", "a.def")),
        format!("object_template_overrides = [{}]", line("", "a.def")),
        format!(
            "object_template_overrides = [{}]",
            line("line = 0, before = 0, ", "a.def")
        ),
        format!(
            "object_template_overrides = [{}]",
            line("line = 0, ", "a.def").replace("\"000000001\"", "\"00000000x\"")
        ),
    ] {
        assert!(
            matches!(parse(&text), Err(RmgPackError::Invalid(_))),
            "{text}"
        );
    }
    assert!(matches!(
        compile(&[("[pack]\nid = \"Bad Id\"", "")], PROFILE),
        Err(RmgPackError::Core(_))
    ));
}

#[test]
fn the_packs_must_be_the_cores() {
    let core = core(&[SYNTHETIC_CORE, SYNTHETIC_EXT_CORE], PROFILE);
    let base = RmgPack::empty(manifest(SYNTHETIC_CORE));
    let ext = RmgPack::empty(manifest(SYNTHETIC_EXT_CORE));
    let mut stranger = base.clone();
    stranger.manifest.version = String::from("9.9.9");
    let mut orphan = ext.clone();
    orphan.manifest.depends = vec![String::from("elsewhere")];
    for (packs, error) in [
        (
            vec![base.clone(), stranger],
            RmgCompileError::UnknownPack(String::from("synthetic@9.9.9")),
        ),
        (
            vec![base.clone(), base.clone()],
            RmgCompileError::DuplicatePack(String::from("synthetic")),
        ),
        (
            vec![base.clone()],
            RmgCompileError::MissingPack(String::from("synthetic-ext")),
        ),
        (
            vec![base.clone(), orphan],
            RmgCompileError::UnknownDependency {
                pack: String::from("synthetic-ext"),
                dependency: String::from("elsewhere"),
            },
        ),
    ] {
        assert_eq!(RmgRuleset::compile(&core, &packs), Err(error));
    }
    assert!(RmgRuleset::compile(&core, &[ext, base]).is_ok());
}

#[test]
fn references_must_be_of_the_table_and_the_closure() {
    let two = |ext: &str| {
        compile_err(
            &[(SYNTHETIC_CORE, SYNTHETIC_RMG), (SYNTHETIC_EXT_CORE, ext)],
            PROFILE,
        )
    };
    let pack = || String::from("synthetic-ext");
    assert_eq!(
        two(&format!(
            "object_template_overrides = [{}]",
            line("line = 3, ", "a.def")
        )),
        RmgCompileError::ObjectLine {
            pack: pack(),
            line: 3
        }
    );
    assert_eq!(
        two(&format!(
            "object_templates = [{}]",
            line("before = 4, ", "a.def")
        )),
        RmgCompileError::ObjectLine {
            pack: pack(),
            line: 4
        }
    );
    assert_eq!(
        two(&format!(
            "object_template_overrides = [{}, {}]",
            line("line = 1, ", "a.def"),
            line("line = 1, ", "b.def")
        )),
        RmgCompileError::DuplicateObjectOverride {
            pack: pack(),
            line: 1
        }
    );
    assert_eq!(
        two("[[connection_override]]\ntemplate = 2\nconnection = 0\nzones = [1, 1]\n"),
        RmgCompileError::Template {
            pack: pack(),
            template: 2
        }
    );
    assert_eq!(
        two("[[connection_override]]\ntemplate = 1\nconnection = 2\nzones = [1, 1]\n"),
        RmgCompileError::Connection {
            pack: pack(),
            template: 1,
            connection: 2
        }
    );
    for flag in ["quirk.battle.no_luck_without_hero", "quirk.content.unknown"] {
        assert_eq!(
            two(&format!(
                "[[connection_override]]\ntemplate = 1\nconnection = 0\nfixes = \"{flag}\"\nzones \
                 = [1, 1]\n"
            )),
            RmgCompileError::NotADataQuirk(String::from(flag))
        );
    }
    assert_eq!(
        two(&SYNTHETIC_RMG.replacen("adjacent = [5, 5]", "adjacent = [5]", 1)),
        RmgCompileError::ObstacleScores {
            pack: pack(),
            obstacle: 1
        }
    );
    assert_eq!(
        two(&SYNTHETIC_RMG.replacen("overlap = [5, 0]", "overlap = [5, 0, 1]", 1)),
        RmgCompileError::ObstacleScores {
            pack: pack(),
            obstacle: 0
        }
    );
    // `lone` loads first and `synthetic` does not depend on it: its lines and templates are not
    // of `synthetic`'s closure.
    let lone = format!(
        "object_templates = [{}]\n{}",
        line("", "lone.def"),
        template("Lone", &[&connection([1, 1])])
    );
    let after_lone = |rmg: &str| compile_err(&[(LONE_CORE, &lone), (SYNTHETIC_CORE, rmg)], PROFILE);
    let synthetic = || String::from("synthetic");
    assert_eq!(
        after_lone(&format!(
            "object_template_overrides = [{}]",
            line("line = 0, ", "a.def")
        )),
        RmgCompileError::ObjectLine {
            pack: synthetic(),
            line: 0
        }
    );
    assert_eq!(
        after_lone(&format!(
            "object_templates = [{}]",
            line("before = 0, ", "a.def")
        )),
        RmgCompileError::ObjectLine {
            pack: synthetic(),
            line: 0
        }
    );
    assert_eq!(
        after_lone("[[connection_override]]\ntemplate = 0\nconnection = 0\nzones = [1, 1]\n"),
        RmgCompileError::Template {
            pack: synthetic(),
            template: 0
        }
    );
}

#[test]
fn every_error_has_a_message() {
    let pack = || String::from("p");
    let compile_errors = [
        RmgCompileError::UnknownPack(pack()),
        RmgCompileError::DuplicatePack(pack()),
        RmgCompileError::MissingPack(pack()),
        RmgCompileError::UnknownDependency {
            pack: pack(),
            dependency: pack(),
        },
        RmgCompileError::ObjectLine {
            pack: pack(),
            line: 1,
        },
        RmgCompileError::DuplicateObjectOverride {
            pack: pack(),
            line: 1,
        },
        RmgCompileError::Template {
            pack: pack(),
            template: 1,
        },
        RmgCompileError::Connection {
            pack: pack(),
            template: 1,
            connection: 2,
        },
        RmgCompileError::NotADataQuirk(pack()),
        RmgCompileError::ObstacleScores {
            pack: pack(),
            obstacle: 1,
        },
        RmgCompileError::GeneratorTables(pack()),
    ];
    for e in compile_errors.iter().cloned() {
        assert!(!e.to_string().is_empty());
        assert!(!RmgPackError::from(e).to_string().is_empty());
    }
    for e in [
        RmgPackError::Syntax(pack()),
        RmgPackError::Invalid(pack()),
        RmgPackError::Core(PackError::Syntax(pack())),
    ] {
        assert!(!e.to_string().is_empty());
    }
}

/// The original tables' counts (survey `E14-generator.md` §3.1's sanity table): templates, zones
/// by kind, connections and the never-kept ones (columns 81 to 84 empty), `objects.txt`'s lines,
/// the obstacles.
#[test]
fn the_original_tables_are_the_surveys() {
    let counts = |packs: &[(&str, &str)]| {
        let part = compile(packs, "base:faithful").unwrap();
        let zones = part.templates().iter().flat_map(|t| &t.zones);
        let mut kinds = [0; 4];
        zones.for_each(|z| kinds[usize::try_from(z.kind.value()).unwrap()] += 1);
        let connections: Vec<_> = part
            .templates()
            .iter()
            .flat_map(|t| &t.connections)
            .collect();
        let never = connections.iter().filter(|c| c.humans[1] == 0).count();
        assert!(part.obstacles().iter().all(|o| o.adjacent.len() == 109));
        (
            part.templates().len(),
            kinds,
            connections.len(),
            never,
            part.object_templates().len(),
            part.obstacles().len(),
        )
    };
    assert_eq!(
        counts(&ORIGINAL_PACKS[..2]),
        (50, [222, 30, 308, 56], 833, 6, 1221, 109)
    );
    assert_eq!(
        counts(&ORIGINAL_PACKS),
        (53, [236, 30, 323, 57], 869, 7, 1326, 109)
    );
    let part = compile(&ORIGINAL_PACKS, "base:faithful").unwrap();
    let names: Vec<_> = part.templates()[50..]
        .iter()
        .map(|t| t.name.as_str())
        .collect();
    assert_eq!(
        names,
        ["Golems Aplenty", "Monk's Retreat", "Rise of Phoenix"]
    );
    assert!(
        part.templates()
            .iter()
            .any(|t| t.name == "Meeting \nin Muzgob")
    );
    assert_eq!(
        part.templates()
            .iter()
            .filter(|t| t.name == "6LM10")
            .count(),
        2
    );
}

/// RMG-QUIRK-22: 8XM8's six decimal first zones as `c_atoi` reads them in `faithful`, the series
/// continued in `strict-fixed` (the gated override of the AB pack), in AB's table and SoD's.
#[test]
fn eight_xm8s_zone_ids_follow_the_flag() {
    let first_zones = |packs: &[(&str, &str)], profile| {
        let part = compile(packs, profile).unwrap();
        let [t] = part
            .templates()
            .iter()
            .filter(|t| t.name == "8XM8")
            .collect::<Vec<_>>()[..]
        else {
            panic!("one 8XM8");
        };
        t.connections[10..16]
            .iter()
            .map(|c| c.zones)
            .collect::<Vec<_>>()
    };
    for packs in [&ORIGINAL_PACKS[..2], &ORIGINAL_PACKS[..]] {
        assert_eq!(
            first_zones(packs, "base:faithful"),
            [[5, 37], [6, 38], [6, 41], [7, 42], [7, 47], [7, 48]]
        );
        assert_eq!(
            first_zones(packs, "base:strict-fixed"),
            [[6, 37], [6, 38], [7, 41], [7, 42], [8, 47], [8, 48]]
        );
    }
}

mod digest;
mod generate;
