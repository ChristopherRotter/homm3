//! Tests of what the game applies of increment 7's pickups, teachers and finds (E08 T62a; ADR-0017
//! point 10): a visit's hero changes and experience with its level-ups, the gift's price and the
//! find's question as offers, the choices and the University as decisions with their typed
//! answers (T63a), a won guarded artifact and a beaten monster's artifact.

use h3_adventure::{Guards, Holds, MonsterSetup, ObjectSetup, OfferKind, TileMask};
use h3_contracts::{Army, BattleOutcome, BattleResult, BattleSide, HeroOutcome};
use h3_heroes::SlotState;
use h3_heroes::{ArtifactInstance, HeroChange};

use super::decisions::{ACCEPT, BLUE, DECLINE, GOLD, RED, after_steps, move_hero, play};
use super::*;

pub(super) const HERO: HeroId = HeroId::new(0);
/// The tile the objects stand on, beside red's hero at (0, 0).
pub(super) const AT: Pos = Pos::new(1, 0, 0);
const RING: ArtifactTypeId = ArtifactTypeId::from_raw(4);

/// An object of type `k` at [`AT`], holding `holds`.
pub(super) fn reward_at(rules: &GameRuleset, k: &str, holds: Holds) -> ObjectSetup {
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
pub(super) fn game(rules: &GameRuleset, object: ObjectSetup) -> GameState {
    let mut setup = setup();
    setup.map.objects = vec![object];
    GameState::new(rules, &setup, 7).unwrap()
}

/// Red's hero visits the object at [`AT`]: the events after its step.
pub(super) fn visit(state: &mut GameState, rules: &GameRuleset) -> Vec<GameEvent> {
    after_steps(play(state, rules, RED, move_hero(0, &[AT])).unwrap())
}

/// The change told to red of its hero.
pub(super) fn changed(change: HeroChange) -> GameEvent {
    GameEvent::HeroChanged {
        player: RED,
        hero: HERO,
        change,
    }
}

/// Red's gold.
pub(super) fn gold(state: &GameState) -> u32 {
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

/// The answer command of `answer`.
pub(super) const fn answered(answer: Answer) -> GameCommand {
    GameCommand::Answer(answer)
}

/// Every answer of another kind than `kind`'s, each rejected as `WrongAnswer` by a decision of
/// that kind (E08 T63a), with the state unchanged ([`play`] checks it).
fn wrong_kinds(state: &mut GameState, rules: &GameRuleset, right: &[Answer]) {
    let all = [
        Answer::Offer(Consent::Accept),
        Answer::Offer(Consent::Decline),
        Answer::LevelUp(SkillOffer::First),
        Answer::Chest(ChestChoice::Gold),
        Answer::Arena(ArenaChoice::Attack),
        Answer::School(SchoolChoice::Leave),
        Answer::University(UniversityAnswer::Leave),
        Answer::Close,
    ];
    for answer in all.into_iter().filter(|a| !right.contains(a)) {
        let refused = play(state, rules, RED, answered(answer));
        assert_eq!(refused, Err(GameCommandError::WrongAnswer), "{answer:?}");
    }
}

/// The open decision's view as red sees it.
fn seen(state: &GameState, rules: &GameRuleset) -> Option<DecisionView> {
    observe(state, rules, Viewer::Player(RED)).decision
}

// covers: ADV-PIK-005, GAM-DEC-005
#[test]
fn a_chests_choice_opens_as_a_decision_and_gives_its_gold_or_experience() {
    let rules = rules();
    let chest = rules.adventure().object_type_by_key("t:chest").unwrap();
    let mut state = game(&rules, reward_at(&rules, "t:chest", Holds::Nothing));
    let opened = GameEvent::ChoiceOpened {
        player: RED,
        hero: HERO,
        object_type: chest,
        anchor: AT,
    };
    // The chest leaves the map with the visit; the choice is red's alone, shown with both amounts.
    assert_eq!(visit(&mut state, &rules).last(), Some(&opened));
    assert!(state.adventure().is_removed(0));
    let choice = state.choice(RED).unwrap().clone();
    let ChoiceKind::Chest {
        gold: amount,
        experience,
    } = choice.kind
    else {
        panic!("{choice:?}")
    };
    assert_eq!((amount, experience), ((GOLD, 1500), 1000));
    assert_eq!(seen(&state, &rules), Some(DecisionView::Choice(choice)));
    assert_eq!(observe(&state, &rules, Viewer::Player(BLUE)).decision, None);
    assert_eq!(project(&opened, Viewer::Player(BLUE)), None);
    let gold_answer = Answer::Chest(ChestChoice::Gold);
    let experience_answer = Answer::Chest(ChestChoice::Experience);
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(RED)),
        [answered(gold_answer), answered(experience_answer)]
    );
    assert_eq!(default_answer(&state, &rules, RED), Some(gold_answer));
    wrong_kinds(&mut state, &rules, &[gold_answer, experience_answer]);
    let refused = play(&mut state, &rules, RED, GameCommand::EndTurn);
    assert_eq!(refused, Err(GameCommandError::DecisionOpen));
    // Hashed with the decision.
    let mut experience = state.clone();
    let mut fresh = game(&rules, reward_at(&rules, "t:chest", Holds::Nothing));
    assert_ne!(state.state_hash(), fresh.state_hash());
    let events = play(&mut state, &rules, RED, answered(gold_answer)).unwrap();
    let closed = GameEvent::DecisionClosed { player: RED };
    let gained = GameEvent::ResourceGained {
        player: RED,
        resource: GOLD,
        amount: 1500,
    };
    assert_eq!(events, [closed, gained]);
    assert_eq!((gold(&state), state.choice(RED)), (3500, None));
    let events = play(&mut experience, &rules, RED, answered(experience_answer)).unwrap();
    assert_eq!(
        events[..2],
        [closed, changed(HeroChange::GainExperience(1000))]
    );
    assert_eq!(gold(&experience), 2000);
    // The other answer is no longer legal.
    let refused = play(&mut fresh, &rules, RED, answered(gold_answer));
    assert_eq!(refused, Err(GameCommandError::NoDecision));
}

