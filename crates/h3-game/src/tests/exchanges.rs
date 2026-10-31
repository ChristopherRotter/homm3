//! Tests of the meetings and the army dialogs (GAM-DEC-003, GAM-DEC-004, GAM-MAN-001, -002;
//! E08 T51b): a step toward an own hero opens a meeting with Scholar first, during which both
//! heroes' armies and artifacts are managed, also between them; an accepted join without room
//! opens the army dialog with the group as a small army; `Decline` closes either, the group's rest
//! lost; the listing per hero and the split's query with either open; the hash and the
//! observation.

use h3_adventure::{
    AdventureEvent, AdventureEventKind, MonsterSetup, ObjectSetup, ObjectTypeId, TileMask, Viewers,
};
use h3_heroes::{ArtifactInstance, ArtifactPlace, Position, wearing};

use super::decisions::{ACCEPT, CLOSE, DECLINE, GOLD, after_steps, move_hero};
use super::manage::{
    BLUE, BLUE_HERO, RED, RED_HERO, SECOND, accepts, army, art, artifact_move, group, imp,
    manage_error, manage_rules, manage_setup, refused, run, slot,
};
use super::*;

const GOG: CreatureTypeId = CreatureTypeId::from_raw(1);
const RED_ACTOR: Actor = Actor::Player(RED);

/// [`manage_setup`]'s game: red's hero at (0, 0) with 9 imps in slot 1, its second hero beside
/// it at (1, 0) with 3 imps in slot 0, both wearing the ring and carrying the band and the charm.
fn game(rules: &GameRuleset) -> GameState {
    GameState::new(rules, &manage_setup(rules), 1).unwrap()
}

/// Red's hero's step onto its second hero's tile: the meeting.
fn meet(game: &mut GameState, rules: &GameRuleset) -> Vec<GameEvent> {
    run(game, rules, RED_ACTOR, move_hero(0, &[Pos::new(1, 0, 0)])).unwrap()
}

// covers: GAM-MET-001, GAM-DEC-003, GAM-DEC-001
#[test]
fn a_step_toward_an_own_hero_opens_a_meeting_its_decline_closes() {
    let rules = manage_rules();
    let mut game = game(&rules);
    let heroes = [RED_HERO, SECOND];
    let events = meet(&mut game, &rules);
    let opened = GameEvent::MeetingOpened {
        player: RED,
        heroes,
    };
    assert_eq!(after_steps(events.clone()), [opened]);
    assert_eq!(project(&opened, Viewer::Player(RED)), Some(opened));
    assert_eq!(project(&opened, Viewer::Player(BLUE)), None);
    assert_eq!(game.meeting(RED), Some(heroes));
    assert_eq!((game.meeting(BLUE), game.army_dialog(RED)), (None, None));
    assert!(game.has_decision(RED));
    assert_eq!(
        game.adventure().hero(RED_HERO).unwrap().pos,
        Pos::new(0, 0, 0)
    );
    // Its owner sees it; the other player does not.
    let seen = |player| observe(&game, &rules, Viewer::Player(player)).decision;
    assert_eq!(seen(RED), Some(DecisionView::Meeting { heroes }));
    assert_eq!(seen(BLUE), None);
    let view = seen(RED).unwrap();
    assert_eq!((view.offer(), view.level_up()), (None, None));
    // `Close` alone is listed and the default; no move, no end of the turn.
    assert_eq!(legal_commands(&game, &rules, RED_ACTOR), [CLOSE]);
    assert_eq!(default_answer(&game, &rules, RED), Some(Answer::Close));
    assert!(legal_moves(&game, &rules, RED_ACTOR, RED_HERO).is_none());
    for command in [GameCommand::EndTurn, move_hero(0, &[Pos::new(0, 1, 0)])] {
        assert_eq!(
            refused(&mut game, &rules, RED_ACTOR, command),
            GameCommandError::DecisionOpen
        );
    }
    for answer in [
        ACCEPT,
        DECLINE,
        GameCommand::Answer(Answer::LevelUp(SkillOffer::First)),
    ] {
        assert_eq!(
            refused(&mut game, &rules, RED_ACTOR, answer),
            GameCommandError::WrongAnswer
        );
    }
    // The hash sees it.
    let mut fresh = self::game(&rules);
    assert_ne!(game.state_hash(), fresh.state_hash());
    let closed = run(&mut game, &rules, RED_ACTOR, CLOSE).unwrap();
    assert_eq!(closed, [GameEvent::DecisionClosed { player: RED }]);
    assert_eq!(project(&closed[0], Viewer::Player(BLUE)), None);
    assert_eq!(game.meeting(RED), None);
    assert_eq!(
        legal_commands(&game, &rules, RED_ACTOR),
        [GameCommand::EndTurn]
    );
    // Another step meets again; the meeting cost the step's points as T50's visit.
    let points = game.adventure().hero(RED_HERO).unwrap().movement;
    meet(&mut fresh, &rules);
    assert_eq!(fresh.adventure().hero(RED_HERO).unwrap().movement, points);
    meet(&mut game, &rules);
    assert_eq!(game.meeting(RED), Some(heroes));
}

