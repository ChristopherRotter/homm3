//! Tests of the management commands (GAM-MAN-001, GAM-MAN-002; E08 T51a): a player's army,
//! formation, flag and artifact commands on one hero at a time in its turn, told to it alone;
//! the per-hero listing and the split's query; the tactics flag in battle (GAM-BAT-009).

use h3_adventure::AdventureCommand;
use h3_battle::observe as observe_battle;
use h3_contracts::BattleSide;
use h3_heroes::{
    ArtifactInstance, ArtifactMove, ArtifactPlace, CombineError, HeroDetails, MoveError, Position,
    apply_change, assemble_changes, disassemble_changes, move_changes, wearing,
};

use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const RED_HERO: HeroId = HeroId::new(0);
const BLUE_HERO: HeroId = HeroId::new(1);
const SECOND: HeroId = HeroId::new(2);

/// The rules with seven army slots, `t:band` (a ring), `t:pair` (a combination of the ring and
/// the band, worn as a ring) and `t:tactics` (Tactics 1 to 3).
fn manage_rules() -> GameRuleset {
    let mut pack = core_pack(false);
    pack.artifacts.push(artifact("t:band", "t:ring_slot"));
    pack.artifacts.push(ArtifactDef {
        parts: vec![key("t:ring"), key("t:band")],
        ..artifact("t:pair", "t:ring_slot")
    });
    pack.skills.push(SecondarySkillDef {
        key: key("t:tactics"),
        name: "Tactics".into(),
        levels: [1, 2, 3].map(|level| vec![HeroEffect::Tactics { level }]),
    });
    let core = Arc::new(Ruleset::compile(&[pack], "t:faithful").unwrap());
    let part = part_with(&core, vec![88, 25], 7);
    let battle = battle_part(&core, 7);
    GameRuleset::new(core, Arc::new(part), Arc::new(battle)).unwrap()
}

fn art(rules: &GameRuleset, k: &str) -> ArtifactInstance {
    rules.core().artifact_by_key(k).unwrap().into()
}

/// [`setup`] with red's hero wearing the ring (position 2) and carrying the band and the charm,
/// and a second red hero at (1, 0) with three imps.
fn manage_setup(rules: &GameRuleset) -> GameSetup {
    let mut setup = setup();
    setup.heroes[0].details.artifacts = Some(HeroArtifactsSetup {
        worn: wearing(rules.core(), &[art(rules, "t:ring")]),
        backpack: vec![art(rules, "t:band"), art(rules, "t:charm")],
    });
    setup.heroes.push(GameHeroSetup {
        pos: Pos::new(1, 0, 0),
        army: HeroArmy::Custom(vec![imp(0, 3)]),
        ..setup.heroes[0].clone()
    });
    setup
}

fn game(rules: &GameRuleset) -> GameState {
    GameState::new(rules, &manage_setup(rules), 1).unwrap()
}

const fn imp(slot: u8, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature: IMP,
        count,
    }
}

const fn slot(hero: HeroId, slot: u8) -> ArmySlot {
    ArmySlot {
        army: ArmyRef::Hero(hero),
        slot,
    }
}

const fn group(slot: u8) -> ArmySlot {
    ArmySlot {
        army: ArmyRef::Group,
        slot,
    }
}

const fn army(command: ArmyCommand) -> GameCommand {
    GameCommand::Manage(Manage::Army(command))
}

const fn artifact_move(from: (HeroId, ArtifactPlace), to: (HeroId, ArtifactPlace)) -> GameCommand {
    GameCommand::Manage(Manage::Artifact { from, to })
}

/// Applies `actor`'s `command` and returns its events.
fn run(
    game: &mut GameState,
    rules: &GameRuleset,
    actor: Actor,
    command: GameCommand,
) -> Result<Vec<GameEvent>, GameCommandError> {
    let mut events = Vec::new();
    apply(game, rules, actor, command, &mut events).map(|()| events)
}

