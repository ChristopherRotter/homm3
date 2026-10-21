//! Tests of the policy on a small ruleset built in code (an application crate takes no pack
//! adapter as a dev-dependency): the dispatch of the stack's action and the shooter (BAT-AI-001,
//! BAT-AI-002), the area shooters' value at a hex with and without the original's slip
//! (BAT-AI-003, BAT-QUIRK-25), the fallback, and a property over random battles. Sieges (the
//! Cyclops's wall shot, BAT-AI-004), special battlefields and the harvested creatures:
//! `h3-pack-battle`'s `harvested_computer.rs`.

use std::string::ToString;
use std::sync::Arc;
use std::vec;
use std::vec::Vec;

use h3_battle::{
    BattleState, Command, Field, Hex, LegalCommands, NoopSink, StackId, apply, computer,
    legal_commands, validate,
};
use h3_content::{
    ContentKey, ContentPack, CreatureDef, CreatureInfoDef, CreatureTypeId, FactionDef,
    HeroClassDef, HeroTypeDef, HeroTypeId, PackManifest, ProfileDef, ResourceDef, Ruleset,
    TerrainDef,
};
use h3_content_battle::{
    AbilityDef, AreaShot, BattleCreatureDef, BattlePack, BattleParamsDef, BattleRuleset,
};
use h3_contracts::{
    Army, ArmyStack, BallistaControl, BattleHero, BattleSetup, BattleSide, Rate, TentControl,
};
use h3_kernel::Rng;
use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};
use proptest::test_runner::Config;

use super::{POLICY_VERSION, checked, choose, unchecked};

fn key(text: &str) -> ContentKey {
    ContentKey::parse(text).unwrap()
}

fn manifest() -> PackManifest {
    PackManifest {
        id: "t".into(),
        version: "1".into(),
        depends: vec![],
    }
}

/// A test creature: its key, speed, Fight Value and flags; its attack, defense, damage, hit
/// points and shots; its ability (the First Aid Tent's, an area shot).
type Creature = (
    &'static str,
    u8,
    u32,
    &'static [&'static str],
    [u16; 4],
    u32,
    u16,
    Option<AbilityDef>,
);

/// The First Aid Tent's healing.
const HEALS: Option<AbilityDef> = Some(AbilityDef::FirstAid {
    min_hp: 1,
    max_hp: 25,
});

/// The test creatures, by handle: walkers (the imp, the gog, the runner, faster than the field is
/// wide), the archer, the Ballista and the First Aid Tent; the area shooters (the magog's
/// fireball, the undead lich's death cloud); fast walkers that are fire-immune (the efreet),
/// lifeless (the golem) and undead (the zombie); a slow two-hex walker (the cavalier).
const CREATURES: [Creature; 12] = [
    ("imp", 5, 100, &[], [2, 3, 1, 2], 4, 0, None),
    ("gog", 3, 900, &[], [4, 4, 2, 3], 13, 0, None),
    ("runner", 20, 300, &["alive"], [5, 5, 2, 4], 10, 0, None),
    ("archer", 4, 300, &["shooter"], [6, 3, 2, 3], 10, 12, None),
    (
        "ballista",
        0,
        600,
        &["war_machine", "two_hex", "shooter"],
        [10, 10, 2, 3],
        250,
        24,
        None,
    ),
    (
        "tent",
        0,
        50,
        &["war_machine", "two_hex"],
        [0, 0, 0, 0],
        75,
        0,
        HEALS,
    ),
    (
        "magog",
        4,
        400,
        &["shooter", "area_attack", "alive"],
        [7, 4, 2, 4],
        13,
        24,
        Some(AbilityDef::AreaShot {
            effect: AreaShot::Fireball,
        }),
    ),
    (
        "lich",
        4,
        900,
        &["shooter", "area_attack", "undead"],
        [13, 10, 11, 13],
        30,
        24,
        Some(AbilityDef::AreaShot {
            effect: AreaShot::DeathCloud,
        }),
    ),
    (
        "efreet",
        20,
        300,
        &["fire_immune", "alive"],
        [5, 5, 2, 4],
        10,
        0,
        None,
    ),
    ("golem", 20, 300, &[], [5, 5, 2, 4], 10, 0, None),
    ("zombie", 20, 300, &["undead"], [5, 5, 2, 4], 10, 0, None),
    (
        "cavalier",
        2,
        1900,
        &["two_hex", "alive"],
        [15, 15, 15, 25],
        100,
        0,
        None,
    ),
];

