//! Tests of the hire dialog through the game (GAM-DEC-008; E08 T95b, increment 10's design note
//! §13): an accepted hire question at an own dwelling or a Refugee Camp opens it as its owner's
//! decision, told and shown to the owner alone and hashed; the legal set and `hire_range` give the
//! most of each creature, bounded by its count, each resource of the treasury and the room in the
//! army, every amount in the range accepted; each refusal leaves the state as it was; a purchase
//! pays, joins and keeps a dialog of several creatures open, closes one of a single creature;
//! `Close`, the system's default; no move or end of the turn while it is open; the guard's visit
//! after it closes (GAM-DEC-007).

use h3_adventure::{
    AdventureEvent, AdventureEventKind, HireError, HireTarget, Holds, ObjectSetup, ObjectTypeId,
    Viewers,
};
use h3_content::ResourceId;

use super::decisions::{ACCEPT, CLOSE, move_hero, play};
use super::dwellings::{ANCHOR, BLUE, GOG, RED, RED_HERO, hut_type, rules_edited};
use super::manage::imp;
use super::rewards::{guard, reward_at, starts_battle};
use super::*;

const GOLD: ResourceId = ResourceId::from_raw(0);
const GEM: ResourceId = ResourceId::from_raw(1);

/// [`rules_edited`]'s dwelling rules (the hut of imps and gogs, the free creatures up to level 1)
/// with a second resource, gems, the gogs (level 2) growing 6 a week for 2 gems (listed first:
/// the price's order is the core's) and 100 gold each (a gem mine, as the abandoned mine needs a
/// mine of each resource), and a Refugee Camp `t:camp` of level 2: the gogs.
fn rules() -> GameRuleset {
    rules_edited(5, |core, part| {
        core.resources.push(ResourceDef {
            key: key("t:gem"),
            name: "Gem".into(),
        });
        part.object_types.push(entered(
            "t:gem_mine",
            ObjectKindDef::Mine {
                resource: key("t:gem"),
                daily: 1,
                battle_terrain: None,
            },
        ));
        core.creatures[1].info.growth = 6;
        core.creatures[1].info.cost = vec![(key("t:gem"), 2), (key("t:gold"), 100)];
        part.object_types.push(ObjectTypeDef {
            from_above: false,
            ..entered("t:camp", ObjectKindDef::RefugeeCamp { levels: [2, 2] })
        });
    })
}

fn camp_type(rules: &GameRuleset) -> ObjectTypeId {
    rules.adventure().object_type_by_key("t:camp").unwrap()
}

/// Red's hut at [`ANCHOR`].
fn own_hut(rules: &GameRuleset) -> ObjectSetup {
    ObjectSetup {
        owner: Some(RED),
        ..reward_at(rules, "t:hut", Holds::Nothing)
    }
}

/// [`setup`] with `objects`, red's hero at (0, 0) with `army` (2 army slots), 2 000 gold and
/// `gems` gems.
fn game_of(
    rules: &GameRuleset,
    objects: Vec<ObjectSetup>,
    army: Vec<ArmyStack>,
    gems: u32,
) -> GameState {
    let mut setup = setup();
    setup.heroes[0].army = HeroArmy::Custom(army);
    setup.map.objects = objects;
    let mut state = GameState::new(rules, &setup, 7).unwrap();
    state.set_resource(RED, GEM, gems);
    state
}

/// `count` gogs in `slot`.
const fn gogs(slot: u8, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature: GOG,
        count,
    }
}

/// Red's hero steps onto [`ANCHOR`] and accepts the hire question: the acceptance's events.
fn open(state: &mut GameState, rules: &GameRuleset) -> Vec<GameEvent> {
    play(state, rules, RED, move_hero(0, &[ANCHOR])).unwrap();
    play(state, rules, RED, ACCEPT).unwrap()
}

/// [`game_of`] with red's hut and its hire dialog open: 6 gogs to buy, the imps free.
fn hut_open(rules: &GameRuleset, army: Vec<ArmyStack>, gems: u32) -> GameState {
    let mut state = game_of(rules, vec![own_hut(rules)], army, gems);
    open(&mut state, rules);
    state
}

const fn buy(creature: CreatureTypeId, amount: u32) -> GameCommand {
    GameCommand::Answer(Answer::Hire(HireAnswer::Buy { creature, amount }))
}

