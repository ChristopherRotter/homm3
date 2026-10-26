//! Test fixtures shared by the crate's tests. The core rulesets are built in code (domain crates
//! take no pack adapter as a dev-dependency).

extern crate std;

use std::string::ToString;
use std::vec;
use std::vec::Vec;

use core::num::{NonZeroU8, NonZeroU16, NonZeroU32};

use h3_content::{
    ContentKey, ContentPack, CreatureDef, CreatureInfoDef, CreatureTypeId, FactionDef,
    HeroEffectDef, HeroTypeId, PackManifest, ProfileDef, ResourceDef, Ruleset, SecondarySkillDef,
    SkillId, TerrainDef, TerrainId,
};
use h3_contracts::{ArmyStack, HeroId, PlayerId};
use h3_heroes::{HeroDetails, OfferLevels};
use h3_kernel::{Rng, StreamId};

use crate::ruleset::{
    AdventurePack, AdventureParamsDef, AdventureRuleset, BattleTerrainsDef, MonsterParamsDef,
    ObjectKindDef, ObjectTypeDef, ObjectTypeId, Passage, PickupDef, RiverDef, RoadDef, RoadId,
    StartingResourcesDef, TerrainMovementDef, TerrainWalkerDef, Visit,
};
use crate::setup::{
    AdventureSetup, HeroSetup, MapSetup, MonsterSetup, ObjectSetup, Pos, TileMask, TileSetup,
};
use crate::state::AdventureState;

/// The setup stream of seed 0, which the tests build states with.
pub(crate) fn draws() -> Rng {
    Rng::for_stream(0, StreamId::SETUP)
}

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

fn creature(k: &str, speed: u8) -> CreatureDef {
    CreatureDef {
        key: key(k),
        speed,
        info: CreatureInfoDef::default(),
    }
}

/// The resources `t:wood` and `t:gold` of [`rules`] and [`movement_rules`].
fn resources() -> Vec<ResourceDef> {
    ["t:wood", "t:gold"]
        .iter()
        .map(|k| ResourceDef {
            key: key(k),
            name: "Resource".into(),
        })
        .collect()
}

/// The starting resources of [`rules`] and [`movement_rules`] (E08 T13a): one difficulty.
fn starting_resources() -> Vec<StartingResourcesDef> {
    vec![StartingResourcesDef {
        human: vec![(key("t:gold"), 1000)],
        computer: vec![],
    }]
}

/// The object types of [`rules`] and [`movement_rules`] (E08 T12a), in handle order.
/// The test rulesets' most creatures in a stack (ADV-ARM-007): the least a part may set.
pub(crate) const MAX_STACK: u32 = crate::MAX_MONSTER_COUNT;

pub(crate) const WOOD_PILE: ObjectTypeId = ObjectTypeId::from_raw(0);
pub(crate) const GOLD_PILE: ObjectTypeId = ObjectTypeId::from_raw(1);
pub(crate) const ANY_PILE: ObjectTypeId = ObjectTypeId::from_raw(2);
pub(crate) const FIRE: ObjectTypeId = ObjectTypeId::from_raw(3);
pub(crate) const TOWER: ObjectTypeId = ObjectTypeId::from_raw(4);
pub(crate) const GUARD: ObjectTypeId = ObjectTypeId::from_raw(5);
pub(crate) const CHART: ObjectTypeId = ObjectTypeId::from_raw(6);
pub(crate) const OPEN_CHART: ObjectTypeId = ObjectTypeId::from_raw(7);
pub(crate) const TENT: ObjectTypeId = ObjectTypeId::from_raw(8);
pub(crate) const GATE: ObjectTypeId = ObjectTypeId::from_raw(9);
pub(crate) const MONSTER: ObjectTypeId = ObjectTypeId::from_raw(10);
pub(crate) const RANDOM_MONSTER: ObjectTypeId = ObjectTypeId::from_raw(11);
pub(crate) const HALLOW: ObjectTypeId = ObjectTypeId::from_raw(12);
pub(crate) const BARRENS: ObjectTypeId = ObjectTypeId::from_raw(13);