const IMP: usize = 0;
const GOG: usize = 1;
const RUNNER: usize = 2;
const ARCHER: usize = 3;
const BALLISTA: usize = 4;
const TENT: usize = 5;
const MAGOG: usize = 6;
const LICH: usize = 7;
const EFREET: usize = 8;
const GOLEM: usize = 9;
const ZOMBIE: usize = 10;
const CAVALIER: usize = 11;

/// The core ruleset of the test creatures, one hero type and the profiles `t:faithful` (the
/// original's slips in its estimates, BAT-QUIRK-25) and `t:fixed` (none), in `profile`.
fn core(profile: &str) -> Ruleset {
    let mut pack = ContentPack::new(manifest());
    pack.terrains = vec![TerrainDef {
        key: key("t:grass"),
        name: "Grass".into(),
    }];
    pack.creatures = CREATURES
        .iter()
        .map(|&(k, speed, fight_value, flags, ..)| CreatureDef {
            key: key(&["t:", k].concat()),
            speed,
            info: CreatureInfoDef {
                fight_value,
                flags: flags.iter().map(|f| f.to_string()).collect(),
                ..CreatureInfoDef::default()
            },
        })
        .collect();
    pack.factions = vec![FactionDef {
        key: key("t:town"),
        name: "Town".into(),
        native_terrain: None,
    }];
    pack.hero_classes = vec![HeroClassDef {
        key: key("t:knight"),
        name: "Knight".into(),
        faction: key("t:town"),
        primary: [1, 1, 1, 1],
        primary_gain: [[25, 25, 25, 25], [25, 25, 25, 25]],
        skill_weights: vec![],
        offer_intervals: [6, 4],
    }];
    pack.heroes = vec![HeroTypeDef {
        key: key("t:hero"),
        name: "Hero".into(),
        class: key("t:knight"),
        skills: vec![],
        spell_book: false,
        spells: vec![],
        army: vec![],
        war_machines: vec![],
        specialty: vec![],
    }];
    pack.resources = vec![ResourceDef {
        key: key("t:gold"),
        name: "Gold".into(),
    }];
    pack.profiles = vec![
        ProfileDef {
            key: key("t:faithful"),
            quirks: vec![("quirk.battle.original_ai_slips".into(), true)],
        },
        ProfileDef {
            key: key("t:fixed"),
            quirks: vec![],
        },
    ];
    Ruleset::compile(&[pack], profile).unwrap()
}

/// The test ruleset in the profile `t:faithful`.
fn ruleset() -> BattleRuleset {
    ruleset_as("t:faithful")
}

/// The test ruleset in `profile`: [`core`] and its battle part on the original's field, without
/// morale and luck.
fn ruleset_as(profile: &str) -> BattleRuleset {
    let creatures = CREATURES
        .iter()
        .map(
            |&(k, _, _, _, [attack, defense, min, max], hit_points, shots, ref ability)| {
                BattleCreatureDef {
                    key: key(&["t:", k].concat()),
                    attack,
                    defense,
                    damage_min: min,
                    damage_max: max,
                    hit_points,
                    shots,
                    casts: 0,
                    abilities: ability.clone().into_iter().collect(),
                }
            },
        )
        .collect();
    let pack = BattlePack {
        creatures,
        battle: Some(BattleParamsDef {
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
            morale_dice: 24,
            good_morale_chances: vec![0, 0, 0],
            bad_morale_chances: vec![0, 0, 0],
            luck_dice: 24,
            good_luck_chances: vec![0, 0, 0],
            war_machine_positions: vec![[1, 3], [1, 1], [1, 9]],
            loose_formation: (1..=7u8)
                .map(|n| (0..n).map(|i| i * 11 / n).collect())
                .collect(),
            tight_formation: (1..=7u8)
                .map(|n| (0..n).map(|i| i * 11 / n).collect())
                .collect(),
            tactics_zone_columns: vec![3, 5, 7],
            surrender: None,
            hero_casts_per_round: 1,
            side_stack_limit: 20,
            siege: None,
        }),
        ..BattlePack::new(manifest())
    };
    BattleRuleset::compile(Arc::new(core(profile)), &[pack]).unwrap()
}

fn creature(handle: usize) -> CreatureTypeId {
    CreatureTypeId::from_index(handle).unwrap()
}

