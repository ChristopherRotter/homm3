//! Tests of the mines' visits through the game (E08 T84a; increment 9's design note §7, §13, §14):
//! an own mine's visit opens the Leave Guards dialog, whose close keeps its group on the mine; a
//! neutral abandoned mine's guards are asked, then fought as a monster the system plays, a win
//! flagging it after the winner's levels; an owned mine's guards are their owner's side, played by
//! that player outside its turn.

use h3_adventure::{AdventureEvent, AdventureEventKind, Holds, ObjectSetup, TileMask, Viewers};

use super::decisions::{ACCEPT, CLOSE, GOLD, after_steps, move_hero, play};
use super::manage::{army, group, imp, slot};
use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const RED_HERO: HeroId = HeroId::new(0);
const BLUE_HERO: HeroId = HeroId::new(1);
const GOG: CreatureTypeId = CreatureTypeId::from_raw(1);

/// A mine of type `k` at `anchor`, `owner`'s or neutral, entered there.
fn mine(rules: &GameRuleset, k: &str, anchor: Pos, owner: Option<PlayerId>) -> ObjectSetup {
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
        holds: if k == "t:abandoned" {
            Holds::Resources(vec![GOLD])
        } else {
            Holds::Nothing
        },
        owner,
    }
}

/// [`setup`] with `objects`, red's hero at (0, 0) with `imps` imps in slot 1, blue's at (2, 2)
/// with 2 gogs in slot 0 and 5 imps in slot 1.
fn game(rules: &GameRuleset, imps: u32, objects: Vec<ObjectSetup>) -> GameState {
    let mut setup = setup();
    setup.heroes[0].army = HeroArmy::Custom(vec![imp(1, imps)]);
    setup.heroes[1].army = HeroArmy::Custom(vec![
        ArmyStack {
            slot: 0,
            creature: GOG,
            count: 2,
        },
        imp(1, 5),
    ]);
    setup.map.objects = objects;
    GameState::new(rules, &setup, 7).unwrap()
}

/// Answers every decision `player` has open with its default answer; the events.
fn answer_all(state: &mut GameState, rules: &GameRuleset, player: PlayerId) -> Vec<GameEvent> {
    let mut events = Vec::new();
    while let Some(answer) = default_answer(state, rules, player) {
        events.extend(play(state, rules, player, GameCommand::Answer(answer)).unwrap());
    }
    events
}

/// Plays red's battle to its end: the owner of the side to act (the system for an unowned side),
/// each the last legal command that stays in the battle; all the events.
fn fight(state: &mut GameState, rules: &GameRuleset) -> Vec<GameEvent> {
    let mut all = Vec::new();
    for _ in 0..4000 {
        let Some(battle) = state.battle(RED) else {
            return all;
        };
        let actor = battle.owner_to_act().map_or(Actor::System, Actor::Player);
        let listed = legal_commands(state, rules, actor);
        let fights = |c: &&GameCommand| !matches!(c, GameCommand::Battle(c) if c.leaves_battle());
        let command = listed.iter().rfind(fights).unwrap().clone();
        apply(state, rules, actor, command, &mut all).unwrap();
    }
    panic!("the battle did not end");
}

/// The adventure's events among `events`.
fn adventure(events: &[GameEvent]) -> Vec<AdventureEvent> {
    events
        .iter()
        .filter_map(|e| match e {
            GameEvent::Adventure(event) => Some(*event),
            _ => None,
        })
        .collect()
}