/// [`rules`]' object types: a wood pile (3–6), a gold pile (5–10 × 100), a random pile of the two,
/// a fire (2–3 wood and 100 gold per wood), a tower revealing radius 8, a guard of key 0, two
/// Cartographers of grass for 1 000 gold (E08 T14a): a chart not visited from above, as the
/// original's, and an open chart that is; a tent of key 0, entered and not from above, and a gate
/// of key 0 (E08 T15a); a monster of the map's creature, a random monster of level 1 and two
/// grounds fought on the battlefields `t:hallow` and `t:barrens` (E08 T20a2a).
fn object_types() -> Vec<ObjectTypeDef> {
    let n = |v| NonZeroU32::new(v).unwrap();
    let ty = |k: &str, visit, kind| ObjectTypeDef {
        key: key(k),
        name: "Object".into(),
        visit,
        from_above: visit == Visit::Neighbour,
        kind,
    };
    let pile = |resource: &str, amount, multiplier| {
        ObjectKindDef::Pickup(PickupDef::Resource {
            resource: key(resource),
            amount,
            multiplier: n(multiplier),
        })
    };
    vec![
        ty("t:wood_pile", Visit::Neighbour, pile("t:wood", [3, 6], 1)),
        ty(
            "t:gold_pile",
            Visit::Neighbour,
            pile("t:gold", [5, 10], 100),
        ),
        ty(
            "t:any_pile",
            Visit::Neighbour,
            ObjectKindDef::Pickup(PickupDef::OneOf(vec![
                key("t:wood_pile"),
                key("t:gold_pile"),
            ])),
        ),
        ty(
            "t:fire",
            Visit::Neighbour,
            ObjectKindDef::Pickup(PickupDef::Paired {
                amount: [2, 3],
                resources: vec![key("t:wood")],
                paired: key("t:gold"),
                multiplier: n(100),
            }),
        ),
        ty(
            "t:tower",
            Visit::Enter,
            ObjectKindDef::RevealArea { radius: 8 },
        ),
        ty(
            "t:guard",
            Visit::Neighbour,
            ObjectKindDef::KeyGuard { key: 0 },
        ),
        ty("t:chart", Visit::Enter, chart()),
        ObjectTypeDef {
            from_above: true,
            ..ty("t:open_chart", Visit::Enter, chart())
        },
        ty("t:tent", Visit::Enter, ObjectKindDef::KeyTent { key: 0 }),
        ty(
            "t:gate",
            Visit::Neighbour,
            ObjectKindDef::KeyGate { key: 0 },
        ),
        ty(
            "t:monster",
            Visit::Neighbour,
            ObjectKindDef::Monster { levels: None },
        ),
        ty(
            "t:random_monster",
            Visit::Neighbour,
            ObjectKindDef::Monster {
                levels: Some([1, 1]),
            },
        ),
        ty(
            "t:hallow",
            Visit::Enter,
            ObjectKindDef::Ground {
                battlefield: key("t:hallow"),
                no_native: false,
            },
        ),
        ty(
            "t:barrens",
            Visit::Enter,
            ObjectKindDef::Ground {
                battlefield: key("t:barrens"),
                no_native: true,
            },
        ),
    ]
}

/// The monsters' parameters of [`rules`] and [`movement_rules`] (E08 T20a2a): the original's five
/// dispositions and nine words of a count; a group's price to join in `t:gold` (E08 T20b1).
pub(crate) fn monster_params() -> MonsterParamsDef {
    MonsterParamsDef {
        aggression: vec![[-4, -4], [1, 7], [1, 10], [4, 10], [10, 10]],
        count_words: vec![5, 10, 20, 50, 100, 250, 500, 1000],
        price_resource: key("t:gold"),
    }
}

/// A Cartographer's kind: grass for 1 000 gold.
fn chart() -> ObjectKindDef {
    ObjectKindDef::RevealTerrain {
        terrains: vec![key("t:grass")],
        price: vec![(key("t:gold"), 1000)],
    }
}