/// The rules of [`manage_rules`] with Scholar (`t:scholar`: a meeting teaches spells up to level
/// 2, 3, 4).
fn scholar_rules() -> GameRuleset {
    let mut pack = core_pack(false);
    pack.skills.push(SecondarySkillDef {
        key: key("t:scholar"),
        name: "Scholar".into(),
        levels: [2, 3, 4].map(|level| vec![HeroEffect::MeetingSpells { level }]),
    });
    let core = Arc::new(Ruleset::compile(&[pack], "t:faithful").unwrap());
    let part = part_with(&core, vec![88, 25], 7);
    let battle = battle_part(&core, 7);
    GameRuleset::new(core, Arc::new(part), Arc::new(battle)).unwrap()
}

// covers: HER-SPL-007, GAM-DEC-003
#[test]
fn scholar_teaches_both_heroes_before_the_meeting_opens_the_visitors_first() {
    let rules = scholar_rules();
    let core = rules.core();
    let id = |k: &str| core.skill_by_key(k).unwrap();
    let spell = |k: &str| core.spell_by_key(k).unwrap();
    let book: ArtifactInstance = core.artifact_by_key("t:book").unwrap().into();
    let mut setup = setup();
    // Red's hero: Advanced Scholar (level 3) and Basic Sense (it learns level 3), Gust; the
    // second hero: Gate (level 3) and the base learning (level 2). Both wear the book.
    setup.heroes[0].details = HeroDetailsSetup {
        skills: Some(vec![(id("t:scholar"), 2), (id("t:sense"), 1)]),
        spells: Some(vec![spell("t:gust")]),
        artifacts: Some(HeroArtifactsSetup {
            worn: wearing(core, &[book]),
            backpack: vec![],
        }),
        ..HeroDetailsSetup::default()
    };
    setup.heroes.push(GameHeroSetup {
        pos: Pos::new(1, 0, 0),
        details: HeroDetailsSetup {
            spells: Some(vec![spell("t:gate")]),
            ..setup.heroes[0].details.clone()
        },
        ..setup.heroes[0].clone()
    });
    setup.heroes[2].details.skills = Some(vec![]);
    let mut game = GameState::new(&rules, &setup, 1).unwrap();
    let learned = |hero, k| GameEvent::HeroChanged {
        player: RED,
        hero,
        change: HeroChange::LearnSpell(spell(k)),
    };
    let events = after_steps(meet(&mut game, &rules));
    assert_eq!(
        events,
        [
            learned(RED_HERO, "t:gate"),
            learned(SECOND, "t:gust"),
            GameEvent::MeetingOpened {
                player: RED,
                heroes: [RED_HERO, SECOND]
            }
        ]
    );
    let spells = |hero| game.hero(hero).unwrap().spells.clone();
    assert_eq!(spells(RED_HERO), [spell("t:gust"), spell("t:gate")]);
    assert_eq!(spells(SECOND), [spell("t:gate"), spell("t:gust")]);
    // A second meeting teaches nothing more.
    run(&mut game, &rules, RED_ACTOR, CLOSE).unwrap();
    assert_eq!(after_steps(meet(&mut game, &rules)).len(), 1);
}

