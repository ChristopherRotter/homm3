//! Test fixtures: a small ruleset built in code (the battle crate cannot depend on the pack
//! adapter) and helpers to set up and drive battles.

use std::string::String;
use std::vec;
use std::vec::Vec;

use h3_content::{
    AbilityDef, BattleParamsDef, ContentKey, ContentPack, CreatureDef, CreatureInfoDef,
    PackManifest, ProfileDef, Ruleset,
};
use h3_contracts::{Army, ArmyStack, BattleSetup, BattleSide};

use crate::{BattleState, Command, Event, Field, Hex, StackId, apply};

/// `(name, attack, defense, damage_min, damage_max, hit_points, speed, traits)`; traits are
/// creature flag names, ability names without parameters (`teleports`, …), `fireball` /
/// `death_cloud` for an area shot, and `shots=N`.
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
];

/// The abilities and other non-flag traits of a test creature.
fn parse_traits(traits: &[&str]) -> (Vec<String>, Vec<AbilityDef>, u16) {
    let mut flags = Vec::new();
    let mut abilities = Vec::new();
    let mut shots = 0;
    for t in traits {
        match *t {
            "teleports" => abilities.push(AbilityDef::Teleports),
            "no_distance_penalty" => abilities.push(AbilityDef::NoDistancePenalty),
            "fireball" => abilities.push(AbilityDef::AreaShot {
                effect: h3_content::AreaShot::Fireball,
            }),
            "death_cloud" => abilities.push(AbilityDef::AreaShot {
                effect: h3_content::AreaShot::DeathCloud,
            }),
            t if t.starts_with("shots=") => shots = t[6..].parse().unwrap(),
            t => flags.push(String::from(t)),
        }
    }
    (flags, abilities, shots)
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
        formation: vec![
            vec![5],
            vec![3, 7],
            vec![2, 5, 8],
            vec![1, 4, 6, 9],
            vec![0, 2, 5, 8, 10],
            vec![0, 2, 4, 6, 8, 10],
            vec![0, 2, 4, 5, 6, 8, 10],
        ],
    }
}

pub fn ruleset() -> Ruleset {
    let creatures = CREATURES
        .iter()
        .map(
            |&(name, attack, defense, damage_min, damage_max, hit_points, speed, traits)| {
                let (flags, abilities, shots) = parse_traits(traits);
                CreatureDef {
                    key: ContentKey::new("t", name).unwrap(),
                    attack,
                    defense,
                    damage_min,
                    damage_max,
                    hit_points,
                    speed,
                    info: CreatureInfoDef {
                        flags,
                        abilities,
                        shots,
                        ..CreatureInfoDef::default()
                    },
                }
            },
        )
        .collect();
    let pack = ContentPack {
        creatures,
        profiles: vec![ProfileDef {
            key: ContentKey::new("t", "faithful").unwrap(),
            quirks: vec![],
        }],
        battle: Some(battle_params()),
        ..ContentPack::new(PackManifest {
            id: String::from("t"),
            version: String::from("1"),
            depends: vec![],
        })
    };
    Ruleset::compile(&[pack], "t:faithful").unwrap()
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

pub fn setup(rules: &Ruleset, attacker: &[Spec], defender: &[Spec]) -> BattleSetup {
    BattleSetup {
        attacker: army(rules, attacker),
        defender: army(rules, defender),
        obstacles: vec![],
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