/// Applies `actor`'s `command`, which must be refused with `error`, nothing changed or told.
fn refused(
    game: &mut GameState,
    rules: &GameRuleset,
    actor: Actor,
    command: GameCommand,
) -> GameCommandError {
    let before = game.clone();
    let mut events = Vec::new();
    let error = apply(game, rules, actor, command.clone(), &mut events).unwrap_err();
    assert_eq!((&*game, events.len()), (&before, 0), "{command:?}");
    error
}

fn manage_error(error: ManageError) -> GameCommandError {
    GameCommandError::Manage(error)
}

// covers: GAM-MAN-001, ADV-ARM-001, ADV-ARM-003, ADV-ARM-004
#[test]
fn a_player_rearranges_a_heros_army_in_its_turn() {
    let rules = manage_rules();
    let mut game = game(&rules);
    let red = Actor::Player(RED);
    let told = |at, stack| GameEvent::StackChanged {
        player: RED,
        at,
        stack,
    };
    // A split: slot 1 keeps 5 imps, slot 3 takes 4; each slot told to red, the stack's first.
    let split = ArmyCommand::Split {
        from: slot(RED_HERO, 1),
        to: slot(RED_HERO, 3),
        count: 4,
    };
    let events = run(&mut game, &rules, red, army(split)).unwrap();
    assert_eq!(
        events,
        [
            told(slot(RED_HERO, 1), Some(imp(1, 5))),
            told(slot(RED_HERO, 3), Some(imp(3, 4)))
        ]
    );
    assert_eq!(
        game.adventure().hero(RED_HERO).unwrap().army,
        [imp(1, 5), imp(3, 4)]
    );
    // Told to red alone.
    for event in &events {
        assert_eq!(project(event, Viewer::Player(RED)), Some(*event));
        assert_eq!(project(event, Viewer::Player(BLUE)), None);
    }
    // A merge empties the slot it leaves; the second hero's army is its own.
    let merge = ArmyCommand::Move {
        from: slot(RED_HERO, 3),
        to: slot(RED_HERO, 1),
    };
    let events = run(&mut game, &rules, red, army(merge)).unwrap();
    assert_eq!(
        events,
        [
            told(slot(RED_HERO, 3), None),
            told(slot(RED_HERO, 1), Some(imp(1, 9)))
        ]
    );
    let moved = ArmyCommand::Move {
        from: slot(SECOND, 0),
        to: slot(SECOND, 6),
    };
    run(&mut game, &rules, red, army(moved)).unwrap();
    assert_eq!(game.adventure().hero(SECOND).unwrap().army, [imp(6, 3)]);
    // Refused, nothing changed or told: the hero's last stack, two heroes or a hero and the
    // group without a meeting or an army dialog, the group alone, another player's hero, an
    // empty slot; another player outside its turn; the system actor.
    let dismiss = |at| army(ArmyCommand::Dismiss { at });
    let to = |to| {
        army(ArmyCommand::Move {
            from: slot(RED_HERO, 1),
            to,
        })
    };
    let cases = [
        (
            dismiss(slot(RED_HERO, 1)),
            ManageError::Army(ArmyCommandError::LastStack),
        ),
        (to(slot(SECOND, 0)), ManageError::NotTogether),
        (to(group(0)), ManageError::NotTogether),
        (
            dismiss(group(0)),
            ManageError::Army(ArmyCommandError::NoGroup),
        ),
        (
            dismiss(slot(BLUE_HERO, 0)),
            ManageError::NotYourHero(BLUE_HERO),
        ),
        (
            army(ArmyCommand::Move {
                from: slot(RED_HERO, 5),
                to: slot(RED_HERO, 6),
            }),
            ManageError::Army(ArmyCommandError::NoStack(5)),
        ),
    ];
    for (command, error) in cases {
        assert_eq!(
            refused(&mut game, &rules, red, command),
            manage_error(error)
        );
    }
    assert_eq!(
        refused(
            &mut game,
            &rules,
            Actor::Player(BLUE),
            dismiss(slot(BLUE_HERO, 0))
        ),
        GameCommandError::NotYourTurn(BLUE)
    );
    assert_eq!(
        refused(&mut game, &rules, Actor::System, dismiss(slot(RED_HERO, 1))),
        GameCommandError::NotForSystem
    );
    // The errors read well.
    assert_eq!(
        manage_error(ManageError::NotTogether).to_string(),
        "the two armies are not together"
    );
    assert_eq!(
        manage_error(ManageError::NotYourHero(BLUE_HERO)).to_string(),
        "hero 1 is not the player's on the map"
    );
    assert_eq!(
        manage_error(ManageError::Army(ArmyCommandError::LastStack)).to_string(),
        "a hero's last stack would leave its army"
    );
}

