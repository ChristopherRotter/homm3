//! Tests of the allies' meeting (GAM-TEM-003; E10 T3b1, increment 1's design note §5): a step of
//! red's hero toward blue's on red's team opens the meeting as red's decision, followed by blue's
//! hero's parts, which red's observation shows while it is open (the user's call at T3b's start);
//! onto blue's hero only gifts, each told to blue too, every other command naming it refused
//! with `AllyHero`, nothing changed; the listing holds the gifts alone; the Scholar's lessons told
//! to each hero's owner.

use h3_heroes::{ArtifactInstance, ArtifactPlace, Position, wearing};

use super::decisions::{CLOSE, after_steps, move_hero};
use super::exchanges::scholar_rules;
use super::manage::{
    BLUE, RED, RED_HERO, SECOND, art, artifact_move, imp, manage_rules, manage_setup, refused, run,
    slot,
};
use super::*;

const RED_ACTOR: Actor = Actor::Player(RED);

/// [`manage_setup`] with its second hero blue's (on red's team when `allied`): red's hero at
/// (0, 0) with 9 imps in slot 1, blue's hero 2 beside it at (1, 0) with 3 imps in slot 0, both
/// wearing the ring and carrying the band and the charm.
fn setup_of(rules: &GameRuleset, allied: bool) -> GameSetup {
    let mut setup = manage_setup(rules);
    setup.heroes[2].owner = BLUE;
    if allied {
        setup.players[1].team = 0;
    }
    setup
}

fn game(rules: &GameRuleset) -> GameState {
    GameState::new(rules, &setup_of(rules, true), 1).unwrap()
}

/// Red's hero's step onto blue's hero's tile: its events but the steps.
fn meet(game: &mut GameState, rules: &GameRuleset) -> Vec<GameEvent> {
    after_steps(run(game, rules, RED_ACTOR, move_hero(0, &[Pos::new(1, 0, 0)])).unwrap())
}

/// What `player` sees of hero `hero`.
fn info(game: &GameState, rules: &GameRuleset, player: PlayerId, hero: HeroId) -> HeroInfo {
    observe(game, rules, Viewer::Player(player))
        .heroes
        .into_iter()
        .find(|info| info.id == hero)
        .unwrap()
}

// covers: GAM-TEM-003, GAM-MET-001, GAM-DEC-003
#[test]
fn a_step_toward_an_ally_s_hero_opens_the_visitor_s_meeting_with_that_hero_shown() {
    let rules = manage_rules();
    let mut game = game(&rules);
    let heroes = [RED_HERO, SECOND];
    let events = meet(&mut game, &rules);
    // The meeting red's, then blue's hero as red sees it, part by part.
    let on = game.adventure().hero(SECOND).unwrap();
    let parts: Vec<GameEvent> = MeetingPart::of(&on.army, game.hero(SECOND).unwrap())
        .map(|part| GameEvent::MeetingPartSeen {
            player: RED,
            hero: SECOND,
            part,
        })
        .collect();
    let opened = GameEvent::MeetingOpened {
        player: RED,
        heroes,
    };
    assert_eq!(events[0], opened);
    assert_eq!(events[1..], parts[..]);
    // The head, the stack, the ring worn, the band and the charm carried.
    assert_eq!(parts.len(), 5);
    assert!(matches!(
        parts[0],
        GameEvent::MeetingPartSeen {
            part: MeetingPart::Head { positions, .. },
            ..
        } if usize::from(positions) == game.hero(SECOND).unwrap().worn.len()
    ));
    assert_eq!(
        parts[1],
        GameEvent::MeetingPartSeen {
            player: RED,
            hero: SECOND,
            part: MeetingPart::Stack(imp(0, 3)),
        }
    );
    for event in &events {
        assert_eq!(project(event, Viewer::Player(RED)), Some(*event));
        assert_eq!(project(event, Viewer::Player(BLUE)), None);
    }
    assert_eq!(game.meeting(RED), Some(heroes));
    assert_eq!(game.meeting_ally_of(RED), Some(SECOND));
    assert_eq!(
        (game.meeting(BLUE), game.meeting_ally_of(BLUE)),
        (None, None)
    );
    // While it is open red sees blue's hero's details and army; blue its own as before.
    let seen = info(&game, &rules, RED, SECOND);
    assert_eq!(seen.details.as_deref(), game.hero(SECOND));
    assert_eq!(seen.army, Some(vec![imp(0, 3)]));
    let own = info(&game, &rules, BLUE, SECOND);
    assert_eq!(
        (own.details.as_deref(), own.army),
        (game.hero(SECOND), None)
    );
    assert_eq!(
        observe(&game, &rules, Viewer::Player(RED)).decision,
        Some(DecisionView::Meeting { heroes })
    );
    // Closed, red's view of it is the map's again.
    run(&mut game, &rules, RED_ACTOR, CLOSE).unwrap();
    let seen = info(&game, &rules, RED, SECOND);
    assert_eq!((seen.details, seen.army), (None, None));
    // An own hero's meeting tells no part (`exchanges.rs`); another team's is a battle.
    let mut apart = GameState::new(&rules, &setup_of(&rules, false), 1).unwrap();
    let events = meet(&mut apart, &rules);
    assert!(apart.meeting(RED).is_none());
    assert!(
        events
            .iter()
            .all(|e| !matches!(e, GameEvent::MeetingOpened { .. }))
    );
    assert!(apart.battle(RED).is_some());
}

