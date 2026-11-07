//! Tests of the town pack format (E09 T5a1), on the synthetic packs' files (E09 T5b2a).

use h3_content::FactionId;
use h3_towns::{Building, BuildingId, BuildingSet, TownRecord};

use super::*;

const PROFILE: &str = "synthetic:faithful";

/// The synthetic pack's file: the parameters and a record of `vale` (its two levels).
const VALE: &str = SYNTHETIC_TOWNS;

/// The synthetic expansion's file: an override of `vale`.
const VALE_OVERRIDE: &str = SYNTHETIC_EXT_TOWNS;

fn compiled() -> TownRuleset {
    compile(
        &[
            (h3_pack::SYNTHETIC_CORE, VALE),
            (h3_pack::SYNTHETIC_EXT_CORE, VALE_OVERRIDE),
        ],
        PROFILE,
    )
    .unwrap()
}

fn core(texts: &[&str]) -> Ruleset {
    let packs: Vec<_> = texts
        .iter()
        .map(|t| h3_pack::parse_pack(t).unwrap())
        .collect();
    Ruleset::compile(&packs, PROFILE).unwrap()
}

fn vale(part: &TownRuleset) -> &TownRecord {
    &part.towns()[0]
}

fn building<'a>(record: &'a TownRecord, key: &str) -> &'a Building {
    record
        .building(record.building_by_key(key).unwrap())
        .unwrap()
}

#[test]
fn the_synthetic_record_compiles_with_its_override() {
    let core = core(&[h3_pack::SYNTHETIC_CORE, h3_pack::SYNTHETIC_EXT_CORE]);
    let part = compiled();
    assert_eq!(part.core_hash(), core.content_hash());
    assert_eq!(part.towns().len(), 1);
    let record = vale(&part);
    assert_eq!(Some(record.faction), core.faction_by_key("synthetic:vale"));
    assert_eq!(part.town(record.faction), Some(record));
    assert_eq!(
        (record.moat, record.siege_obstacles),
        (Moat::Damaging, true)
    );
    let shard = core
        .resources()
        .iter()
        .position(|r| r.key.as_str() == "synthetic:shard")
        .unwrap();
    assert_eq!(record.silo.len(), 1);
    assert_eq!((record.silo[0].0.index(), record.silo[0].1), (shard, 3));
    let levels: Vec<_> = record
        .creatures
        .iter()
        .map(|l| (l.base, l.upgrade))
        .collect();
    assert_eq!(
        levels,
        [
            (core.creature_by_key("synthetic:spearman").unwrap(), None),
            (core.creature_by_key("synthetic:brute").unwrap(), None),
        ]
    );
    assert_eq!(part.params().unwrap().capitol_demoted_to, "city_hall");
    assert_eq!(part.params().unwrap().neutral_garrison[1].count, [5, 7]);
    // E09 T8c2: the synthetic pack's Visions parameters and the expansion's creature.
    let params = part.params().unwrap();
    assert_eq!((params.visions_factor, params.visions_min_radius), (2, 4));
    let lurker = core.creature_by_key("synthetic-ext:lurker").unwrap();
    assert_eq!(params.visions_creatures, [lurker]);
}

#[test]
fn the_roles_parse() {
    let part = compiled();
    let record = vale(&part);
    let village = building(record, "village_hall");
    assert_eq!(
        (village.name.as_str(), village.slot, village.roles.hall),
        ("Village Hall", Some(10), Some(1))
    );
    let capitol = building(record, "capitol");
    assert_eq!(capitol.supersedes.bits(), 0b111);
    assert_eq!(building(record, "castle").roles.fort, Some(3));
    let tavern = building(record, "tavern").roles;
    assert!(tavern.tavern && tavern.morale == 1);
    assert!(building(record, "silo").roles.silo);
    let grail = building(record, "grail").roles;
    assert!(grail.grail && grail.escape_tunnel && grail.luck == 2);
    assert_eq!(
        grail.defender,
        DefenderBonus {
            attack: 2,
            defense: 0,
            power: 0,
            mana: 150,
        }
    );
    assert_eq!(grail.reveal, Some(Reveal::All));
    assert_eq!(
        building(record, "lookout").roles.reveal,
        Some(Reveal::Radius(20))
    );
    assert_eq!(
        building(record, "dwelling_2").roles.dwelling,
        Some(Dwelling {
            level: 2,
            upgraded: false,
        })
    );
    let horde_up = building(record, "horde_up");
    let horde = record.building_by_key("horde").unwrap();
    assert_eq!(horde_up.forbidden_with, Some(horde));
    let dwelling_2 = record.building_by_key("dwelling_2").unwrap();
    assert_eq!(
        horde_up.auto_with,
        BuildingSet::EMPTY.with(horde).with(dwelling_2)
    );
    assert!(horde_up.supersedes.contains(horde) && horde_up.supersedes.contains(dwelling_2));
    assert_eq!(building(record, "horde").auto_with, BuildingSet::EMPTY);
    assert_eq!(
        record.building(record.capitol_demoted_to).unwrap().key,
        "city_hall"
    );
    assert!(
        record
            .building(BuildingId::from_index(63).unwrap())
            .is_none()
    );
}