// covers: GAM-DEC-004, ADV-MIN-002
#[test]
fn an_own_mine_opens_leave_guards_whose_close_keeps_the_group_on_it() {
    let rules = rules();
    let anchor = Pos::new(1, 1, 0);
    let ty = rules.adventure().object_type_by_key("t:mine").unwrap();
    let mut state = game(&rules, 9, vec![mine(&rules, "t:mine", anchor, Some(RED))]);
    let events = play(&mut state, &rules, RED, move_hero(0, &[anchor])).unwrap();
    let opened = GameEvent::LeaveGuardsOpened {
        player: RED,
        hero: RED_HERO,
        object_type: ty,
        anchor,
    };
    // No guards yet: the dialog opens with an empty group.
    assert_eq!(after_steps(events), [opened]);
    assert_eq!(project(&opened, Viewer::Player(RED)), Some(opened));
    assert_eq!(project(&opened, Viewer::Player(BLUE)), None);
    assert_eq!(state.army_dialog(RED), Some((RED_HERO, &[][..])));
    assert_eq!(state.leave_guards(RED), Some((ty, anchor)));
    assert_eq!(state.leave_guards(BLUE), None);
    assert_eq!(
        observe(&state, &rules, Viewer::Player(RED)).decision,
        Some(DecisionView::ArmyDialog {
            hero: RED_HERO,
            group: vec![],
            mine: Some((ty, anchor)),
        })
    );
    // Four imps left there; the hash sees the dialog's holder and group.
    let open = state.state_hash();
    let split = ArmyCommand::Split {
        from: slot(RED_HERO, 1),
        to: group(0),
        count: 4,
    };
    play(&mut state, &rules, RED, army(split)).unwrap();
    assert_eq!(state.army_dialog(RED), Some((RED_HERO, &[imp(0, 4)][..])));
    assert_ne!(state.state_hash(), open);
    // The close keeps them on the mine, nothing lost; every player that sees it told the hint.
    let events = play(&mut state, &rules, RED, CLOSE).unwrap();
    assert_eq!(
        events,
        [
            GameEvent::DecisionClosed { player: RED },
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::GuardsChanged {
                    object_type: ty,
                    anchor,
                    creature: Some(IMP),
                    count_word: Some(0),
                },
                seen_by: Viewers::NONE.with(RED).with(BLUE),
            }),
        ]
    );
    let adventure = state.adventure();
    assert_eq!(adventure.mine_guards(rules.adventure(), 0), [imp(0, 4)]);
    assert_eq!(adventure.hero(RED_HERO).unwrap().army, [imp(1, 5)]);
    assert!(adventure.check_owners(rules.adventure()));
    assert!(!state.has_decision(RED));
}

// covers: ADV-MIN-002, GAM-BAT-001, GAM-DEC-007
#[test]
fn a_neutral_mine_s_guards_fight_as_a_monster_and_a_win_flags_it_after_the_levels() {
    let rules = rules();
    let anchor = Pos::new(1, 1, 0);
    let ty = rules.adventure().object_type_by_key("t:abandoned").unwrap();
    let mut state = game(&rules, 300, vec![mine(&rules, "t:abandoned", anchor, None)]);
    let guards = state.adventure().mine_guards(rules.adventure(), 0);
    assert_eq!(guards.len(), 1);
    assert_eq!((guards[0].slot, guards[0].creature), (0, GOG));
    assert!((3..=5).contains(&guards[0].count));
    let events = play(&mut state, &rules, RED, move_hero(0, &[anchor])).unwrap();
    assert_eq!(
        after_steps(events),
        [GameEvent::OfferOpened {
            player: RED,
            hero: RED_HERO,
            object_type: ty,
            anchor,
        }]
    );
    // Accepted: the battle the system plays for the guards, red's alone.
    let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::BattleStarted { seen_by, .. } if *seen_by == Viewers::NONE.with(RED)
    )));
    let battle = state.battle(RED).unwrap();
    assert_eq!(battle.owners(), [Some(RED), None]);
    assert_eq!(battle.heroes(), [Some(RED_HERO), None]);
    let mut events = fight(&mut state, &rules);
    events.extend(answer_all(&mut state, &rules, RED));
    // Won: the guards gone, then (after any level) the flag.
    let told = adventure(&events);
    let at = |pred: fn(&AdventureEventKind) -> bool| told.iter().position(|e| pred(&e.kind));
    let gone = at(|k| {
        matches!(
            k,
            AdventureEventKind::GuardsChanged {
                count_word: None,
                ..
            }
        )
    });
    let flagged = at(|k| matches!(k, AdventureEventKind::ObjectFlagged { owner: RED, .. }));
    assert!(gone.unwrap() < flagged.unwrap(), "{told:?}");
    let ended = events
        .iter()
        .position(|e| matches!(e, GameEvent::BattleEnded { .. }))
        .unwrap();
    let flag_event = events
        .iter()
        .position(|e| {
            matches!(
                e,
                GameEvent::Adventure(AdventureEvent {
                    kind: AdventureEventKind::ObjectFlagged { .. },
                    ..
                })
            )
        })
        .unwrap();
    assert!(ended < flag_event);
    let adventure = state.adventure();
    assert_eq!(adventure.owner(0), Some(RED));
    assert!(adventure.mine_guards(rules.adventure(), 0).is_empty());
    assert_eq!(adventure.hero(RED_HERO).unwrap().pos, anchor);
}