/// The rules the setup and state tests use: the terrains `t:grass` (land), `t:sea` (water) and
/// `t:rock` (impassable), in that order; the resources `t:wood` and `t:gold`; the creatures
/// `t:imp` (speed 5) and `t:gog` (speed 2); one road and one river; a land table of 1300, 1360,
/// 1430, 1500 for speeds 0 to 3; maps up to 8 × 8 on two levels, two players, armies of three
/// slots; the object types of [`object_types`].
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
        resources: resources(),
        profiles: vec![ProfileDef {
            key: key("t:faithful"),
            quirks: vec![],
        }],
        ..ContentPack::new(manifest("t", &[]))
    };
    let core = Ruleset::compile(&[core], "t:faithful").unwrap();
    let cost = |c| NonZeroU16::new(c).unwrap();
    let movement = |k: &str, passage| TerrainMovementDef {
        terrain: key(k),
        passage,
        pathfinding: None,
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
        object_types: object_types(),
        params: Some(AdventureParamsDef {
            native_cost: cost(100),
            land_points: vec![1300, 1360, 1430, 1500],
            scouting_radius: 5,
            max_map_size: cost(8),
            max_levels: NonZeroU8::new(2).unwrap(),
            max_players: NonZeroU8::new(2).unwrap(),
            army_slots: NonZeroU8::new(3).unwrap(),
            max_stack_count: NonZeroU32::new(MAX_STACK).unwrap(),
            starting_stack_chances: vec![88, 25],
            battle: None,
            monsters: Some(monster_params()),
            fly_cost: [cost(140), cost(140), cost(120), cost(100)],
            water_walk_cost: [cost(140), cost(140), cost(120), cost(100)],
        }),
        starting_resources: starting_resources(),
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
/// ADV-QUIRK-1 to -4, -17 and -18 on) or `t:strict` (all off): the terrains grass (100), sand (150; 125, 100,
/// 100 by Pathfinding level), swamp (175; 150, 125, 100), sea (water) and rock; a dirt road (75) and a cobblestone road (50); the native cost
/// 100; the creatures above (speeds 4, 4, 3, 6); the land table 1000, 1100, 1200, 1300, 1500 for
/// speeds 0 to 4; maps up to 16 × 16 on two levels, two players, armies of seven slots; the
/// resources and object types of [`rules`].
pub(crate) fn movement_rules(faithful: bool) -> (Ruleset, AdventureRuleset) {
    movement_rules_up_to(faithful, 16)
}

/// [`movement_rules`] with maps up to `size` × `size`.
pub(crate) fn movement_rules_up_to(faithful: bool, size: u16) -> (Ruleset, AdventureRuleset) {
    movement_rules_edited(faithful, size, |_| {})
}

/// [`movement_rules_up_to`] with the core pack changed by `edit` before it compiles (E08
/// T19a2b: upgrades).
pub(crate) fn movement_rules_edited(
    faithful: bool,
    size: u16,
    edit: impl FnOnce(&mut ContentPack),
) -> (Ruleset, AdventureRuleset) {
    movement_rules_with(faithful, size, edit, |_| {})
}

/// [`movement_rules`] with the skills of [`hero_skills`] and a terrain walker: an army with a
/// wolf counts sand as swamp (E08 T40; invented, as the original's Nomads count sand as dirt).
pub(crate) fn hero_rules(faithful: bool) -> (Ruleset, AdventureRuleset) {
    movement_rules_with(
        faithful,
        16,
        |core| core.skills = hero_skills(),
        |part| {
            part.terrain_walkers = vec![TerrainWalkerDef {
                creature: key("t:wolf"),
                terrain: key("t:sand"),
                counts_as: key("t:swamp"),
            }];
        },
    )
}

/// The skills of [`hero_rules`], in handle order: Logistics (+10 %, 20 %, 30 % of the land
/// table's entry), Pathfinding (levels 1 to 3) and Scouting (radius +1 to +3), as the original's
/// (E08 T40); flying and water walking at levels 1 to 3 (E08 T41: invented skills, as the
/// original's levels come from Angel Wings, Boots of Levitation and the spells, whose levels the
/// fly and water-walk prices are by: 140, 120, 100 at levels 1 to 3).
pub(crate) fn hero_skills() -> Vec<SecondarySkillDef> {
    let skill = |k: &str, effect: fn(u8) -> HeroEffectDef| SecondarySkillDef {
        key: key(k),
        name: "Skill".into(),
        levels: [1, 2, 3].map(|level| vec![effect(level)]),
    };
    vec![
        skill("t:logistics", |level| HeroEffectDef::LandMovement {
            bp: 1000 * u16::from(level),
        }),
        skill("t:pathfinding", |level| HeroEffectDef::Pathfinding {
            level,
        }),
        skill("t:scouting", |tiles| HeroEffectDef::ScoutingRadius {
            tiles,
        }),
        skill("t:flying", |level| HeroEffectDef::Flying { level }),
        skill("t:water_walking", |level| HeroEffectDef::WaterWalking {
            level,
        }),
    ]
}

/// The skills of [`hero_rules`].
pub(crate) const LOGISTICS: SkillId = SkillId::from_raw(0);
pub(crate) const PATHFINDING: SkillId = SkillId::from_raw(1);
pub(crate) const SCOUTING: SkillId = SkillId::from_raw(2);
pub(crate) const FLYING: SkillId = SkillId::from_raw(3);
pub(crate) const WATER_WALKING: SkillId = SkillId::from_raw(4);

/// A hero's details with these secondary skills and nothing else (a hero type the core lacks:
/// no specialty), for the derivations of [`hero_rules`] (E08 T40).
pub(crate) fn hero_details(skills: &[(SkillId, u8)]) -> HeroDetails {
    HeroDetails {
        hero_type: HeroTypeId::from_raw(u16::MAX),
        level: 1,
        experience: 0,
        offered: OfferLevels::default(),
        primary: [1, 1, 1, 1],
        skills: skills.to_vec(),
        worn: vec![],
        backpack: vec![],
        spells: vec![],
        mana: 0,
    }
}

/// [`movement_rules_edited`] with the adventure pack changed by `edit_part` too.
fn movement_rules_with(
    faithful: bool,
    size: u16,
    edit: impl FnOnce(&mut ContentPack),
    edit_part: impl FnOnce(&mut AdventurePack),
) -> (Ruleset, AdventureRuleset) {
    let adventure_quirks = [
        "quirk.adventure.diagonal_check_ignores_roads",
        "quirk.adventure.diagonal_check_ignores_native",
        "quirk.adventure.native_skips_leading_neutrals",
        "quirk.adventure.fringe_shows_objects",
        "quirk.adventure.drop_ignores_artifacts",
        "quirk.adventure.radius_waits_for_a_step",
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
        resources: resources(),
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
        ..ContentPack::new(manifest("t", &[]))
    };
    let mut core = core;
    edit(&mut core);
    let profile = if faithful { "t:faithful" } else { "t:strict" };
    let core = Ruleset::compile(&[core], profile).unwrap();
    let cost = |c| NonZeroU16::new(c).unwrap();
    let movement = |k: &str, passage| TerrainMovementDef {
        terrain: key(k),
        passage,
        pathfinding: None,
    };
    let road = |k: &str, c| RoadDef {
        key: key(k),
        name: "Road".into(),
        cost: cost(c),
    };
    let part = AdventurePack {
        terrains: vec![
            movement("t:grass", Passage::Land(cost(100))),
            // The original's sand and swamp by Pathfinding level (E08 T29).
            TerrainMovementDef {
                pathfinding: Some([cost(125), cost(100), cost(100)]),
                ..movement("t:sand", Passage::Land(cost(150)))
            },
            TerrainMovementDef {
                pathfinding: Some([cost(150), cost(125), cost(100)]),
                ..movement("t:swamp", Passage::Land(cost(175)))
            },
            movement("t:sea", Passage::Water(cost(100))),
            movement("t:rock", Passage::Impassable),
        ],
        roads: vec![road("t:dirt_road", 75), road("t:cobble_road", 50)],
        object_types: object_types(),
        rivers: vec![RiverDef {
            key: key("t:clear_river"),
            name: "River".into(),
        }],
        params: Some(AdventureParamsDef {
            native_cost: cost(100),
            land_points: vec![1000, 1100, 1200, 1300, 1500],
            scouting_radius: 5,
            max_map_size: cost(size),
            max_levels: NonZeroU8::new(2).unwrap(),
            max_players: NonZeroU8::new(2).unwrap(),
            army_slots: NonZeroU8::new(7).unwrap(),
            max_stack_count: NonZeroU32::new(MAX_STACK).unwrap(),
            starting_stack_chances: vec![88, 25],
            // E08 T19a: battles underground on swamp, on the coast on sand and the shore.
            battle: Some(BattleTerrainsDef {
                underground: key("t:swamp"),
                coast: key("t:sand"),
                coast_battlefield: key("t:shore"),
            }),
            monsters: Some(monster_params()),
            fly_cost: [cost(140), cost(140), cost(120), cost(100)],
            water_walk_cost: [cost(140), cost(140), cost(120), cost(100)],
        }),
        starting_resources: starting_resources(),
        ..AdventurePack::new(manifest("t", &[]))
    };
    let mut part = part;
    edit_part(&mut part);
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
        formation: h3_content::Formation::Loose,
    }
}

