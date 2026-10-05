//! Test fixtures: a small ruleset built in code (the battle crate cannot depend on the pack
//! adapter) and helpers to set up and drive battles.

use std::string::String;
use std::vec;
use std::vec::Vec;

use h3_content::{
    AbilityDef, AuraTarget, BattleParamsDef, ContentKey, ContentPack, CreatureDef, CreatureFlag,
    CreatureFlags, CreatureInfoDef, FactionDef, Formation, HeroClassDef, HeroTypeDef, ObstacleDef,
    PackManifest, ProfileDef, ResourceDef, Ruleset, SpellArea, SpellCategory, SpellDef,
    SpellEffect, SpellMagic, SpellPolarity, SpellSchoolDef, SpellTarget, StartingStackDef,
    SurrenderDef, TargetConditions, TargetSide, TerrainDef,
};
use h3_contracts::{Army, ArmyStack, BattleHero, BattleSetup, BattleSide};

use crate::{BattleState, Command, Event, Field, Hex, StackId, apply};

/// `(name, attack, defense, damage_min, damage_max, hit_points, speed, traits)`; traits are
/// creature flag names, ability names (`teleports`, `jousting=5`, `hate=fixed:50`, …),
/// `fireball` / `death_cloud` for an area shot, and `shots=N`.
pub type Creature = (
    &'static str,
    u16,
    u16,
    u16,
    u16,
    u32,
    u8,
    &'static [&'static str],
);

