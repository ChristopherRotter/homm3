//! Test fixtures shared by the crate's tests. The core rulesets are built in code (domain crates
//! take no pack adapter as a dev-dependency); the core still needs battle parameters until the
//! content split (ADR-0015 point 3).

extern crate std;

use std::string::ToString;
use std::vec;

use core::num::{NonZeroU8, NonZeroU16};

use h3_content::{
    BattleParamsDef, ContentKey, ContentPack, CreatureDef, CreatureInfoDef, PackManifest,
    ProfileDef, Ruleset, TerrainDef,
};

use crate::ruleset::{
    AdventurePack, AdventureParamsDef, AdventureRuleset, Passage, RiverDef, RoadDef,
    TerrainMovementDef,
};

pub(crate) fn key(text: &str) -> ContentKey {
    ContentKey::parse(text).unwrap()
}

pub(crate) fn manifest(id: &str, depends: &[&str]) -> PackManifest {
    PackManifest {
        id: id.into(),
        version: "1".into(),
        depends: depends.iter().map(|d| (*d).to_string()).collect(),
    }
}

pub(crate) fn battle() -> BattleParamsDef {
    BattleParamsDef {
        field_width: 17,
        field_height: 11,
        side_columns: 1,
        army_slots: 3,
        damage_roll_samples: 10,
        attack_bonus_per_point_percent: 5,
        attack_bonus_cap_percent: 300,
        defense_reduction_per_point_permille: 25,
        defense_reduction_cap_permille: 700,
        defend_bonus_percent: 20,
        defend_bonus_min: 1,
        ranged_full_damage_distance: 10,
        ranged_penalty_percent: 50,
        melee_penalty_percent: 50,
        morale_dice: 24,
        good_morale_chances: vec![1, 2, 3],
        bad_morale_chances: vec![2, 4, 6],
        luck_dice: 24,
        good_luck_chances: vec![1, 2, 3],
        war_machine_positions: vec![[1, 3], [1, 1], [1, 9]],
        loose_formation: vec![vec![5], vec![3, 7], vec![2, 5, 8]],
        tight_formation: vec![vec![5], vec![4, 6], vec![4, 5, 6]],
        tactics_zone_columns: vec![3, 5, 7],
        surrender: None,
        hero_casts_per_round: 1,
        side_stack_limit: 20,
    }
}

fn creature(k: &str) -> CreatureDef {
    CreatureDef {
        key: key(k),
        attack: 1,
        defense: 1,
        damage_min: 1,
        damage_max: 1,
        hit_points: 1,
        speed: 5,
        info: CreatureInfoDef::default(),
    }
}

/// The rules the setup and state tests use: the terrains `t:grass` (land), `t:sea` (water) and
/// `t:rock` (impassable), in that order; the creatures `t:imp` and `t:gog`; one road and one
/// river; maps up to 8 × 8 on two levels, two players, armies of three slots.
pub(crate) fn rules() -> (Ruleset, AdventureRuleset) {
    let core = ContentPack {
        terrains: ["t:grass", "t:sea", "t:rock"]
            .iter()
            .map(|k| TerrainDef {
                key: key(k),
                name: "Terrain".into(),
            })
            .collect(),
        creatures: vec![creature("t:imp"), creature("t:gog")],
        profiles: vec![ProfileDef {
            key: key("t:faithful"),
            quirks: vec![],
        }],
        battle: Some(battle()),
        ..ContentPack::new(manifest("t", &[]))
    };
    let core = Ruleset::compile(&[core], "t:faithful").unwrap();
    let cost = |c| NonZeroU16::new(c).unwrap();
    let movement = |k: &str, passage| TerrainMovementDef {
        terrain: key(k),
        passage,
    };
    let part = AdventurePack {
        terrains: vec![
            movement("t:grass", Passage::Land(cost(100))),
            movement("t:sea", Passage::Water(cost(100))),
            movement("t:rock", Passage::Impassable),
        ],
        roads: vec![RoadDef {
            key: key("t:dirt_road"),
            name: "Road".into(),
            cost: cost(75),
        }],
        rivers: vec![RiverDef {
            key: key("t:clear_river"),
            name: "River".into(),
        }],
        params: Some(AdventureParamsDef {
            native_cost: cost(100),
            land_points: vec![1500],
            scouting_radius: 5,
            max_map_size: cost(8),
            max_levels: NonZeroU8::new(2).unwrap(),
            max_players: NonZeroU8::new(2).unwrap(),
            army_slots: NonZeroU8::new(3).unwrap(),
        }),
        ..AdventurePack::new(manifest("t", &[]))
    };
    let part = AdventureRuleset::compile(&core, &[part]).unwrap();
    (core, part)
}
