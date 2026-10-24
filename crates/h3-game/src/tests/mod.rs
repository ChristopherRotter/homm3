//! Tests of the game's rules bundle and setup (E08 T6a); the state, the starting armies' draw
//! and the turns in `state.rs` (T6b); the observation and the projection in `observe.rs`
//! (T8b1); the legal commands in `legal.rs` (T8b2a); the pending decisions in `decisions.rs`
//! (T14b); battles in `battles.rs` (T19a); the level-ups in play in `level_ups.rs` (T30). The rulesets are built in code (an application crate
//! takes no pack adapter as a dev-dependency).

mod aftermath;
mod battles;
mod day;
mod decisions;
mod heroes;
mod legal;
mod level_ups;
mod levels;
mod monsters;
mod moves;
mod observe;
mod state;

extern crate std;

use std::string::ToString;
use std::vec;
use std::vec::Vec;

use alloc::sync::Arc;
use core::num::{NonZeroU8, NonZeroU16, NonZeroU32};

use h3_adventure::{
    AdventurePack, AdventureParamsDef, AdventureRuleset, ArmyError, BattleTerrainsDef, MapSetup,
    MonsterParamsDef, ObjectKindDef, ObjectTypeDef, Passage, PickupDef, Pos, SetupError,
    StandError, StartingResourcesDef, TerrainMovementDef, TileSetup, Visit,
};
use h3_content::{
    ArtifactClass, ArtifactDef, ArtifactSlotDef, ContentKey, ContentPack, CreatureDef,
    CreatureInfoDef, CreatureTypeId, FactionDef, Formation, HeroClassDef, HeroEffect,
    HeroParamsDef, HeroTypeDef, HeroTypeId, PackManifest, ProfileDef, RaisedCreature, ResourceDef,
    Ruleset, SecondarySkillDef, SkillId, SpellCategory, SpellDef, SpellSchoolDef, StartingStackDef,
    TerrainDef, TerrainId,
};
use h3_content_battle::{
    AbilityDef, BattleCreatureDef, BattlePack, BattleParamsDef, BattleRuleset, BattleSpellDef,
    BattlefieldDef, ObstacleDef, SpellAi, SpellMagic, SpellPolarity, SurrenderDef,
};
use h3_contracts::{ArmyStack, HeroId, PlayerId};

use super::*;
use crate::treasury::Treasuries;

fn key(text: &str) -> ContentKey {
    ContentKey::parse(text).unwrap()
}

fn creature(k: &str, speed: u8) -> CreatureDef {
    CreatureDef {
        key: key(k),
        speed,
        info: CreatureInfoDef::default(),
    }
}

fn war_machine(k: &str) -> CreatureDef {
    CreatureDef {
        info: CreatureInfoDef {
            flags: vec!["war_machine".into()],
            ..CreatureInfoDef::default()
        },
        ..creature(k, 0)
    }
}

fn manifest() -> PackManifest {
    PackManifest {
        id: "t".into(),
        version: "1".into(),
        depends: vec![],
    }
}

fn hero_type(k: &str, army: &[(&str, [u16; 2])], war_machines: &[&str]) -> HeroTypeDef {
    HeroTypeDef {
        key: key(k),
        name: "Hero".into(),
        class: key("t:knight"),
        skills: vec![],
        spell_book: false,
        spells: vec![],
        army: army
            .iter()
            .map(|(c, count)| StartingStackDef {
                creature: key(c),
                count: *count,
            })
            .collect(),
        war_machines: war_machines.iter().map(|m| key(m)).collect(),
        specialty: vec![],
    }
}

/// A core ruleset: the terrains `t:grass` and `t:rock`, the creatures `t:imp` (speed 5, 50
/// gold), `t:gog` (speed 2, 100 gold) and the war machine `t:cart`, and the hero types `t:orrin` (imp 2-5, gog
/// 1-3), `t:loner` (no army), `t:crowd` (three stacks) and `t:gunner` (imp 1-1, the cart, gog
/// 2-2), in that order; `extra` adds a creature. The profile `t:faithful` enables no quirk,
/// `t:tile` ADV-QUIRK-5's (E08 T19a).
fn core(extra: bool) -> Ruleset {
    core_with(extra, "t:faithful")
}