pub const CREATURES: &[Creature] = &[
    ("fixed", 10, 10, 2, 2, 10, 5, &[]),
    ("slow", 1, 1, 1, 1, 5, 2, &[]),
    ("fast", 5, 5, 1, 1, 3, 9, &[]),
    ("tank", 0, 20, 1, 1, 100, 4, &[]),
    ("range", 5, 5, 1, 9, 10, 5, &[]),
    ("spear", 4, 5, 1, 3, 10, 4, &[]),
    ("brute", 7, 4, 2, 5, 18, 5, &[]),
    ("wisp", 3, 2, 1, 2, 4, 9, &[]),
    ("wide", 6, 6, 2, 4, 25, 5, &["two_hex"]),
    ("flyer", 5, 3, 1, 3, 8, 7, &["flying"]),
    ("dragon", 12, 12, 5, 8, 60, 8, &["two_hex", "flying"]),
    ("porter", 8, 6, 2, 4, 15, 6, &["teleports"]),
    ("archer", 6, 3, 2, 3, 10, 4, &["shooter", "shots=12"]),
    (
        "sniper",
        6,
        3,
        2,
        3,
        10,
        4,
        &[
            "shooter",
            "shots=3",
            "strikes_twice",
            "no_melee_penalty",
            "no_distance_penalty",
        ],
    ),
    (
        "magog",
        7,
        4,
        2,
        4,
        13,
        6,
        &["shooter", "shots=24", "area_attack", "fireball"],
    ),
    (
        "lich",
        13,
        10,
        11,
        13,
        30,
        6,
        &[
            "shooter",
            "shots=12",
            "area_attack",
            "death_cloud",
            "undead",
        ],
    ),
    ("salamander", 5, 5, 2, 3, 12, 5, &["fire_immune", "alive"]),
    // Melee abilities (BAT-ATK-002..004, BAT-RET-002/003, BAT-DMG-004..009, BAT-TRN-005): all
    // 10/10 with damage 2 and 10 HP, so against `fixed` or `serf` a strike deals 2 per creature.
    ("serf", 10, 10, 2, 2, 10, 5, &["alive"]),
    ("twin", 10, 10, 2, 2, 10, 5, &["strikes_twice"]),
    ("naga", 10, 10, 2, 2, 10, 5, &["no_enemy_retaliation"]),
    (
        "griffin",
        10,
        10,
        2,
        2,
        10,
        5,
        &["flying", "extra_retaliations=1"],
    ),
    (
        "royal",
        10,
        10,
        2,
        2,
        10,
        5,
        &["flying", "unlimited_retaliations"],
    ),
    (
        "hydra",
        10,
        10,
        2,
        2,
        10,
        5,
        &[
            "two_hex",
            "multi_headed",
            "attacks_all_adjacent",
            "no_enemy_retaliation",
        ],
    ),
    (
        "cerberus",
        10,
        10,
        2,
        2,
        10,
        5,
        &[
            "two_hex",
            "multi_headed",
            "three_headed_attack",
            "no_enemy_retaliation",
        ],
    ),
    ("drake", 10, 10, 2, 2, 10, 5, &["two_hex", "breath_attack"]),
    (
        "harpy",
        10,
        10,
        2,
        2,
        10,
        6,
        &["flying", "strike_and_return"],
    ),
    ("knight", 10, 10, 2, 2, 10, 7, &["jousting=5"]),
    ("pikeman", 10, 10, 2, 2, 10, 4, &["charge_immunity"]),
    ("behemoth", 10, 10, 2, 2, 10, 5, &["defense_reduction=40"]),
    ("dread", 10, 10, 2, 2, 10, 5, &["death_blow=20"]),
    ("doom", 10, 10, 2, 2, 10, 5, &["death_blow=100"]),
    (
        "vampire",
        10,
        10,
        2,
        2,
        10,
        5,
        &["life_drain=100", "undead"],
    ),
    ("gorgon", 10, 10, 2, 2, 10, 5, &["death_stare=10"]),
    ("starer", 10, 10, 2, 2, 10, 5, &["death_stare=100"]),
    ("wight", 10, 10, 2, 2, 10, 5, &["regeneration", "undead"]),
    ("hater", 10, 10, 2, 2, 10, 5, &["hate=fixed:50"]),
    // Morale and luck (BAT-MOR-001..004, BAT-LCK-001/002).
    (
        "pikeman_c",
        10,
        10,
        2,
        2,
        10,
        5,
        &["faction=castle", "alive"],
    ),
    (
        "angel",
        10,
        10,
        2,
        2,
        10,
        5,
        &["faction=castle", "alive", "morale_aura=1:allies"],
    ),
    (
        "archangel",
        10,
        10,
        2,
        2,
        10,
        5,
        &["faction=castle", "alive", "morale_aura=1:allies"],
    ),
    ("imp", 10, 10, 2, 2, 10, 5, &["faction=inferno", "alive"]),
    (
        "devil",
        10,
        10,
        2,
        2,
        10,
        5,
        &["faction=inferno", "alive", "luck_aura=-1:enemies"],
    ),
    (
        "skeleton",
        10,
        10,
        2,
        2,
        10,
        5,
        &["faction=necropolis", "undead", "no_morale"],
    ),
    (
        "bone_dragon",
        10,
        10,
        2,
        2,
        10,
        5,
        &[
            "faction=necropolis",
            "undead",
            "no_morale",
            "morale_aura=-1:enemies",
        ],
    ),
    (
        "minotaur",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "minimum_morale=1"],
    ),
    (
        "halfling",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "minimum_luck=1"],
    ),
    ("lucky", 10, 10, 2, 2, 10, 5, &["alive", "minimum_luck=3"]),
    // War machines (BAT-WAR-001..004).
    (
        "ballista",
        10,
        10,
        2,
        3,
        250,
        0,
        &[
            "two_hex",
            "shooter",
            "war_machine",
            "no_morale",
            "shots=24",
            "ai=600",
        ],
    ),
    (
        "tent",
        0,
        0,
        0,
        0,
        75,
        0,
        // The minimum morale has no effect (no morale); it puts another ability before first aid.
        &[
            "two_hex",
            "war_machine",
            "no_morale",
            "minimum_morale=1",
            "first_aid=1:25",
        ],
    ),
    (
        "cart",
        0,
        5,
        0,
        0,
        100,
        0,
        &["war_machine", "no_morale", "ammo_supply"],
    ),
    ("valuable", 10, 10, 2, 2, 10, 5, &["alive", "ai=1000"]),
    ("gadget", 0, 0, 0, 0, 10, 0, &["war_machine"]),
    // A shooting war machine with an area shot: it can hit its own side (BAT-END-002).
    (
        "firebomb",
        10,
        10,
        50,
        50,
        100,
        0,
        &[
            "shooter",
            "war_machine",
            "no_morale",
            "area_attack",
            "fireball",
            "shots=4",
        ],
    ),
    // A creature and its upgrade (hero statistic effects, BAT-HER-005).
    (
        "squire",
        10,
        12,
        6,
        9,
        35,
        5,
        &["alive", "upgrades=paladin"],
    ),
    ("paladin", 12, 12, 7, 10, 35, 6, &["alive"]),
    // A level-7 creature (spell specialties divide by the creature's level, BAT-MAG-004).
    ("titan", 20, 20, 10, 20, 300, 7, &["alive", "level=7"]),
];