fn army(stacks: &[(usize, u32)]) -> Army {
    Army {
        stacks: (0u8..)
            .zip(stacks)
            .map(|(slot, &(handle, count))| ArmyStack {
                slot,
                creature: creature(handle),
                count,
            })
            .collect(),
    }
}

/// A hero with `tactics`, commanding its Ballista and its First Aid Tent.
fn hero(tactics: u8) -> BattleHero {
    let mut hero = BattleHero::new(HeroTypeId::from_index(0).unwrap());
    hero.tactics = tactics;
    hero.war_machines.ballista = Some(BallistaControl {
        level: 1,
        shots: 1,
        double_damage_chance_bp: 0,
    });
    hero.war_machines.tent = Some(TentControl {
        heal_bonus: Rate::bp(0),
    });
    hero
}

/// A setup of the two armies without heroes, on Normal.
fn setup(attacker: &[(usize, u32)], defender: &[(usize, u32)]) -> BattleSetup {
    BattleSetup {
        attacker: army(attacker),
        defender: army(defender),
        difficulty: 1,
        ..BattleSetup::default()
    }
}

/// Plays `command` for the side to act.
fn play(state: &mut BattleState, r: &BattleRuleset, command: Command) {
    let side = state.side_to_act().unwrap();
    apply(state, r, side, command, &mut NoopSink).unwrap();
}

/// The active stack's creature handle.
fn active(state: &BattleState) -> usize {
    state
        .stack(state.active().unwrap())
        .unwrap()
        .creature
        .index()
}

/// The policy's command for the side to act, from a fixed stream.
fn decide(state: &BattleState, r: &BattleRuleset) -> Command {
    choose(
        state,
        r,
        state.side_to_act().unwrap(),
        &mut Rng::from_seed(5),
    )
}

// covers: BAT-AI-001, BAT-AI-002
#[test]
fn a_shooter_shoots_the_computers_target_at_its_front_hex() {
    let r = ruleset();
    // The imps (speed 5) act first, then the defender's archers (4), then the gogs (3): the
    // imps defend, then the archers shoot.
    let mut state =
        BattleState::new(&r, &setup(&[(IMP, 10), (GOG, 2)], &[(ARCHER, 5)]), 1).unwrap();
    assert_eq!(active(&state), IMP);
    play(&mut state, &r, Command::Defend);
    assert_eq!(active(&state), ARCHER);
    let archer = state.active().unwrap();
    let target = computer::shot_target(&state, &r, archer).unwrap();
    let field = Field::new(r.battle().field);
    let at = state.stack(target).unwrap().footprint(field).front;
    assert_eq!(decide(&state, &r), Command::Shoot { target, at });
    assert!(validate(&state, &r, BattleSide::Defender, decide(&state, &r)).is_ok());
}

// covers: BAT-AI-002
#[test]
fn the_shooter_takes_the_better_value() {
    let r = ruleset();
    // One gog (Fight Value 900) and one imp (100): the archers kill the gog's worth first.
    let mut state =
        BattleState::new(&r, &setup(&[(IMP, 1), (GOG, 1)], &[(ARCHER, 20)]), 2).unwrap();
    play(&mut state, &r, Command::Defend);
    let Command::Shoot { target, .. } = decide(&state, &r) else {
        panic!("no shot");
    };
    assert_eq!(state.stack(target).unwrap().creature.index(), GOG);
}

/// The creature handle of stack `id`.
fn handle(state: &BattleState, id: StackId) -> usize {
    state.stack(id).unwrap().creature.index()
}

/// The battle of `attacker` against the defender's fast walker `escort` and its thirty area
/// shooters `shooter`, at the shooters' first turn: the walker acts first and, when `near` picks
/// a hex it may walk to from the attacker's first stack's footprint, walks there (else it
/// defends); the attacker's stacks faster than the shooters defend.
fn escorted(
    r: &BattleRuleset,
    attacker: &[(usize, u32)],
    (escort, count): (usize, u32),
    shooter: usize,
    near: Option<fn(Field, h3_battle::Footprint, Hex) -> bool>,
) -> BattleState {
    let mut state =
        BattleState::new(r, &setup(attacker, &[(escort, count), (shooter, 30)]), 8).unwrap();
    assert_eq!(active(&state), escort);
    let field = Field::new(r.battle().field);
    let first = state.stacks()[0].footprint(field);
    let mut legal = LegalCommands::new();
    legal_commands(&state, r, BattleSide::Defender, &mut legal);
    let walk = near.map(|near| {
        legal
            .iter()
            .find_map(|c| match *c {
                Command::Move { to } if near(field, first, to) => Some(Command::Move { to }),
                _ => None,
            })
            .unwrap()
    });
    play(&mut state, r, walk.unwrap_or(Command::Defend));
    while active(&state) != shooter {
        play(&mut state, r, Command::Defend);
    }
    state
}