#[test]
fn the_requirements_costs_and_the_shipyard_parse() {
    // E09 T16a: the direct requirements closed, the costs in the core's resource order.
    let part = compiled();
    let record = vale(&part);
    let id = |k: &str| record.building_by_key(k).unwrap();
    let set = |keys: &[&str]| {
        keys.iter()
            .fold(BuildingSet::EMPTY, |set, &k| set.with(id(k)))
    };
    assert_eq!(
        building(record, "capitol").requires,
        set(&[
            "city_hall",
            "town_hall",
            "tavern",
            "castle",
            "citadel",
            "fort"
        ])
    );
    assert_eq!(
        building(record, "horde_up").requires,
        set(&["horde", "dwelling_2", "dwelling_1", "fort"])
    );
    assert_eq!(
        building(record, "village_hall").requires,
        BuildingSet::EMPTY
    );
    let core = core(&[h3_pack::SYNTHETIC_CORE, h3_pack::SYNTHETIC_EXT_CORE]);
    let resource = |k: &str| {
        let index = core.resources().iter().position(|r| r.key.as_str() == k);
        index.and_then(h3_content::ResourceId::from_index).unwrap()
    };
    let (coin, shard) = (resource("synthetic:coin"), resource("synthetic:shard"));
    assert_eq!(
        building(record, "capitol").cost,
        [(coin, 10_000), (shard, 10)]
    );
    assert_eq!(building(record, "horde_up").cost, [(coin, 1000)]);
    assert!(building(record, "grail").cost.is_empty());
    let shipyard = building(record, "shipyard");
    assert!(shipyard.roles.shipyard);
    assert_eq!(
        (shipyard.requires, shipyard.cost.as_slice()),
        (set(&["tavern"]), [(coin, 2000), (shard, 20)].as_slice())
    );
    assert_eq!(
        record.buildings.iter().filter(|b| b.roles.shipyard).count(),
        1
    );
    // A cost's key of another pack, as a silo's.
    let text = VALE.replace(
        "cost = { coin = 2500 }",
        r#"cost = { "synthetic:shard" = 7 }"#,
    );
    let pack = parse(&text).unwrap();
    let town_hall = pack.towns[0]
        .buildings
        .iter()
        .find(|b| b.key == "town_hall")
        .unwrap();
    assert_eq!(town_hall.requires, ["tavern"]);
    assert_eq!(
        town_hall
            .cost
            .iter()
            .map(|(k, a)| (k.as_str(), *a))
            .collect::<Vec<_>>(),
        [("synthetic:shard", 7)]
    );
}

#[test]
fn an_upgrade_and_a_parsed_pack_keep_what_they_say() {
    let manifest = h3_pack::parse_pack(h3_pack::SYNTHETIC_CORE)
        .unwrap()
        .manifest;
    let text = VALE.replace(
        r#"creatures = [["spearman"], ["synthetic:brute"]]"#,
        r#"creatures = [["spearman", "brute"]]"#,
    );
    let pack = parse_towns(&manifest, &text).unwrap();
    assert_eq!(pack.manifest, manifest);
    let level = &pack.towns[0].creatures[0];
    assert_eq!(
        level.upgrade.as_ref().map(ContentKey::as_str),
        Some("synthetic:brute")
    );
    assert_eq!(pack.towns[0].silo.len(), 2);
    assert!(pack.overrides.is_empty() && pack.params.is_some());
    let ext = parse_towns(&manifest, VALE_OVERRIDE).unwrap();
    let patch = &ext.overrides[0].patch;
    assert_eq!(patch.moat, Some(Moat::Damaging));
    assert_eq!(patch.siege_obstacles, Some(true));
    // A moat each way.
    for (word, moat) in [("absent", Moat::Absent), ("harmless", Moat::Harmless)] {
        let text = VALE_OVERRIDE.replace("\"damaging\"", &format!("\"{word}\""));
        let pack = parse_towns(&manifest, &text).unwrap();
        assert_eq!(pack.overrides[0].patch.moat, Some(moat));
    }
}

