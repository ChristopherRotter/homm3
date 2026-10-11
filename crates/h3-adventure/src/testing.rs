//! Test fixtures shared by the crate's tests. The core rulesets are built in code (domain crates
//! take no pack adapter as a dev-dependency); the core still needs battle parameters until the
//! content split (ADR-0015 point 3).

extern crate std;

use std::string::ToString;
use std::vec;
use std::vec::Vec;

use core::num::{NonZeroU8, NonZeroU16};

use h3_content::{
    BattleParamsDef, ContentKey, ContentPack, CreatureDef, CreatureInfoDef, CreatureTypeId,
    FactionDef, PackManifest, ProfileDef, Ruleset, TerrainDef, TerrainId,
};
use h3_contracts::{ArmyStack, HeroId, PlayerId};

use crate::ruleset::{
    AdventurePack, AdventureParamsDef, AdventureRuleset, Passage, RiverDef, RoadDef, RoadId,
    TerrainMovementDef,
};
use crate::setup::{
    AdventureSetup, HeroSetup, MapSetup, ObjectKind, ObjectSetup, Pos, TileMask, TileSetup,
};
use crate::state::AdventureState;

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

fn creature(k: &str, speed: u8) -> CreatureDef {
    CreatureDef {
        key: key(k),
        attack: 1,
        defense: 1,
        damage_min: 1,
        damage_max: 1,
        hit_points: 1,
        speed,
        info: CreatureInfoDef::default(),
    }
}

/// The rules the setup and state tests use: the terrains `t:grass` (land), `t:sea` (water) and
/// `t:rock` (impassable), in that order; the creatures `t:imp` (speed 5) and `t:gog` (speed 2);
/// one road and one river; a land table of 1300, 1360, 1430, 1500 for speeds 0 to 3; maps up to
/// 8 × 8 on two levels, two players, armies of three slots.
pub(crate) fn rules() -> (Ruleset, AdventureRuleset) {
    let core = ContentPack {
        terrains: ["t:grass", "t:sea", "t:rock"]
            .iter()
            .map(|k| TerrainDef {
                key: key(k),
                name: "Terrain".into(),
            })
            .collect(),
        creatures: vec![creature("t:imp", 5), creature("t:gog", 2)],
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
            land_points: vec![1300, 1360, 1430, 1500],
            scouting_radius: 5,
            max_map_size: cost(8),
            max_levels: NonZeroU8::new(2).unwrap(),
            max_players: NonZeroU8::new(2).unwrap(),
            army_slots: NonZeroU8::new(3).unwrap(),
            starting_stack_chances: vec![88, 25],
        }),
        ..AdventurePack::new(manifest("t", &[]))
    };
    let part = AdventureRuleset::compile(&core, &[part]).unwrap();
    (core, part)
}

/// The terrains of [`movement_rules`], in core order.
pub(crate) const GRASS: TerrainId = TerrainId::from_raw(0);
pub(crate) const SAND: TerrainId = TerrainId::from_raw(1);
pub(crate) const SWAMP: TerrainId = TerrainId::from_raw(2);
pub(crate) const SEA: TerrainId = TerrainId::from_raw(3);
pub(crate) const ROCK: TerrainId = TerrainId::from_raw(4);
/// The roads of [`movement_rules`].
pub(crate) const DIRT_ROAD: RoadId = RoadId::from_raw(0);
pub(crate) const COBBLE_ROAD: RoadId = RoadId::from_raw(1);
/// The creatures of [`movement_rules`]: a pikeman of the castle (native to grass), a gnoll of the
/// fortress (swamp), a neutral peasant, a wolf of a faction without a native terrain.
pub(crate) const PIKEMAN: CreatureTypeId = CreatureTypeId::from_raw(0);
pub(crate) const GNOLL: CreatureTypeId = CreatureTypeId::from_raw(1);
pub(crate) const PEASANT: CreatureTypeId = CreatureTypeId::from_raw(2);
pub(crate) const WOLF: CreatureTypeId = CreatureTypeId::from_raw(3);

fn faction_creature(k: &str, speed: u8, faction: Option<&str>) -> CreatureDef {
    CreatureDef {
        info: CreatureInfoDef {
            faction: faction.map(key),
            ..CreatureInfoDef::default()
        },
        ..creature(k, speed)
    }
}