// covers: GAM-TEM-003, GAM-MAN-001, ADV-ARM-002
#[test]
fn onto_an_ally_s_hero_in_a_meeting_only_gifts_pass_each_told_to_its_owner() {
    let rules = manage_rules();
    let mut game = game(&rules);
    meet(&mut game, &rules);
    let told = |player, at, stack| GameEvent::StackChanged { player, at, stack };
    // Four of red's imps split into blue's empty slot 1: told to red, then blue's slot to blue.
    let split = ArmyCommand::Split {
        from: slot(RED_HERO, 1),
        to: slot(SECOND, 1),
        count: 4,
    };
    let events = run(
        &mut game,
        &rules,
        RED_ACTOR,
        GameCommand::Manage(Manage::Army(split)),
    );
    assert_eq!(
        events.unwrap(),
        [
            told(RED, slot(RED_HERO, 1), Some(imp(1, 5))),
            told(RED, slot(SECOND, 1), Some(imp(1, 4))),
            told(BLUE, slot(SECOND, 1), Some(imp(1, 4))),
        ]
    );
    assert_eq!(
        info(&game, &rules, RED, SECOND).army,
        Some(vec![imp(0, 3), imp(1, 4)])
    );
    // The band from red's backpack onto blue's hero's free ring position, the charm into its
    // backpack: red's removal told to red, blue's hero's placement to blue and red.
    let ring = art(&rules, "t:ring");
    let worn = game.hero(SECOND).unwrap().worn.clone();
    let ring_at = worn.iter().position(|a| *a == Some(ring)).unwrap();
    let band_type = rules.core().artifact_by_key("t:band").unwrap();
    let free = (0..worn.len())
        .find(|&at| takes(&worn, rules.core(), band_type, Position(at)))
        .unwrap();
    let band = artifact_move(
        (RED_HERO, ArtifactPlace::Backpack(0)),
        (SECOND, ArtifactPlace::Worn(Position(free))),
    );
    let events = run(&mut game, &rules, RED_ACTOR, band).unwrap();
    let changed = |player, hero, change| GameEvent::HeroChanged {
        player,
        hero,
        change,
    };
    let placed = HeroChange::PlaceArtifact {
        artifact: art(&rules, "t:band"),
        at: ArtifactPlace::Worn(Position(free)),
    };
    assert_eq!(
        events,
        [
            changed(
                RED,
                RED_HERO,
                HeroChange::RemoveArtifact(ArtifactPlace::Backpack(0))
            ),
            changed(BLUE, SECOND, placed),
            changed(RED, SECOND, placed),
        ]
    );
    let charm = artifact_move(
        (RED_HERO, ArtifactPlace::Backpack(0)),
        (SECOND, ArtifactPlace::Backpack(0)),
    );
    run(&mut game, &rules, RED_ACTOR, charm).unwrap();
    let details = game.hero(SECOND).unwrap();
    assert_eq!(details.worn[free], Some(art(&rules, "t:band")));
    assert_eq!(details.backpack[0], art(&rules, "t:charm"));
    assert_eq!(
        info(&game, &rules, RED, SECOND).details.as_deref(),
        Some(details)
    );
    // Nothing is taken and none of its settings changes: each refused, nothing changed or told.
    let ally = GameCommandError::AllyHero(SECOND);
    let manage = |m| GameCommand::Manage(m);
    for command in [
        // A stack taken, a move and a split onto an occupied slot, a stack dismissed.
        manage(Manage::Army(ArmyCommand::Move {
            from: slot(SECOND, 0),
            to: slot(RED_HERO, 2),
        })),
        manage(Manage::Army(ArmyCommand::Move {
            from: slot(RED_HERO, 1),
            to: slot(SECOND, 0),
        })),
        manage(Manage::Army(ArmyCommand::Split {
            from: slot(RED_HERO, 1),
            to: slot(SECOND, 0),
            count: 1,
        })),
        manage(Manage::Army(ArmyCommand::Dismiss {
            at: slot(SECOND, 0),
        })),
        // An artifact taken, one onto an occupied position.
        artifact_move(
            (SECOND, ArtifactPlace::Worn(Position(ring_at))),
            (RED_HERO, ArtifactPlace::Backpack(0)),
        ),
        artifact_move(
            (RED_HERO, ArtifactPlace::Worn(Position(ring_at))),
            (SECOND, ArtifactPlace::Worn(Position(ring_at))),
        ),
        // Its settings.
        manage(Manage::Formation {
            hero: SECOND,
            formation: Formation::Tight,
        }),
        manage(Manage::Tactics {
            hero: SECOND,
            on: false,
        }),
        manage(Manage::Assemble {
            hero: SECOND,
            at: Position(ring_at),
        }),
        manage(Manage::Disassemble {
            hero: SECOND,
            at: Position(ring_at),
        }),
    ] {
        assert_eq!(refused(&mut game, &rules, RED_ACTOR, command), ally);
    }
    assert_eq!(
        ally.to_string(),
        "hero 2 is an ally's: only gifts into its empty places"
    );
}