/// [`core`] with the profile `profile`.
fn core_with(extra: bool, profile: &str) -> Ruleset {
    Ruleset::compile(&[core_pack(extra)], profile).unwrap()
}

/// [`core`]'s pack, before it compiles (E08 T20b2: the monsters' tests give it AI values).
fn core_pack(extra: bool) -> ContentPack {
    // The imp costs 50 gold and the gog 100: a surrender's price (E08 T19a).
    let costing = |k: &str, speed, gold| CreatureDef {
        info: CreatureInfoDef {
            cost: vec![(key("t:gold"), gold)],
            ..CreatureInfoDef::default()
        },
        ..creature(k, speed)
    };
    let mut creatures = vec![
        costing("t:imp", 5, 50),
        costing("t:gog", 2, 100),
        war_machine("t:cart"),
    ];
    if extra {
        creatures.push(creature("t:elf", 5));
    }
    ContentPack {
        terrains: ["t:grass", "t:rock"]
            .iter()
            .map(|k| TerrainDef {
                key: key(k),
                name: "Terrain".into(),
            })
            .collect(),
        creatures,
        factions: vec![FactionDef {
            key: key("t:town"),
            name: "Town".into(),
            native_terrain: None,
        }],
        hero_classes: vec![HeroClassDef {
            key: key("t:knight"),
            name: "Knight".into(),
            faction: key("t:town"),
            primary: [1, 1, 1, 1],
            primary_gain: [[25, 25, 25, 25], [25, 25, 25, 25]],
            // E08 T29: every skill may be offered at a level-up.
            skill_weights: vec![
                (key("t:wits"), 4),
                (key("t:lore"), 3),
                (key("t:dark"), 2),
                (key("t:sense"), 1),
            ],
            offer_intervals: [6, 4],
        }],
        heroes: vec![
            hero_type("t:orrin", &[("t:imp", [2, 5]), ("t:gog", [1, 3])], &[]),
            hero_type("t:loner", &[], &[]),
            hero_type(
                "t:crowd",
                &[("t:imp", [1, 1]), ("t:imp", [1, 1]), ("t:gog", [1, 1])],
                &[],
            ),
            hero_type(
                "t:gunner",
                &[("t:imp", [1, 1]), ("t:gog", [2, 2])],
                &["t:cart"],
            ),
            // A spell book and Expert Wits (E08 T18b).
            HeroTypeDef {
                skills: vec![(key("t:wits"), 3)],
                spell_book: true,
                ..hero_type("t:sage", &[("t:imp", [1, 1])], &[])
            },
        ],
        resources: vec![ResourceDef {
            key: key("t:gold"),
            name: "Gold".into(),
        }],
        profiles: vec![
            ProfileDef {
                key: key("t:faithful"),
                quirks: vec![],
            },
            ProfileDef {
                key: key("t:tile"),
                quirks: vec![("quirk.adventure.obstacles_from_tile".to_string(), true)],
            },
            // The original's single-precision rates, Q-0005 (E08 T19a2b: Necromancy's count).
            ProfileDef {
                key: key("t:single"),
                quirks: vec![("quirk.battle.single_precision_rates".to_string(), true)],
            },
            // A monster's split from its tile, ADV-QUIRK-6 (E08 T20b2).
            ProfileDef {
                key: key("t:split"),
                quirks: vec![("quirk.adventure.split_from_tile".to_string(), true)],
            },
            // A level's draws from the hero's seed, ADV-QUIRK-9 (E08 T29).
            ProfileDef {
                key: key("t:seeded"),
                quirks: vec![("quirk.adventure.level_up_from_seed".to_string(), true)],
            },
        ],
        // The hero model's content (E08 T18b): a book and a machine slot, the spell book, the
        // cart's artifact, a charm and a scroll without slot, a spell, Wits (+25/50/100 % spell points) and the hero parameters;
        // since T19a2a two ring positions with a ring, and Lore (+5/10/15 % experience).
        artifact_slots: [
            ("t:book_slot", 1, false),
            ("t:machine", 1, false),
            ("t:ring_slot", 2, true),
        ]
        .iter()
        .map(|&(k, count, backpack)| ArtifactSlotDef {
            key: key(k),
            name: "Slot".into(),
            count,
            backpack,
        })
        .collect(),
        artifacts: vec![
            ArtifactDef {
                effects: vec![HeroEffect::SpellBook],
                ..artifact("t:book", "t:book_slot")
            },
            ArtifactDef {
                war_machine: Some(key("t:cart")),
                ..artifact("t:cart_art", "t:machine")
            },
            // Carried in the backpack only; a scroll holds a spell of its own.
            ArtifactDef {
                slot: None,
                ..artifact("t:charm", "t:machine")
            },
            ArtifactDef {
                slot: None,
                effects: vec![HeroEffect::Scroll],
                ..artifact("t:scroll", "t:machine")
            },
            artifact("t:ring", "t:ring_slot"),
        ],
        spell_schools: vec![SpellSchoolDef {
            key: key("t:air"),
            name: "Air".into(),
            protection_rank: 0,
        }],
        spells: vec![
            SpellDef {
                key: key("t:gust"),
                name: "Gust".into(),
                category: SpellCategory::Combat,
                level: 1,
                schools: vec![key("t:air")],
                cost: [5, 4, 4, 4],
                power: 10,
                effect: [10, 10, 20, 30],
                granted_by_name_only: false,
            },
            // A level 3 spell (E08 T19a2b: Eagle Eye's learning needs Basic Sense).
            SpellDef {
                key: key("t:gate"),
                name: "Gate".into(),
                category: SpellCategory::Adventure,
                level: 3,
                schools: vec![key("t:air")],
                cost: [5, 4, 4, 4],
                power: 10,
                effect: [10, 10, 20, 30],
                granted_by_name_only: false,
            },
        ],
        skills: vec![
            SecondarySkillDef {
                key: key("t:wits"),
                name: "Wits".into(),
                levels: [2500, 5000, 10_000].map(|bp| vec![HeroEffect::MaxMana { bp }]),
            },
            SecondarySkillDef {
                key: key("t:lore"),
                name: "Lore".into(),
                levels: [500, 1000, 1500].map(|bp| vec![HeroEffect::ExperienceGain { bp }]),
            },
            // E08 T19a2b: Dark (Necromancy: 10/20/70 %, imps raised) and Sense (Wisdom).
            SecondarySkillDef {
                key: key("t:dark"),
                name: "Dark".into(),
                levels: [1000, 2000, 7000].map(|bp| {
                    vec![
                        HeroEffect::Necromancy {
                            bp,
                            unless_skill: None,
                        },
                        HeroEffect::Raises {
                            priority: 0,
                            creature: RaisedCreature::Creature(key("t:imp")),
                        },
                    ]
                }),
            },
            SecondarySkillDef {
                key: key("t:sense"),
                name: "Sense".into(),
                levels: [1, 2, 3].map(|levels| vec![HeroEffect::LearnLevel { levels }]),
            },
        ],
        hero_params: Some(HeroParamsDef {
            primary_min: [0, 0, 1, 1],
            primary_max: 99,
            backpack: 64,
            learn_level: 2,
            experience: vec![
                0, 1000, 2000, 3200, 4600, 6200, 8000, 10000, 12200, 14700, 17500, 20600,
            ],
            max_level: 74,
            wisdom: key("t:sense"),
            schools: vec![],
        }),
        ..ContentPack::new(manifest())
    }
}