/// The rules the movement tests use, under the profile `t:faithful` (the adventure quirks
/// ADV-QUIRK-1 to -4 on) or `t:strict` (all off): the terrains grass (100), sand (150), swamp
/// (175), sea (water) and rock; a dirt road (75) and a cobblestone road (50); the native cost
/// 100; the creatures above (speeds 4, 4, 3, 6); the land table 1000, 1100, 1200, 1300, 1500 for
/// speeds 0 to 4; maps up to 16 × 16 on two levels, two players, armies of seven slots.
pub(crate) fn movement_rules(faithful: bool) -> (Ruleset, AdventureRuleset) {
    let adventure_quirks = [
        "quirk.adventure.diagonal_check_ignores_roads",
        "quirk.adventure.diagonal_check_ignores_native",
        "quirk.adventure.native_skips_leading_neutrals",
        "quirk.adventure.fringe_shows_objects",
    ];
    let core = ContentPack {
        terrains: ["t:grass", "t:sand", "t:swamp", "t:sea", "t:rock"]
            .iter()
            .map(|k| TerrainDef {
                key: key(k),
                name: "Terrain".into(),
            })
            .collect(),
        factions: [
            ("t:castle", Some("t:grass")),
            ("t:fortress", Some("t:swamp")),
            ("t:wild", None),
        ]
        .iter()
        .map(|&(k, native)| FactionDef {
            key: key(k),
            name: "Faction".into(),
            native_terrain: native.map(key),
        })
        .collect(),
        creatures: vec![
            faction_creature("t:pikeman", 4, Some("t:castle")),
            faction_creature("t:gnoll", 4, Some("t:fortress")),
            faction_creature("t:peasant", 3, None),
            faction_creature("t:wolf", 6, Some("t:wild")),
        ],
        profiles: vec![
            ProfileDef {
                key: key("t:faithful"),
                quirks: adventure_quirks
                    .iter()
                    .map(|q| ((*q).to_string(), true))
                    .collect(),
            },
            ProfileDef {
                key: key("t:strict"),
                quirks: vec![],
            },
        ],
        battle: Some(battle()),
        ..ContentPack::new(manifest("t", &[]))
    };
    let profile = if faithful { "t:faithful" } else { "t:strict" };
    let core = Ruleset::compile(&[core], profile).unwrap();
    let cost = |c| NonZeroU16::new(c).unwrap();
    let movement = |k: &str, passage| TerrainMovementDef {
        terrain: key(k),
        passage,
    };
    let road = |k: &str, c| RoadDef {
        key: key(k),
        name: "Road".into(),
        cost: cost(c),
    };
    let part = AdventurePack {
        terrains: vec![
            movement("t:grass", Passage::Land(cost(100))),
            movement("t:sand", Passage::Land(cost(150))),
            movement("t:swamp", Passage::Land(cost(175))),
            movement("t:sea", Passage::Water(cost(100))),
            movement("t:rock", Passage::Impassable),
        ],
        roads: vec![road("t:dirt_road", 75), road("t:cobble_road", 50)],
        rivers: vec![RiverDef {
            key: key("t:clear_river"),
            name: "River".into(),
        }],
        params: Some(AdventureParamsDef {
            native_cost: cost(100),
            land_points: vec![1000, 1100, 1200, 1300, 1500],
            scouting_radius: 5,
            max_map_size: cost(16),
            max_levels: NonZeroU8::new(2).unwrap(),
            max_players: NonZeroU8::new(2).unwrap(),
            army_slots: NonZeroU8::new(7).unwrap(),
            starting_stack_chances: vec![88, 25],
        }),
        ..AdventurePack::new(manifest("t", &[]))
    };
    let part = AdventureRuleset::compile(&core, &[part]).unwrap();
    (core, part)
}

/// An army of one creature per slot, from slot 0.
pub(crate) fn army(creatures: &[CreatureTypeId]) -> Vec<ArmyStack> {
    creatures
        .iter()
        .zip(0..)
        .map(|(&creature, slot)| ArmyStack {
            slot,
            creature,
            count: 1,
        })
        .collect()
}

/// A hero of `owner` on `pos` with an army of `creatures`.
pub(crate) fn hero_at(id: u16, owner: u8, pos: Pos, creatures: &[CreatureTypeId]) -> HeroSetup {
    HeroSetup {
        id: HeroId::new(id),
        owner: PlayerId::new(owner),
        pos,
        army: army(creatures),
    }
}

/// The tile and object a character of [`world`] stands for.
fn tile_of(c: char) -> (TerrainId, Option<RoadId>, Option<(u64, u64)>) {
    match c {
        'G' => (GRASS, Some(COBBLE_ROAD), None),
        'd' => (GRASS, Some(DIRT_ROAD), None),
        's' => (SAND, None, None),
        'S' => (SAND, Some(COBBLE_ROAD), None),
        'w' => (SWAMP, None, None),
        'W' => (SWAMP, Some(COBBLE_ROAD), None),
        '~' => (SEA, None, None),
        '#' => (ROCK, None, None),
        'B' => (GRASS, None, Some((1, 0))),
        'V' => (GRASS, None, Some((0, 1))),
        'T' => (GRASS, None, Some((1, 1))),
        _ => (GRASS, None, None),
    }
}

/// An adventure setup for two players from text rows, one character per tile (level by level,
/// `levels` levels of equal height): `g` (or any other character) grass, `s` sand, `w` swamp, `~` sea, `#` rock; `G`, `S`,
/// `W` the same with a cobblestone road, `d` grass with a dirt road; on grass, `B` an object's
/// blocked tile, `V` its visitable tile, `T` both (a town's gate).
pub(crate) fn world(rows: &[&str], levels: u8, heroes: Vec<HeroSetup>) -> AdventureSetup {
    let height = rows.len() / usize::from(levels);
    let width = rows[0].chars().count();
    let mut tiles = Vec::new();
    let mut objects = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            let (terrain, road, object) = tile_of(c);
            tiles.push(TileSetup {
                terrain,
                road,
                river: None,
            });
            if let Some((blocked, visitable)) = object {
                objects.push(ObjectSetup {
                    anchor: Pos::new(
                        u16::try_from(x).unwrap(),
                        u16::try_from(index % height).unwrap(),
                        u8::try_from(index / height).unwrap(),
                    ),
                    blocked: TileMask::from_bits(blocked),
                    visitable: TileMask::from_bits(visitable),
                    kind: ObjectKind::Blocker,
                });
            }
        }
    }
    AdventureSetup {
        players: 2,
        map: MapSetup {
            width: u16::try_from(width).unwrap(),
            height: u16::try_from(height).unwrap(),
            levels,
            tiles,
            objects,
        },
        heroes,
    }
}

/// The state of [`world`] under `rules`.
pub(crate) fn world_state(
    rules: &(Ruleset, AdventureRuleset),
    rows: &[&str],
    heroes: Vec<HeroSetup>,
) -> AdventureState {
    AdventureState::new(&rules.0, &rules.1, &world(rows, 1, heroes)).unwrap()
}
