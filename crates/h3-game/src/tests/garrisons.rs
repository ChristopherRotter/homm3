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
    // Then red's changed view of the garrison (E08 T114b, ADV-GAR-006: empty now); blue's, at
    // level 0, did not change.
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
            GameEvent::GarrisonChanged {
                player: RED,
                object_type: ty,
                anchor,
                owner: Some(RED),
                level: OWNER_LEVEL,
                slots: true,
                removable: Some(true),
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
            // E08 T114b (ADV-GAR-006): every viewer's view, its owner changed.
            GameEvent::GarrisonChanged {
                player: RED,
                owner: Some(RED),
                level: OWNER_LEVEL,
                slots: true,
                removable: Some(true),
                ..
            },
            GameEvent::GarrisonChanged {
                player: BLUE,
                owner: Some(RED),
                level: 0,
                slots: false,
                removable: None,
                ..
            },
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

/// What `viewer` is told of its view `view` of a map garrison (E08 T114b, ADV-GAR-006): its
/// `GarrisonChanged`, then a `GarrisonStackSeen` for each stack its level shows, in slot order.
pub(crate) fn garrison_telling(viewer: PlayerId, view: &MapGarrisonView) -> Vec<GameEvent> {
    let (object_type, anchor) = (view.object_type, view.anchor);
    let army = view.garrison.army.as_deref();
    let mut told = vec![GameEvent::GarrisonChanged {
        player: viewer,
        object_type,
        anchor,
        owner: view.owner,
        level: view.garrison.level,
        slots: army.is_some(),
        removable: view.removable,
    }];
    told.extend(view.garrison.stacks.iter().enumerate().map(|(i, &stack)| {
        GameEvent::GarrisonStackSeen {
            player: viewer,
            object_type,
            anchor,
            stack,
            slot: army.map(|army| army[i].slot),
        }
    }));
    told
}

/// `viewer`'s views of the map garrisons.
fn garrisons_seen(
    state: &GameState,
    rules: &GameRuleset,
    viewer: PlayerId,
) -> Vec<MapGarrisonView> {
    observe(state, rules, Viewer::Player(viewer)).garrisons
}

/// The three garrisons of [`views`]: red's removable one on (1, 0) with 20 imps in slot 0,
/// blue's anti-magic non-removable one on (3, 0) with 5 imps in slot 1, a neutral empty one on
/// (0, 2).
fn three(rules: &GameRuleset) -> Vec<ObjectSetup> {
    vec![
        garrison(
            rules,
            "t:garrison",
            Pos::new(1, 0, 0),
            Some(RED),
            vec![imp(0, 20)],
        ),
        fixed(
            rules,
            ("t:anti_garrison", Pos::new(3, 0, 0)),
            Some(BLUE),
            vec![imp(1, 5)],
            false,
        ),
        garrison(rules, "t:garrison", Pos::new(0, 2, 0), None, vec![]),
    ]
}

// covers: ADV-GAR-006
#[test]
fn every_viewer_sees_a_map_garrison_at_its_level_and_its_flag_by_its_team() {
    let rules = rules();
    let state = game(&rules, 9, three(&rules));
    let (plain, anti) = (
        rules.adventure().object_type_by_key("t:garrison").unwrap(),
        rules
            .adventure()
            .object_type_by_key("t:anti_garrison")
            .unwrap(),
    );
    let word = |count| {
        rules
            .adventure()
            .params()
            .monsters()
            .map(|w| w.count_word(count))
    };
    let own = |slot, count| GarrisonView {
        level: OWNER_LEVEL,
        stacks: vec![GarrisonStackView {
            creature: imp(0, 1).creature,
            count_word: word(count),
            count: Some(count),
        }],
        army: Some(vec![imp(slot, count)]),
    };
    // Another team's at the viewer's taverns' level (none here: level 0, not even whether it is
    // empty), its flag hidden; a neutral one nobody's team.
    let unseen = GarrisonView {
        level: 0,
        stacks: vec![],
        army: None,
    };
    let view = |object_type, anchor, owner, garrison, removable| MapGarrisonView {
        object_type,
        anchor,
        owner,
        garrison,
        removable,
    };
    let (red_one, blue_one, neutral) = (Pos::new(1, 0, 0), Pos::new(3, 0, 0), Pos::new(0, 2, 0));
    assert_eq!(
        garrisons_seen(&state, &rules, RED),
        [
            view(plain, red_one, Some(RED), own(0, 20), Some(true)),
            view(anti, blue_one, Some(BLUE), unseen.clone(), None),
            view(plain, neutral, None, unseen.clone(), None),
        ]
    );
    assert_eq!(
        garrisons_seen(&state, &rules, BLUE),
        [
            view(plain, red_one, Some(RED), unseen.clone(), None),
            view(anti, blue_one, Some(BLUE), own(1, 5), Some(false)),
            view(plain, neutral, None, unseen, None),
        ]
    );
    // Each view's events go to its viewer alone.
    for event in garrison_telling(RED, &garrisons_seen(&state, &rules, RED)[0]) {
        assert_eq!(project(&event, Viewer::Player(RED)), Some(event));
        assert_eq!(project(&event, Viewer::Player(BLUE)), None);
    }
}

// covers: ADV-GAR-006
#[test]
fn a_garrison_explored_since_a_grid_is_revealed_with_its_view() {
    let rules = rules();
    let state = game(&rules, 9, three(&rules));
    let grid = state.adventure().explored(RED).unwrap().clone();
    let (width, height, levels) = (grid.width(), grid.height(), grid.levels());
    let seen = garrisons_seen(&state, &rules, RED);
    let revealed = |since: &BitGrid| reveal(&state, &rules, Viewer::Player(RED), since).garrisons;
    // From nothing explored: every garrison, as `observe` shows it; through the grid now: none; a
    // grid of another size counts as none explored.
    assert_eq!(revealed(&BitGrid::new(width, height, levels)), seen);
    assert_eq!(revealed(&grid), []);
    assert_eq!(revealed(&BitGrid::new(width + 1, height, levels)), seen);
    // A grid with one tile of the neutral garrison explored leaves it out.
    let mut since = BitGrid::new(width, height, levels);
    since.set(Pos::new(0, 2, 0), true);
    assert_eq!(revealed(&since), seen[..2]);
}

// covers: ADV-GAR-006, ADV-GAR-002
#[test]
fn a_garrison_battle_s_end_and_its_capture_tell_each_viewer_its_changed_view() {
    let rules = rules();
    let anchor = Pos::new(1, 0, 0);
    let object = garrison(&rules, "t:garrison", anchor, Some(BLUE), vec![imp(0, 1)]);
    let mut state = game(&rules, 300, vec![object]);
    play(&mut state, &rules, RED, move_hero(0, &[anchor])).unwrap();
    let events = win(&mut state, &rules);
    // At the battle's end blue's view by slot, emptied (red's, at level 0, did not change); after
    // the winner's levels the flag, every viewer's view, then the exchange.
    let ended = events
        .iter()
        .position(|e| matches!(e, GameEvent::BattleEnded { .. }))
        .unwrap();
    let told: Vec<GameEvent> = events[ended..]
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::GarrisonChanged { .. } | GameEvent::GarrisonStackSeen { .. }
            )
        })
        .copied()
        .collect();
    let emptied = GameEvent::GarrisonChanged {
        player: BLUE,
        object_type: rules.adventure().object_type_by_key("t:garrison").unwrap(),
        anchor,
        owner: Some(BLUE),
        level: OWNER_LEVEL,
        slots: true,
        removable: Some(true),
    };
    let mut expected = vec![emptied];
    for viewer in [RED, BLUE] {
        expected.extend(garrison_telling(
            viewer,
            &garrisons_seen(&state, &rules, viewer)[0],
        ));
    }
    assert_eq!(told, expected);
    assert_eq!(garrisons_seen(&state, &rules, BLUE)[0].owner, Some(RED));
    let opened = events
        .iter()
        .position(|e| matches!(e, GameEvent::ExchangeOpened { .. }));
    let last = events
        .iter()
        .rposition(|e| matches!(e, GameEvent::GarrisonChanged { .. }));
    assert!(last < opened);
}