fn artifact(k: &str, slot: &str) -> ArtifactDef {
    ArtifactDef {
        key: key(k),
        name: "Artifact".into(),
        class: ArtifactClass::Treasure,
        cost: vec![],
        slot: Some(key(slot)),
        war_machine: None,
        needs_skill: None,
        effects: vec![],
        parts: vec![],
    }
}

/// The adventure part for `core`: maps up to 4 × 4 on one level, three players, armies of two
/// slots, a land table of 1300, 1360, 1430, 1500 for speeds 0 to 3, the original's starting
/// stack chances, a gold pile `t:pile` (5 to 10 hundreds), a Cartographer `t:chart`, a tent
/// `t:tent` and a guard `t:guard` of key 0.
fn part(core: &Ruleset) -> AdventureRuleset {
    part_with(core, vec![88, 25], 2)
}

/// [`part`] with other starting stack chances and army slots.
fn part_with(core: &Ruleset, chances: Vec<u8>, slots: u8) -> AdventureRuleset {
    let cost = NonZeroU16::new(100).unwrap();
    let pack = AdventurePack {
        terrains: vec![
            TerrainMovementDef {
                terrain: key("t:grass"),
                passage: Passage::Land(cost),
                pathfinding: None,
            },
            TerrainMovementDef {
                terrain: key("t:rock"),
                passage: Passage::Impassable,
                pathfinding: None,
            },
        ],
        // A gold pile, and a Cartographer of grass for 2 500 gold (E08 T14b).
        object_types: vec![
            ObjectTypeDef {
                key: key("t:pile"),
                name: "Pile".into(),
                visit: Visit::Neighbour,
                from_above: true,
                kind: ObjectKindDef::Pickup(PickupDef::Resource {
                    resource: key("t:gold"),
                    amount: [5, 10],
                    multiplier: NonZeroU32::new(100).unwrap(),
                }),
            },
            ObjectTypeDef {
                key: key("t:chart"),
                name: "Chart".into(),
                visit: Visit::Enter,
                from_above: false,
                kind: ObjectKindDef::RevealTerrain {
                    terrains: vec![key("t:grass")],
                    price: vec![(key("t:gold"), 2500)],
                },
            },
            // A Keymaster's tent and a border guard of key 0 (E08 T15a).
            ObjectTypeDef {
                key: key("t:tent"),
                name: "Tent".into(),
                visit: Visit::Enter,
                from_above: false,
                kind: ObjectKindDef::KeyTent { key: 0 },
            },
            ObjectTypeDef {
                key: key("t:guard"),
                name: "Guard".into(),
                visit: Visit::Neighbour,
                from_above: true,
                kind: ObjectKindDef::KeyGuard { key: 0 },
            },
            // A special ground fought on the battlefield `t:holy` (E08 T20a2a).
            ObjectTypeDef {
                key: key("t:holy_ground"),
                name: "Holy ground".into(),
                visit: Visit::Enter,
                from_above: true,
                kind: ObjectKindDef::Ground {
                    battlefield: key("t:holy"),
                    no_native: false,
                },
            },
            // A wandering monster of the map's creature (E08 T20b1).
            ObjectTypeDef {
                key: key("t:monster"),
                name: "Monster".into(),
                visit: Visit::Neighbour,
                from_above: true,
                kind: ObjectKindDef::Monster { levels: None },
            },
        ],
        params: Some(AdventureParamsDef {
            native_cost: cost,
            land_points: vec![1300, 1360, 1430, 1500],
            scouting_radius: 5,
            max_map_size: NonZeroU16::new(4).unwrap(),
            max_levels: NonZeroU8::new(1).unwrap(),
            max_players: NonZeroU8::new(3).unwrap(),
            army_slots: NonZeroU8::new(slots).unwrap(),
            max_stack_count: NonZeroU32::new(1_000_000).unwrap(),
            starting_stack_chances: chances,
            // E08 T19a: a battle on a coast tile is fought on rock and the shore.
            battle: Some(BattleTerrainsDef {
                underground: key("t:grass"),
                coast: key("t:rock"),
                coast_battlefield: key("t:shore"),
            }),
            // The original's five dispositions and a sixth of aggression 3; the price to join in
            // gold (E08 T20b1).
            monsters: Some(MonsterParamsDef {
                aggression: vec![[-4, -4], [1, 7], [1, 10], [4, 10], [10, 10], [3, 3]],
                count_words: vec![5, 10],
                price_resource: key("t:gold"),
            }),
            fly_cost: [cost; 4],
            water_walk_cost: [cost; 4],
        }),
        // Difficulty 0 starts with 3 000 gold for a human, 1 with 2 000 (E08 T13a).
        starting_resources: [3000, 2000]
            .map(|gold| StartingResourcesDef {
                human: vec![(key("t:gold"), gold)],
                computer: vec![(key("t:gold"), 1)],
            })
            .to_vec(),
        ..AdventurePack::new(manifest())
    };
    AdventureRuleset::compile(core, &[pack]).unwrap()
}