/// The tile and object (blocked and visitable masks, object type) a character of [`world`]
/// stands for.
#[allow(clippy::type_complexity)]
fn tile_of(
    c: char,
) -> (
    TerrainId,
    Option<RoadId>,
    Option<(u64, u64, Option<ObjectTypeId>)>,
) {
    match c {
        'G' => (GRASS, Some(COBBLE_ROAD), None),
        'd' => (GRASS, Some(DIRT_ROAD), None),
        's' => (SAND, None, None),
        'S' => (SAND, Some(COBBLE_ROAD), None),
        'w' => (SWAMP, None, None),
        'W' => (SWAMP, Some(COBBLE_ROAD), None),
        '~' => (SEA, None, None),
        '#' => (ROCK, None, None),
        'B' => (GRASS, None, Some((1, 0, None))),
        'V' => (GRASS, None, Some((0, 1, None))),
        'T' => (GRASS, None, Some((1, 1, None))),
        'P' => (GRASS, None, Some((1, 1, Some(WOOD_PILE)))),
        'F' => (GRASS, None, Some((1, 1, Some(FIRE)))),
        'O' => (GRASS, None, Some((0, 1, Some(TOWER)))),
        'K' => (GRASS, None, Some((1, 1, Some(GUARD)))),
        'C' => (GRASS, None, Some((1, 1, Some(CHART)))),
        'c' => (GRASS, None, Some((1, 1, Some(OPEN_CHART)))),
        'Y' => (GRASS, None, Some((1, 1, Some(TENT)))),
        'X' => (GRASS, None, Some((1, 1, Some(GATE)))),
        'M' => (GRASS, None, Some((1, 1, Some(MONSTER)))),
        _ => (GRASS, None, None),
    }
}

