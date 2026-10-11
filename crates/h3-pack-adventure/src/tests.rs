//! Tests of the adventure pack format (E08 T4).

use std::error::Error;

use super::*;

const PROFILE: &str = "base:faithful";

fn manifest(id: &str) -> PackManifest {
    PackManifest {
        id: id.into(),
        version: "1".into(),
        depends: vec![],
    }
}

fn parse(text: &str) -> Result<AdventurePack, AdventurePackError> {
    parse_adventure(&manifest("p"), text)
}

fn cost(value: u16) -> NonZeroU16 {
    NonZeroU16::new(value).unwrap()
}

// covers: GAM-SET-001
#[test]
fn the_original_starting_armies_fit_the_draw() {
    let (core, part) = compile(&ORIGINAL_PACKS, PROFILE).unwrap();
    assert_eq!(part.params().starting_stack_chances(), [88, 25]);
    // The draw reads a hero type's starting slots as its first stack, its war machines, then the
    // other stacks: the original's hero table has three slots and every war machine in the
    // second (Heroes3.exe 0x679DD0 + 0x24; E08 T6b). The packs keep that shape.
    let mut machines = 0;
    for hero in core.hero_types() {
        let slots = (hero.army.len(), hero.war_machines.len());
        assert!(
            slots.0 >= 1 && slots.0 + slots.1 <= 3,
            "{}: {slots:?}",
            hero.key.as_str()
        );
        if slots.1 > 0 {
            machines += 1;
            assert_eq!(slots, (2, 1), "{}", hero.key.as_str());
        }
    }
    assert_eq!(machines, 9);
}

// covers: ADV-MOV-001, ADV-MOV-003, ADV-MOV-004, ADV-MOV-007, ADV-FOG-001
#[test]
fn the_base_pack_holds_the_sourced_values() {
    let (core, part) = compile(&ORIGINAL_PACKS[..1], PROFILE).unwrap();
    let params = part.params();
    assert_eq!(
        params.land_points_table(),
        [
            1300, 1360, 1430, 1500, 1560, 1630, 1700, 1760, 1830, 1900, 1960, 2000
        ]
    );
    assert_eq!(params.land_points(20), 2000);
    assert_eq!(params.native_cost(), cost(100));
    assert_eq!(params.scouting_radius(), 5);
    let passages: Vec<(&str, Passage)> = part
        .terrains()
        .iter()
        .map(|t| (t.key.as_str(), t.passage))
        .collect();
    let land = |c| Passage::Land(cost(c));
    assert_eq!(
        passages,
        [
            ("base:dirt", land(100)),
            ("base:desert", land(150)),
            ("base:grass", land(100)),
            ("base:snow", land(150)),
            ("base:swamp", land(175)),
            ("base:rough", land(125)),
            ("base:subterranean", land(100)),
            ("base:lava", land(100)),
            ("base:water", Passage::Water(cost(100))),
            ("base:rock", Passage::Impassable),
        ]
    );
    assert_eq!(part.terrains().len(), core.terrains().len());
    let roads: Vec<(&str, u16)> = part
        .roads()
        .iter()
        .map(|r| (r.key.as_str(), r.cost.get()))
        .collect();
    assert_eq!(
        roads,
        [
            ("base:dirt_road", 75),
            ("base:gravel_road", 65),
            ("base:cobblestone_road", 50)
        ]
    );
    let rivers: Vec<&str> = part.rivers().iter().map(|r| r.key.as_str()).collect();
    assert_eq!(
        rivers,
        [
            "base:clear_river",
            "base:icy_river",
            "base:muddy_river",
            "base:lava_river"
        ]
    );
}

#[test]
fn the_expansions_change_nothing_yet() {
    let (_, base) = compile(&ORIGINAL_PACKS[..1], PROFILE).unwrap();
    let (ab_core, ab) = compile(&ORIGINAL_PACKS[..2], PROFILE).unwrap();
    let (sod_core, sod) = compile(&ORIGINAL_PACKS, PROFILE).unwrap();
    assert_eq!(ab.content_hash(), base.content_hash());
    assert_eq!(sod.content_hash(), base.content_hash());
    // The parts still belong to their own core rulesets.
    assert_eq!(ab.core_hash(), ab_core.content_hash());
    assert_eq!(sod.core_hash(), sod_core.content_hash());
    assert_ne!(ab.core_hash(), sod.core_hash());
}

#[test]
fn the_synthetic_packs_compile() {
    let (_, part) = compile(
        &[
            (h3_pack::SYNTHETIC_EXT_PACK, SYNTHETIC_EXT_ADVENTURE),
            (h3_pack::SYNTHETIC_PACK, SYNTHETIC_ADVENTURE),
        ],
        "synthetic:faithful",
    )
    .unwrap();
    let roads: Vec<&str> = part.roads().iter().map(|r| r.key.as_str()).collect();
    assert_eq!(roads, ["synthetic:trail", "synthetic-ext:causeway"]);
    assert_eq!(part.river_by_key("synthetic-ext:creek").unwrap().raw(), 1);
    assert_eq!(part.params().land_points(9), 1700);
}