// covers: GAM-MAN-001, ADV-ARM-006
#[test]
fn the_formation_and_the_flag_are_set_and_told_to_the_owner_alone() {
    let rules = manage_rules();
    let mut game = game(&rules);
    let red = Actor::Player(RED);
    let formation = GameCommand::Manage(Manage::Formation {
        hero: RED_HERO,
        formation: Formation::Tight,
    });
    let set = run(&mut game, &rules, red, formation).unwrap();
    assert_eq!(
        set,
        [GameEvent::FormationSet {
            player: RED,
            hero: RED_HERO,
            formation: Formation::Tight
        }]
    );
    let flag = GameCommand::Manage(Manage::Tactics {
        hero: RED_HERO,
        on: false,
    });
    let off = run(&mut game, &rules, red, flag).unwrap();
    assert_eq!(
        off,
        [GameEvent::TacticsSet {
            player: RED,
            hero: RED_HERO,
            on: false
        }]
    );
    let on_map = game.adventure().hero(RED_HERO).unwrap();
    assert_eq!(
        (on_map.formation, on_map.tactics),
        (Formation::Tight, false)
    );
    // The owner sees both; another player's view of the events is nothing.
    let seen = observe(&game, &rules, Viewer::Player(RED));
    let own = seen.adventure.hero(RED_HERO).unwrap().own.as_ref().unwrap();
    assert_eq!((own.formation, own.tactics), (Formation::Tight, false));
    for event in set.iter().chain(&off) {
        assert_eq!(project(event, Viewer::Player(RED)), Some(*event));
        assert_eq!(project(event, Viewer::Player(BLUE)), None);
    }
    // Another player's hero.
    for command in [
        Manage::Formation {
            hero: BLUE_HERO,
            formation: Formation::Loose,
        },
        Manage::Tactics {
            hero: BLUE_HERO,
            on: false,
        },
    ] {
        assert_eq!(
            refused(&mut game, &rules, red, GameCommand::Manage(command)),
            manage_error(ManageError::NotYourHero(BLUE_HERO))
        );
    }
}

/// `details` after `changes`, as `h3-heroes` applies them.
fn changed(rules: &GameRuleset, details: &HeroDetails, changes: &[HeroChange]) -> HeroDetails {
    let mut details = details.clone();
    for &change in changes {
        apply_change(&mut details, rules.core(), change);
    }
    details
}

/// The events of `changes` to red's hero.
fn told(changes: &[HeroChange]) -> Vec<GameEvent> {
    changes
        .iter()
        .map(|&change| GameEvent::HeroChanged {
            player: RED,
            hero: RED_HERO,
            change,
        })
        .collect()
}