// covers: ADV-TCH-001, GAM-DEC-005
#[test]
fn the_arena_is_answered_with_one_of_its_skills_and_cannot_be_left() {
    let rules = rules();
    let mut state = game(&rules, reward_at(&rules, "t:arena", Holds::Nothing));
    visit(&mut state, &rules);
    let (attack, defense) = (
        Answer::Arena(ArenaChoice::Attack),
        Answer::Arena(ArenaChoice::Defense),
    );
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(RED)),
        [answered(attack), answered(defense)]
    );
    assert_eq!(default_answer(&state, &rules, RED), Some(attack));
    wrong_kinds(&mut state, &rules, &[attack, defense]);
    // Hashed with the decision.
    let fresh = game(&rules, reward_at(&rules, "t:arena", Holds::Nothing));
    assert_ne!(state.state_hash(), fresh.state_hash());
    let mut first = state.clone();
    let before = state.hero(HERO).unwrap().primary;
    let events = play(&mut state, &rules, RED, answered(defense)).unwrap();
    let closed = GameEvent::DecisionClosed { player: RED };
    assert_eq!(
        events,
        [closed, changed(HeroChange::GainPrimary([0, 2, 0, 0]))]
    );
    assert_eq!(state.hero(HERO).unwrap().primary[1], before[1] + 2);
    assert!(state.adventure().is_marked(HERO, 0));
    let events = play(&mut first, &rules, RED, answered(attack)).unwrap();
    assert_eq!(
        events,
        [closed, changed(HeroChange::GainPrimary([2, 0, 0, 0]))]
    );
    // Marked: a second visit opens nothing.
    assert!(visit(&mut state, &rules).is_empty());
}

// covers: ADV-TCH-002, GAM-DEC-005
#[test]
fn a_school_opens_when_its_price_is_paid_and_may_be_left() {
    let rules = rules();
    let school = rules.adventure().object_type_by_key("t:school").unwrap();
    let mut state = game(&rules, reward_at(&rules, "t:school", Holds::Nothing));
    visit(&mut state, &rules);
    let answers = [
        SchoolChoice::First,
        SchoolChoice::Second,
        SchoolChoice::Leave,
    ]
    .map(Answer::School);
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(RED)),
        answers.map(answered)
    );
    assert_eq!(default_answer(&state, &rules, RED), Some(answers[2]));
    wrong_kinds(&mut state, &rules, &answers);
    let Some(DecisionView::Choice(choice)) = seen(&state, &rules) else {
        panic!("no choice")
    };
    assert_eq!(choice.price(), [(GOLD, 1000)]);
    // Hashed with the decision.
    let fresh = game(&rules, reward_at(&rules, "t:school", Holds::Nothing));
    assert_ne!(state.state_hash(), fresh.state_hash());
    // Left: nothing paid, no mark, nothing given.
    let mut taken = state.clone();
    let mut first = state.clone();
    let closed = GameEvent::DecisionClosed { player: RED };
    let events = play(&mut state, &rules, RED, answered(answers[2])).unwrap();
    assert_eq!(events, [closed]);
    assert_eq!(gold(&state), 2000);
    assert!(!state.adventure().is_marked(HERO, 0));
    // Taken: the price, then the skill and the mark.
    let events = play(&mut taken, &rules, RED, answered(answers[1])).unwrap();
    let paid = GameEvent::ResourcePaid {
        player: RED,
        resource: GOLD,
        amount: 1000,
    };
    assert_eq!(
        events,
        [closed, paid, changed(HeroChange::GainPrimary([0, 0, 0, 1]))]
    );
    assert_eq!(gold(&taken), 1000);
    assert!(taken.adventure().is_marked(HERO, 0));
    let events = play(&mut first, &rules, RED, answered(answers[0])).unwrap();
    assert_eq!(events[2], changed(HeroChange::GainPrimary([0, 0, 1, 0])));
    // A treasury that cannot pay: refused, told to the player, no decision.
    let mut poor = game(&rules, reward_at(&rules, "t:school", Holds::Nothing));
    poor.set_resource(RED, GOLD, 999);
    let events = visit(&mut poor, &rules);
    let refused = GameEvent::OfferRefused {
        player: RED,
        hero: HERO,
        object_type: school,
        anchor: AT,
    };
    assert_eq!(events, [refused]);
    assert!(!poor.has_decision(RED));
}