/// Beside the attacker's first stack (its front hex).
fn beside(field: Field, first: h3_battle::Footprint, hex: Hex) -> bool {
    field.distance(hex, first.front) == 1
}

// covers: BAT-AI-003
#[test]
fn an_area_shooter_does_not_hit_its_own_side_for_less() {
    for profile in ["t:faithful", "t:fixed"] {
        let r = ruleset_as(profile);
        // Alone, the gog is the magogs' better shot (as the archers', above).
        let state = escorted(&r, &[(GOG, 1), (IMP, 1)], (RUNNER, 30), MAGOG, None);
        let Command::Shoot { target, at } = decide(&state, &r) else {
            panic!("no shot");
        };
        assert_eq!(
            (handle(&state, target), at),
            (GOG, state.stack(target).unwrap().hex)
        );
        // With thirty runners beside it, the gog can strike at once (its value is undivided),
        // but the fireball's kills among the runners cost more than it is worth: its value is
        // below 0, so it is no target; the imp is shot instead.
        let state = escorted(&r, &[(GOG, 1), (IMP, 1)], (RUNNER, 30), MAGOG, Some(beside));
        let Command::Shoot { target, .. } = decide(&state, &r) else {
            panic!("no shot");
        };
        assert_eq!(handle(&state, target), IMP, "{profile}");
        // With the gog alone there is no target: the magogs defend.
        let state = escorted(&r, &[(GOG, 1)], (RUNNER, 30), MAGOG, Some(beside));
        assert_eq!(
            unchecked(&state, &r, BattleSide::Defender, &mut Rng::from_seed(1)),
            Command::Defend
        );
    }
}

// covers: BAT-AI-003
#[test]
fn the_original_estimates_an_area_shot_without_the_immunities() {
    // The fireball spares fire-immune stacks (BAT-RNG-004), the death cloud lifeless ones; the
    // original's estimate (slip 1) counts the efreets beside the gog for the magogs and the golems
    // for the undead liches, so it finds no target: they defend. Without the slip they shoot.
    for (escort, shooter) in [(EFREET, MAGOG), (GOLEM, LICH)] {
        for (profile, shoots) in [("t:faithful", false), ("t:fixed", true)] {
            let r = ruleset_as(profile);
            let state = escorted(&r, &[(GOG, 1)], (escort, 30), shooter, Some(beside));
            let command = decide(&state, &r);
            assert_eq!(
                matches!(command, Command::Shoot { .. }),
                shoots,
                "{profile}: {command:?}"
            );
        }
    }
    // Both leave the undead out when they are beside the target (the slip: an undead shooter
    // leaves out the undead; the rule: the cloud spares them), but not the undead target itself.
    for profile in ["t:faithful", "t:fixed"] {
        let r = ruleset_as(profile);
        let state = escorted(&r, &[(GOG, 1)], (ZOMBIE, 30), LICH, Some(beside));
        assert!(
            matches!(decide(&state, &r), Command::Shoot { .. }),
            "{profile}"
        );
        let mut state = BattleState::new(&r, &setup(&[(ZOMBIE, 5)], &[(LICH, 10)]), 9).unwrap();
        play(&mut state, &r, Command::Defend);
        let Command::Shoot { target, .. } = decide(&state, &r) else {
            panic!("no shot");
        };
        assert_eq!(handle(&state, target), ZOMBIE);
    }
}

/// Beside the attacker's first stack's position (its rear hex) but not its front hex.
fn behind(field: Field, first: h3_battle::Footprint, hex: Hex) -> bool {
    first
        .rear
        .is_some_and(|rear| field.distance(hex, rear) == 1 && field.distance(hex, first.front) > 1)
}