/// The traits of a test creature as creature info: flags, abilities, shots and faction.
fn parse_traits(traits: &[&str]) -> CreatureInfoDef {
    let mut flags = Vec::new();
    let mut abilities = Vec::new();
    let mut shots = 0;
    let mut faction = None;
    let mut ai = 0;
    let mut upgrades = Vec::new();
    let mut level = 1;
    for t in traits {
        let (name, value) = t.split_once('=').unwrap_or((t, "0"));
        let n = || value.parse::<u16>().unwrap();
        let signed = || value.parse::<i8>().unwrap();
        let aura = || {
            let (amount, affects) = value.split_once(':').unwrap();
            let affects = match affects {
                "allies" => AuraTarget::Allies,
                _ => AuraTarget::Enemies,
            };
            (amount.parse::<i8>().unwrap(), affects)
        };
        let ability = match name {
            "ai" => {
                ai = value.parse().unwrap();
                continue;
            }
            "first_aid" => {
                let (min, max) = value.split_once(':').unwrap();
                AbilityDef::FirstAid {
                    min_hp: min.parse().unwrap(),
                    max_hp: max.parse().unwrap(),
                }
            }
            "ammo_supply" => AbilityDef::AmmoSupply,
            "faction" => {
                faction = Some(ContentKey::new("t", value).unwrap());
                continue;
            }
            "upgrades" => {
                upgrades.push(ContentKey::new("t", value).unwrap());
                continue;
            }
            "level" => {
                level = value.parse().unwrap();
                continue;
            }
            "morale_aura" => {
                let (amount, affects) = aura();
                AbilityDef::MoraleAura { amount, affects }
            }
            "luck_aura" => {
                let (amount, affects) = aura();
                AbilityDef::LuckAura { amount, affects }
            }
            "minimum_morale" => AbilityDef::MinimumMorale { value: signed() },
            "minimum_luck" => AbilityDef::MinimumLuck { value: signed() },
            "teleports" => AbilityDef::Teleports,
            "no_distance_penalty" => AbilityDef::NoDistancePenalty,
            "fireball" => AbilityDef::AreaShot {
                effect: h3_content::AreaShot::Fireball,
            },
            "death_cloud" => AbilityDef::AreaShot {
                effect: h3_content::AreaShot::DeathCloud,
            },
            "hate" => {
                let (target, percent) = value.split_once(':').unwrap();
                AbilityDef::Hate {
                    target: ContentKey::new("t", target).unwrap(),
                    percent: percent.parse().unwrap(),
                }
            }
            "jousting" => AbilityDef::Jousting {
                percent_per_hex: n(),
            },
            "charge_immunity" => AbilityDef::ChargeImmunity,
            "defense_reduction" => AbilityDef::DefenseReduction {
                percent: u8::try_from(n()).unwrap(),
            },
            "death_blow" => AbilityDef::DeathBlow {
                chance_percent: u8::try_from(n()).unwrap(),
            },
            "life_drain" => AbilityDef::LifeDrain { percent: n() },
            "regeneration" => AbilityDef::Regeneration,
            "death_stare" => AbilityDef::DeathStare {
                chance_percent: u8::try_from(n()).unwrap(),
            },
            "extra_retaliations" => AbilityDef::ExtraRetaliations {
                count: u8::try_from(n()).unwrap(),
            },
            "unlimited_retaliations" => AbilityDef::UnlimitedRetaliations,
            "strike_and_return" => AbilityDef::StrikeAndReturn,
            "attacks_all_adjacent" => AbilityDef::AttacksAllAdjacent,
            "three_headed_attack" => AbilityDef::ThreeHeadedAttack,
            "shots" => {
                shots = n();
                continue;
            }
            flag => {
                flags.push(String::from(flag));
                continue;
            }
        };
        abilities.push(ability);
    }
    CreatureInfoDef {
        flags,
        abilities,
        shots,
        faction,
        ai_value: ai,
        upgrades,
        level,
        ..CreatureInfoDef::default()
    }
}