/// The battle part for `core` (E08 T19a): the imp (2/3, 1-2, 4 hit points), the gog (4/4, 2-3,
/// 13) and the elf, and the cart as an ammunition supply; the gust's battle row (castable by no
/// one); `slots` army slots, the original's field and war machine places, spread formations, no
/// morale or luck, surrenders at half the cost in gold; the shore; a boulder of two hexes on grass
/// and rock.
fn battle_part(core: &Arc<Ruleset>, slots: u8) -> BattleRuleset {
    battle_part_with(core, slots, |_| {})
}

/// [`battle_part`] with its pack edited.
fn battle_part_with(
    core: &Arc<Ruleset>,
    slots: u8,
    edit: impl FnOnce(&mut BattlePack),
) -> BattleRuleset {
    let row = |k: &str, attack, defense, damage: [u16; 2], hit_points| BattleCreatureDef {
        key: key(k),
        attack,
        defense,
        damage_min: damage[0],
        damage_max: damage[1],
        hit_points,
        shots: 0,
        casts: 0,
        abilities: vec![],
    };
    let mut creatures = vec![
        row("t:imp", 2, 3, [1, 2], 4),
        row("t:gog", 4, 4, [2, 3], 13),
        BattleCreatureDef {
            abilities: vec![AbilityDef::AmmoSupply],
            ..row("t:cart", 0, 5, [0, 0], 100)
        },
    ];
    if core.creature_count() > 3 {
        creatures.push(row("t:elf", 9, 5, [3, 5], 15));
    }
    let pack = BattlePack {
        // The coast's and, since E08 T20a2a, the holy ground's.
        battlefields: ["t:shore", "t:holy"]
            .map(|k| BattlefieldDef {
                key: key(k),
                own_obstacles: false,
                blocked: vec![],
                morale: vec![],
                luck: vec![],
                no_morale: false,
                no_luck: false,
                no_native: false,
                max_spell_level: None,
                no_creature_casts: false,
                expert_schools: vec![],
                expert_other_casts: false,
            })
            .to_vec(),
        obstacles: vec![ObstacleDef {
            key: key("t:boulder"),
            terrains: vec![key("t:grass"), key("t:rock")],
            battlefields: vec![],
            width: 2,
            height: 1,
            blocked: vec![(0, 0), (1, 0)],
            absolute: false,
        }],
        creatures,
        spells: vec![BattleSpellDef {
            key: key("t:gust"),
            polarity: SpellPolarity::Neutral,
            magic: SpellMagic::default(),
            counters: vec![],
            dispels: vec![],
            spares: vec![],
            tests_as: None,
            summons: None,
            ai: SpellAi::default(),
        }],
        battle: Some(BattleParamsDef {
            field_width: 17,
            field_height: 11,
            side_columns: 1,
            army_slots: slots,
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
            good_morale_chances: vec![0, 0, 0],
            bad_morale_chances: vec![0, 0, 0],
            luck_dice: 24,
            good_luck_chances: vec![0, 0, 0],
            war_machine_positions: vec![[1, 3], [1, 1], [1, 9]],
            loose_formation: (1..=slots)
                .map(|n| (0..n).map(|i| i * 11 / n).collect())
                .collect(),
            tight_formation: (1..=slots)
                .map(|n| (0..n).map(|i| i * 11 / n).collect())
                .collect(),
            tactics_zone_columns: vec![3, 5, 7],
            surrender: Some(SurrenderDef {
                resource: key("t:gold"),
                cost_percent: 50,
            }),
            hero_casts_per_round: 1,
            side_stack_limit: 20,
            siege: None,
        }),
        ..BattlePack::new(manifest())
    };
    let mut pack = pack;
    edit(&mut pack);
    BattleRuleset::compile(Arc::clone(core), &[pack]).unwrap()
}