// covers: BAT-AI-003
#[test]
fn an_area_shot_at_a_two_hex_stack_takes_its_better_hex() {
    let r = ruleset();
    let field = Field::new(r.battle().field);
    // Alone, the cavalier is worth the same at both its hexes: the shot aims at its position.
    let state = escorted(&r, &[(CAVALIER, 1), (IMP, 1)], (RUNNER, 30), MAGOG, None);
    let cavalier = state.stacks()[0].footprint(field);
    let shot = |target, at| Command::Shoot { target, at };
    assert_eq!(decide(&state, &r), shot(StackId(0), cavalier.rear.unwrap()));
    // With the runners beside its position only, its front hex is worth strictly more.
    let state = escorted(
        &r,
        &[(CAVALIER, 1), (IMP, 1)],
        (RUNNER, 30),
        MAGOG,
        Some(behind),
    );
    assert_eq!(decide(&state, &r), shot(StackId(0), cavalier.front));
}

// covers: BAT-AI-001, BAT-AI-005
#[test]
fn a_walker_attacks_and_a_blocked_shooter_strikes_back() {
    let r = ruleset();
    // The runner (speed 20) reaches the archers in its first turn and attacks them.
    let mut state = BattleState::new(&r, &setup(&[(RUNNER, 1)], &[(ARCHER, 5)]), 3).unwrap();
    assert_eq!(active(&state), RUNNER);
    let attack = decide(&state, &r);
    assert!(
        matches!(
            attack,
            Command::Attack {
                target: StackId(1),
                ..
            }
        ),
        "{attack:?}"
    );
    assert!(validate(&state, &r, BattleSide::Attacker, attack).is_ok());
    play(&mut state, &r, attack);
    // Blocked by the runner beside them, the archers cannot shoot: a walker's turn, which strikes
    // the runner from where they stand.
    assert_eq!(active(&state), ARCHER);
    let archers = state.active().unwrap();
    assert!(!computer::shoots(&state, &r, archers));
    let from = state.stack(archers).unwrap().hex;
    assert_eq!(
        decide(&state, &r),
        Command::Attack {
            target: StackId(0),
            from
        }
    );
}

// covers: BAT-AI-001, BAT-AI-002
#[test]
fn the_commanded_ballista_shoots_and_the_tent_defends() {
    let r = ruleset();
    // The runner wounds the gogs in its first turn, so that the commanded tent has a patient and
    // takes its turns (BAT-WAR-007); the gogs defend (the walker is not under test here).
    let mut setup = setup(&[(GOG, 2)], &[(RUNNER, 1)]);
    setup.heroes = [Some(hero(0)), None];
    setup.war_machines = [vec![creature(BALLISTA), creature(TENT)], vec![]];
    let mut state = BattleState::new(&r, &setup, 4).unwrap();
    let mut seen = [false; 2];
    let mut legal = LegalCommands::new();
    for _ in 0..8 {
        let Some(side) = state.side_to_act() else {
            break;
        };
        let command = if side == BattleSide::Defender {
            legal_commands(&state, &r, side, &mut legal);
            legal
                .iter()
                .copied()
                .find(|c| {
                    matches!(
                        c,
                        Command::Attack {
                            target: StackId(0),
                            ..
                        }
                    )
                })
                .unwrap_or(Command::Defend)
        } else if active(&state) == GOG {
            Command::Defend
        } else {
            decide(&state, &r)
        };
        match active(&state) {
            BALLISTA => {
                seen[0] = true;
                assert!(matches!(command, Command::Shoot { .. }), "{command:?}");
            }
            TENT => {
                seen[1] = true;
                assert_eq!(command, Command::Defend);
            }
            _ => {}
        }
        play(&mut state, &r, command);
    }
    assert_eq!(seen, [true, true]);
}

// covers: BAT-AI-001
#[test]
fn the_tactics_phase_ends_at_once() {
    let r = ruleset();
    let mut setup = setup(&[(IMP, 3)], &[(GOG, 2)]);
    setup.heroes = [Some(hero(2)), None];
    let state = BattleState::new(&r, &setup, 5).unwrap();
    assert!(state.tactics().is_some());
    assert_eq!(decide(&state, &r), Command::EndTactics);
}