pub fn battle_params() -> BattleParamsDef {
    BattleParamsDef {
        field_width: 17,
        field_height: 11,
        side_columns: 1,
        army_slots: 7,
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
        // Morale and luck never trigger by default (tests stay independent of them); morale and
        // luck tests and the property tests use `ruleset_with`.
        morale_dice: 24,
        good_morale_chances: vec![0, 0, 0],
        bad_morale_chances: vec![0, 0, 0],
        luck_dice: 24,
        good_luck_chances: vec![0, 0, 0],
        war_machine_positions: vec![[1, 3], [1, 1], [1, 9]],
        loose_formation: vec![
            vec![5],
            vec![3, 7],
            vec![2, 5, 8],
            vec![1, 4, 6, 9],
            vec![0, 2, 5, 8, 10],
            vec![0, 2, 4, 6, 8, 10],
            vec![0, 2, 4, 5, 6, 8, 10],
        ],
        // The original's tight formation (VCMI layout).
        tight_formation: vec![
            vec![5],
            vec![4, 6],
            vec![4, 5, 6],
            vec![2, 4, 6, 8],
            vec![2, 4, 5, 6, 8],
            vec![0, 2, 4, 6, 8, 10],
            vec![0, 2, 4, 5, 6, 8, 10],
        ],
        tactics_zone_columns: vec![3, 5, 7],
        surrender: Some(SurrenderDef {
            resource: ContentKey::new("t", "gold").unwrap(),
            cost_percent: 50,
        }),
        hero_casts_per_round: 1,
    }
}

/// Gold costs of some test creatures (BAT-FLE-002): those of the original's Pikemen, Pixies and
/// war machines; the others cost nothing.
pub const COSTS: &[(&str, u32)] = &[
    ("fixed", 60),
    ("flyer", 25),
    ("ballista", 2500),
    ("cart", 1000),
    ("tent", 750),
];

fn obstacle(
    name: &str,
    terrains: &[&str],
    width: u8,
    height: u8,
    blocked: &[(i8, i8)],
    absolute: bool,
) -> ObstacleDef {
    ObstacleDef {
        key: ContentKey::new("t", name).unwrap(),
        terrains: terrains
            .iter()
            .map(|t| ContentKey::new("t", t).unwrap())
            .collect(),
        width,
        height,
        blocked: blocked.to_vec(),
        absolute,
    }
}

pub fn ruleset() -> Ruleset {
    ruleset_with("t:faithful", |_| {})
}

/// The test ruleset with profile `profile` (`t:faithful` without quirks, `t:quirky` with
/// Q-0003, `t:single` with Q-0005) and edited battle parameters.
pub fn ruleset_with(profile: &str, edit: impl FnOnce(&mut BattleParamsDef)) -> Ruleset {
    let creatures = CREATURES
        .iter()
        .map(
            |&(name, attack, defense, damage_min, damage_max, hit_points, speed, traits)| {
                CreatureDef {
                    key: ContentKey::new("t", name).unwrap(),
                    attack,
                    defense,
                    damage_min,
                    damage_max,
                    hit_points,
                    speed,
                    info: CreatureInfoDef {
                        cost: COSTS
                            .iter()
                            .filter(|(n, _)| *n == name)
                            .map(|&(_, gold)| (ContentKey::new("t", "gold").unwrap(), gold))
                            .collect(),
                        ..parse_traits(traits)
                    },
                }
            },
        )
        .collect();
    let mut battle = battle_params();
    edit(&mut battle);
    let pack = ContentPack {
        creatures,
        factions: [
            ("castle", Some("plain")),
            ("inferno", Some("lava")),
            ("necropolis", None),
        ]
        .iter()
        .map(|&(f, native)| FactionDef {
            key: ContentKey::new("t", f).unwrap(),
            name: String::from(f),
            native_terrain: native.map(|t| ContentKey::new("t", t).unwrap()),
        })
        .collect(),
        terrains: ["plain", "lava", "void", "tall"]
            .iter()
            .map(|t| TerrainDef {
                key: ContentKey::new("t", t).unwrap(),
                name: String::from(*t),
            })
            .collect(),
        obstacles: vec![
            obstacle("stone", &["plain", "lava"], 2, 1, &[(0, 0), (1, 0)], false),
            obstacle(
                "cliff",
                &["lava"],
                0,
                0,
                &[(5, 4), (6, 4), (7, 4), (8, 4)],
                true,
            ),
            obstacle("tower", &["tall"], 1, 10, &[(0, 0)], false),
        ],
        profiles: vec![
            ProfileDef {
                key: ContentKey::new("t", "faithful").unwrap(),
                quirks: vec![],
            },
            ProfileDef {
                key: ContentKey::new("t", "quirky").unwrap(),
                quirks: vec![(String::from("quirk.battle.no_luck_without_hero"), true)],
            },
            ProfileDef {
                key: ContentKey::new("t", "single").unwrap(),
                quirks: vec![(String::from("quirk.battle.single_precision_rates"), true)],
            },
            ProfileDef {
                key: ContentKey::new("t", "unlimited").unwrap(),
                quirks: vec![(String::from("quirk.battle.unlimited_ballista_shots"), true)],
            },
        ],
        battle: Some(battle),
        spell_schools: ["air", "fire"]
            .iter()
            .map(|&name| SpellSchoolDef {
                key: ContentKey::new("t", name).unwrap(),
                name: String::from(name),
            })
            .collect(),
        spells: SPELLS.iter().map(|&spell| spell_def(spell)).collect(),
        hero_classes: vec![HeroClassDef {
            key: ContentKey::new("t", "knight").unwrap(),
            name: String::from("Knight"),
            faction: ContentKey::new("t", "castle").unwrap(),
            primary: [2, 2, 1, 1],
            primary_gain: [[35, 45, 10, 10], [30, 30, 20, 20]],
            skill_weights: vec![],
        }],
        heroes: vec![HeroTypeDef {
            key: ContentKey::new("t", "orrin").unwrap(),
            name: String::from("Orrin"),
            class: ContentKey::new("t", "knight").unwrap(),
            skills: vec![],
            spell_book: false,
            spells: vec![],
            army: vec![StartingStackDef {
                creature: ContentKey::new("t", "fixed").unwrap(),
                count: [1, 2],
            }],
            war_machines: vec![],
            specialty: vec![],
        }],
        resources: vec![ResourceDef {
            key: ContentKey::new("t", "gold").unwrap(),
            name: String::from("Gold"),
        }],
        ..ContentPack::new(PackManifest {
            id: String::from("t"),
            version: String::from("1"),
            depends: vec![],
        })
    };
    Ruleset::compile(&[pack], profile).unwrap()
}

