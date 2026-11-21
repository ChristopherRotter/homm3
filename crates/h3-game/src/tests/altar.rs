//! Tests of the Altar of Sacrifice's decision (GAM-DEC-012; E08 T134a, increment 14's design note
//! §14): an Altar's visit opens it for its visitor's owner alone, it blocks the turn until its
//! `Leave` (the system's default), a sacrifice of artifacts (ADV-ALT-002) or creatures
//! (ADV-ALT-003) named whole in one answer (A-48 fixed as `ADV-QUIRK-50`) keeps it open, its
//! levels given at once but a level-up's, before which the Altar closes and after which it
//! returns (GAM-DEC-007 amended, decision 371); the view with the experience (ADV-ALT-001). A
//! rejected answer changes nothing ([`play`] checks it).

use h3_adventure::{AltarDef, AltarFactionDef, Holds, ObjectKindDef};
use h3_heroes::{HeroDetails, Position};

use super::decisions::{BLUE, RED, play};
use super::rewards::{AT, HERO, answered, changed, reward_at, visit};
use super::*;

const BOOK: ArtifactTypeId = ArtifactTypeId::from_raw(0);
const CHARM: ArtifactTypeId = ArtifactTypeId::from_raw(2);
const RING: ArtifactTypeId = ArtifactTypeId::from_raw(4);
/// The core's Learning ("Lore": 5 %, 10 %, 15 %).
const LORE: SkillId = SkillId::from_raw(1);

/// [`rules`] with the ring a major artifact, the imp's AI value 80 (10 experience at the Altar),
/// and an entered, kind-marked `t:altar` with the base pack's values (1 000 to 6 000 by class, 0
/// for a special one; the book kept; 23 places; 5 × ⌊AI value / 40⌋), the core's one faction's
/// heroes sacrificing both.
fn altar_rules() -> GameRuleset {
    altar_rules_of(23)
}

/// [`altar_rules`] with an Altar of `places` places.
fn altar_rules_of(places: u8) -> GameRuleset {
    let mut pack = core_pack(false);
    pack.artifacts[RING.index()].class = ArtifactClass::Major;
    pack.creatures[IMP.index()].info.ai_value = 80;
    let core = Arc::new(Ruleset::compile(&[pack], "t:faithful").unwrap());
    let mut adventure = adventure_pack(vec![88, 25], 2);
    adventure.object_types.push(ObjectTypeDef {
        kind_mark: true,
        ..entered(
            "t:altar",
            ObjectKindDef::Altar(AltarDef {
                experience: [1000, 1500, 3000, 6000, 0],
                keeps: vec![key("t:book")],
                places,
                creature_factor: 5,
                creature_divisor: 40,
            }),
        )
    });
    adventure.altar_factions = vec![AltarFactionDef {
        faction: key("t:town"),
        artifacts: true,
        creatures: true,
    }];
    let part = AdventureRuleset::compile(&core, &[adventure]).unwrap();
    let battle = battle_part(&core, 7);
    GameRuleset::new(
        Arc::clone(&core),
        Arc::new(part),
        Arc::new(battle),
        no_towns(&core),
    )
    .unwrap()
}

/// A game with the Altar at [`AT`], red's hero of `experience` and `skills` wearing the book and
/// the ring and carrying two charms visiting it: the game and the events of the visit.
fn at_altar(
    rules: &GameRuleset,
    experience: u32,
    skills: Vec<(SkillId, u8)>,
) -> (GameState, Vec<GameEvent>) {
    at_altar_with(rules, (experience, skills), setup())
}

/// [`at_altar`] from `setup` (its hero's army).
fn at_altar_with(
    rules: &GameRuleset,
    (experience, skills): (u32, Vec<(SkillId, u8)>),
    mut setup: GameSetup,
) -> (GameState, Vec<GameEvent>) {
    setup.map.objects = vec![reward_at(rules, "t:altar", Holds::Nothing)];
    setup.heroes[0].details = HeroDetailsSetup {
        experience: Some(experience),
        skills: Some(skills),
        artifacts: Some(HeroArtifactsSetup {
            worn: vec![Some(BOOK.into()), None, Some(RING.into()), None],
            backpack: vec![CHARM.into(), CHARM.into()],
        }),
        ..HeroDetailsSetup::default()
    };
    let mut state = GameState::new(rules, &setup, 7).unwrap();
    let events = visit(&mut state, rules);
    (state, events)
}

/// An Altar's answer as red's command.
const fn altar(answer: AltarAnswer) -> GameCommand {
    answered(Answer::Altar(answer))
}

