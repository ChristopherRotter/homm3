//! Tests of the rests (GAM-DEC-007, E08 T63b): what a visit still owes its hero runs once its
//! owner has no open decision: a reward record's experience after its army dialog, a won guards'
//! battle's reward after the winner's levels; each held rest in the hash. The guard's visit after
//! the whole visit of an entered object (ADV-MON-006): `rewards.rs`.

use std::boxed::Box;

use h3_adventure::{Guards, Holds, ObjectSetup, RewardRecord};
use h3_contracts::{BattleOutcome, StackOrigin, StackRecord};
use h3_heroes::{ArtifactInstance, HeroChange, HeroReward};

use super::decisions::{ACCEPT, CLOSE, RED, play};
use super::rewards::{AT, HERO, answered, changed, guard, reward_at, visit, won};
use super::*;

const GOG: CreatureTypeId = CreatureTypeId::from_raw(1);
const RING: ArtifactTypeId = ArtifactTypeId::from_raw(4);
const FIRST: Answer = Answer::LevelUp(SkillOffer::First);

/// `count` of `creature` in `slot`.
const fn stack(slot: u8, creature: CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

/// The index of the first of `events` that `matches`.
fn first(events: &[GameEvent], matches: impl Fn(&GameEvent) -> bool) -> Option<usize> {
    events.iter().position(matches)
}

/// Whether `event` gives red's hero a level or opens one.
const fn levels(event: &GameEvent) -> bool {
    matches!(
        event,
        GameEvent::LevelGained { .. } | GameEvent::LevelUpOpened { .. }
    )
}

/// A box at [`AT`] whose record gives 1 000 experience and three gogs, which find no room in red's
/// hero's two slots full of imps.
fn crowded(rules: &GameRuleset) -> GameState {
    let mut setup = setup();
    setup.heroes[0].army = HeroArmy::Custom(vec![stack(0, IMP, 1), stack(1, IMP, 9)]);
    let record = RewardRecord {
        hero: HeroReward {
            experience: 1000,
            ..HeroReward::default()
        },
        creatures: vec![stack(1, GOG, 3)],
        ..RewardRecord::default()
    };
    setup.map.objects = vec![reward_at(rules, "t:box", Holds::Record(Box::new(record)))];
    GameState::new(rules, &setup, 7).unwrap()
}

// covers: GAM-DEC-007, ADV-RWD-001, GAM-DEC-004
#[test]
fn a_records_experience_waits_for_its_army_dialog() {
    let rules = rules();
    let mut state = crowded(&rules);
    let before = state.hero(HERO).unwrap().experience;
    visit(&mut state, &rules);
    let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
    // The dialog opens; the experience waits (before E08 T63b it came first, with its levels).
    assert!(state.army_dialog(RED).is_some());
    let gained = |e: &GameEvent| *e == changed(HeroChange::GainExperience(1000));
    assert_eq!(first(&events, gained), None);
    assert_eq!(state.hero(HERO).unwrap().experience, before);
    // Closed: the group's rest lost, then the experience with its level.
    let events = play(&mut state, &rules, RED, CLOSE).unwrap();
    let lost = first(&events, |e| matches!(e, GameEvent::GroupLost { .. })).unwrap();
    let experience = first(&events, gained).unwrap();
    let level = first(&events, levels).unwrap();
    assert!(lost < experience && experience < level, "{events:?}");
    assert_eq!(state.hero(HERO).unwrap().experience, before + 1000);
}

/// The game with red's hero at 950 experience and an artifact at [`AT`] guarded by two imps.
fn guarded_ring(rules: &GameRuleset) -> GameState {
    let mut setup = setup();
    setup.heroes[0].details.experience = Some(950);
    setup.map.objects = vec![ObjectSetup {
        guards: Some(Guards {
            army: vec![stack(0, IMP, 2)],
            question: true,
        }),
        ..reward_at(rules, "t:artifact", Holds::Artifact(RING.into()))
    }];
    GameState::new(rules, &setup, 5).unwrap()
}

/// Red's victory at [`AT`] over guards that lost ten gogs: enough experience for a level.
fn won_a_level() -> BattleOutcome {
    BattleOutcome {
        stacks: [
            vec![],
            vec![StackRecord {
                origin: StackOrigin::Army(0),
                creature: GOG,
                start: 10,
                end: 0,
            }],
        ],
        ..won()
    }
}

// covers: GAM-DEC-007, ADV-PIK-007, GAM-BAT-008
#[test]
fn a_won_guards_reward_comes_after_the_winners_levels() {
    let rules = rules();
    let mut state = guarded_ring(&rules);
    let sides = ([Some(RED), None], [Some(HERO), None], AT);
    let mut events = Vec::new();
    state.end_battle(&rules, sides, &won_a_level(), &mut events);
    state.run_rests(&rules, &mut events);
    let ring = changed(HeroChange::AddArtifact(ArtifactInstance::from(RING)));
    // The level has two offers: the artifact waits for its answer (before E08 T63b it came
    // first), the guards beaten, the artifact still on the map.
    assert!(
        state.level_up(RED).is_some() && !events.contains(&ring),
        "{events:?}"
    );
    assert!(!state.adventure().is_removed(0) && state.adventure().guards_left(0).is_empty());
    let events = play(&mut state, &rules, RED, answered(FIRST)).unwrap();
    let level = first(&events, levels).unwrap();
    let picked = first(&events, |e| *e == ring).unwrap();
    assert!(level < picked, "{events:?}");
    assert!(state.adventure().is_removed(0));
    assert_eq!(state.hero(HERO).unwrap().level, 2);
}

// covers: GAM-DEC-007
#[test]
fn the_hash_sees_each_rest() {
    let rules = rules();
    // A record's experience held behind its army dialog.
    let mut experience = crowded(&rules);
    visit(&mut experience, &rules);
    play(&mut experience, &rules, RED, ACCEPT).unwrap();
    // A guard's visit held behind an entered gift's level-up.
    let mut setup = setup();
    setup.map.objects = vec![
        reward_at(&rules, "t:in_stone", Holds::Nothing),
        guard(&rules),
    ];
    let mut guarded = GameState::new(&rules, &setup, 7).unwrap();
    visit(&mut guarded, &rules);
    // A won battle's reward held until the winner's levels are given.
    let mut won = guarded_ring(&rules);
    let sides = ([Some(RED), None], [Some(HERO), None], AT);
    won.end_battle(&rules, sides, &won_a_level(), &mut Vec::new());
    for state in [experience, guarded, won] {
        assert_eq!(state.held_rests(), 1);
        let mut bare = state.clone();
        bare.drop_rests();
        assert_ne!(state.state_hash(), bare.state_hash());
    }
}