fn rules() -> GameRuleset {
    rules_with(vec![88, 25], 2)
}

/// [`rules`] with other starting stack chances and army slots.
fn rules_with(chances: Vec<u8>, slots: u8) -> GameRuleset {
    let core = Arc::new(core(false));
    let part = part_with(&core, chances, slots);
    let battle = battle_part(&core, 7);
    GameRuleset::new(core, Arc::new(part), Arc::new(battle)).unwrap()
}

const IMP: CreatureTypeId = CreatureTypeId::from_raw(0);
const ORRIN: HeroTypeId = HeroTypeId::from_raw(0);

/// A 4 × 4 grass map with rock at (3, 3); two players (red and blue), a hero each: red's
/// Orrin with a custom army, blue's with its starting army.
fn setup() -> GameSetup {
    let mut tiles = vec![
        TileSetup {
            terrain: TerrainId::from_raw(0),
            road: None,
            river: None,
            coast: false,
        };
        16
    ];
    tiles[15].terrain = TerrainId::from_raw(1);
    GameSetup {
        players: vec![PlayerSetup { colour: 0 }, PlayerSetup { colour: 1 }],
        difficulty: 1,
        map: MapSetup {
            width: 4,
            height: 4,
            levels: 1,
            tiles,
            objects: vec![],
        },
        heroes: vec![
            GameHeroSetup {
                hero_type: ORRIN,
                owner: PlayerId::new(0),
                pos: Pos::new(0, 0, 0),
                army: HeroArmy::Custom(vec![ArmyStack {
                    slot: 1,
                    creature: IMP,
                    count: 9,
                }]),
                formation: Formation::Loose,
                details: HeroDetailsSetup::default(),
            },
            GameHeroSetup {
                hero_type: ORRIN,
                owner: PlayerId::new(1),
                pos: Pos::new(2, 2, 0),
                army: HeroArmy::Starting,
                formation: Formation::Loose,
                details: HeroDetailsSetup::default(),
            },
        ],
        level_limit: 0,
        banned_skills: vec![],
    }
}