// covers: GAM-TEM-003, GAM-MAN-002
#[test]
fn a_meeting_with_an_ally_s_hero_lists_the_gifts_alone() {
    let rules = manage_rules();
    let mut game = game(&rules);
    meet(&mut game, &rules);
    // Blue's hero's own management is none of red's.
    assert!(legal_management(&game, &rules, RED_ACTOR, SECOND).is_empty());
    let listed = legal_management(&game, &rules, RED_ACTOR, RED_HERO);
    let details = game.hero(SECOND).unwrap();
    let mut gifts = 0;
    for command in &listed {
        match *command {
            Manage::Army(ArmyCommand::Move { to, .. } | ArmyCommand::Split { to, .. })
                if to.army == ArmyRef::Hero(SECOND) =>
            {
                assert!(
                    game.adventure()
                        .hero(SECOND)
                        .unwrap()
                        .army
                        .iter()
                        .all(|s| s.slot != to.slot)
                );
                gifts += 1;
            }
            Manage::Artifact {
                to: (SECOND, place),
                ..
            } => {
                if let ArtifactPlace::Worn(Position(at)) = place {
                    assert_eq!(details.worn[at], None, "{command:?}");
                }
                gifts += 1;
            }
            _ => {}
        }
    }
    assert!(gifts > 0);
    // Each listed command passes.
    for command in listed {
        let mut tried = game.clone();
        run(&mut tried, &rules, RED_ACTOR, GameCommand::Manage(command)).unwrap();
    }
}

// covers: GAM-TEM-003, HER-SPL-007
#[test]
fn scholar_teaches_an_ally_s_hero_as_one_s_own_each_lesson_told_to_its_owner() {
    let rules = scholar_rules();
    let core = rules.core();
    let id = |k: &str| core.skill_by_key(k).unwrap();
    let spell = |k: &str| core.spell_by_key(k).unwrap();
    let book: ArtifactInstance = core.artifact_by_key("t:book").unwrap().into();
    let mut setup = setup();
    setup.players[1].team = 0;
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
        owner: BLUE,
        pos: Pos::new(1, 0, 0),
        details: HeroDetailsSetup {
            spells: Some(vec![spell("t:gate")]),
            ..setup.heroes[0].details.clone()
        },
        ..setup.heroes[0].clone()
    });
    setup.heroes[2].details.skills = Some(vec![]);
    let mut game = GameState::new(&rules, &setup, 1).unwrap();
    let learned = |player, hero, k| GameEvent::HeroChanged {
        player,
        hero,
        change: HeroChange::LearnSpell(spell(k)),
    };
    let events = meet(&mut game, &rules);
    assert_eq!(
        events[..3],
        [
            learned(RED, RED_HERO, "t:gate"),
            learned(BLUE, SECOND, "t:gust"),
            GameEvent::MeetingOpened {
                player: RED,
                heroes: [RED_HERO, SECOND]
            }
        ]
    );
    assert_eq!(
        game.hero(SECOND).unwrap().spells,
        [spell("t:gate"), spell("t:gust")]
    );
}