#[test]
fn packs_without_the_file_compile_to_the_empty_part() {
    let synthetic = compile(
        &[
            (h3_pack::SYNTHETIC_CORE, ""),
            (h3_pack::SYNTHETIC_EXT_CORE, ""),
        ],
        PROFILE,
    )
    .unwrap();
    let core = core(&[h3_pack::SYNTHETIC_CORE, h3_pack::SYNTHETIC_EXT_CORE]);
    assert_eq!(synthetic, TownRuleset::empty(&core));
    // The original core packs, which have no `towns.toml` until E09 T5b.
    let original = compile(
        &[
            (h3_pack::BASE_CORE, ""),
            (h3_pack::AB_CORE, ""),
            (h3_pack::SOD_CORE, ""),
        ],
        "base:faithful",
    )
    .unwrap();
    assert!(original.towns().is_empty() && original.params().is_none());
    assert!(original.town(FactionId::from_index(0).unwrap()).is_none());
}

fn parse(text: &str) -> Result<TownPack, TownPackError> {
    let manifest = h3_pack::parse_pack(h3_pack::SYNTHETIC_CORE)
        .unwrap()
        .manifest;
    parse_towns(&manifest, text)
}

#[test]
fn the_parser_is_strict() {
    let syntax = |text: &str| matches!(parse(text), Err(TownPackError::Syntax(_)));
    assert!(syntax("[town_params]\nbogus = 1"));
    assert!(syntax("[town_visions]\nbogus = 1"));
    assert!(syntax("[town_visions]\ncreatures = \"rogue\""));
    assert!(syntax(
        &VALE.replace("moat = \"harmless\"", "moat = \"deep\"")
    ));
    assert!(syntax(
        &VALE.replace("reveal = \"all\"", "reveal = \"some\"")
    ));
    assert!(syntax(&VALE.replace("reveal = 20", "reveal = 300")));
    assert!(syntax(
        &VALE.replace("defender = { attack = 2,", "defender = { speed = 2,")
    ));
    assert!(syntax(
        &VALE.replace("dwelling = [1, false]", "dwelling = [1]")
    ));
    assert!(syntax(&VALE.replace(
        r#"auto_with = ["horde", "dwelling_2"]"#,
        r#"auto_with = "horde""#
    )));
    assert!(syntax(
        &VALE.replace(r#"requires = ["tavern"]"#, r#"requires = "tavern""#)
    ));
    assert!(syntax(
        &VALE.replace("cost = { coin = 2500 }", "cost = { coin = -1 }")
    ));
    assert!(syntax(
        &VALE.replace("cost = { coin = 2500 }", "cost = 2500")
    ));
    assert!(syntax(&VALE.replace("shipyard = true", "shipyard = 1")));
    let key = |text: &str| matches!(parse(text), Err(TownPackError::Key(_)));
    assert!(key(
        &VALE.replace("faction = \"vale\"", "faction = \"Vale\"")
    ));
    assert!(key(&VALE.replace("gold = \"coin\"", "gold = \"Coin\"")));
    assert!(key(&VALE.replace("[\"spearman\"]", "[\"Spearman\"]")));
    assert!(key(
        &VALE.replace("[\"spearman\"]", "[\"spearman\", \"x:\"]")
    ));
    assert!(key(&VALE.replace("shard = 1", "Shard = 1")));
    assert!(key(
        &VALE.replace("cost = { coin = 2500 }", "cost = { Coin = 2500 }")
    ));
    assert!(key(&VALE_OVERRIDE.replace("synthetic:vale", "Vale")));
    assert!(key(&VALE_OVERRIDE.replace("\"lurker\"", "\"Lurker\"")));
    assert!(key(
        &VALE_OVERRIDE.replace("synthetic:shard", "synthetic:Shard")
    ));
    for levels in ["[[]]", r#"[["spearman", "brute", "brute"]]"#] {
        let text = VALE.replace(r#"[["spearman"], ["synthetic:brute"]]"#, levels);
        assert_eq!(
            parse(&text),
            Err(TownPackError::CreatureLevel("vale".into()))
        );
    }
}

#[test]
fn compile_errors_name_their_stage() {
    let core = compile(&[("not toml", "")], PROFILE);
    assert!(matches!(
        core,
        Err(TownPackError::Core(PackError::Syntax(_)))
    ));
    let profile = compile(&[(h3_pack::SYNTHETIC_CORE, "")], "synthetic:none");
    assert!(matches!(
        profile,
        Err(TownPackError::Core(PackError::Compile(_)))
    ));
    let towns = compile(&[(h3_pack::SYNTHETIC_CORE, "town = 1")], PROFILE);
    assert!(matches!(towns, Err(TownPackError::Syntax(_))));
    let missing = VALE.replace("[town_params]", "[unused]");
    let missing = compile(&[(h3_pack::SYNTHETIC_CORE, &missing)], PROFILE);
    assert!(matches!(missing, Err(TownPackError::Syntax(_))));
    let without = VALE
        .split("[[town]]")
        .nth(1)
        .map(|records| format!("[[town]]{records}"))
        .unwrap();
    let error = compile(&[(h3_pack::SYNTHETIC_CORE, &without)], PROFILE).unwrap_err();
    assert_eq!(
        error,
        TownPackError::Compile(TownCompileError::MissingParams)
    );
    assert_eq!(error.to_string(), "no pack provides the town parameters");
}

#[test]
fn every_error_has_a_message() {
    let key = ContentKey::parse("Bad").unwrap_err();
    let cases = [
        (
            TownPackError::Syntax("why".into()),
            "syntax: why".to_owned(),
        ),
        (TownPackError::Key(key.clone()), key.to_string()),
        (
            TownPackError::CreatureLevel("vale".into()),
            "town `vale`: a creature level holds the creature and at most one upgrade".to_owned(),
        ),
        (
            TownPackError::Core(PackError::Syntax("why".into())),
            "core packs: syntax: why".to_owned(),
        ),
    ];
    for (error, message) in cases {
        assert_eq!(error.to_string(), message);
    }
    let error: &dyn std::error::Error = &TownPackError::Syntax(String::new());
    assert!(error.source().is_none());
}

/// The original packs' profile.
const ORIGINAL: &str = "base:faithful";

/// The original core packs, compiled with the original profile.
fn original_core(texts: &[&str]) -> Ruleset {
    let packs: Vec<_> = texts
        .iter()
        .map(|t| h3_pack::parse_pack(t).unwrap())
        .collect();
    Ruleset::compile(&packs, ORIGINAL).unwrap()
}

#[test]
fn the_original_packs_compile_with_every_faction() {
    // E09 T5b2b2: the harvested files compile in every expansion combination.
    let roe = compile(&ORIGINAL_PACKS[..1], ORIGINAL).unwrap();
    let ab = compile(&ORIGINAL_PACKS[..2], ORIGINAL).unwrap();
    let sod = compile(&ORIGINAL_PACKS, ORIGINAL).unwrap();
    assert!(compile(&ORIGINAL_PACKS, "base:strict-fixed").is_ok());
    let core = original_core(&[h3_pack::BASE_CORE, h3_pack::AB_CORE, h3_pack::SOD_CORE]);
    assert_eq!(sod.core_hash(), core.content_hash());
    assert_eq!(
        (roe.towns().len(), ab.towns().len(), sod.towns().len()),
        (8, 9, 9)
    );
    let faction = |key: &str| core.faction_by_key(key).unwrap();
    // Each faction: seven levels, each with its upgrade; its listed buildings (survey E09-towns
    // §16.1's counts) and the Village Hall; its creatures the faction's of each level.
    let factions = [
        ("base:castle", 35),
        ("base:rampart", 37),
        ("base:tower", 36),
        ("base:inferno", 37),
        ("base:necropolis", 36),
        ("base:dungeon", 36),
        ("base:stronghold", 34),
        ("base:fortress", 34),
        ("ab:conflux", 35),
    ];
    for (key, listed) in factions {
        let record = sod.town(faction(key)).unwrap();
        assert_eq!(record.creatures.len(), 7, "{key}");
        assert!(
            record.creatures.iter().all(|l| l.upgrade.is_some()),
            "{key}"
        );
        assert_eq!(record.buildings.len(), listed + 1, "{key}");
        let hall = building(record, "village_hall");
        assert_eq!((hall.roles.hall, hall.slot), (Some(1), None), "{key}");
        assert_eq!(
            record.building(record.capitol_demoted_to).unwrap().key,
            "city_hall"
        );
    }
    // The moats by edition (E09 decision 20): RoE and AB rules in base and ab, SoD's overrides.
    let siege = |part: &TownRuleset, key: &str| {
        let record = part.town(faction(key)).unwrap();
        (record.moat, record.siege_obstacles)
    };
    assert_eq!(siege(&roe, "base:castle"), (Moat::Harmless, false));
    assert_eq!(siege(&roe, "base:stronghold"), (Moat::Absent, true));
    assert_eq!(siege(&ab, "ab:conflux"), (Moat::Harmless, false));
    for (key, _) in factions {
        assert_eq!(siege(&sod, key), (Moat::Damaging, false), "{key}");
    }
    // The parameters.
    let params = sod.params().unwrap();
    assert_eq!(params.hall_gold, [500, 1000, 2000, 4000]);
    assert_eq!((params.grail_gold, params.reveal_radius), (5000, 5));
    assert_eq!(params.level_2_dwelling_chance, 30);
    assert_eq!(params.neutral_garrison[3].count, [1, 3]);
    assert_eq!(
        core.resources()[params.gold.index()].key.as_str(),
        "base:gold"
    );
    // Visions (E09 T8c2): the expert factor and least radius; the Rogue from AB on.
    assert_eq!((params.visions_factor, params.visions_min_radius), (3, 3));
    let rogue = core.creature_by_key("ab:rogue").unwrap();
    assert!(roe.params().unwrap().visions_creatures.is_empty());
    let ab_rogue =
        original_core(&[h3_pack::BASE_CORE, h3_pack::AB_CORE]).creature_by_key("ab:rogue");
    assert_eq!(ab.params().unwrap().visions_creatures, [ab_rogue.unwrap()]);
    assert_eq!(params.visions_creatures, [rogue]);
    // Roles of one faction each (survey §16.6), the hordes' upgrade (decision 29).
    let castle = sod.town(faction("base:castle")).unwrap();
    let brotherhood = building(castle, "brotherhood_of_the_sword");
    assert!(brotherhood.roles.tavern && brotherhood.roles.morale == 2);
    let tavern = castle.building_by_key("tavern").unwrap();
    assert!(brotherhood.supersedes.contains(tavern));
    let horde = castle.building_by_key("griffin_bastion").unwrap();
    let upgraded = building(castle, "upg_griffin_bastion");
    assert_eq!(upgraded.forbidden_with, Some(horde));
    let dwelling = castle.building_by_key("upg_griffin_tower").unwrap();
    assert_eq!(
        upgraded.auto_with,
        BuildingSet::default().with(horde).with(dwelling)
    );
    let rampart = sod.town(faction("base:rampart")).unwrap();
    assert_eq!(building(rampart, "fountain_of_fortune").roles.luck, 2);
    let tower = sod.town(faction("base:tower")).unwrap();
    assert_eq!(
        building(tower, "lookout_tower").roles.reveal,
        Some(Reveal::Radius(20))
    );
    let skyship = building(tower, "skyship");
    assert!(skyship.roles.grail && skyship.roles.reveal == Some(Reveal::All));
    assert_eq!(skyship.roles.defender.mana, 150);
    let stronghold = sod.town(faction("base:stronghold")).unwrap();
    assert!(building(stronghold, "escape_tunnel").roles.escape_tunnel);
    // TD-8 by the code (decision 30).
    let fortress = sod.town(faction("base:fortress")).unwrap();
    assert_eq!(
        building(fortress, "glyphs_of_fear").roles.defender.attack,
        2
    );
    assert_eq!(
        building(fortress, "blood_obelisk").roles.defender.defense,
        2
    );
    let conflux = sod.town(faction("ab:conflux")).unwrap();
    assert!(building(conflux, "aurora_borealias").roles.grail);
    the_original_requirements_costs_and_shipyards(&sod, &core);
}

/// E09 T16b's harvest in the shipped packs (design §25): every closure inside its record, never
/// itself, the Capitol's holding the Town Hall; the closures of the Capitol and the upgraded
/// level-7 dwelling of every faction as survey `E09-construction.md` §6's lists give them (TD-9:
/// the Conflux's upgraded Altar of Fire keeps the list's upgraded Altar of Air, decision 41); §5's
/// costs (the neutral rows, a faction's special rows, a dwelling's rows); the Shipyards.
fn the_original_requirements_costs_and_shipyards(sod: &TownRuleset, core: &Ruleset) {
    let keys = |record: &TownRecord, set: BuildingSet| -> Vec<String> {
        let mut keys: Vec<String> = set
            .iter()
            .map(|b| record.building(b).unwrap().key.clone())
            .collect();
        keys.sort_unstable();
        keys
    };
    for record in sod.towns() {
        for (i, b) in record.buildings.iter().enumerate() {
            let id = BuildingId::from_index(i).unwrap();
            assert!(!b.requires.contains(id), "{}", b.key);
            assert!(
                b.requires
                    .iter()
                    .all(|r| r.index() < record.buildings.len()),
                "{}",
                b.key
            );
        }
        let capitol = building(record, "capitol");
        assert!(
            capitol
                .requires
                .contains(record.building_by_key("town_hall").unwrap())
        );
        assert_eq!(
            keys(record, capitol.requires),
            [
                "blacksmith",
                "castle",
                "citadel",
                "city_hall",
                "fort",
                "mage_guild_level_1",
                "marketplace",
                "tavern",
                "town_hall"
            ]
        );
    }
    let faction = |key: &str| sod.town(core.faction_by_key(key).unwrap()).unwrap();
    let level_7: [(&str, &str, &[&str]); 9] = [
        (
            "base:castle",
            "upg_portal_of_glory",
            &[
                "barracks",
                "blacksmith",
                "fort",
                "guardhouse",
                "mage_guild_level_1",
                "monastery",
                "portal_of_glory",
            ],
        ),
        (
            "base:rampart",
            "upg_dragon_cliffs",
            &[
                "centaur_stables",
                "dendroid_arches",
                "dragon_cliffs",
                "enchanted_spring",
                "fort",
                "homestead",
                "mage_guild_level_1",
                "mage_guild_level_2",
                "mage_guild_level_3",
                "unicorn_glade",
            ],
        ),
        (
            "base:tower",
            "upg_cloud_temple",
            &[
                "altar_of_wishes",
                "cloud_temple",
                "fort",
                "golden_pavilion",
                "golem_factory",
                "mage_guild_level_1",
                "mage_tower",
                "parapet",
                "workshop",
            ],
        ),
        (
            "base:inferno",
            "upg_forsaken_palace",
            &[
                "demon_gate",
                "fire_lake",
                "forsaken_palace",
                "fort",
                "hall_of_sins",
                "hell_hole",
                "imp_crucible",
                "mage_guild_level_1",
            ],
        ),
        (
            "base:necropolis",
            "upg_dragon_vault",
            &[
                "cursed_temple",
                "dragon_vault",
                "estate",
                "fort",
                "graveyard",
                "hall_of_darkness",
                "mage_guild_level_1",
                "mausoleum",
            ],
        ),
        (
            "base:dungeon",
            "upg_dragon_cave",
            &[
                "chapel_of_stilled_voices",
                "dragon_cave",
                "fort",
                "harpy_loft",
                "labyrinth",
                "mage_guild_level_1",
                "mage_guild_level_2",
                "mage_guild_level_3",
                "manticore_lair",
                "pillar_of_eyes",
                "warren",
            ],
        ),
        (
            "base:stronghold",
            "upg_behemoth_lair",
            &[
                "behemoth_lair",
                "cliff_nest",
                "fort",
                "goblin_barracks",
                "wolf_pen",
            ],
        ),
        (
            "base:fortress",
            "upg_hydra_pond",
            &[
                "basilisk_pit",
                "fort",
                "gnoll_hut",
                "hydra_pond",
                "lizard_den",
                "serpent_fly_hive",
                "wyvern_nest",
            ],
        ),
        (
            "ab:conflux",
            "upg_pyre",
            &[
                "altar_of_air",
                "altar_of_earth",
                "altar_of_fire",
                "altar_of_thought",
                "altar_of_water",
                "fort",
                "mage_guild_level_1",
                "magic_lantern",
                "pyre",
            ],
        ),
    ];
    for (f, key, closure) in level_7 {
        let record = faction(f);
        assert_eq!(keys(record, building(record, key).requires), closure, "{f}");
    }
    let conflux = faction("ab:conflux");
    let fire = building(conflux, "upg_altar_of_fire");
    for required in ["altar_of_fire", "upg_altar_of_air"] {
        assert!(
            fire.requires
                .contains(conflux.building_by_key(required).unwrap())
        );
    }
    // The costs in the core's order, each resource by its key.
    let cost = |record: &TownRecord, key: &str| -> Vec<(String, u32)> {
        building(record, key)
            .cost
            .iter()
            .map(|&(r, n)| (core.resources()[r.index()].key.as_str().to_owned(), n))
            .collect()
    };
    let amounts = |a: [u32; 7]| -> Vec<(String, u32)> {
        [
            "wood", "mercury", "ore", "sulfur", "crystal", "gems", "gold",
        ]
        .iter()
        .zip(a)
        .filter(|&(_, n)| n > 0)
        .map(|(r, n)| (format!("base:{r}"), n))
        .collect()
    };
    let castle = faction("base:castle");
    for (key, a) in [
        ("mage_guild_level_1", [5, 0, 5, 0, 0, 0, 2000]),
        ("mage_guild_level_2", [5, 4, 5, 4, 4, 4, 1000]),
        ("mage_guild_level_3", [5, 6, 5, 6, 6, 6, 1000]),
        ("mage_guild_level_4", [5, 8, 5, 8, 8, 8, 1000]),
        ("tavern", [5, 0, 0, 0, 0, 0, 500]),
        ("shipyard", [20, 0, 0, 0, 0, 0, 2000]),
        ("fort", [20, 0, 20, 0, 0, 0, 5000]),
        ("citadel", [0, 0, 5, 0, 0, 0, 2500]),
        ("castle", [10, 0, 10, 0, 0, 0, 5000]),
        ("village_hall", [0, 0, 0, 0, 0, 0, 1]),
        ("town_hall", [0, 0, 0, 0, 0, 0, 2500]),
        ("city_hall", [0, 0, 0, 0, 0, 0, 5000]),
        ("capitol", [0, 0, 0, 0, 0, 0, 10_000]),
        ("marketplace", [5, 0, 0, 0, 0, 0, 500]),
        ("resource_silo", [0, 0, 5, 0, 0, 0, 5000]),
        ("blacksmith", [5, 0, 0, 0, 0, 0, 1000]),
        ("griffin_bastion", [0, 0, 0, 0, 0, 0, 1000]),
        ("upg_griffin_bastion", [0, 0, 0, 0, 0, 0, 1000]),
    ] {
        assert_eq!(cost(castle, key), amounts(a), "{key}");
    }
    let guild_5 = cost(faction("base:tower"), "mage_guild_level_5");
    assert_eq!(guild_5, amounts([5, 10, 5, 10, 10, 10, 1000]));
    let rampart = faction("base:rampart");
    for key in ["dendroid_saplings", "upg_dendroid_saplings"] {
        assert_eq!(cost(rampart, key), amounts([0, 0, 0, 0, 0, 0, 2000]));
    }
    for (key, a) in [
        ("upg_altar_of_air", [2, 2, 0, 0, 0, 2, 1500]),
        ("upg_altar_of_water", [0, 5, 5, 0, 0, 0, 2000]),
        ("upg_altar_of_fire", [0, 5, 5, 0, 0, 0, 2000]),
        ("upg_altar_of_earth", [0, 0, 0, 5, 0, 0, 1000]),
    ] {
        assert_eq!(cost(conflux, key), amounts(a), "{key}");
    }
    // The Grail building alone is free; the Shipyard of the four factions that list it (§6) is
    // the building of the role `shipyard`.
    for (f, shipyard) in [
        ("base:castle", true),
        ("base:rampart", false),
        ("base:tower", false),
        ("base:inferno", false),
        ("base:necropolis", true),
        ("base:dungeon", false),
        ("base:stronghold", false),
        ("base:fortress", true),
        ("ab:conflux", true),
    ] {
        let record = faction(f);
        for b in &record.buildings {
            assert_eq!(b.cost.is_empty(), b.roles.grail, "{}", b.key);
        }
        let shipyards: Vec<&str> = record
            .buildings
            .iter()
            .filter(|b| b.roles.shipyard)
            .map(|b| b.key.as_str())
            .collect();
        assert_eq!(
            shipyards,
            if shipyard { &["shipyard"][..] } else { &[] },
            "{f}"
        );
    }
}