// covers: GAM-MAN-001, HER-ART-005, HER-ART-006
#[test]
fn a_heros_artifacts_move_assemble_and_disassemble_through_its_changes() {
    let rules = manage_rules();
    let core = rules.core();
    let mut game = game(&rules);
    let red = Actor::Player(RED);
    let (band, charm) = (art(&rules, "t:band"), art(&rules, "t:charm"));
    let pair = art(&rules, "t:pair");
    // The band from the backpack onto the second ring position.
    let before = game.hero(RED_HERO).unwrap().clone();
    let moved = ArtifactMove {
        from: ArtifactPlace::Backpack(0),
        to: ArtifactPlace::Worn(Position(3)),
    };
    let changes = move_changes(&before, None, core, moved).unwrap().from;
    let events = run(
        &mut game,
        &rules,
        red,
        artifact_move((RED_HERO, moved.from), (RED_HERO, moved.to)),
    )
    .unwrap();
    assert_eq!(events, told(&changes));
    let after = game.hero(RED_HERO).unwrap();
    assert_eq!(after, &changed(&rules, &before, &changes));
    assert_eq!(
        (after.worn[3], &after.backpack[..]),
        (Some(band), &[charm][..])
    );
    // The ring completes the pair: assembled, then disassembled again.
    let worn = after.clone();
    let changes = assemble_changes(&worn, core, Position(2)).unwrap();
    let assemble = Manage::Assemble {
        hero: RED_HERO,
        at: Position(2),
    };
    let events = run(&mut game, &rules, red, GameCommand::Manage(assemble)).unwrap();
    assert_eq!(events, told(&changes));
    let assembled = game.hero(RED_HERO).unwrap().clone();
    assert_eq!(assembled, changed(&rules, &worn, &changes));
    assert_eq!(assembled.worn[2], Some(pair));
    let changes = disassemble_changes(&assembled, core, Position(2)).unwrap();
    let disassemble = Manage::Disassemble {
        hero: RED_HERO,
        at: Position(2),
    };
    let events = run(&mut game, &rules, red, GameCommand::Manage(disassemble)).unwrap();
    assert_eq!(events, told(&changes));
    assert_eq!(
        game.hero(RED_HERO).unwrap(),
        &changed(&rules, &assembled, &changes)
    );
    // Told to red alone.
    assert!(
        events
            .iter()
            .all(|e| project(e, Viewer::Player(BLUE)).is_none())
    );
    // Refused, nothing changed or told: an empty place, two heroes without a meeting, another
    // player's hero on either end, a hero not on the map; a part of no complete combination, an
    // empty position, an artifact that is no combination.
    let backpack = |hero| (hero, ArtifactPlace::Backpack(0));
    let ring_at = (RED_HERO, ArtifactPlace::Worn(Position(2)));
    let cases = [
        (
            artifact_move((RED_HERO, ArtifactPlace::Backpack(5)), ring_at),
            ManageError::Artifact(MoveError::NoArtifact(ArtifactPlace::Backpack(5))),
        ),
        (
            artifact_move(ring_at, backpack(SECOND)),
            ManageError::NotTogether,
        ),
        (
            artifact_move(ring_at, backpack(BLUE_HERO)),
            ManageError::NotYourHero(BLUE_HERO),
        ),
        (
            artifact_move(backpack(BLUE_HERO), ring_at),
            ManageError::NotYourHero(BLUE_HERO),
        ),
        (
            artifact_move(backpack(HeroId::new(9)), ring_at),
            ManageError::NotYourHero(HeroId::new(9)),
        ),
        (
            GameCommand::Manage(Manage::Assemble {
                hero: SECOND,
                at: Position(0),
            }),
            ManageError::Combine(CombineError::NoArtifact(Position(0))),
        ),
        (
            GameCommand::Manage(Manage::Disassemble {
                hero: RED_HERO,
                at: Position(2),
            }),
            ManageError::Combine(CombineError::NotCombination(Position(2))),
        ),
        (
            GameCommand::Manage(Manage::Assemble {
                hero: BLUE_HERO,
                at: Position(2),
            }),
            ManageError::NotYourHero(BLUE_HERO),
        ),
        (
            GameCommand::Manage(Manage::Disassemble {
                hero: BLUE_HERO,
                at: Position(2),
            }),
            ManageError::NotYourHero(BLUE_HERO),
        ),
    ];
    for (command, error) in cases {
        assert_eq!(
            refused(&mut game, &rules, red, command),
            manage_error(error)
        );
    }
    assert_eq!(
        manage_error(ManageError::Artifact(MoveError::SamePlace)).to_string(),
        "the artifact is there already"
    );
    assert_eq!(
        manage_error(ManageError::Combine(CombineError::NotCombination(
            Position(2)
        )))
        .to_string(),
        "the artifact at 2 is no combination"
    );
}