const HIRE_CLOSE: GameCommand = GameCommand::Answer(Answer::Hire(HireAnswer::Close));

/// The hire dialog `player` sees.
fn seen(state: &GameState, rules: &GameRuleset, player: PlayerId) -> Option<HireView> {
    match observe(state, rules, Viewer::Player(player)).decision {
        Some(DecisionView::Hire(view)) => Some(view),
        _ => None,
    }
}

fn joined(creature: CreatureTypeId, count: u32) -> GameEvent {
    GameEvent::Adventure(AdventureEvent {
        kind: AdventureEventKind::CreaturesJoined {
            hero: RED_HERO,
            creature,
            count,
        },
        seen_by: Viewers::NONE.with(RED),
    })
}

const fn paid(resource: ResourceId, amount: u32) -> GameEvent {
    GameEvent::ResourcePaid {
        player: RED,
        resource,
        amount,
    }
}

// covers: GAM-DEC-008, ADV-HIR-002, ADV-RFG-001
#[test]
fn an_accepted_hire_question_opens_the_dialog_to_its_owner_alone() {
    let rules = rules();
    let mut state = game_of(&rules, vec![own_hut(&rules)], vec![imp(1, 9)], 100);
    let events = open(&mut state, &rules);
    // After the free imps, the dialog: told to red alone, by the hut's type and anchor.
    let opened = GameEvent::HireOpened {
        player: RED,
        target: HireTarget::Hero(RED_HERO),
        object_type: hut_type(&rules),
        anchor: ANCHOR,
    };
    assert_eq!(events[1..], [joined(IMP, 3), opened]);
    assert_eq!(project(&opened, Viewer::Player(RED)), Some(opened));
    assert_eq!(project(&opened, Viewer::Player(BLUE)), None);
    // Its view: the hero, the hut, each creature with its count and the cost of one in the
    // core's order (gold, then gems); no most. Blue sees nothing.
    let view = HireView {
        target: HireTarget::Hero(RED_HERO),
        object_type: hut_type(&rules),
        anchor: ANCHOR,
        creatures: vec![
            HireCreature {
                creature: IMP,
                count: 0,
                cost: vec![(GOLD, 50)],
            },
            HireCreature {
                creature: GOG,
                count: 6,
                cost: vec![(GOLD, 100), (GEM, 2)],
            },
        ],
    };
    assert_eq!(seen(&state, &rules, RED), Some(view));
    assert_eq!(observe(&state, &rules, Viewer::Player(BLUE)).decision, None);
    // `Close` (the system's default): `DecisionClosed`; the decision was hashed.
    assert_eq!(
        default_answer(&state, &rules, RED),
        Some(Answer::Hire(HireAnswer::Close))
    );
    assert_eq!(default_answer(&state, &rules, BLUE), None);
    let mut closed = state.clone();
    let events = play(&mut closed, &rules, RED, HIRE_CLOSE).unwrap();
    assert_eq!(events, [GameEvent::DecisionClosed { player: RED }]);
    assert!(!closed.has_decision(RED));
    assert_eq!(seen(&closed, &rules, RED), None);
    assert_ne!(closed.state_hash(), state.state_hash());
    // A Refugee Camp's: its week's creature, the gogs, nothing free.
    let camp = reward_at(&rules, "t:camp", Holds::Nothing);
    let mut state = game_of(&rules, vec![camp], vec![imp(1, 9)], 100);
    let events = open(&mut state, &rules);
    assert_eq!(
        events.last(),
        Some(&GameEvent::HireOpened {
            player: RED,
            target: HireTarget::Hero(RED_HERO),
            object_type: camp_type(&rules),
            anchor: ANCHOR,
        })
    );
    let creatures = seen(&state, &rules, RED).map(|view| view.creatures);
    assert_eq!(
        creatures,
        Some(vec![HireCreature {
            creature: GOG,
            count: 6,
            cost: vec![(GOLD, 100), (GEM, 2)],
        }])
    );
}

