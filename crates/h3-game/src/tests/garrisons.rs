//! Tests of the map's garrisons through the game (E08 T114a1; increment 12's design note §6, §7,
//! §14): another's or a neutral garrison is fought at once, unsplit, its owner's side or the
//! system's; a battle on an anti-magic garrison's entrance, the garrison's own or a hero's
//! standing in it, is fought without magic; a win flags the garrison after the winner's levels,
//! a loss leaves its survivors. Since E08 T114a2 (design note §13) the exchange (GAM-DEC-009):
//! opened at a team's garrison, after an empty one's flag and after a won battle's levels; its
//! army commands name the hero's army and the garrison's by the one owner rule, its `Close` only
//! closes.

use h3_adventure::{
    AdventureEvent, AdventureEventKind, ArmyCommand, ArmyCommandError, ArmyRef, ArmySlot, Holds,
    ObjectSetup, TileMask,
};

use super::decisions::{after_steps, move_hero, play, revisit};
use super::manage::imp;
use super::mines::{adventure, fight};
use super::rewards::{answered, win};
use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const RED_HERO: HeroId = HeroId::new(0);

/// A garrison of type `k` at `anchor`, entered there, `owner`'s or neutral, holding `army`,
/// removable.
fn garrison(
    rules: &GameRuleset,
    k: &str,
    anchor: Pos,
    owner: Option<PlayerId>,
    army: Vec<ArmyStack>,
) -> ObjectSetup {
    fixed(rules, (k, anchor), owner, army, true)
}

/// [`garrison`] with its removable flag `removable`.
fn fixed(
    rules: &GameRuleset,
    (k, anchor): (&str, Pos),
    owner: Option<PlayerId>,
    army: Vec<ArmyStack>,
    removable: bool,
) -> ObjectSetup {
    ObjectSetup {
        scenery: Default::default(),
        anchor,
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.adventure().object_type_by_key(k),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds: Holds::Garrison { army, removable },
        owner,
    }
}

/// [`setup`] with `objects`, red's hero at (0, 0) with `imps` imps in slot 1, blue's at (2, 2)
/// with its starting army.
fn game(rules: &GameRuleset, imps: u32, objects: Vec<ObjectSetup>) -> GameState {
    let mut setup = setup();
    setup.heroes[0].army = HeroArmy::Custom(vec![imp(1, imps)]);
    setup.map.objects = objects;
    GameState::new(rules, &setup, 7).unwrap()
}

/// Whether the battle `player` has open is fought without magic, as `player` sees it.
fn anti_magic(state: &GameState, rules: &GameRuleset, player: PlayerId) -> bool {
    observe(state, rules, Viewer::Player(player))
        .battle
        .unwrap()
        .anti_magic
}

// covers: ADV-GAR-002, ADV-GAR-005, GAM-BAT-001, GAM-DEC-007
#[test]
fn a_neutral_anti_magic_garrison_is_fought_without_magic_and_flagged_after_the_win() {
    let rules = rules();
    let anchor = Pos::new(1, 0, 0);
    let ty = rules
        .adventure()
        .object_type_by_key("t:anti_garrison")
        .unwrap();
    let held = vec![imp(1, 1)];
    let object = garrison(&rules, "t:anti_garrison", anchor, None, held.clone());
    let mut state = game(&rules, 300, vec![object]);
    // No question: the battle at once, the system playing the garrison's unsplit army.
    let events = play(&mut state, &rules, RED, move_hero(0, &[anchor])).unwrap();
    assert!(matches!(
        after_steps(events)[..],
        [GameEvent::BattleStarted { .. }, ..]
    ));
    let battle = state.battle(RED).unwrap();
    assert_eq!(battle.owners(), [Some(RED), None]);
    assert_eq!(battle.heroes(), [Some(RED_HERO), None]);
    assert!(anti_magic(&state, &rules, RED));
    let events = win(&mut state, &rules);
    // Won: after the levels, the flag with its reveal; the army gone; then the exchange
    // (ADV-GAR-003), told after the flag.
    let told = adventure(&events);
    assert!(told.iter().any(|e| e.kind
        == AdventureEventKind::ObjectFlagged {
            object_type: ty,
            anchor,
            owner: RED,
        }));
    let opened = GameEvent::ExchangeOpened {
        player: RED,
        hero: RED_HERO,
        garrison: 0,
        object_type: ty,
        anchor,
        removable: true,
    };
    let at = |e: &GameEvent| events.iter().position(|x| x == e);
    assert!(at(&opened) > events.iter().position(|e| matches!(e, GameEvent::Adventure(a) if matches!(a.kind, AdventureEventKind::ObjectFlagged { .. }))));
    let adventure = state.adventure();
    assert_eq!(adventure.owner(0), Some(RED));
    assert_eq!(adventure.garrison_army(0), []);
    assert_eq!(adventure.hero(RED_HERO).unwrap().pos, anchor);
    assert_eq!(state.exchange(RED).map(|e| e.garrison), Some(0));
    play(&mut state, &rules, RED, answered(Answer::Close)).unwrap();
    assert!(!state.has_decision(RED));
}