/// Every management command of red on `hero` but the splits, among its army's slots, the
/// formations, the flag's values, `places` worn positions and backpack indices up to `carried`,
/// and the positions.
fn candidates(rules: &GameRuleset, hero: HeroId, (worn, carried): (usize, usize)) -> Vec<Manage> {
    let slots = rules.adventure().params().army_slots();
    let mut all = Vec::new();
    for from in 0..slots {
        for to in 0..slots {
            all.push(Manage::Army(ArmyCommand::Move {
                from: slot(hero, from),
                to: slot(hero, to),
            }));
        }
        all.push(Manage::Army(ArmyCommand::Dismiss {
            at: slot(hero, from),
        }));
    }
    for formation in [Formation::Loose, Formation::Tight] {
        all.push(Manage::Formation { hero, formation });
    }
    for on in [true, false] {
        all.push(Manage::Tactics { hero, on });
    }
    let places: Vec<ArtifactPlace> = (0..worn)
        .map(|at| ArtifactPlace::Worn(Position(at)))
        .chain((0..=carried).map(ArtifactPlace::Backpack))
        .collect();
    for &from in &places {
        for &to in &places {
            all.push(Manage::Artifact {
                from: (hero, from),
                to: (hero, to),
            });
        }
    }
    for at in (0..worn).map(Position) {
        all.push(Manage::Assemble { hero, at });
        all.push(Manage::Disassemble { hero, at });
    }
    all
}

/// Whether `apply` accepts `command` of `actor` in `game`.
fn accepts(game: &GameState, rules: &GameRuleset, actor: Actor, command: Manage) -> bool {
    let mut copy = game.clone();
    apply(
        &mut copy,
        rules,
        actor,
        GameCommand::Manage(command),
        &mut NoopSink,
    )
    .is_ok()
}

// covers: GAM-MAN-002
#[test]
fn the_listing_gives_a_heros_commands_apply_accepts_but_the_splits() {
    let rules = manage_rules();
    let mut game = game(&rules);
    let red = Actor::Player(RED);
    let check = |game: &GameState| {
        let listed = legal_management(game, &rules, red, RED_HERO);
        let accepted: Vec<Manage> = candidates(&rules, RED_HERO, (4, 3))
            .into_iter()
            .filter(|&command| accepts(game, &rules, red, command))
            .collect();
        assert_eq!(listed.len(), accepted.len());
        assert!(accepted.iter().all(|command| listed.contains(command)));
        listed
    };
    // The imps' moves onto the six other slots (none dismissed: the last stack); two formations
    // and two flags; the ring onto the other ring position and the backpack's three indices, the
    // band onto either ring position and after the charm, the charm before the band.
    let listed = check(&game);
    assert_eq!(listed.len(), 6 + 2 + 2 + 8);
    assert_eq!(
        listed[0],
        Manage::Army(ArmyCommand::Move {
            from: slot(RED_HERO, 1),
            to: slot(RED_HERO, 0)
        })
    );
    // With the band worn, the pair assembles; once assembled, it disassembles.
    let wear = artifact_move(
        (RED_HERO, ArtifactPlace::Backpack(0)),
        (RED_HERO, ArtifactPlace::Worn(Position(3))),
    );
    run(&mut game, &rules, red, wear).unwrap();
    let assemble = Manage::Assemble {
        hero: RED_HERO,
        at: Position(2),
    };
    assert!(check(&game).contains(&assemble));
    run(&mut game, &rules, red, GameCommand::Manage(assemble)).unwrap();
    assert!(check(&game).contains(&Manage::Disassemble {
        hero: RED_HERO,
        at: Position(2)
    }));
    // None for the system actor, another player's hero, a hero not on the map, a player outside
    // its turn.
    assert_eq!(legal_management(&game, &rules, Actor::System, RED_HERO), []);
    assert_eq!(legal_management(&game, &rules, red, BLUE_HERO), []);
    assert_eq!(legal_management(&game, &rules, red, HeroId::new(9)), []);
    let blue = Actor::Player(BLUE);
    assert_eq!(legal_management(&game, &rules, blue, BLUE_HERO), []);
    run(&mut game, &rules, red, GameCommand::EndTurn).unwrap();
    assert_eq!(legal_management(&game, &rules, red, RED_HERO), []);
    assert!(!legal_management(&game, &rules, blue, BLUE_HERO).is_empty());
}