/// A test spell: name, category, schools, target and side.
pub type SpellSpec = (
    &'static str,
    SpellCategory,
    &'static [&'static str],
    SpellTarget,
    TargetSide,
);

/// The test spells (BAT-MAG-001 to BAT-MAG-004), all costing 5/4/4/4 and dealing 10 per power
/// plus 10/10/20/30: `bolt` damages one enemy stack; `smite` any stack that is not undead, in two
/// schools; `gaze` is a creature ability; `ward` has no effects and `portal` is an adventure
/// spell; `nova` damages an area (not castable before E04b T2d); `rally` reaches only own stacks.
pub const SPELLS: &[SpellSpec] = &[
    (
        "bolt",
        SpellCategory::Combat,
        &["air"],
        SpellTarget::Stack,
        TargetSide::Enemy,
    ),
    (
        "gaze",
        SpellCategory::Ability,
        &[],
        SpellTarget::None,
        TargetSide::Any,
    ),
    (
        "smite",
        SpellCategory::Combat,
        &["air", "fire"],
        SpellTarget::Stack,
        TargetSide::Any,
    ),
    (
        "ward",
        SpellCategory::Combat,
        &["fire"],
        SpellTarget::None,
        TargetSide::Any,
    ),
    (
        "portal",
        SpellCategory::Adventure,
        &["air"],
        SpellTarget::None,
        TargetSide::Any,
    ),
    (
        "nova",
        SpellCategory::Combat,
        &["fire"],
        SpellTarget::Hex,
        TargetSide::Any,
    ),
    (
        "rally",
        SpellCategory::Combat,
        &["air"],
        SpellTarget::Stack,
        TargetSide::Own,
    ),
];