// covers: ADV-TCH-007, GAM-DEC-006, HER-CHG-004
#[test]
fn a_university_sells_its_open_slots_until_it_is_left() {
    let rules = rules();
    let university = rules
        .adventure()
        .object_type_by_key("t:university")
        .unwrap();
    let mut state = game(&rules, reward_at(&rules, "t:university", Holds::Nothing));
    state.set_resource(RED, GOLD, 4000);
    let opened = GameEvent::UniversityOpened {
        player: RED,
        hero: HERO,
        object_type: university,
        anchor: AT,
    };
    assert_eq!(visit(&mut state, &rules), [opened]);
    assert_eq!(project(&opened, Viewer::Player(BLUE)), None);
    let visit_of = state.university(RED).unwrap().clone();
    assert_eq!(visit_of.price, [(GOLD, 2000)]);
    let Some(DecisionView::University { visit, slots }) = seen(&state, &rules) else {
        panic!("no University")
    };
    assert_eq!(visit, visit_of);
    let open: Vec<u8> = (0..)
        .zip(&slots)
        .filter(|&(_, &slot)| slot == SlotState::Open)
        .map(|(i, _)| i)
        .collect();
    assert!(!open.is_empty(), "{slots:?}");
    let buy = |slot| Answer::University(UniversityAnswer::Buy(slot));
    let leave = Answer::University(UniversityAnswer::Leave);
    let listed: Vec<GameCommand> = open
        .iter()
        .map(|&slot| answered(buy(slot)))
        .chain([answered(leave)])
        .collect();
    assert_eq!(legal_commands(&state, &rules, Actor::Player(RED)), listed);
    assert_eq!(default_answer(&state, &rules, RED), Some(leave));
    wrong_kinds(&mut state, &rules, &[leave]);
    // Hashed with the decision.
    let mut fresh = game(&rules, reward_at(&rules, "t:university", Holds::Nothing));
    fresh.set_resource(RED, GOLD, 4000);
    assert_ne!(state.state_hash(), fresh.state_hash());
    // A slot it lacks is closed.
    let refused = play(&mut state, &rules, RED, answered(buy(9)));
    assert_eq!(refused, Err(GameCommandError::SlotClosed));
    // A treasury that cannot pay: no purchase listed, and one is refused.
    let slot = open[0];
    let mut poor = state.clone();
    poor.set_resource(RED, GOLD, 1999);
    assert_eq!(
        legal_commands(&poor, &rules, Actor::Player(RED)),
        [answered(leave)]
    );
    let refused = play(&mut poor, &rules, RED, answered(buy(slot)));
    assert_eq!(refused, Err(GameCommandError::CannotPay));
    // A purchase: the price, then the skill at Basic; the University stays open.
    let skill = visit.skills[usize::from(slot)];
    let events = play(&mut state, &rules, RED, answered(buy(slot))).unwrap();
    let paid = GameEvent::ResourcePaid {
        player: RED,
        resource: GOLD,
        amount: 2000,
    };
    let learned = changed(HeroChange::RaiseSkill { skill, level: 1 });
    assert_eq!(events, [paid, learned]);
    assert_eq!(gold(&state), 2000);
    assert!(state.university(RED).is_some());
    // The bought slot is known now: closed.
    let Some(DecisionView::University { slots, .. }) = seen(&state, &rules) else {
        panic!("no University")
    };
    assert_eq!(slots[usize::from(slot)], SlotState::Known);
    let refused = play(&mut state, &rules, RED, answered(buy(slot)));
    assert_eq!(refused, Err(GameCommandError::SlotClosed));
    // Left: closed, told to red.
    let events = play(&mut state, &rules, RED, answered(leave)).unwrap();
    assert_eq!(events, [GameEvent::DecisionClosed { player: RED }]);
    assert!(!state.has_decision(RED));
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
    // The open offer is hashed with its kind and price.
    let mut declined = state.clone();
    play(&mut declined, &rules, RED, DECLINE).unwrap();
    assert_ne!(state.state_hash(), declined.state_hash());
    assert!(!declined.adventure().is_marked(HERO, 0));
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
    let mut declined = state.clone();
    play(&mut declined, &rules, RED, DECLINE).unwrap();
    assert_ne!(state.state_hash(), declined.state_hash());
    assert_eq!(declined.hero(HERO).unwrap().bonuses.morale, 0);
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
pub(super) fn won() -> BattleOutcome {
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
        state.run_rests(&rules, &mut events);
        let ring = ArtifactInstance::from(RING);
        assert!(events.contains(&changed(HeroChange::AddArtifact(ring))));
        assert!(state.hero(HERO).unwrap().worn.contains(&Some(ring)));
        assert_eq!(gold(&state), 2000 + gained);
        assert!(state.adventure().is_removed(0));
    }
}