// covers: GAM-MAN-001, GAM-DEC-003, ADV-ARM-002
#[test]
fn in_a_meeting_stacks_and_artifacts_move_between_its_two_heroes() {
    let rules = manage_rules();
    let mut game = game(&rules);
    meet(&mut game, &rules);
    let told = |at, stack| GameEvent::StackChanged {
        player: RED,
        at,
        stack,
    };
    // Red's hero's imps merge into the second hero's: its last stack may not, so 4 split off first.
    let split = ArmyCommand::Split {
        from: slot(RED_HERO, 1),
        to: slot(SECOND, 2),
        count: 4,
    };
    let events = run(&mut game, &rules, RED_ACTOR, army(split)).unwrap();
    assert_eq!(
        events,
        [
            told(slot(RED_HERO, 1), Some(imp(1, 5))),
            told(slot(SECOND, 2), Some(imp(2, 4)))
        ]
    );
    let last = ArmyCommand::Move {
        from: slot(RED_HERO, 1),
        to: slot(SECOND, 0),
    };
    assert_eq!(
        refused(&mut game, &rules, RED_ACTOR, army(last)),
        manage_error(ManageError::Army(ArmyCommandError::LastStack))
    );
    let merge = ArmyCommand::Move {
        from: slot(SECOND, 2),
        to: slot(SECOND, 0),
    };
    run(&mut game, &rules, RED_ACTOR, army(merge)).unwrap();
    let armies = |game: &GameState| {
        [RED_HERO, SECOND].map(|hero| game.adventure().hero(hero).unwrap().army.clone())
    };
    assert_eq!(armies(&game), [vec![imp(1, 5)], vec![imp(0, 7)]]);
    // The second hero's ring onto red's hero's free ring position: the second hero's removal,
    // then red's hero's placement, each told to red.
    let ring = art(&rules, "t:ring");
    let moved = artifact_move(
        (SECOND, ArtifactPlace::Worn(Position(2))),
        (RED_HERO, ArtifactPlace::Worn(Position(3))),
    );
    let events = run(&mut game, &rules, RED_ACTOR, moved).unwrap();
    let changed = |hero, change| GameEvent::HeroChanged {
        player: RED,
        hero,
        change,
    };
    assert_eq!(
        events,
        [
            changed(
                SECOND,
                HeroChange::RemoveArtifact(ArtifactPlace::Worn(Position(2)))
            ),
            changed(
                RED_HERO,
                HeroChange::PlaceArtifact {
                    artifact: ring,
                    at: ArtifactPlace::Worn(Position(3))
                }
            ),
        ]
    );
    let worn = |hero| game.hero(hero).unwrap().worn[2..4].to_vec();
    assert_eq!(worn(RED_HERO), [Some(ring), Some(ring)]);
    assert_eq!(worn(SECOND), [None, None]);
    // Either hero's formation and flag.
    let flag = Manage::Tactics {
        hero: SECOND,
        on: false,
    };
    run(&mut game, &rules, RED_ACTOR, GameCommand::Manage(flag)).unwrap();
    // Refused: a hero the meeting does not hold (another player's included), the group; nothing
    // changed or told.
    let blue_move = ArmyCommand::Move {
        from: slot(SECOND, 0),
        to: slot(BLUE_HERO, 1),
    };
    let to_group = ArmyCommand::Move {
        from: slot(SECOND, 0),
        to: group(0),
    };
    let blue_flag = Manage::Tactics {
        hero: BLUE_HERO,
        on: false,
    };
    let cases = [
        (army(blue_move), GameCommandError::DecisionOpen),
        (
            GameCommand::Manage(blue_flag),
            GameCommandError::DecisionOpen,
        ),
        (
            artifact_move(
                (RED_HERO, ArtifactPlace::Backpack(0)),
                (BLUE_HERO, ArtifactPlace::Backpack(0)),
            ),
            GameCommandError::DecisionOpen,
        ),
        (
            army(to_group),
            manage_error(ManageError::Army(ArmyCommandError::NoGroup)),
        ),
    ];
    for (command, error) in cases {
        assert_eq!(refused(&mut game, &rules, RED_ACTOR, command), error);
    }
    // Closed, two heroes are apart again.
    run(&mut game, &rules, RED_ACTOR, CLOSE).unwrap();
    let back = ArmyCommand::Move {
        from: slot(SECOND, 0),
        to: slot(RED_HERO, 1),
    };
    assert_eq!(
        refused(&mut game, &rules, RED_ACTOR, army(back)),
        manage_error(ManageError::NotTogether)
    );
    let back = artifact_move(
        (RED_HERO, ArtifactPlace::Worn(Position(3))),
        (SECOND, ArtifactPlace::Worn(Position(2))),
    );
    assert_eq!(
        refused(&mut game, &rules, RED_ACTOR, back),
        manage_error(ManageError::NotTogether)
    );
}