fn error(edit: impl FnOnce(&mut GameSetup)) -> GameSetupError {
    let mut setup = setup();
    edit(&mut setup);
    setup.validate(&rules()).unwrap_err()
}

#[test]
fn a_setup_with_custom_and_starting_armies_validates() {
    assert_eq!(setup().validate(&rules()), Ok(()));
    // Players may skip colours.
    let mut setup = setup();
    setup.players[1].colour = 2;
    assert_eq!(setup.validate(&rules()), Ok(()));
}

#[test]
fn the_game_checks_its_part_before_the_adventure() {
    // The difficulty first: the ruleset has rows 0 and 1.
    assert_eq!(
        error(|s| {
            s.difficulty = 2;
            s.players.clear();
        }),
        GameSetupError::Difficulty(2)
    );
    let player = PlayerSetup { colour: 0 };
    assert_eq!(
        error(|s| s.players = vec![player; 256]),
        GameSetupError::Players(256)
    );
    assert_eq!(
        error(|s| s.players[1].colour = 0),
        GameSetupError::Colour(1)
    );
    assert_eq!(error(|s| s.players.reverse()), GameSetupError::Colour(1));
    assert_eq!(
        error(|s| s.players[1].colour = 3),
        GameSetupError::Colour(1)
    );
    // The banned skills after the players (E08 T29): the test pack has four.
    let unknown = SkillId::from_raw(4);
    assert_eq!(
        error(|s| {
            s.banned_skills = vec![SkillId::from_raw(0), unknown];
            s.heroes[1].hero_type = HeroTypeId::from_raw(5);
        }),
        GameSetupError::BannedSkill(unknown)
    );
    let many = (usize::from(u16::MAX) + 2, usize::from(u16::MAX) + 1);
    let hero = setup().heroes[0].clone();
    assert_eq!(
        error(|s| s.heroes = vec![hero.clone(); many.0]),
        GameSetupError::Heroes(many.0)
    );
    // As many heroes as ids pass the game's count and reach the adventure's checks.
    assert!(matches!(
        error(|s| s.heroes = vec![hero; many.1]),
        GameSetupError::Adventure(SetupError::HeroTile { .. })
    ));
    assert_eq!(
        error(|s| s.heroes[1].hero_type = HeroTypeId::from_raw(5)),
        GameSetupError::UnknownHeroType(HeroId::new(1))
    );
    assert_eq!(
        error(|s| {
            s.heroes[0].hero_type = HeroTypeId::from_raw(5);
            s.heroes[0].army = HeroArmy::Starting;
        }),
        GameSetupError::UnknownHeroType(HeroId::new(0))
    );
    assert_eq!(
        error(|s| s.heroes[1].hero_type = HeroTypeId::from_raw(1)),
        GameSetupError::StartingArmy(HeroId::new(1))
    );
}

