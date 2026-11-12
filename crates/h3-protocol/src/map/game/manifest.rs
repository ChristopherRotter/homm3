//! The whole game's manifest (E16 M5 T4; ADR-0018 point 10): the rules' identity from the
//! game's ruleset, the map from its caller (the protocol reads no files), the key tables.

use alloc::string::String;
use alloc::vec::Vec;

use h3_content::FactionId;
use h3_game::{GameRuleset, GameSetup};
use h3_kernel::hash::{HASH_ALGORITHM, HASH_VERSION};
use h3_kernel::rng::{RNG_ALGORITHM, RNG_VERSION};

use crate::PROTOCOL_VERSION;
use crate::map::{field, key_tables as battle_key_tables, wide};
use crate::wire::game::{Computer, GameKeyTables, GameManifest, MapIdentity, MapOffer};
use crate::wire::{Algorithm, PackRef};

/// What a whole game on `rules` and the map `map` is made of: the adventure replay header's
/// fields (`h3_sim::adventure::replay::ReplayHeader`) with the battle part's hash, the map, the
/// battlefield's size and the key tables.
pub fn manifest(rules: &GameRuleset, map: MapIdentity) -> GameManifest {
    let core = rules.core();
    GameManifest {
        protocol: PROTOCOL_VERSION,
        engine: String::from(env!("CARGO_PKG_VERSION")),
        packs: core
            .packs()
            .iter()
            .map(|(id, version)| PackRef {
                id: id.clone(),
                version: version.clone(),
            })
            .collect(),
        profile: String::from(core.profile().as_str()),
        ruleset_hash: wide(core.content_hash()),
        battle_ruleset_hash: wide(rules.battle().content_hash()),
        adventure_ruleset_hash: wide(rules.adventure().content_hash()),
        town_ruleset_hash: wide(rules.towns().content_hash()),
        rng: Algorithm {
            name: String::from(RNG_ALGORITHM),
            version: RNG_VERSION,
        },
        hash: Algorithm {
            name: String::from(HASH_ALGORITHM),
            version: HASH_VERSION,
        },
        battle_state_hash_version: h3_battle::BATTLE_STATE_HASH_VERSION,
        adventure_state_hash_version: h3_adventure::ADVENTURE_STATE_HASH_VERSION,
        game_state_hash_version: h3_game::GAME_STATE_HASH_VERSION,
        map,
        field: field(rules.battle()),
        keys: battle_key_tables(rules.battle()),
        game_keys: key_tables(rules),
    }
}

/// The lobby's map offer `id` (E16 M5 T9a): the map `map` named `name`, its size, levels and
/// players from `setup` (each player a seat, by colour), the game's difficulty from it too, on
/// `rules` (the manifest's packs, profile and core hash); `computer` plays its unowned battle
/// sides (T10).
pub fn map_offer(
    (id, name): (&str, String),
    (rules, setup): (&GameRuleset, &GameSetup),
    map: MapIdentity,
    computer: Computer,
) -> MapOffer {
    let GameManifest {
        packs,
        profile,
        ruleset_hash,
        ..
    } = manifest(rules, map);
    MapOffer {
        id: String::from(id),
        name,
        size: setup.map.width,
        levels: setup.map.levels,
        players: setup.players.iter().map(|p| p.colour).collect(),
        difficulty: setup.difficulty,
        packs,
        profile,
        ruleset_hash,
        map,
        computer,
    }
}

/// The content keys by handle of the kinds only the whole game names (the manifest's
/// [`GameKeyTables`]).
pub fn key_tables(rules: &GameRuleset) -> GameKeyTables {
    let (core, adventure, towns) = (rules.core(), rules.adventure(), rules.towns());
    let texts = |keys: &mut dyn Iterator<Item = &h3_content::ContentKey>| {
        keys.map(|k| String::from(k.as_str())).collect()
    };
    GameKeyTables {
        object_types: texts(&mut adventure.object_types().iter().map(|o| &o.key)),
        resources: texts(&mut core.resources().iter().map(|r| &r.key)),
        roads: texts(&mut adventure.roads().iter().map(|r| &r.key)),
        rivers: texts(&mut adventure.rivers().iter().map(|r| &r.key)),
        buildings: (0..core.factions().len())
            .map(|i| {
                FactionId::from_index(i)
                    .and_then(|faction| towns.town(faction))
                    .map_or_else(Vec::new, |record| {
                        record.buildings.iter().map(|b| b.key.clone()).collect()
                    })
            })
            .collect(),
    }
}