/// Every management command of red on `hero` but the splits, `toward` another army: its army's
/// moves onto its own and the other army's slots and its dismissals, the formations and the flag's
/// values, its artifacts' moves onto its own places and the other hero's (`worn` positions,
/// backpack indices up to `carried`), its assemblies and disassemblies; with the group, the group's
/// moves onto its own and the hero's slots and its dismissals.
fn candidates(
    rules: &GameRuleset,
    hero: HeroId,
    toward: Option<ArmyRef>,
    (worn, carried): (usize, usize),
) -> Vec<Manage> {
    let slots = rules.adventure().params().army_slots();
    let own = ArmyRef::Hero(hero);
    let at = |army, slot| ArmySlot { army, slot };
    let mut sources = vec![own];
    sources.extend(toward.filter(|&army| army == ArmyRef::Group));
    let mut all = Vec::new();
    for &source in &sources {
        let other = if source == own { toward } else { Some(own) };
        for from in 0..slots {
            for target in core::iter::once(source).chain(other) {
                for to in 0..slots {
                    all.push(Manage::Army(ArmyCommand::Move {
                        from: at(source, from),
                        to: at(target, to),
                    }));
                }
            }
            all.push(Manage::Army(ArmyCommand::Dismiss {
                at: at(source, from),
            }));
        }
    }
    for formation in [Formation::Loose, Formation::Tight] {
        all.push(Manage::Formation { hero, formation });
    }
    for on in [true, false] {
        all.push(Manage::Tactics { hero, on });
    }
    let places: Vec<ArtifactPlace> = (0..worn)
        .map(|p| ArtifactPlace::Worn(Position(p)))
        .chain((0..=carried).map(ArtifactPlace::Backpack))
        .collect();
    let receivers = core::iter::once(hero).chain(match toward {
        Some(ArmyRef::Hero(other)) => Some(other),
        _ => None,
    });
    for receiver in receivers {
        for &from in &places {
            for &to in &places {
                all.push(Manage::Artifact {
                    from: (hero, from),
                    to: (receiver, to),
                });
            }
        }
    }
    for p in (0..worn).map(Position) {
        all.push(Manage::Assemble { hero, at: p });
        all.push(Manage::Disassemble { hero, at: p });
    }
    all
}

/// Checks that `legal_management` lists for red's `hero` exactly the commands among
/// [`candidates`] that `apply` accepts, and returns the listing.
fn listed(
    game: &GameState,
    rules: &GameRuleset,
    hero: HeroId,
    toward: Option<ArmyRef>,
) -> Vec<Manage> {
    let listed = legal_management(game, rules, RED_ACTOR, hero);
    let accepted: Vec<Manage> = candidates(rules, hero, toward, (4, 3))
        .into_iter()
        .filter(|&command| accepts(game, rules, RED_ACTOR, command))
        .collect();
    assert_eq!(listed.len(), accepted.len(), "{hero:?}");
    assert!(accepted.iter().all(|command| listed.contains(command)));
    listed
}