// covers: ADV-GAR-002, GAM-BAT-001
#[test]
fn an_owned_garrison_defends_as_its_owner_s_side_and_keeps_its_survivors() {
    let rules = rules();
    let anchor = Pos::new(1, 0, 0);
    let held = vec![imp(0, 40)];
    let object = garrison(&rules, "t:garrison", anchor, Some(BLUE), held);
    let mut state = game(&rules, 1, vec![object]);
    play(&mut state, &rules, RED, move_hero(0, &[anchor])).unwrap();
    // Blue's side without a hero, played by blue outside its turn; with magic.
    let battle = state.battle(RED).unwrap();
    assert_eq!(battle.owners(), [Some(RED), Some(BLUE)]);
    assert_eq!(battle.heroes(), [Some(RED_HERO), None]);
    assert!(!anti_magic(&state, &rules, BLUE));
    fight(&mut state, &rules);
    // Lost: red's hero off the map, the garrison blue's with its survivors.
    let adventure = state.adventure();
    assert_eq!(adventure.hero(RED_HERO), None);
    assert_eq!(adventure.owner(0), Some(BLUE));
    let left = adventure.garrison_army(0);
    assert!(matches!(left, [ArmyStack { slot: 0, count, .. }] if (1..=40).contains(count)));
}

// covers: ADV-GAR-005
#[test]
fn a_hero_standing_in_an_anti_magic_garrison_is_attacked_without_magic() {
    let rules = rules();
    let entrance = Pos::new(2, 1, 0);
    let object = garrison(&rules, "t:anti_garrison", entrance, Some(BLUE), vec![]);
    let mut state = game(&rules, 9, vec![object]);
    // Blue's hero steps into its own garrison (crossed, visited where its move ends: its exchange,
    // closed); then red attacks it there.
    play(&mut state, &rules, RED, GameCommand::EndTurn).unwrap();
    play(&mut state, &rules, BLUE, move_hero(1, &[entrance])).unwrap();
    assert_eq!(
        state.adventure().hero(HeroId::new(1)).unwrap().pos,
        entrance
    );
    assert!(state.exchange(BLUE).is_some());
    play(&mut state, &rules, BLUE, answered(Answer::Close)).unwrap();
    play(&mut state, &rules, BLUE, GameCommand::EndTurn).unwrap();
    let path = [Pos::new(1, 0, 0), entrance];
    play(&mut state, &rules, RED, move_hero(0, &path)).unwrap();
    assert_eq!(state.battle(RED).unwrap().heroes()[1], Some(HeroId::new(1)));
    assert!(anti_magic(&state, &rules, RED));
}

/// A slot of `army`.
const fn at(army: ArmyRef, slot: u8) -> ArmySlot {
    ArmySlot { army, slot }
}

/// The garrison at object index 0.
const GARRISON: ArmyRef = ArmyRef::MapGarrison(0);

/// Red's hero's army.
const RED_ARMY: ArmyRef = ArmyRef::Hero(RED_HERO);

/// Red's army command.
fn army(command: ArmyCommand) -> GameCommand {
    GameCommand::Manage(Manage::Army(command))
}

/// An army command's refusal as the game gives it.
const fn refused(error: ArmyCommandError) -> GameCommandError {
    GameCommandError::Manage(ManageError::Army(error))
}