/// A sacrifice of the artifacts at `places`.
fn artifacts(places: &[ArtifactPlace]) -> GameCommand {
    let set = places
        .iter()
        .fold(AltarPlaces::NONE, |set, &p| set.with(p).unwrap());
    altar(AltarAnswer::Artifacts(set))
}

const fn worn(p: usize) -> ArtifactPlace {
    ArtifactPlace::Worn(Position(p))
}

/// The core's four skills at Expert, every one a class's level-up offers: its levels offer none.
fn expert() -> Vec<(SkillId, u8)> {
    (0..4).map(|skill| (SkillId::from_raw(skill), 3)).collect()
}

/// The refusal of an Altar's answer.
const fn refused(error: AltarError) -> Result<Vec<GameEvent>, GameCommandError> {
    Err(GameCommandError::Altar(error))
}

/// Red's hero's details.
fn details(state: &GameState) -> &HeroDetails {
    &state.heroes()[usize::from(HERO.raw())]
}

/// The Altar's view red sees.
fn view(state: &GameState, rules: &GameRuleset) -> AltarView {
    match observe(state, rules, Viewer::Player(RED)).decision {
        Some(DecisionView::Altar(view)) => view,
        other => panic!("{other:?}"),
    }
}

// covers: GAM-DEC-012, ADV-ALT-001, ADV-ALT-002, ADV-ALT-003, ADV-BON-009
#[test]
fn an_altars_visit_opens_its_decision_for_its_owner_until_it_is_left() {
    let rules = altar_rules();
    let ty = rules.adventure().object_type_by_key("t:altar").unwrap();
    let (mut state, events) = at_altar(&rules, 0, vec![(LORE, 1)]);
    let opened = GameEvent::AltarOpened {
        player: RED,
        hero: HERO,
        object_type: ty,
        anchor: AT,
    };
    // The team's kind mark, then the decision.
    let known = GameEvent::Adventure(h3_adventure::AdventureEvent {
        kind: h3_adventure::AdventureEventKind::KindKnown { object_type: ty },
        seen_by: h3_adventure::Viewers::NONE.with(RED),
    });
    assert_eq!(events, [known, opened]);
    assert_eq!(project(&opened, Viewer::Player(BLUE)), None);
    let open = state.altar(RED).copied().unwrap();
    assert_eq!((open.hero, open.object_type, open.anchor), (HERO, ty, AT));
    assert_eq!(state.altar(BLUE), None);
    // The view: both screens, the 23 places, each artifact with its experience with Basic Learning
    // (the book kept, a charm's 1 050, the major ring's 3 150), the nine imps less one at 10 each,
    // 84 with Learning; 1 000 to the next level.
    let artifact = |place, artifact, experience| AltarArtifact {
        place,
        artifact,
        experience,
    };
    let seen = AltarView {
        hero: HERO,
        object_type: ty,
        anchor: AT,
        offers: AltarOffers {
            screens: AltarScreens {
                artifacts: true,
                creatures: true,
            },
            places: 23,
            artifacts: vec![
                artifact(worn(0), BOOK, None),
                artifact(worn(2), RING, Some(3150)),
                artifact(ArtifactPlace::Backpack(0), CHARM, Some(1050)),
                artifact(ArtifactPlace::Backpack(1), CHARM, Some(1050)),
            ],
            stacks: vec![AltarStack {
                slot: 1,
                each: 10,
                most: 8,
                experience: 84,
            }],
            to_next_level: 1000,
        },
    };
    assert_eq!(view(&state, &rules), seen);
    // Listed: the two alls (E08 T135a: the ring and both charms, the book kept; the imps but
    // one), then its `Leave`, the system's default; the turn waits for it.
    let leave = Answer::Altar(AltarAnswer::Leave);
    let back = ArtifactPlace::Backpack;
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(RED)),
        [
            artifacts(&[worn(2), back(0), back(1)]),
            altar(AltarAnswer::Creatures([0, 8, 0, 0, 0, 0, 0])),
            answered(leave)
        ]
    );
    assert_eq!(default_answer(&state, &rules, RED), Some(leave));
    let other_kinds = [
        Answer::Offer(Consent::Decline),
        Answer::Market(MarketAnswer::Leave),
        Answer::Tavern(TavernAnswer::Close),
        Answer::Close,
    ];
    for answer in other_kinds {
        let wrong = play(&mut state, &rules, RED, answered(answer));
        assert_eq!(wrong, Err(GameCommandError::WrongAnswer), "{answer:?}");
    }
    let ended = play(&mut state, &rules, RED, GameCommand::EndTurn);
    assert_eq!(ended, Err(GameCommandError::DecisionOpen));
    // Hashed with the decision (game state hash v36).
    let mut left = state.clone();
    assert_eq!(
        play(&mut left, &rules, RED, answered(leave)),
        Ok(vec![GameEvent::DecisionClosed { player: RED }])
    );
    assert_ne!(state.state_hash(), left.state_hash());
    assert_eq!(left.altar(RED), None);
    assert_eq!(default_answer(&left, &rules, RED), None);
    // Leaving loses nothing: nothing lies on the altar between commands (A-48 fixed).
    assert_eq!(details(&left), details(&state));
    assert_eq!(
        left.adventure().hero(HERO).unwrap().army,
        state.adventure().hero(HERO).unwrap().army
    );
}

