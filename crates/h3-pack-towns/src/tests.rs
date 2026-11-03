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
    assert!(key(&VALE_OVERRIDE.replace("synthetic:vale", "Vale")));
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