// covers: GAM-MAN-002, GAM-DEC-003
#[test]
fn a_meetings_listing_and_split_query_reach_the_other_hero() {
    let rules = manage_rules();
    let mut game = game(&rules);
    let alone = listed(&game, &rules, RED_HERO, None);
    meet(&mut game, &rules);
    let red = listed(&game, &rules, RED_HERO, Some(ArmyRef::Hero(SECOND)));
    let second = listed(&game, &rules, SECOND, Some(ArmyRef::Hero(RED_HERO)));
    // Each hero's one stack stays (it may only swap, and both are imps); their artifacts reach
    // the other hero's places.
    assert!(red.len() > alone.len());
    let merge = Manage::Army(ArmyCommand::Move {
        from: slot(RED_HERO, 3),
        to: slot(SECOND, 0),
    });
    assert!(!red.iter().any(|m| matches!(m, Manage::Army(ArmyCommand::Move { to, .. }) if to.army == ArmyRef::Hero(SECOND))));
    // Split, red's hero's second stack merges into the second hero's or moves to its free slots.
    let split = ArmyCommand::Split {
        from: slot(RED_HERO, 1),
        to: slot(RED_HERO, 3),
        count: 4,
    };
    run(&mut game, &rules, RED_ACTOR, army(split)).unwrap();
    let red = listed(&game, &rules, RED_HERO, Some(ArmyRef::Hero(SECOND)));
    assert!(red.contains(&merge));
    assert!(second.contains(&Manage::Artifact {
        from: (SECOND, ArtifactPlace::Backpack(0)),
        to: (RED_HERO, ArtifactPlace::Backpack(2))
    }));
    // None for another player's hero or a hero the meeting does not hold.
    assert_eq!(legal_management(&game, &rules, RED_ACTOR, BLUE_HERO), []);
    let range = |from, to| split_range(&game, &rules, RED_ACTOR, (from, to));
    assert_eq!(range(slot(RED_HERO, 1), slot(SECOND, 4)), Some(1..=4));
    assert_eq!(range(slot(SECOND, 0), slot(RED_HERO, 1)), Some(1..=7));
    assert_eq!(range(slot(RED_HERO, 1), slot(BLUE_HERO, 4)), None);
    assert_eq!(range(slot(RED_HERO, 1), group(4)), None);
}

/// An object of type `key` at `anchor`, blocking and visited there.
fn object(rules: &GameRuleset, key: &str, anchor: Pos, monster: MonsterSetup) -> ObjectSetup {
    ObjectSetup {
        scenery: Default::default(),
        anchor,
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.adventure().object_type_by_key(key),
        amount: None,
        drawn: TileMask::default(),
        monster: Some(monster),
        guards: None,
        holds: Default::default(),
        owner: None,
    }
}

const fn gogs(slot: u8, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature: GOG,
        count,
    }
}

/// Red's hero (0) with `army` in a two-slot army, beside 4 friendly imps (aggression 0) at
/// (1, 1) with a treasure of 100 gold.
fn dialog_game(rules: &GameRuleset, army: Vec<ArmyStack>) -> GameState {
    let mut setup = setup();
    setup.heroes[0].army = HeroArmy::Custom(army);
    setup.map.objects = vec![object(
        rules,
        "t:monster",
        Pos::new(1, 1, 0),
        MonsterSetup {
            creature: Some(IMP),
            count: 4,
            disposition: 0,
            never_flees: false,
            no_growth: false,
            treasure: vec![(GOLD, 100)],
            artifact: None,
        },
    )];
    GameState::new(rules, &setup, 7).unwrap()
}

fn monster_type(rules: &GameRuleset) -> ObjectTypeId {
    rules.adventure().object_type_by_key("t:monster").unwrap()
}