// covers: GAM-DEC-001, GAM-DEC-012
#[test]
fn another_players_answer_to_an_altars_open_decision_is_refused() {
    // Red's Altar open; blue (no decision of its own) answers it with a valid-looking answer of
    // its kind: refused, nothing changed or told ([`play`] checks it).
    let rules = altar_rules();
    let (mut state, _) = at_altar(&rules, 0, vec![(LORE, 1)]);
    let answers = [
        artifacts(&[worn(2)]),
        altar(AltarAnswer::Creatures([0, 8, 0, 0, 0, 0, 0])),
        altar(AltarAnswer::Leave),
    ];
    for answer in answers {
        assert_eq!(
            play(&mut state, &rules, BLUE, answer.clone()),
            Err(GameCommandError::NoDecision),
            "{answer:?}"
        );
    }
    assert!(state.altar(RED).is_some());
    assert_eq!(state.altar(BLUE), None);
}

// covers: ADV-ALT-002, GAM-DEC-012, HER-EXP-001
#[test]
fn a_sacrifice_of_artifacts_removes_them_gives_their_experience_and_keeps_the_altar_open() {
    let rules = altar_rules();
    // Level 14 at 28 784 (past the pack's twelve each step a fifth more), 34 140 the next; every
    // skill at Expert, so the levels at load raise none (Expert Learning: 15 %).
    let (mut state, _) = at_altar(&rules, 28_784, expert());
    let before = details(&state).experience;
    // The ring and the second charm: removed worn first, then the backpack's from the last; 3 450
    // and 1 150 together, below the next level.
    let back = ArtifactPlace::Backpack(1);
    let refusals = [
        (artifacts(&[worn(0)]), AltarError::Kept(BOOK)),
        (artifacts(&[worn(1)]), AltarError::Places),
        (artifacts(&[]), AltarError::Nothing),
    ];
    for (command, error) in refusals {
        assert_eq!(play(&mut state, &rules, RED, command), refused(error));
    }
    let events = play(&mut state, &rules, RED, artifacts(&[back, worn(2)])).unwrap();
    let removed = |place| changed(HeroChange::RemoveArtifact(place));
    let gained = changed(HeroChange::GainExperience(3450 + 1150));
    assert_eq!(events, [removed(worn(2)), removed(back), gained]);
    let after = details(&state);
    assert_eq!(after.experience, before + 4600);
    assert_eq!(after.worn[2], None);
    assert_eq!(after.backpack, [CHARM.into()]);
    // The Altar stays open for more.
    assert!(state.altar(RED).is_some());
    assert_eq!(
        GameCommandError::Altar(AltarError::Nothing).to_string(),
        "the sacrifice gives no experience"
    );
}

// covers: ADV-ALT-003, GAM-DEC-012
#[test]
fn a_sacrifice_of_creatures_lowers_its_stacks_and_keeps_the_army_a_creature() {
    let rules = altar_rules();
    let (mut state, _) = at_altar(&rules, 0, vec![(LORE, 1)]);
    let all = altar(AltarAnswer::Creatures([0, 9, 0, 0, 0, 0, 0]));
    assert_eq!(
        play(&mut state, &rules, RED, all),
        refused(AltarError::LastCreature)
    );
    let empty = altar(AltarAnswer::Creatures([1, 0, 0, 0, 0, 0, 0]));
    assert_eq!(
        play(&mut state, &rules, RED, empty),
        refused(AltarError::Amount(0))
    );
    // Five imps of 10: 52.5 with Basic Learning, truncated.
    let five = altar(AltarAnswer::Creatures([0, 5, 0, 0, 0, 0, 0]));
    let events = play(&mut state, &rules, RED, five).unwrap();
    let at = ArmySlot {
        army: ArmyRef::Hero(HERO),
        slot: 1,
    };
    let lowered = GameEvent::StackChanged {
        player: RED,
        at,
        stack: Some(ArmyStack {
            slot: 1,
            creature: IMP,
            count: 4,
        }),
    };
    let gained = changed(HeroChange::GainExperience(52));
    assert_eq!(events, [lowered, gained]);
    assert_eq!(details(&state).experience, 52);
    assert!(state.altar(RED).is_some());
}