// covers: GAM-MAN-002, ADV-ARM-003
#[test]
fn the_split_query_gives_one_armys_counts_in_the_players_turn() {
    let rules = manage_rules();
    let game = game(&rules);
    let red = Actor::Player(RED);
    let range = |actor, from, to| split_range(&game, &rules, actor, (from, to));
    // Nine imps: one to eight into an empty slot.
    assert_eq!(
        range(red, slot(RED_HERO, 1), slot(RED_HERO, 3)),
        Some(1..=8)
    );
    // None for the system actor, a player outside its turn, two armies without a meeting, an
    // empty slot.
    assert_eq!(
        range(Actor::System, slot(RED_HERO, 1), slot(RED_HERO, 3)),
        None
    );
    assert_eq!(
        range(Actor::Player(BLUE), slot(BLUE_HERO, 0), slot(BLUE_HERO, 3)),
        None
    );
    assert_eq!(range(red, slot(RED_HERO, 1), slot(SECOND, 3)), None);
    assert_eq!(range(red, slot(RED_HERO, 0), slot(RED_HERO, 3)), None);
}

/// Red's Tactics `levels.0` hero attacking blue's of `levels.1`, each flag set by its owner in
/// its turn first (`flags`): the battle's tactics phase's side, and both sides' Tactics.
fn tactics_battle(levels: (u8, u8), flags: (bool, bool)) -> (Option<BattleSide>, [u8; 2]) {
    let rules = manage_rules();
    let tactics = rules.core().skill_by_key("t:tactics").unwrap();
    let mut setup = setup();
    for (hero, level) in setup.heroes.iter_mut().zip([levels.0, levels.1]) {
        hero.details.skills = Some(vec![(tactics, level)]);
    }
    let mut game = GameState::new(&rules, &setup, 3).unwrap();
    // Red sets its flag, blue its own in its turn; the next day red attacks.
    for (player, hero, on) in [(RED, RED_HERO, flags.0), (BLUE, BLUE_HERO, flags.1)] {
        let flag = GameCommand::Manage(Manage::Tactics { hero, on });
        run(&mut game, &rules, Actor::Player(player), flag).unwrap();
        run(
            &mut game,
            &rules,
            Actor::Player(player),
            GameCommand::EndTurn,
        )
        .unwrap();
    }
    let attack = GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: RED_HERO,
        path: vec![Pos::new(1, 1, 0), Pos::new(2, 2, 0)],
    });
    run(&mut game, &rules, Actor::Player(RED), attack).unwrap();
    let battle = game.battle(RED).unwrap().state();
    let seen = |side| observe_battle(battle, rules.battle(), side);
    let level = |side| seen(side).own_hero.map_or(0, |hero| hero.tactics);
    (
        seen(BattleSide::Attacker).tactics.map(|phase| phase.side),
        [level(BattleSide::Attacker), level(BattleSide::Defender)],
    )
}

// covers: GAM-BAT-009, BAT-TAC-001
#[test]
fn a_flag_off_denies_the_tactics_phase_its_side_would_get() {
    let attacker = Some(BattleSide::Attacker);
    let defender = Some(BattleSide::Defender);
    // Both flags on: the greater level's side places its army.
    assert_eq!(tactics_battle((2, 1), (true, true)), (attacker, [2, 1]));
    assert_eq!(tactics_battle((1, 3), (true, true)), (defender, [1, 3]));
    // That side's flag off: both levels go as 0, no phase.
    assert_eq!(tactics_battle((2, 1), (false, true)), (None, [0, 0]));
    assert_eq!(tactics_battle((1, 3), (true, false)), (None, [0, 0]));
    // The other side's flag does not matter, nor a flag where the levels cancel.
    assert_eq!(tactics_battle((2, 1), (true, false)), (attacker, [2, 1]));
    assert_eq!(tactics_battle((2, 2), (false, false)), (None, [2, 2]));
}