// covers: ADV-MIN-002, GAM-BAT-001, GAM-BAT-010
#[test]
fn an_owned_mine_s_guards_are_their_owner_s_side_played_outside_its_turn() {
    let rules = rules();
    let anchor = Pos::new(3, 1, 0);
    let ty = rules.adventure().object_type_by_key("t:mine").unwrap();
    let mut state = game(
        &rules,
        300,
        vec![mine(&rules, "t:mine", anchor, Some(BLUE))],
    );
    play(&mut state, &rules, RED, GameCommand::EndTurn).unwrap();
    // Blue leaves its five imps there and steps back off the entrance.
    play(&mut state, &rules, BLUE, move_hero(1, &[anchor])).unwrap();
    let to_group = ArmyCommand::Move {
        from: slot(BLUE_HERO, 1),
        to: group(0),
    };
    play(&mut state, &rules, BLUE, army(to_group)).unwrap();
    play(&mut state, &rules, BLUE, CLOSE).unwrap();
    play(&mut state, &rules, BLUE, move_hero(1, &[Pos::new(2, 2, 0)])).unwrap();
    play(&mut state, &rules, BLUE, GameCommand::EndTurn).unwrap();
    assert_eq!(
        state.adventure().mine_guards(rules.adventure(), 0),
        [imp(0, 5)]
    );
    // Red's hero is asked, then fights blue's side, which blue owns and plays.
    let path = [Pos::new(1, 1, 0), Pos::new(2, 1, 0), anchor];
    let events = play(&mut state, &rules, RED, move_hero(0, &path)).unwrap();
    assert_eq!(
        after_steps(events),
        [GameEvent::OfferOpened {
            player: RED,
            hero: RED_HERO,
            object_type: ty,
            anchor,
        }]
    );
    let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
    let both = Viewers::NONE.with(RED).with(BLUE);
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::BattleStarted { seen_by, .. } if *seen_by == both
    )));
    let battle = state.battle(RED).unwrap();
    assert_eq!(battle.owners(), [Some(RED), Some(BLUE)]);
    assert_eq!(battle.heroes(), [Some(RED_HERO), None]);
    assert_eq!(state.battle(BLUE).map(OpenBattle::tile), Some(anchor));
    assert!(state.unowned_battle().is_none());
    let mut events = fight(&mut state, &rules);
    events.extend(answer_all(&mut state, &rules, RED));
    // Red's 300 imps win: blue's guards gone, the mine red's, both told.
    let flagged = adventure(&events)
        .into_iter()
        .find(|e| matches!(e.kind, AdventureEventKind::ObjectFlagged { .. }))
        .unwrap();
    assert_eq!(
        flagged,
        AdventureEvent {
            kind: AdventureEventKind::ObjectFlagged {
                object_type: ty,
                anchor,
                owner: RED,
            },
            seen_by: both,
        }
    );
    let adventure = state.adventure();
    assert_eq!(adventure.owner(0), Some(RED));
    assert!(adventure.mine_guards(rules.adventure(), 0).is_empty());
}
