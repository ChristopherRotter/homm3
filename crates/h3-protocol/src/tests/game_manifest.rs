//! The whole game's manifest (E16 M5 T4): it names what an adventure replay's header names, with
//! the battle part's hash and the map a replay names; its key tables follow the ruleset's
//! handles; it round-trips, and so do a game record's header (T5b) and a seat's `GameJoined`
//! (T8a) holding it.

use std::string::String;
use std::vec::Vec;
use std::{format, fs};

use h3_content::FactionId;
use h3_kernel::handle::Handle;
use h3_sim::adventure::replay::map_content_hash;

use super::game::{RULES, round_trips};
use crate::map;
use crate::map::game::{key_tables, manifest, map_offer};
use crate::wire::game::{Computer, GameRecordHeader, MapIdentity, MapOffer};
use crate::wire::{ServerMessage, WireU64};

/// A golden adventure replay's map, as the manifest names it.
fn golden_map() -> MapIdentity {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let file = root.join("fixtures/replays/adventure/towns-capitol-s1.toml");
    let replay = h3_files::adventure::parse_replay(&fs::read_to_string(file).unwrap()).unwrap();
    let bytes = fs::read(root.join(&replay.map.path)).unwrap();
    assert_eq!(map_content_hash(&bytes), replay.map.content_hash);
    MapIdentity {
        content_hash: WireU64(replay.map.content_hash),
        import_version: replay.map.import_version,
    }
}

#[test]
fn the_game_manifest_names_what_an_adventure_replay_names() {
    let rules = &*RULES;
    let at = golden_map();
    let m = manifest(rules, at);
    let h = h3_sim::adventure::replay::ReplayHeader::current(rules);
    assert_eq!(m.protocol, crate::PROTOCOL_VERSION);
    assert_eq!(m.engine, h.engine_version);
    let packs: Vec<(String, String)> = m
        .packs
        .iter()
        .map(|p| (p.id.clone(), p.version.clone()))
        .collect();
    assert_eq!(packs, h.packs);
    assert_eq!(m.profile, h.profile);
    assert_eq!(
        (
            m.ruleset_hash.0,
            m.adventure_ruleset_hash.0,
            m.town_ruleset_hash.0
        ),
        (
            h.ruleset_hash,
            h.adventure_ruleset_hash,
            h.town_ruleset_hash
        )
    );
    // Every part's hash (ADR-0015 point 2): the battle part's too, as a battle's manifest has it.
    let battle = map::manifest(rules.battle());
    assert_eq!(m.battle_ruleset_hash, battle.battle_ruleset_hash);
    assert_eq!(
        (m.rng.name.as_str(), m.rng.version),
        (h.rng_algorithm.as_str(), h.rng_version)
    );
    assert_eq!(
        (m.hash.name.as_str(), m.hash.version),
        (h.hash_algorithm.as_str(), h.hash_version)
    );
    assert_eq!(
        (
            m.battle_state_hash_version,
            m.adventure_state_hash_version,
            m.game_state_hash_version
        ),
        (
            battle.battle_state_hash_version,
            h.adventure_state_hash_version,
            h.game_state_hash_version
        )
    );
    assert_eq!(m.map, at);
    assert_eq!(at.import_version, h3_map::MAP_IMPORT_VERSION);
    assert_eq!((&m.field, &m.keys), (&battle.field, &battle.keys));
    assert_eq!(m.game_keys, key_tables(rules));
    round_trips(&m);
}

#[test]
fn a_map_offer_names_the_maps_size_players_and_rules() {
    let rules = &*RULES;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (_, setup) =
        h3_files::adventure::load_map(&root, "fixtures/maps/e08-duel-36.h3m", rules).unwrap();
    let at = golden_map();
    // E16 M5 T10: the offer names its computer.
    let all_seeing = ("duel", String::from("Duel"));
    let offer = map_offer(all_seeing, (rules, &setup), at, Computer::AllSeeing);
    let m = manifest(rules, at);
    assert_eq!(
        offer,
        MapOffer {
            id: String::from("duel"),
            name: String::from("Duel"),
            size: 36,
            levels: 2,
            players: Vec::from([0, 1]),
            difficulty: 1,
            packs: m.packs,
            profile: m.profile,
            ruleset_hash: m.ruleset_hash,
            map: at,
            computer: Computer::AllSeeing,
        }
    );
    round_trips(&offer);
    round_trips(&Computer::Fair);
}