#[test]
fn the_adventure_checks_the_rest() {
    assert_eq!(
        error(|s| s.players.push(PlayerSetup { colour: 3 })),
        GameSetupError::Colour(2)
    );
    let mut three = setup();
    three.players = (0..3).map(|colour| PlayerSetup { colour }).collect();
    assert_eq!(three.validate(&rules()), Ok(()));
    assert_eq!(
        error(|s| s.players.clear()),
        GameSetupError::Adventure(SetupError::Players(0))
    );
    assert_eq!(
        error(|s| s.heroes[1].pos = Pos::new(3, 3, 0)),
        GameSetupError::Adventure(SetupError::HeroTile {
            hero: HeroId::new(1),
            error: StandError::Terrain,
        })
    );
    // A starting army is checked whole by the game: three stacks do not fit two slots, whatever
    // the draw; a custom army goes to the adventure's checks.
    assert_eq!(
        error(|s| s.heroes[1].hero_type = HeroTypeId::from_raw(2)),
        GameSetupError::StartingArmy(HeroId::new(1))
    );
    assert_eq!(
        error(|s| s.heroes[0].army = HeroArmy::Custom(vec![ArmyStack {
            slot: 2,
            creature: IMP,
            count: 1
        }])),
        GameSetupError::Adventure(SetupError::Army {
            hero: HeroId::new(0),
            error: ArmyError::SlotOutOfRange(2),
        })
    );
    assert_eq!(
        error(|s| s.heroes[0].army = HeroArmy::Custom(vec![])),
        GameSetupError::Adventure(SetupError::Army {
            hero: HeroId::new(0),
            error: ArmyError::Empty,
        })
    );
}

#[test]
fn the_parts_must_belong_to_the_core() {
    let other = Arc::new(core(true));
    let part = Arc::new(part(&other));
    let battle = Arc::new(battle_part(&other, 7));
    let ours = Arc::new(core(false));
    assert_eq!(
        GameRuleset::new(Arc::clone(&ours), Arc::clone(&part), Arc::clone(&battle)),
        Err(RulesetMismatch)
    );
    // The battle part too (E08 T19a).
    assert_eq!(
        GameRuleset::new(
            Arc::clone(&other),
            Arc::clone(&part),
            Arc::new(battle_part(&ours, 7))
        ),
        Err(RulesetMismatch)
    );
    // The coast's battlefield must be the battle part's, and every special ground's (E08
    // T20a2a).
    let fieldless = battle_part_with(&other, 7, |pack| pack.battlefields.clear());
    assert_eq!(
        GameRuleset::new(Arc::clone(&other), Arc::clone(&part), Arc::new(fieldless)),
        Err(RulesetMismatch)
    );
    let unholy = battle_part_with(&other, 7, |pack| pack.battlefields.truncate(1));
    assert_eq!(
        GameRuleset::new(Arc::clone(&other), Arc::clone(&part), Arc::new(unholy)),
        Err(RulesetMismatch)
    );
    let rules = GameRuleset::new(other, part, battle).unwrap();
    assert_eq!(rules.core().creature_count(), 4);
    assert_eq!(rules.adventure().params().max_players(), 3);
    assert_eq!(rules.battle().battle().army_slots, 7);
}

#[test]
fn the_errors_read_well() {
    let hero = HeroId::new(2);
    let cases: Vec<(GameSetupError, &str)> = vec![
        (
            GameSetupError::Difficulty(5),
            "the ruleset has no difficulty 5",
        ),
        (GameSetupError::Players(300), "300 players are too many"),
        (
            GameSetupError::Colour(1),
            "player 1's colour is out of order",
        ),
        (GameSetupError::Heroes(70_000), "70000 heroes are too many"),
        (
            GameSetupError::UnknownHeroType(hero),
            "hero 2 has an unknown hero type",
        ),
        (
            GameSetupError::StartingArmy(hero),
            "hero 2's starting army does not fit an army",
        ),
        (
            GameSetupError::Hero(hero, h3_heroes::DetailsError::LevelZero),
            "hero 2: the level must be at least 1",
        ),
        (
            GameSetupError::BannedSkill(SkillId::from_raw(4)),
            "banned skill 4 is not the ruleset's",
        ),
        (
            GameSetupError::Adventure(SetupError::Players(0)),
            "adventure: 0 players are outside the bounds",
        ),
    ];
    for (error, text) in cases {
        assert_eq!(error.to_string(), text);
    }
    assert_eq!(
        RulesetMismatch.to_string(),
        "the ruleset parts do not fit the core ruleset or each other"
    );
}