// covers: GAM-DEC-009, GAM-MAN-001, ADV-GAR-003
#[test]
fn a_team_s_garrison_opens_the_exchange_whose_commands_name_both_armies() {
    let rules = rules();
    let anchor = Pos::new(1, 0, 0);
    let ty = rules.adventure().object_type_by_key("t:garrison").unwrap();
    // The test part's two army slots: the garrison's imps in slot 0, the hero's in slot 1.
    let held = vec![imp(0, 20)];
    let object = garrison(&rules, "t:garrison", anchor, Some(RED), held.clone());
    let mut state = game(&rules, 9, vec![object]);
    // Outside the exchange the garrison is named with no hero holding it.
    let into = army(ArmyCommand::Move {
        from: at(RED_ARMY, 1),
        to: at(GARRISON, 1),
    });
    assert_eq!(
        play(&mut state, &rules, RED, into.clone()),
        Err(GameCommandError::Manage(ManageError::NotTogether))
    );
    let alone = army(ArmyCommand::Dismiss {
        at: at(GARRISON, 0),
    });
    assert_eq!(
        play(&mut state, &rules, RED, alone),
        Err(GameCommandError::Manage(ManageError::NotTogether))
    );
    // The move ends on it: the exchange, its army's stacks told after it, the owner's alone.
    let events = play(&mut state, &rules, RED, move_hero(0, &[anchor])).unwrap();
    let opened = GameEvent::ExchangeOpened {
        player: RED,
        hero: RED_HERO,
        garrison: 0,
        object_type: ty,
        anchor,
        removable: true,
    };
    let changed = GameEvent::StackChanged {
        player: RED,
        at: at(GARRISON, 0),
        stack: Some(imp(0, 20)),
    };
    assert_eq!(after_steps(events), [opened, changed]);
    assert_eq!(project(&opened, Viewer::Player(BLUE)), None);
    let view = observe(&state, &rules, Viewer::Player(RED)).decision;
    assert_eq!(
        view,
        Some(DecisionView::Exchange {
            hero: RED_HERO,
            garrison: 0,
            object: (ty, anchor),
            removable: true,
            army: held,
        })
    );
    // Its answer is `Close` (its default too); the turn cannot end meanwhile.
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(RED)),
        [GameCommand::Answer(Answer::Close)]
    );
    assert_eq!(default_answer(&state, &rules, RED), Some(Answer::Close));
    assert_eq!(
        play(&mut state, &rules, RED, GameCommand::EndTurn),
        Err(GameCommandError::DecisionOpen)
    );
    // Meanwhile red's commands name the hero's army and the garrison's, each slot told.
    let listed = legal_management(&state, &rules, Actor::Player(RED), RED_HERO);
    // The hero's last stack stays; the garrison's moves into the hero's empty slot.
    let to_garrison = Manage::Army(ArmyCommand::Move {
        from: at(RED_ARMY, 1),
        to: at(GARRISON, 1),
    });
    let from_garrison = Manage::Army(ArmyCommand::Move {
        from: at(GARRISON, 0),
        to: at(RED_ARMY, 0),
    });
    assert!(listed.contains(&from_garrison) && !listed.contains(&to_garrison));
    let events = play(&mut state, &rules, RED, GameCommand::Manage(from_garrison)).unwrap();
    assert_eq!(
        events,
        [
            GameEvent::StackChanged {
                player: RED,
                at: at(GARRISON, 0),
                stack: None,
            },
            GameEvent::StackChanged {
                player: RED,
                at: at(RED_ARMY, 0),
                stack: Some(imp(0, 20)),
            },
        ]
    );
    assert_eq!(state.adventure().garrison_army(0), []);
    let range = split_range(
        &state,
        &rules,
        Actor::Player(RED),
        (at(RED_ARMY, 0), at(GARRISON, 1)),
    );
    assert_eq!(range, Some(1..=19));
    play(&mut state, &rules, RED, into.clone()).unwrap();
    // Another garrison, the group or another hero are not held by it.
    let other = army(ArmyCommand::Move {
        from: at(RED_ARMY, 0),
        to: at(ArmyRef::MapGarrison(1), 0),
    });
    assert_eq!(
        play(&mut state, &rules, RED, other),
        Err(GameCommandError::DecisionOpen)
    );
    let group = army(ArmyCommand::Move {
        from: at(RED_ARMY, 0),
        to: at(ArmyRef::Group, 0),
    });
    assert_eq!(
        play(&mut state, &rules, RED, group),
        Err(refused(ArmyCommandError::NoGroup))
    );
    let blue = army(ArmyCommand::Dismiss {
        at: at(ArmyRef::Hero(HeroId::new(1)), 0),
    });
    assert_eq!(
        play(&mut state, &rules, RED, blue),
        Err(GameCommandError::DecisionOpen)
    );
    // Its close only closes: the garrison keeps what the commands left there.
    let events = play(&mut state, &rules, RED, answered(Answer::Close)).unwrap();
    assert_eq!(events, [GameEvent::DecisionClosed { player: RED }]);
    assert_eq!(state.adventure().garrison_army(0), [imp(1, 9)]);
    assert_eq!(
        play(&mut state, &rules, RED, into.clone()),
        Err(GameCommandError::Manage(ManageError::NotTogether))
    );
    // The hero standing in it revisits it: the exchange again.
    let events = play(&mut state, &rules, RED, revisit(0)).unwrap();
    assert!(matches!(
        after_steps(events)[..],
        [
            GameEvent::ExchangeOpened { .. },
            GameEvent::StackChanged { .. }
        ]
    ));
}