/// An adventure setup for two players from text rows, one character per tile (level by level,
/// `levels` levels of equal height): `g` (or any other character) grass, `s` sand, `w` swamp, `~` sea, `#` rock; `G`, `S`,
/// `W` the same with a cobblestone road, `d` grass with a dirt road; on grass, `B` an object's
/// blocked tile, `V` its visitable tile, `T` both (a town's gate), `P` a wood pile blocking and
/// visited on its tile, `F` a fire likewise, `O` a tower's visitable tile (entered), `K` a guard
/// blocking and visited on its tile, `C` a chart (a Cartographer not visited from above) likewise,
/// `c` an open chart (one that is) likewise, `Y` a tent and `X` a gate likewise, `M` five of the
/// first creature as a monster likewise (aggressive; E08 T20a2b).
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
                coast: false,
            });
            if let Some((blocked, visitable, object_type)) = object {
                objects.push(ObjectSetup {
                    scenery: Default::default(),
                    anchor: Pos::new(
                        u16::try_from(x).unwrap(),
                        u16::try_from(index % height).unwrap(),
                        u8::try_from(index / height).unwrap(),
                    ),
                    blocked: TileMask::from_bits(blocked),
                    visitable: TileMask::from_bits(visitable),
                    object_type,
                    amount: None,
                    drawn: TileMask::default(),
                    monster: (object_type == Some(MONSTER)).then(|| MonsterSetup {
                        creature: Some(CreatureTypeId::from_raw(0)),
                        count: 5,
                        disposition: 2,
                        never_flees: false,
                        no_growth: false,
                        treasure: vec![],
                    }),
                    guards: None,
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
        difficulty: 0,
    }
}

/// The state of a setup as the game builds it (E08 T40): [`AdventureState::new`] with
/// [`draws`], then the start of the game ([`AdventureState::start_game`]) with an empty store of
/// heroes (no levels at load: each hero counts without skills, artifacts or a specialty).
pub(crate) fn started(
    rules: &(Ruleset, AdventureRuleset),
    setup: &AdventureSetup,
) -> Result<AdventureState, crate::SetupError> {
    let mut state = AdventureState::new(&rules.0, &rules.1, setup, &mut draws())?;
    state.start_game(&rules.0, &rules.1, &[]);
    Ok(state)
}

/// The state of [`world`] under `rules`, started ([`started`]).
pub(crate) fn world_state(
    rules: &(Ruleset, AdventureRuleset),
    rows: &[&str],
    heroes: Vec<HeroSetup>,
) -> AdventureState {
    started(rules, &world(rows, 1, heroes)).unwrap()
}
