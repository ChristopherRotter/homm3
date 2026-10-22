//! Tests of the hero's day in the game (E08 T40): the adventure's start of the game after the
//! levels at load reads the store (ADR-0017 point 1), and each turn start of the owner again
//! (ADV-MOV-001, ADV-MOV-002; survey `E08-hero-movement.md` §1).

use h3_adventure::day_points;
use h3_heroes::ArtifactInstance;

use super::*;

const ONE: HeroId = HeroId::new(1);
/// The hero type `t:rider` of [`rules`].
const RIDER: HeroTypeId = HeroTypeId::from_raw(5);

/// [`rules`](super::rules) with the content of the day's points: the skill `t:roads` (Logistics:
/// +10 %, 20 %, 30 %), the artifact `t:boots` (+600 land points) in the ring slot, and the hero
/// type `t:rider` (two gogs) specialist in the gog (one faster) and in Logistics (5 % of itself
/// per level), as the original's Kyrre and creature specialists.
fn rules() -> GameRuleset {
    let mut pack = core_pack(false);
    pack.skills.push(SecondarySkillDef {
        key: key("t:roads"),
        name: "Roads".into(),
        levels: [1000, 2000, 3000].map(|bp| vec![HeroEffect::LandMovement { bp }]),
    });
    pack.artifacts.push(ArtifactDef {
        effects: vec![HeroEffect::LandMovementPoints { points: 600 }],
        ..artifact("t:boots", "t:ring_slot")
    });
    pack.heroes.push(HeroTypeDef {
        specialty: vec![
            HeroEffect::CreatureSpecialty {
                creature: key("t:gog"),
                bp_per_step: 1,
                speed: 1,
            },
            HeroEffect::SkillSpecialty {
                skill: key("t:roads"),
                bp_per_level: 500,
            },
        ],
        ..hero_type("t:rider", &[("t:gog", [2, 2])], &[])
    });
    let core = Arc::new(Ruleset::compile(&[pack], "t:faithful").unwrap());
    let part = part_with(&core, vec![88, 25], 2);
    let battle = battle_part(&core, 7);
    GameRuleset::new(core, Arc::new(part), Arc::new(battle)).unwrap()
}

/// The game of [`setup`] with blue's hero a rider with Basic Logistics, the boots and the custom
/// experience `experience`.
fn game(rules: &GameRuleset, experience: u32) -> GameState {
    let roads = rules.core().skill_by_key("t:roads").unwrap();
    let boots: ArtifactInstance = rules.core().artifact_by_key("t:boots").unwrap().into();
    let mut setup = setup();
    let hero = &mut setup.heroes[1];
    hero.hero_type = RIDER;
    hero.army = HeroArmy::Starting;
    hero.details.skills = Some(vec![(roads, 1)]);
    hero.details.artifacts = Some(HeroArtifactsSetup {
        worn: vec![boots],
        backpack: vec![],
    });
    hero.details.experience = Some(experience);
    GameState::new(rules, &setup, 3).unwrap()
}

fn points(state: &GameState) -> u32 {
    state.adventure().hero(ONE).unwrap().movement
}

// covers: ADV-MOV-001, ADV-MOV-002, GAM-SET-002
#[test]
fn the_days_points_come_from_the_details_after_the_levels_at_load() {
    let rules = rules();
    // Level 1: the gogs (speed 2) count one faster, the table's 1500 at speed 3 (not 1430); Basic
    // Logistics grown by 5 % of itself (10.5 %: 1657); the boots' 600 after the rate.
    let state = game(&rules, 0);
    assert_eq!(state.hero(ONE).unwrap().level, 1);
    assert_eq!(points(&state), 2257);
    // Levels at load (experience 3 200: level 4): the points read the details as the levels left
    // them (the original's `placed_hero_setup` 0x4D8B30), not the level-1 hero's.
    let state = game(&rules, 3200);
    let details = state.hero(ONE).unwrap();
    assert_eq!(details.level, 4);
    let army = state.adventure().hero(ONE).unwrap().army.clone();
    let expected = day_points(rules.core(), rules.adventure(), Some(details), &army);
    assert_eq!(points(&state), expected);
    let mut at_one = details.clone();
    at_one.level = 1;
    assert_ne!(
        day_points(rules.core(), rules.adventure(), Some(&at_one), &army),
        expected
    );
    // Its owner's turn start sets the same points again, from the store (red ends its turn).
    let mut state = state;
    state
        .end_turn(&rules, PlayerId::new(0), &mut NoopSink)
        .unwrap();
    assert!(state.is_acting(PlayerId::new(1)));
    assert_eq!(points(&state), expected);
}