// covers: GAM-DEC-012, GAM-DEC-002, GAM-DEC-007
#[test]
fn a_sacrifices_level_up_closes_the_altar_and_its_visit_returns_after_it() {
    let rules = altar_rules();
    let (mut state, _) = at_altar(&rules, 0, vec![(LORE, 1)]);
    // The ring's 3 150: levels 2 and 3, each with two offers. The Altar closes before the first.
    let events = play(&mut state, &rules, RED, artifacts(&[worn(2)])).unwrap();
    let closed = GameEvent::DecisionClosed { player: RED };
    let at = events.iter().position(|e| *e == closed).unwrap();
    assert!(matches!(
        events[at + 1],
        GameEvent::LevelUpOpened { level: 2, .. }
    ));
    assert_eq!(events.len(), at + 2);
    assert_eq!(state.altar(RED), None);
    assert_eq!(state.level_up(RED).map(|view| view.level), Some(2));
    // The second level-up replaces the first; once it is answered the Altar opens again.
    let first = || answered(Answer::LevelUp(SkillOffer::First));
    let events = play(&mut state, &rules, RED, first()).unwrap();
    assert!(
        events
            .iter()
            .all(|e| !matches!(e, GameEvent::AltarOpened { .. }))
    );
    assert_eq!(state.level_up(RED).map(|view| view.level), Some(3));
    let events = play(&mut state, &rules, RED, first()).unwrap();
    let ty = rules.adventure().object_type_by_key("t:altar").unwrap();
    let opened = GameEvent::AltarOpened {
        player: RED,
        hero: HERO,
        object_type: ty,
        anchor: AT,
    };
    assert_eq!(events.last(), Some(&opened));
    assert_eq!(state.altar(RED).map(|visit| visit.hero), Some(HERO));
    assert_eq!(details(&state).level, 3);
    // A hero whose levels offer nothing gains them at once, the Altar staying open: the ring's
    // 3 450 with Expert Learning, levels 2 to 4.
    let (mut state, _) = at_altar(&rules, 0, expert());
    let events = play(&mut state, &rules, RED, artifacts(&[worn(2)])).unwrap();
    assert!(!events.contains(&closed));
    let gained = events
        .iter()
        .filter(|e| matches!(e, GameEvent::LevelGained { .. }))
        .count();
    assert_eq!(gained, 3);
    assert!(state.altar(RED).is_some());
}

// covers: GAM-DEC-012, ADV-ALT-002, ADV-ALT-003
#[test]
fn the_listed_alls_fill_the_altars_places_and_keep_the_lowest_slot_a_creature() {
    // Two places; four imps in slot 0 and one in slot 1 (the rules' two). Level 14 with every skill at Expert, so
    // no sacrifice here reaches the next level.
    let rules = altar_rules_of(2);
    let mut setup = setup();
    let stack = |slot, count| ArmyStack {
        slot,
        creature: IMP,
        count,
    };
    setup.heroes[0].army = HeroArmy::Custom(vec![stack(0, 4), stack(1, 1)]);
    let (mut state, _) = at_altar_with(&rules, (28_784, expert()), setup);
    let listed = |state: &GameState| legal_commands(state, &rules, Actor::Player(RED));
    // The ring and the first charm fill the two places (the book kept); every imp but one of
    // the lowest occupied slot's.
    let back = ArtifactPlace::Backpack;
    let creatures = altar(AltarAnswer::Creatures([3, 1, 0, 0, 0, 0, 0]));
    let leave = altar(AltarAnswer::Leave);
    assert_eq!(
        listed(&state),
        [
            artifacts(&[worn(2), back(0)]),
            creatures.clone(),
            leave.clone()
        ]
    );
    for command in listed(&state) {
        let mut probe = state.clone();
        assert!(
            play(&mut probe, &rules, RED, command.clone()).is_ok(),
            "{command:?}"
        );
    }
    // The army's lone imp gives nothing: the creatures' all is listed no more.
    play(&mut state, &rules, RED, creatures).unwrap();
    assert_eq!(listed(&state), [artifacts(&[worn(2), back(0)]), leave]);
}