// covers: GAM-DEC-008, ADV-ARM-007
#[test]
fn the_most_is_bounded_by_the_count_each_resource_and_the_room() {
    let rules = rules();
    let red = Actor::Player(RED);
    let most = |state: &GameState| {
        let range = hire_range(state, &rules, RED, GOG);
        let legal = legal_commands(state, &rules, red);
        let listed = (*range.end() > 0).then(|| buy(GOG, *range.end()));
        assert_eq!(
            legal,
            listed.into_iter().chain([HIRE_CLOSE]).collect::<Vec<_>>()
        );
        // Every amount in the range is accepted.
        for amount in range.clone() {
            let mut bought = state.clone();
            play(&mut bought, &rules, RED, buy(GOG, amount)).unwrap();
        }
        *range.end()
    };
    // By the count: 6 (2 000 gold pay for 20, 100 gems for 50); the imps' count of 0 lists none.
    let state = hut_open(&rules, vec![imp(1, 9)], 100);
    assert_eq!(most(&state), 6);
    assert!(hire_range(&state, &rules, RED, IMP).is_empty());
    // By the gold, then by the gems: 3 and 1, then none.
    let mut poor = state.clone();
    poor.set_resource(RED, GOLD, 399);
    assert_eq!(most(&poor), 3);
    poor.set_resource(RED, GEM, 3);
    assert_eq!(most(&poor), 1);
    poor.set_resource(RED, GEM, 1);
    assert_eq!(most(&poor), 0);
    // By the room: both slots taken, a stack of gogs one short of a stack's most.
    let state = hut_open(&rules, vec![gogs(0, 999_999), imp(1, 9)], 100);
    assert_eq!(most(&state), 1);
    // No room at all: both slots taken by imps.
    let state = hut_open(&rules, vec![imp(0, 1), imp(1, 9)], 100);
    assert_eq!(most(&state), 0);
    // Empty for a creature the source does not sell, for another player, without a dialog.
    let state = hut_open(&rules, vec![imp(1, 9)], 100);
    assert!(hire_range(&state, &rules, RED, CreatureTypeId::from_raw(9)).is_empty());
    assert!(hire_range(&state, &rules, BLUE, GOG).is_empty());
    let mut closed = state;
    play(&mut closed, &rules, RED, HIRE_CLOSE).unwrap();
    assert!(hire_range(&closed, &rules, RED, GOG).is_empty());
}

// covers: GAM-DEC-008, ADV-HIR-002, ADV-ARM-007
#[test]
fn each_refused_purchase_leaves_the_state_as_it_was() {
    let rules = rules();
    let state = hut_open(&rules, vec![gogs(0, 999_999), imp(1, 9)], 100);
    let mut poor = state.clone();
    poor.set_resource(RED, GOLD, 0);
    let other = CreatureTypeId::from_raw(9);
    for (state, command, error) in [
        (
            &state,
            buy(other, 1),
            GameCommandError::Hire(HireError::NotOffered(other)),
        ),
        (
            &state,
            buy(IMP, 1),
            GameCommandError::Hire(HireError::Amount(1)),
        ),
        (
            &state,
            buy(GOG, 0),
            GameCommandError::Hire(HireError::Amount(0)),
        ),
        // The amount before the room, the room before the treasury.
        (
            &state,
            buy(GOG, 7),
            GameCommandError::Hire(HireError::Amount(7)),
        ),
        (
            &state,
            buy(GOG, 2),
            GameCommandError::Hire(HireError::NoRoom),
        ),
        (
            &poor,
            buy(GOG, 2),
            GameCommandError::Hire(HireError::NoRoom),
        ),
        (&poor, buy(GOG, 1), GameCommandError::CannotPay),
        // The exchange dialogs' `Close` and another kind's answer.
        (&state, CLOSE, GameCommandError::WrongAnswer),
        (
            &state,
            GameCommand::Answer(Answer::University(UniversityAnswer::Leave)),
            GameCommandError::WrongAnswer,
        ),
    ] {
        let mut refused = state.clone();
        assert_eq!(play(&mut refused, &rules, RED, command), Err(error));
        assert_eq!(refused.state_hash(), state.state_hash());
    }
    // Gems short: refused too.
    let mut no_gems = state.clone();
    no_gems.set_resource(RED, GEM, 1);
    let mut refused = no_gems.clone();
    let error = play(&mut refused, &rules, RED, buy(GOG, 1));
    assert_eq!(error, Err(GameCommandError::CannotPay));
    assert_eq!(refused.state_hash(), no_gems.state_hash());
    // A hire answer to another kind of decision: the hire question itself.
    let mut asked = game_of(&rules, vec![own_hut(&rules)], vec![imp(1, 9)], 100);
    play(&mut asked, &rules, RED, move_hero(0, &[ANCHOR])).unwrap();
    assert_eq!(
        play(&mut asked, &rules, RED, HIRE_CLOSE),
        Err(GameCommandError::WrongAnswer)
    );
    // Each refusal reads.
    assert_eq!(
        GameCommandError::Hire(HireError::NoRoom).to_string(),
        "the army has no room for the creatures"
    );
}