// covers: BAT-AI-001
#[test]
fn the_other_side_and_a_battle_that_is_over_get_defend() {
    let r = ruleset();
    let mut state = BattleState::new(&r, &setup(&[(RUNNER, 50)], &[(IMP, 1)]), 6).unwrap();
    let other = state.side_to_act().unwrap().opponent();
    let mut rng = Rng::from_seed(1);
    assert_eq!(unchecked(&state, &r, other, &mut rng), Command::Defend);
    assert_eq!(choose(&state, &r, other, &mut rng), Command::Defend);
    for _ in 0..10 {
        let Some(side) = state.side_to_act() else {
            break;
        };
        let mut legal = LegalCommands::new();
        legal_commands(&state, &r, side, &mut legal);
        let attack = legal
            .iter()
            .copied()
            .find(|c| matches!(c, Command::Attack { .. }))
            .unwrap_or(Command::Defend);
        play(&mut state, &r, attack);
    }
    assert!(state.is_over());
    for side in BattleSide::BOTH {
        assert_eq!(unchecked(&state, &r, side, &mut rng), Command::Defend);
    }
}

// covers: BAT-AI-001
#[test]
fn a_refused_choice_falls_back_to_defend() {
    let r = ruleset();
    let state = BattleState::new(&r, &setup(&[(IMP, 3)], &[(GOG, 2)]), 7).unwrap();
    let side = state.side_to_act().unwrap();
    let refused = Command::Shoot {
        target: StackId(1),
        at: Hex(0),
    };
    assert!(validate(&state, &r, side, refused).is_err());
    assert_eq!(checked(&state, &r, side, refused), Command::Defend);
    assert_eq!(checked(&state, &r, side, Command::Wait), Command::Wait);
    assert_eq!(POLICY_VERSION, 3);
}

/// The handles an army draws from: every creature but the war machines.
const ARMY: [usize; 10] = [
    IMP, GOG, RUNNER, ARCHER, MAGOG, LICH, EFREET, GOLEM, ZOMBIE, CAVALIER,
];

proptest! {
    #![proptest_config(Config::with_cases(48))]

    /// Exit criterion 1 on the test ruleset: random battles of both sides, every difficulty,
    /// with and without heroes (Tactics, a commanded Ballista and First Aid Tent): at every step
    /// the original's choice for the side to act is a command the engine accepts (no fallback
    /// fires), and the same state and stream give the same command and leave the same stream.
    /// Two steps in three play it, the third a random legal command that stays in the battle.
    // covers: BAT-AI-001, BAT-AI-002, BAT-AI-003
    #[test]
    fn the_choice_is_legal_and_the_same_for_the_same_stream(
        seed in any::<u64>(),
        draws in any::<u64>(),
        fixed: bool,
    ) {
        let r = ruleset_as(if fixed { "t:fixed" } else { "t:faithful" });
        let mut rng = Rng::from_seed(draws);
        let draw_army = |rng: &mut Rng| -> Vec<(usize, u32)> {
            (0..=rng.below(4))
                .map(|_| (ARMY[rng.below(10) as usize], 1 + rng.below(40)))
                .collect()
        };
        let attacker = draw_army(&mut rng);
        let defender = draw_army(&mut rng);
        let mut setup = setup(&attacker, &defender);
        for side in 0..2 {
            if rng.below(2) == 0 {
                setup.heroes[side] = Some(hero(u8::try_from(rng.below(4)).unwrap()));
                setup.war_machines[side] = [BALLISTA, TENT]
                    .into_iter()
                    .filter(|_| rng.below(2) == 0)
                    .map(creature)
                    .collect();
            }
            setup.human[side] = rng.below(2) == 0;
        }
        setup.difficulty = u8::try_from(rng.below(5)).unwrap();
        let mut state = BattleState::new(&r, &setup, seed).unwrap();
        let mut seat = Rng::from_seed(seed ^ draws);
        let mut legal = LegalCommands::new();
        for _ in 0..300 {
            let Some(side) = state.side_to_act() else {
                break;
            };
            let proposed = unchecked(&state, &r, side, &mut seat.clone());
            prop_assert!(
                validate(&state, &r, side, proposed).is_ok(),
                "refused: {proposed:?}"
            );
            let (mut a, mut b) = (seat.clone(), seat.clone());
            let command = choose(&state, &r, side, &mut a);
            prop_assert_eq!(command, choose(&state, &r, side, &mut b));
            prop_assert_eq!(command, proposed);
            prop_assert_eq!(&a, &b);
            seat = a;
            let played = if rng.below(3) == 0 {
                legal_commands(&state, &r, side, &mut legal);
                let staying: Vec<Command> =
                    legal.iter().copied().filter(|c| !c.leaves_battle()).collect();
                staying[rng.below(u32::try_from(staying.len()).unwrap()) as usize]
            } else {
                command
            };
            apply(&mut state, &r, side, played, &mut NoopSink).unwrap();
        }
    }
}
