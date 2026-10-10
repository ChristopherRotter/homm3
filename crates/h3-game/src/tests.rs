//! Tests of the game's rules bundle and setup (E08 T6a). The rulesets are built in code (an
//! application crate takes no pack adapter as a dev-dependency); the core still needs battle
//! parameters until the content split (ADR-0015 point 3).

extern crate std;

use std::string::ToString;
use std::vec;
use std::vec::Vec;

use alloc::sync::Arc;
use core::num::{NonZeroU8, NonZeroU16};

use h3_adventure::{
    AdventurePack, AdventureParamsDef, AdventureRuleset, ArmyError, MapSetup, Passage, Pos,
    SetupError, StandError, TerrainMovementDef, TileSetup,
};
use h3_content::{
    BattleParamsDef, ContentKey, ContentPack, CreatureDef, CreatureInfoDef, CreatureTypeId,
    FactionDef, HeroClassDef, HeroTypeDef, HeroTypeId, PackManifest, ProfileDef, Ruleset,
    StartingStackDef, TerrainDef, TerrainId,
};
use h3_contracts::{ArmyStack, HeroId, PlayerId};

use super::*;

fn key(text: &str) -> ContentKey {
    ContentKey::parse(text).unwrap()
}

fn battle() -> BattleParamsDef {
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

fn manifest() -> PackManifest {
    PackManifest {
        id: "t".into(),
        version: "1".into(),
        depends: vec![],
    }
}

fn hero_type(k: &str, army: &[(&str, [u16; 2])]) -> HeroTypeDef {
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
        war_machines: vec![],
        specialty: vec![],
    }
}

/// A core ruleset: the terrains `t:grass` and `t:rock`, the creatures `t:imp` and `t:gog`, and
/// the hero types `t:orrin` (imp 2-5, gog 1-3), `t:loner` (no army) and `t:crowd` (three
/// stacks), in that order; `extra` adds a creature.
fn core(extra: bool) -> Ruleset {
    let mut creatures = vec![creature("t:imp"), creature("t:gog")];
    if extra {
        creatures.push(creature("t:elf"));
    }
    let pack = ContentPack {
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
            skill_weights: vec![],
        }],
        heroes: vec![
            hero_type("t:orrin", &[("t:imp", [2, 5]), ("t:gog", [1, 3])]),
            hero_type("t:loner", &[]),
            hero_type(
                "t:crowd",
                &[("t:imp", [1, 1]), ("t:imp", [1, 1]), ("t:gog", [1, 1])],
            ),
        ],
        profiles: vec![ProfileDef {
            key: key("t:faithful"),
            quirks: vec![],
        }],
        battle: Some(battle()),
        ..ContentPack::new(manifest())
    };
    Ruleset::compile(&[pack], "t:faithful").unwrap()
}

/// The adventure part for `core`: maps up to 4 × 4 on one level, three players, armies of two
/// slots.
fn part(core: &Ruleset) -> AdventureRuleset {
    let cost = NonZeroU16::new(100).unwrap();
    let pack = AdventurePack {
        terrains: vec![
            TerrainMovementDef {
                terrain: key("t:grass"),
                passage: Passage::Land(cost),
            },
            TerrainMovementDef {
                terrain: key("t:rock"),
                passage: Passage::Impassable,
            },
        ],
        params: Some(AdventureParamsDef {
            native_cost: cost,
            land_points: vec![1500],
            scouting_radius: 5,
            max_map_size: NonZeroU16::new(4).unwrap(),
            max_levels: NonZeroU8::new(1).unwrap(),
            max_players: NonZeroU8::new(3).unwrap(),
            army_slots: NonZeroU8::new(2).unwrap(),
        }),
        ..AdventurePack::new(manifest())
    };
    AdventureRuleset::compile(core, &[pack]).unwrap()
}

fn rules() -> GameRuleset {
    let core = core(false);
    let part = part(&core);
    GameRuleset::new(Arc::new(core), Arc::new(part)).unwrap()
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
        };
        16
    ];
    tiles[15].terrain = TerrainId::from_raw(1);
    GameSetup {
        players: vec![PlayerSetup { colour: 0 }, PlayerSetup { colour: 1 }],
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
            },
            GameHeroSetup {
                hero_type: ORRIN,
                owner: PlayerId::new(1),
                pos: Pos::new(2, 2, 0),
                army: HeroArmy::Starting,
            },
        ],
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
        error(|s| s.heroes[1].hero_type = HeroTypeId::from_raw(3)),
        GameSetupError::UnknownHeroType(HeroId::new(1))
    );
    assert_eq!(
        error(|s| {
            s.heroes[0].hero_type = HeroTypeId::from_raw(3);
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
    // A starting army is checked whole: three stacks do not fit two slots.
    assert_eq!(
        error(|s| s.heroes[1].hero_type = HeroTypeId::from_raw(2)),
        GameSetupError::Adventure(SetupError::Army {
            hero: HeroId::new(1),
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
    let other = core(true);
    let part = part(&other);
    assert_eq!(
        GameRuleset::new(Arc::new(core(false)), Arc::new(part.clone())),
        Err(RulesetMismatch)
    );
    let rules = GameRuleset::new(Arc::new(other), Arc::new(part)).unwrap();
    assert_eq!(rules.core().creature_count(), 3);
    assert_eq!(rules.adventure().params().max_players(), 3);
}

#[test]
fn the_errors_read_well() {
    let hero = HeroId::new(2);
    let cases: Vec<(GameSetupError, &str)> = vec![
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
            "hero 2's starting army has no stacks",
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
        "the adventure part was compiled on another core ruleset"
    );
}
