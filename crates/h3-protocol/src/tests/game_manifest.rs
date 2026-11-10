//! The whole game's manifest (E16 M5 T4): it names what an adventure replay's header names, with
//! the battle part's hash and the map a replay names; its key tables follow the ruleset's
//! handles; it round-trips.

use std::string::String;
use std::vec::Vec;
use std::{format, fs};

use h3_content::FactionId;
use h3_kernel::handle::Handle;
use h3_sim::adventure::replay::map_content_hash;

use super::game::{RULES, round_trips};
use crate::map;
use crate::map::game::{key_tables, manifest};
use crate::wire::WireU64;
use crate::wire::game::MapIdentity;

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