// covers: GAM-DEC-008, ADV-HIR-002
#[test]
fn a_purchase_pays_and_joins_and_a_dialog_of_several_creatures_stays_open() {
    let rules = rules();
    let mut state = hut_open(&rules, vec![imp(1, 9)], 100);
    // Two of the six: the price by resource in the core's order, then the gogs join slot 0.
    let events = play(&mut state, &rules, RED, buy(GOG, 2)).unwrap();
    assert_eq!(events, [paid(GOLD, 200), paid(GEM, 4), joined(GOG, 2)]);
    assert_eq!(state.treasury(RED), Some(&[1800, 96][..]));
    let army = &state.adventure().hero(RED_HERO).unwrap().army;
    assert_eq!(army[..], [gogs(0, 2), imp(1, 12)]);
    assert_eq!(state.adventure().dwelling_counts(0), Some(&[0, 4][..]));
    // Still open, its counts read again.
    let view = seen(&state, &rules, RED).unwrap();
    assert_eq!(view.creatures[1].count, 4);
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(RED)),
        [buy(GOG, 4), HIRE_CLOSE]
    );
    // Moves, management and the end of the turn wait for it.
    let formation = GameCommand::Manage(Manage::Formation {
        hero: RED_HERO,
        formation: Formation::Tight,
    });
    for command in [
        GameCommand::EndTurn,
        move_hero(0, &[Pos::new(0, 0, 0)]),
        formation,
    ] {
        let mut refused = state.clone();
        assert_eq!(
            play(&mut refused, &rules, RED, command),
            Err(GameCommandError::DecisionOpen)
        );
    }
    // The rest: onto the stack, the dialog still open (none left to buy) until closed.
    let events = play(&mut state, &rules, RED, buy(GOG, 4)).unwrap();
    assert_eq!(events, [paid(GOLD, 400), paid(GEM, 8), joined(GOG, 4)]);
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(RED)),
        [HIRE_CLOSE]
    );
    play(&mut state, &rules, RED, HIRE_CLOSE).unwrap();
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(RED)),
        [GameCommand::EndTurn]
    );
}

// covers: GAM-DEC-008, ADV-RFG-001
#[test]
fn a_single_creature_s_dialog_closes_after_its_purchase() {
    let rules = rules();
    let camp = reward_at(&rules, "t:camp", Holds::Nothing);
    let mut state = game_of(&rules, vec![camp], vec![imp(1, 9)], 100);
    open(&mut state, &rules);
    let events = play(&mut state, &rules, RED, buy(GOG, 1)).unwrap();
    assert_eq!(
        events,
        [
            paid(GOLD, 100),
            paid(GEM, 2),
            joined(GOG, 1),
            GameEvent::DecisionClosed { player: RED },
        ]
    );
    assert!(!state.has_decision(RED));
    assert_eq!(state.adventure().camp(0), Some((GOG, 5)));
}

// covers: GAM-DEC-008, GAM-DEC-007, ADV-MON-006
#[test]
fn the_guard_visits_once_the_dialog_closes() {
    let rules = rules();
    let mut state = game_of(
        &rules,
        vec![own_hut(&rules), guard(&rules)],
        vec![imp(1, 9)],
        100,
    );
    let events = open(&mut state, &rules);
    assert!(!starts_battle(&events));
    assert!(matches!(events.last(), Some(GameEvent::HireOpened { .. })));
    // A purchase keeps it open and the guard waiting; its closing lets the guard fight.
    let events = play(&mut state, &rules, RED, buy(GOG, 1)).unwrap();
    assert!(!starts_battle(&events));
    let events = play(&mut state, &rules, RED, HIRE_CLOSE).unwrap();
    assert_eq!(events[0], GameEvent::DecisionClosed { player: RED });
    assert!(starts_battle(&events), "{events:?}");
}