/// The definition of a test spell.
fn spell_def((name, category, schools, target, side): SpellSpec) -> SpellDef {
    let castable = !matches!(name, "gaze" | "ward" | "portal");
    let excludes = if name == "smite" {
        CreatureFlags::default().with(CreatureFlag::Undead)
    } else {
        CreatureFlags::default()
    };
    SpellDef {
        key: ContentKey::new("t", name).unwrap(),
        name: String::from(name),
        category,
        level: u8::from(category != SpellCategory::Ability),
        schools: schools
            .iter()
            .map(|s| ContentKey::new("t", s).unwrap())
            .collect(),
        cost: [5, 4, 4, 4],
        power: 10,
        effect: [10, 10, 20, 30],
        polarity: SpellPolarity::Negative,
        magic: SpellMagic {
            target,
            area: (target == SpellTarget::Hex).then_some(SpellArea {
                inner: 0,
                radius: [1; 4],
            }),
            side,
            conditions: TargetConditions {
                requires: CreatureFlags::default(),
                excludes,
            },
            effects: if castable {
                vec![SpellEffect::Damage]
            } else {
                vec![]
            },
            ..SpellMagic::default()
        },
    }
}

/// `(slot, creature name, count)`.
pub type Spec = (u8, &'static str, u32);

pub fn army(rules: &Ruleset, spec: &[Spec]) -> Army {
    Army {
        stacks: spec
            .iter()
            .map(|&(slot, name, count)| ArmyStack {
                slot,
                creature: rules.creature_by_key(&std::format!("t:{name}")).unwrap(),
                count,
            })
            .collect(),
    }
}

/// A battle hero of the test ruleset's only hero type with some primary skills and mana, without
/// Tactics (so that no tactics phase runs, BAT-TAC-001).
pub fn hero(rules: &Ruleset) -> BattleHero {
    let mut hero = BattleHero::new(rules.hero_type_by_key("t:orrin").unwrap());
    [hero.attack, hero.defense, hero.power, hero.knowledge] = [3, 2, 1, 1];
    hero.magic.mana = 15;
    hero.magic.max_mana = 10;
    hero
}

pub fn setup(rules: &Ruleset, attacker: &[Spec], defender: &[Spec]) -> BattleSetup {
    BattleSetup {
        attacker: army(rules, attacker),
        defender: army(rules, defender),
        obstacles: vec![],
        heroes: [None, None],
        terrain: None,
        war_machines: [vec![], vec![]],
        formations: [Formation::Loose; 2],
        funds: [0; 2],
    }
}

pub fn battle(rules: &Ruleset, attacker: &[Spec], defender: &[Spec], seed: u64) -> BattleState {
    BattleState::new(rules, &setup(rules, attacker, defender), seed).unwrap()
}

/// A battle with obstacles at the given `(x, y)` hexes.
pub fn battle_with_obstacles(
    rules: &Ruleset,
    attacker: &[Spec],
    defender: &[Spec],
    obstacles: &[(i32, i32)],
) -> BattleState {
    let mut s = setup(rules, attacker, defender);
    s.obstacles = obstacles.iter().map(|&(x, y)| at(rules, x, y).0).collect();
    BattleState::new(rules, &s, 1).unwrap()
}

pub fn field(rules: &Ruleset) -> Field {
    Field::new(rules.battle().field)
}

pub fn at(rules: &Ruleset, x: i32, y: i32) -> Hex {
    field(rules).hex(x, y).unwrap()
}

/// Moves stack `id` to `(x, y)` directly (test setup only).
pub fn place(state: &mut BattleState, rules: &Ruleset, id: u8, x: i32, y: i32) {
    state.stacks.as_mut_slice()[usize::from(id)].hex = at(rules, x, y);
}

/// Makes `id` the active stack (test setup only).
pub fn activate(state: &mut BattleState, id: u8) {
    state.active = Some(StackId(id));
}

/// Applies `command` for the side to act, returning the events.
pub fn act(
    state: &mut BattleState,
    rules: &Ruleset,
    command: Command,
) -> Result<Vec<Event>, crate::CommandError> {
    let side = state.side_to_act().unwrap_or(BattleSide::Attacker);
    let mut events = Vec::new();
    apply(state, rules, side, command, &mut events)?;
    Ok(events)
}

/// Lets the active stack defend until `id` is active.
pub fn advance_to(state: &mut BattleState, rules: &Ruleset, id: u8) {
    for _ in 0..64 {
        if state.active() == Some(StackId(id)) {
            return;
        }
        act(state, rules, Command::Defend).unwrap();
    }
    panic!("stack {id} never became active");
}

/// The order in which stacks act when everybody defends, for `turns` turns.
pub fn order(state: &mut BattleState, rules: &Ruleset, turns: usize) -> Vec<u8> {
    (0..turns)
        .map(|_| {
            let id = state.active().unwrap().0;
            act(state, rules, Command::Defend).unwrap();
            id
        })
        .collect()
}