/// A savage wandering monster of two imps at (2, 0), which guards [`AT`] (ADV-MON-005) and always
/// fights.
pub(super) fn guard(rules: &GameRuleset) -> ObjectSetup {
    ObjectSetup {
        anchor: Pos::new(2, 0, 0),
        monster: Some(MonsterSetup {
            creature: Some(IMP),
            count: 2,
            disposition: 4,
            never_flees: true,
            no_growth: false,
            treasure: vec![],
            artifact: None,
        }),
        ..reward_at(rules, "t:monster", Holds::Nothing)
    }
}

/// Whether `events` start a battle.
pub(super) fn starts_battle(events: &[GameEvent]) -> bool {
    events
        .iter()
        .any(|e| matches!(e, GameEvent::BattleStarted { .. }))
}

// covers: ADV-MON-006, GAM-DEC-005, GAM-DEC-006
#[test]
fn a_guard_waits_for_a_choice_or_a_university_and_follows_a_gift() {
    let rules = rules();
    let with_guard = |k: &str| {
        let mut setup = setup();
        setup.map.objects = vec![reward_at(&rules, k, Holds::Nothing), guard(&rules)];
        GameState::new(&rules, &setup, 7).unwrap()
    };
    // The guard's visit waits for the choice's answer or the University's leaving.
    let leave = Answer::University(UniversityAnswer::Leave);
    for (k, answer) in [
        ("t:in_arena", Answer::Arena(ArenaChoice::Defense)),
        ("t:in_school", Answer::School(SchoolChoice::Leave)),
        ("t:in_university", leave),
    ] {
        let mut state = with_guard(k);
        assert!(!starts_battle(&visit(&mut state, &rules)), "{k}");
        let events = play(&mut state, &rules, RED, answered(answer)).unwrap();
        assert!(starts_battle(&events), "{k}: {events:?}");
    }
    // A purchase keeps the University open and the guard waiting.
    let mut state = with_guard("t:in_university");
    state.set_resource(RED, GOLD, 4000);
    visit(&mut state, &rules);
    if let Some(&GameCommand::Answer(buy)) = legal_commands(&state, &rules, Actor::Player(RED))
        .iter()
        .find(|&c| *c != answered(leave))
    {
        let events = play(&mut state, &rules, RED, answered(buy)).unwrap();
        assert!(!starts_battle(&events));
    }
    // A School the treasury cannot pay: refused, then the guard at once.
    let mut state = with_guard("t:in_school");
    state.set_resource(RED, GOLD, 0);
    let events = visit(&mut state, &rules);
    assert!(matches!(events[0], GameEvent::OfferRefused { .. }) && starts_battle(&events));
    // A gift without a decision: its experience, then the guard's battle (before E08 T63a the
    // monster's answer dropped the hero's gains), once the level-up it brings is answered (before
    // E08 T63b the battle opened beside it, its hero read before the level).
    let mut state = with_guard("t:in_stone");
    let events = visit(&mut state, &rules);
    assert!(events.contains(&changed(HeroChange::GainExperience(1000))));
    assert!(
        state.level_up(RED).is_some() && !starts_battle(&events),
        "{events:?}"
    );
    let first = Answer::LevelUp(SkillOffer::First);
    let events = play(&mut state, &rules, RED, answered(first)).unwrap();
    assert!(starts_battle(&events), "{events:?}");
    let primary = state.hero(HERO).unwrap().primary;
    let fought = state
        .battle(RED)
        .unwrap()
        .state()
        .hero(BattleSide::Attacker);
    let fought = fought.map(|h| [h.attack, h.defense, h.power, h.knowledge]);
    assert_eq!(fought, Some(primary));
}
