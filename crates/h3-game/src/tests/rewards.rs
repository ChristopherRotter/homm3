//! Tests of what the game applies of increment 7's pickups, teachers and finds (E08 T62a; ADR-0017
//! point 10): a visit's hero changes and experience with its level-ups, the gift's price and the
//! find's question as offers, the choices answered by their defaults and the University left
//! until T63 (`ADV-STUB-10`), a won guarded artifact and a beaten monster's artifact.

use h3_adventure::{Guards, Holds, MonsterSetup, ObjectSetup, OfferKind, TileMask};
use h3_contracts::{Army, BattleOutcome, BattleResult, BattleSide, HeroOutcome};
use h3_heroes::{ArtifactInstance, HeroChange};

use super::decisions::{ACCEPT, GOLD, RED, after_steps, move_hero, play};
use super::*;

const HERO: HeroId = HeroId::new(0);
/// The tile the objects stand on, beside red's hero at (0, 0).
const AT: Pos = Pos::new(1, 0, 0);
const RING: ArtifactTypeId = ArtifactTypeId::from_raw(4);

/// An object of type `k` at [`AT`], holding `holds`.
fn reward_at(rules: &GameRuleset, k: &str, holds: Holds) -> ObjectSetup {
    ObjectSetup {
        scenery: Default::default(),
        anchor: AT,
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.adventure().object_type_by_key(k),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds,
    }
}

/// [`setup`] (2 000 gold each) with this object.
fn game(rules: &GameRuleset, object: ObjectSetup) -> GameState {
    let mut setup = setup();
    setup.map.objects = vec![object];
    GameState::new(rules, &setup, 7).unwrap()
}

/// Red's hero visits the object at [`AT`]: the events after its step.
fn visit(state: &mut GameState, rules: &GameRuleset) -> Vec<GameEvent> {
    after_steps(play(state, rules, RED, move_hero(0, &[AT])).unwrap())
}

/// The change told to red of its hero.
fn changed(change: HeroChange) -> GameEvent {
    GameEvent::HeroChanged {
        player: RED,
        hero: HERO,
        change,
    }
}

/// Red's gold.
fn gold(state: &GameState) -> u32 {
    state.treasury(RED).unwrap()[0]
}

// covers: ADV-TCH-001, GAM-DEC-002
#[test]
fn a_gifts_experience_reaches_the_hero_with_its_level() {
    let rules = rules();
    let mut state = game(&rules, reward_at(&rules, "t:stone", Holds::Nothing));
    let before = state.hero(HERO).unwrap().experience;
    let events = visit(&mut state, &rules);
    assert_eq!(events[0], changed(HeroChange::GainExperience(1000)));
    let hero = state.hero(HERO).unwrap();
    assert_eq!(hero.experience, before + 1000);
    assert!(
        hero.level == 2 || state.waiting_for() == Some(RED),
        "the level it brings is given"
    );
    assert!(state.adventure().is_marked(HERO, 0));
}

// covers: ADV-PIK-005, ADV-TCH-001, ADV-TCH-002
#[test]
fn until_their_decisions_the_choices_take_their_defaults() {
    let rules = rules();
    // The chest's gold (the original's plain OK).
    let mut state = game(&rules, reward_at(&rules, "t:chest", Holds::Nothing));
    let events = visit(&mut state, &rules);
    assert!(events.contains(&GameEvent::ResourceGained {
        player: RED,
        resource: GOLD,
        amount: 1500,
    }));
    assert_eq!(gold(&state), 3500);
    assert!(state.adventure().is_removed(0));
    // The Arena's first skill.
    let mut state = game(&rules, reward_at(&rules, "t:arena", Holds::Nothing));
    let attack = state.hero(HERO).unwrap().primary[0];
    let events = visit(&mut state, &rules);
    assert_eq!(events, [changed(HeroChange::GainPrimary([2, 0, 0, 0]))]);
    assert_eq!(state.hero(HERO).unwrap().primary[0], attack + 2);
    // A School and a University are left: nothing paid, no mark, no decision.
    for (k, holds) in [
        ("t:school", Holds::Nothing),
        ("t:university", Holds::Nothing),
    ] {
        let mut state = game(&rules, reward_at(&rules, k, holds));
        assert!(visit(&mut state, &rules).is_empty(), "{k}");
        assert_eq!(gold(&state), 2000);
        assert!(!state.adventure().is_marked(HERO, 0) && state.decision(RED).is_none());
    }
}