/// Red's hero's visit of the imps and its acceptance of their offer to join; the acceptance's
/// events. With the offer open no management command is accepted or listed (`DecisionOpen`).
fn join(game: &mut GameState, rules: &GameRuleset) -> Vec<GameEvent> {
    run(game, rules, RED_ACTOR, move_hero(0, &[Pos::new(1, 1, 0)])).unwrap();
    assert!(game.decision(RED).is_some());
    let flag = GameCommand::Manage(Manage::Tactics {
        hero: HERO,
        on: false,
    });
    assert_eq!(
        refused(game, rules, RED_ACTOR, flag),
        GameCommandError::DecisionOpen
    );
    assert_eq!(legal_management(game, rules, RED_ACTOR, HERO), []);
    assert_eq!(split_range(game, rules, RED_ACTOR, (at(1), at(0))), None);
    run(game, rules, RED_ACTOR, ACCEPT).unwrap()
}

const HERO: HeroId = HeroId::new(0);

const fn at(slot: u8) -> ArmySlot {
    ArmySlot {
        army: ArmyRef::Hero(HERO),
        slot,
    }
}

// covers: ADV-MON-008, GAM-DEC-004, GAM-DEC-001
#[test]
fn a_join_without_room_is_offered_and_opens_the_army_dialog() {
    let rules = rules();
    // With 9 imps the group merges into them: no dialog.
    let mut roomy = dialog_game(&rules, vec![gogs(0, 2), imp(1, 9)]);
    let joined = join(&mut roomy, &rules);
    assert!(
        !joined
            .iter()
            .any(|e| matches!(e, GameEvent::ArmyDialogOpened { .. }))
    );
    assert_eq!(roomy.adventure().hero(HERO).unwrap().army[1], imp(1, 13));
    // With two stacks of gogs the offer is still made; accepted, it opens the army dialog.
    let mut game = dialog_game(&rules, vec![gogs(0, 2), gogs(1, 5)]);
    let stack = imp(0, 4);
    let opened = GameEvent::ArmyDialogOpened {
        player: RED,
        hero: HERO,
        stack,
    };
    let anchor = Pos::new(1, 1, 0);
    // The answer told, the monster's leaving, the treasure, then the dialog.
    assert_eq!(
        join(&mut game, &rules),
        [
            GameEvent::OfferAnswered {
                player: RED,
                object_type: monster_type(&rules),
                anchor,
                accepted: true,
            },
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::ObjectRemoved {
                    object_type: monster_type(&rules),
                    anchor,
                },
                seen_by: Viewers::NONE.with(RED).with(BLUE),
            }),
            GameEvent::ResourceGained {
                player: RED,
                resource: GOLD,
                amount: 100,
            },
            opened,
        ]
    );
    assert_eq!(project(&opened, Viewer::Player(RED)), Some(opened));
    assert_eq!(project(&opened, Viewer::Player(BLUE)), None);
    assert_eq!(game.army_dialog(RED), Some((HERO, &[stack][..])));
    assert_eq!(game.meeting(RED), None);
    assert_eq!(
        observe(&game, &rules, Viewer::Player(RED)).decision,
        Some(DecisionView::ArmyDialog {
            hero: HERO,
            group: vec![stack]
        })
    );
    assert_eq!(observe(&game, &rules, Viewer::Player(BLUE)).decision, None);
    assert_eq!(legal_commands(&game, &rules, RED_ACTOR), [CLOSE]);
    assert_eq!(default_answer(&game, &rules, RED), Some(Answer::Close));
    assert!(legal_moves(&game, &rules, RED_ACTOR, HERO).is_none());
    assert_eq!(
        refused(&mut game, &rules, RED_ACTOR, ACCEPT),
        GameCommandError::WrongAnswer
    );
    // Room made: the five gogs dismissed, 3 imps split into their slot, one left in the group;
    // each slot told, the group's as `ArmyRef::Group`; the hash sees the group.
    let hash = game.state_hash();
    run(
        &mut game,
        &rules,
        RED_ACTOR,
        army(ArmyCommand::Dismiss { at: at(1) }),
    )
    .unwrap();
    let take = ArmyCommand::Split {
        from: group(0),
        to: at(1),
        count: 3,
    };
    let told = |at, stack| GameEvent::StackChanged {
        player: RED,
        at,
        stack,
    };
    assert_eq!(
        run(&mut game, &rules, RED_ACTOR, army(take)).unwrap(),
        [
            told(group(0), Some(imp(0, 1))),
            told(at(1), Some(imp(1, 3)))
        ]
    );
    assert_ne!(game.state_hash(), hash);
    assert_eq!(game.army_dialog(RED), Some((HERO, &[imp(0, 1)][..])));
    assert_eq!(
        game.adventure().hero(HERO).unwrap().army,
        [gogs(0, 2), imp(1, 3)]
    );
    // The group alone: its imp moved to its other slot.
    let within = ArmyCommand::Move {
        from: group(0),
        to: group(1),
    };
    assert_eq!(
        run(&mut game, &rules, RED_ACTOR, army(within)).unwrap(),
        [told(group(0), None), told(group(1), Some(imp(1, 1)))]
    );
    // Refused: another hero than the dialog's; nothing changed or told.
    let blue = Manage::Formation {
        hero: BLUE_HERO,
        formation: Formation::Tight,
    };
    assert_eq!(
        refused(&mut game, &rules, RED_ACTOR, GameCommand::Manage(blue)),
        GameCommandError::DecisionOpen
    );
    let from_blue = ArmyCommand::Move {
        from: slot(BLUE_HERO, 0),
        to: group(0),
    };
    assert_eq!(
        refused(&mut game, &rules, RED_ACTOR, army(from_blue)),
        GameCommandError::DecisionOpen
    );
    // Closed: the group's rest is lost, told after the closing.
    let closed = run(&mut game, &rules, RED_ACTOR, CLOSE).unwrap();
    assert_eq!(
        closed,
        [
            GameEvent::DecisionClosed { player: RED },
            GameEvent::GroupLost {
                player: RED,
                hero: HERO,
                stack: imp(1, 1),
            }
        ]
    );
    for event in &closed {
        assert_eq!(project(event, Viewer::Player(BLUE)), None);
    }
    assert_eq!(game.army_dialog(RED), None);
    assert_eq!(
        legal_commands(&game, &rules, RED_ACTOR),
        [GameCommand::EndTurn]
    );
    // Closed at once, the whole group is lost.
    let mut game = dialog_game(&rules, vec![gogs(0, 2), gogs(1, 5)]);
    join(&mut game, &rules);
    let closed = run(&mut game, &rules, RED_ACTOR, CLOSE).unwrap();
    assert_eq!(
        closed[1],
        GameEvent::GroupLost {
            player: RED,
            hero: HERO,
            stack,
        }
    );
}

// covers: GAM-MAN-002, GAM-DEC-004
#[test]
fn an_army_dialogs_listing_and_split_query_reach_the_group() {
    let rules = rules();
    let mut game = dialog_game(&rules, vec![gogs(0, 2), gogs(1, 5)]);
    join(&mut game, &rules);
    let listed = listed(&game, &rules, HERO, Some(ArmyRef::Group));
    // The hero's gogs onto the group's slots, the group's imps onto the hero's (swaps), the
    // group's dismissal.
    for command in [
        ArmyCommand::Move {
            from: at(1),
            to: group(1),
        },
        ArmyCommand::Move {
            from: group(0),
            to: at(0),
        },
        ArmyCommand::Dismiss { at: group(0) },
    ] {
        assert!(listed.contains(&Manage::Army(command)), "{command:?}");
    }
    let range = |from, to| split_range(&game, &rules, RED_ACTOR, (from, to));
    assert_eq!(range(group(0), group(1)), Some(1..=3));
    assert_eq!(range(at(1), group(1)), Some(1..=4));
    assert_eq!(range(at(1), slot(BLUE_HERO, 1)), None);
    assert_eq!(legal_management(&game, &rules, RED_ACTOR, BLUE_HERO), []);
}