#[test]
fn the_game_key_tables_follow_the_handles() {
    let rules = &*RULES;
    let (core, adventure, towns) = (rules.core(), rules.adventure(), rules.towns());
    let keys = key_tables(rules);
    assert_eq!(keys.object_types.len(), adventure.object_types().len());
    for (i, key) in keys.object_types.iter().enumerate() {
        assert_eq!(
            adventure.object_type_by_key(key).map(Handle::index),
            Some(i)
        );
    }
    for (i, key) in keys.roads.iter().enumerate() {
        assert_eq!(adventure.road_by_key(key).map(Handle::index), Some(i));
    }
    for (i, key) in keys.rivers.iter().enumerate() {
        assert_eq!(adventure.river_by_key(key).map(Handle::index), Some(i));
    }
    let resources: Vec<&str> = core.resources().iter().map(|r| r.key.as_str()).collect();
    assert_eq!(keys.resources, resources);
    // Each faction's town record by the faction's handle, its buildings by index.
    assert_eq!(keys.buildings.len(), core.factions().len());
    let mut with_town = 0;
    for (i, buildings) in keys.buildings.iter().enumerate() {
        let record = towns.town(FactionId::from_index(i).unwrap());
        assert_eq!(buildings.is_empty(), record.is_none(), "faction {i}");
        for (b, key) in buildings.iter().enumerate() {
            let id = record.and_then(|r| r.building_by_key(key));
            assert_eq!(id.map(|id| id.index()), Some(b), "faction {i}: {key}");
        }
        with_town += usize::from(record.is_some());
    }
    assert_eq!(with_town, towns.towns().len());
    let kinds = [
        &keys.object_types,
        &keys.resources,
        &keys.roads,
        &keys.rivers,
    ];
    assert!(kinds.iter().all(|k| !k.is_empty()) && with_town > 0);
}

#[test]
fn the_game_manifests_json_is_serdes_default_form() {
    let at = MapIdentity {
        content_hash: WireU64(u64::MAX),
        import_version: 3,
    };
    let text = serde_json::to_string(&at).unwrap();
    assert_eq!(
        text,
        format!(r#"{{"content_hash":"{}","import_version":3}}"#, u64::MAX)
    );
}

#[test]
fn a_game_records_header_round_trips_in_serdes_default_form() {
    let rules = &*RULES;
    let manifest = manifest(rules, golden_map());
    let header = GameRecordHeader {
        format: crate::RECORD_FORMAT_VERSION,
        game: 4,
        player: 1,
        manifest: manifest.clone(),
        first: 3,
    };
    round_trips(&header);
    let text = serde_json::to_string(&header).unwrap();
    let manifest = serde_json::to_string(&manifest).unwrap();
    assert_eq!(
        text,
        format!(r#"{{"format":4,"game":4,"player":1,"manifest":{manifest},"first":3}}"#)
    );
}

/// E16 M5 T8a (v45): a seat's `GameJoined` carries the game's manifest, as its record's header;
/// its last number since T8b (v46).
#[test]
fn a_seats_joined_round_trips_in_serdes_default_form() {
    let manifest = manifest(&RULES, golden_map());
    let joined = ServerMessage::GameJoined {
        game: 4,
        player: 1,
        token: Some("t".into()),
        manifest: manifest.clone(),
        last: Some(5),
    };
    round_trips(&joined);
    let text = serde_json::to_string(&joined).unwrap();
    let manifest = serde_json::to_string(&manifest).unwrap();
    assert_eq!(
        text,
        format!(
            r#"{{"GameJoined":{{"game":4,"player":1,"token":"t","manifest":{manifest},"last":5}}}}"#
        )
    );
}