// covers: ADV-TCH-003, GAM-DEC-001
#[test]
fn a_gifts_price_opens_as_an_offer_when_the_treasury_can_pay() {
    let rules = rules();
    let tree = rules.adventure().object_type_by_key("t:tree").unwrap();
    let mut state = game(&rules, reward_at(&rules, "t:tree", Holds::Nothing));
    visit(&mut state, &rules);
    let offer = state.decision(RED).unwrap().clone();
    assert_eq!(
        (offer.object_type, offer.kind.clone()),
        (
            tree,
            OfferKind::Gift {
                price: vec![(GOLD, 1000)]
            }
        )
    );
    let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
    assert!(events.contains(&GameEvent::ResourcePaid {
        player: RED,
        resource: GOLD,
        amount: 1000,
    }));
    assert!(events.contains(&changed(HeroChange::GainExperience(1000))));
    assert_eq!(gold(&state), 1000);
    assert!(state.adventure().is_marked(HERO, 0));
    // A price the treasury cannot pay: refused, told to the player.
    let dear = rules.adventure().object_type_by_key("t:dear_tree").unwrap();
    let mut state = game(&rules, reward_at(&rules, "t:dear_tree", Holds::Nothing));
    assert_eq!(
        visit(&mut state, &rules),
        [GameEvent::OfferRefused {
            player: RED,
            hero: HERO,
            object_type: dear,
            anchor: AT,
        }]
    );
}

// covers: ADV-PIK-011, HER-ADV-008
#[test]
fn the_tombs_question_opens_and_its_morale_reaches_the_hero() {
    let rules = rules();
    let mut state = game(&rules, reward_at(&rules, "t:tomb", Holds::Nothing));
    visit(&mut state, &rules);
    assert_eq!(state.decision(RED).unwrap().kind, OfferKind::Search);
    let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
    let morale = HeroChange::Bonus {
        morale: -3,
        luck: 0,
        mark: Some(0),
    };
    assert!(events.contains(&changed(morale)));
    assert_eq!(state.hero(HERO).unwrap().bonuses.morale, -3);
    assert!(state.adventure().is_emptied(0) && state.adventure().knows(RED, 0));
}

/// Red's victory over a side without a hero at [`AT`], keeping its nine imps.
fn won() -> BattleOutcome {
    BattleOutcome {
        result: BattleResult::Victory(BattleSide::Attacker),
        survivors: [
            Army {
                stacks: vec![ArmyStack {
                    slot: 1,
                    creature: IMP,
                    count: 9,
                }],
            },
            Army { stacks: vec![] },
        ],
        war_machines: [vec![], vec![]],
        stacks: [vec![], vec![]],
        surrender_cost: None,
        heroes: [
            Some(HeroOutcome {
                mana: 0,
                learned: vec![],
            }),
            None,
        ],
        rounds: 1,
    }
}

// covers: ADV-PIK-007, ADV-MON-011
#[test]
fn a_won_guarded_artifact_and_a_beaten_monsters_artifact_reach_the_hero() {
    let rules = rules();
    let guards = Guards {
        army: vec![ArmyStack {
            slot: 0,
            creature: IMP,
            count: 2,
        }],
        question: true,
    };
    let guarded = ObjectSetup {
        guards: Some(guards),
        ..reward_at(&rules, "t:artifact", Holds::Artifact(RING.into()))
    };
    let monster = ObjectSetup {
        monster: Some(MonsterSetup {
            creature: Some(IMP),
            count: 2,
            disposition: 0,
            never_flees: true,
            no_growth: false,
            treasure: vec![(GOLD, 100)],
            artifact: Some(RING),
        }),
        ..reward_at(&rules, "t:monster", Holds::Nothing)
    };
    let sides = ([Some(RED), None], [Some(HERO), None], AT);
    for (object, gained) in [(guarded, 0), (monster, 100)] {
        let mut state = game(&rules, object);
        let mut events = Vec::new();
        state.end_battle(&rules, sides, &won(), &mut events);
        let ring = ArtifactInstance::from(RING);
        assert!(events.contains(&changed(HeroChange::AddArtifact(ring))));
        assert!(state.hero(HERO).unwrap().worn.contains(&Some(ring)));
        assert_eq!(gold(&state), 2000 + gained);
        assert!(state.adventure().is_removed(0));
    }
}