// covers: GAM-DEC-009, ADV-GAR-003
#[test]
fn a_non_removable_garrison_s_exchange_only_takes_stacks_into_its_empty_slots() {
    let rules = rules();
    let anchor = Pos::new(1, 0, 0);
    let held = vec![imp(1, 20)];
    let object = fixed(&rules, ("t:garrison", anchor), Some(RED), held, false);
    let mut setup = setup();
    setup.heroes[0].army = HeroArmy::Custom(vec![imp(0, 9), imp(1, 4)]);
    setup.map.objects = vec![object];
    let mut state = GameState::new(&rules, &setup, 7).unwrap();
    play(&mut state, &rules, RED, move_hero(0, &[anchor])).unwrap();
    let Some(DecisionView::Exchange { removable, .. }) =
        observe(&state, &rules, Viewer::Player(RED)).decision
    else {
        panic!("no exchange");
    };
    assert!(!removable);
    let back = army(ArmyCommand::Move {
        from: at(GARRISON, 1),
        to: at(RED_ARMY, 0),
    });
    assert_eq!(
        play(&mut state, &rules, RED, back),
        Err(refused(ArmyCommandError::NotOwn))
    );
    let merge = army(ArmyCommand::Move {
        from: at(RED_ARMY, 1),
        to: at(GARRISON, 1),
    });
    assert_eq!(
        play(&mut state, &rules, RED, merge),
        Err(refused(ArmyCommandError::OnlyEmpty))
    );
    let left = army(ArmyCommand::Move {
        from: at(RED_ARMY, 1),
        to: at(GARRISON, 0),
    });
    play(&mut state, &rules, RED, left).unwrap();
    assert_eq!(state.adventure().garrison_army(0), [imp(0, 4), imp(1, 20)]);
    let listed = legal_management(&state, &rules, Actor::Player(RED), RED_HERO);
    assert!(listed.iter().all(|m| !matches!(
        m,
        Manage::Army(ArmyCommand::Move { from, .. } | ArmyCommand::Split { from, .. })
            if from.army == GARRISON
    ) && !matches!(m, Manage::Army(ArmyCommand::Dismiss { at }) if at.army == GARRISON)));
}

// covers: GAM-DEC-009, ADV-GAR-002, ADV-GAR-003
#[test]
fn an_empty_garrison_s_capture_opens_the_exchange_after_its_flag() {
    let rules = rules();
    let anchor = Pos::new(1, 0, 0);
    let object = garrison(&rules, "t:garrison", anchor, Some(BLUE), vec![]);
    let mut state = game(&rules, 9, vec![object]);
    let before = state.state_hash();
    let events = after_steps(play(&mut state, &rules, RED, move_hero(0, &[anchor])).unwrap());
    assert!(matches!(
        events[..],
        [
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::ObjectFlagged { owner: RED, .. },
                ..
            }),
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::TilesRevealed { .. },
                ..
            }),
            GameEvent::ExchangeOpened {
                player: RED,
                garrison: 0,
                removable: true,
                ..
            },
        ]
    ));
    assert_eq!(state.adventure().owner(0), Some(RED));
    // The open exchange is in the state hash (game state hash v29).
    let mut closed = state.clone();
    play(&mut closed, &rules, RED, answered(Answer::Close)).unwrap();
    assert_ne!(closed.state_hash(), state.state_hash());
    assert_ne!(closed.state_hash(), before);
}