#[test]
fn parses_every_field() {
    let pack = parse(
        r#"
        [params]
        native_cost = 90
        land_points = [1000, 1100]
        scouting_radius = 3
        max_map_size = 72
        max_levels = 1
        max_players = 4
        army_slots = 5
        starting_stack_chances = [80, 20]

        [[terrain]]
        terrain = "plain"
        cost = 110

        [[terrain]]
        terrain = "other:sea"
        passage = "water"
        cost = 120

        [[terrain]]
        terrain = "cliff"
        passage = "impassable"

        [[road]]
        id = "path"
        name = "Path"
        cost = 80

        [[river]]
        id = "stream"
        name = "Stream"
        "#,
    )
    .unwrap();
    assert_eq!(pack.manifest, manifest("p"));
    let terrains: Vec<(&str, Passage)> = pack
        .terrains
        .iter()
        .map(|t| (t.terrain.as_str(), t.passage))
        .collect();
    assert_eq!(
        terrains,
        [
            ("p:plain", Passage::Land(cost(110))),
            ("other:sea", Passage::Water(cost(120))),
            ("p:cliff", Passage::Impassable),
        ]
    );
    assert_eq!(pack.roads[0].key.as_str(), "p:path");
    assert_eq!(
        (pack.roads[0].name.as_str(), pack.roads[0].cost),
        ("Path", cost(80))
    );
    assert_eq!(pack.rivers[0].key.as_str(), "p:stream");
    assert_eq!(pack.rivers[0].name, "Stream");
    assert_eq!(
        pack.params,
        Some(AdventureParamsDef {
            native_cost: cost(90),
            land_points: vec![1000, 1100],
            scouting_radius: 3,
            max_map_size: cost(72),
            max_levels: NonZeroU8::new(1).unwrap(),
            max_players: NonZeroU8::new(4).unwrap(),
            army_slots: NonZeroU8::new(5).unwrap(),
            starting_stack_chances: vec![80, 20],
        })
    );
    assert_eq!(parse("").unwrap(), AdventurePack::new(manifest("p")));
}

#[test]
fn rejects_malformed_files() {
    let syntax = |text: &str| matches!(parse(text), Err(AdventurePackError::Syntax(_)));
    assert!(syntax("[[terrain]]\nterrain = \"a\"\ncost = 1\nspeed = 2"));
    assert!(syntax("[[road]]\nid = \"a\"\nname = \"A\"\ncost = 0"));
    assert!(syntax("[[terrain]]\nterrain = \"a\"\npassage = \"lava\""));
    assert!(syntax(
        "[params]\nnative_cost = 100\nland_points = [1]\nscouting_radius = 300"
    ));
    let bounds = "native_cost = 100\nland_points = [1]\nscouting_radius = 1\nmax_map_size = 1";
    let slots = "max_levels = 1\nmax_players = 1\narmy_slots";
    assert!(syntax(&format!(
        "[params]\n{bounds}\n{slots} = 0\nstarting_stack_chances = []"
    )));
    assert!(syntax(&format!(
        "[params]\n{bounds}\n{slots} = 1\nstarting_stack_chances = [256]"
    )));
    assert!(syntax(&format!("[params]\n{bounds}\n{slots} = 1")));
    assert!(syntax(&format!(
        "[params]\n{bounds}\nmax_levels = 1\nmax_players = 1"
    )));
    assert!(syntax("[unknown]"));
    let cost_error = |text: &str| parse(text).unwrap_err();
    for text in [
        "[[terrain]]\nterrain = \"a\"",
        "[[terrain]]\nterrain = \"a\"\npassage = \"water\"",
        "[[terrain]]\nterrain = \"a\"\npassage = \"impassable\"\ncost = 100",
    ] {
        assert_eq!(
            cost_error(text),
            AdventurePackError::TerrainCost("a".into())
        );
    }
    let key = |text: &str| matches!(parse(text), Err(AdventurePackError::Key(_)));
    assert!(key("[[terrain]]\nterrain = \"Bad:a\"\ncost = 100"));
    assert!(key("[[road]]\nid = \"Bad\"\nname = \"A\"\ncost = 1"));
    assert!(key("[[river]]\nid = \"Bad\"\nname = \"A\""));
}

#[test]
fn compile_reports_each_stage() {
    assert!(matches!(
        compile(&[("[pack]", "")], PROFILE),
        Err(AdventurePackError::Core(PackError::Syntax(_)))
    ));
    assert!(matches!(
        compile(&ORIGINAL_PACKS, "base:unknown"),
        Err(AdventurePackError::Core(PackError::Compile(_)))
    ));
    assert!(matches!(
        compile(&[(h3_pack::BASE_PACK, "[unknown]")], PROFILE),
        Err(AdventurePackError::Syntax(_))
    ));
    assert_eq!(
        compile(&[(h3_pack::BASE_PACK, "")], PROFILE).unwrap_err(),
        AdventurePackError::Compile(AdventureCompileError::MissingTerrain("base:dirt".into()))
    );
}

#[test]
fn errors_explain_themselves() {
    let key = ContentKey::parse("Bad").unwrap_err();
    let cases = [
        (
            AdventurePackError::Syntax("eof".into()),
            "syntax: eof".to_owned(),
        ),
        (AdventurePackError::Key(key.clone()), key.to_string()),
        (
            AdventurePackError::TerrainCost("a".into()),
            "terrain `a`: land and water need a cost, an impassable terrain has none".into(),
        ),
        (
            AdventurePackError::Core(PackError::Syntax("eof".into())),
            "core packs: syntax: eof".into(),
        ),
        (
            AdventurePackError::Compile(AdventureCompileError::MissingParams),
            "no pack provides the adventure parameters".into(),
        ),
    ];
    for (error, text) in cases {
        assert_eq!(error.to_string(), text);
        assert!(error.source().is_none());
    }
    assert_eq!(
        AdventurePackError::from(key.clone()),
        AdventurePackError::Key(key)
    );
}
