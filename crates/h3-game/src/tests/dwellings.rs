//! Tests of the dwellings' visits through the game (E08 T95a1; increment 10's design note §7,
//! §13): a neutral dwelling's visit flags it and opens the hire question as its visitor's owner's
//! decision, hashed with its creatures; accepted, its free creatures join the hero's army, told to
//! the owner alone; declined, nothing more. E08 T95a2 (§7 step 2, §12): a guarded one's guards'
//! question, hashed with its creature and word; accepted, the guards' battle the system plays,
//! whoever owns the dwelling; won, the flag and the hire question after the levels; lost, the
//! hero off the map and the survivors kept as its guards.

use h3_adventure::{
    AdventureEvent, AdventureEventKind, DwellingParams, Holds, ObjectSetup, ObjectTypeId,
    OfferKind, TileMask, Viewers,
};
use h3_contracts::BattleSide;

use super::decisions::{ACCEPT, DECLINE, after_steps, move_hero, play};
use super::manage::imp;
use super::mines::{adventure, fight};
use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const RED_HERO: HeroId = HeroId::new(0);
const GOG: CreatureTypeId = CreatureTypeId::from_raw(1);
/// The dwelling's anchor and entrance, one step right of red's hero.
const ANCHOR: Pos = Pos::new(1, 0, 0);

/// The rules of [`rules`](super::rules) with imps of level 1 growing 3 a week, gogs of level 2
/// growing 2, a dwelling `t:hut` of both (entered, one tile) and the original's dwellings'
/// parameters (the guards from level 5, the free creatures up to level 1).
fn rules() -> GameRuleset {
    rules_with(5)
}

/// [`rules`] with the guards from `guard_level`: from level 2, the hut's 6 gogs in slot 0.
fn rules_with(guard_level: u8) -> GameRuleset {
    let mut pack = core_pack(false);
    pack.creatures[0].info.growth = 3;
    pack.creatures[1].info.level = 2;
    pack.creatures[1].info.growth = 2;
    let core = Arc::new(Ruleset::compile(&[pack], "t:faithful").unwrap());
    let mut part = adventure_pack(vec![88, 25], 2);
    part.object_types.push(ObjectTypeDef {
        from_above: false,
        ..entered(
            "t:hut",
            ObjectKindDef::Dwelling {
                creatures: vec![key("t:imp"), key("t:gog")],
                blocked: vec![(0, 0)],
                visitable: vec![(0, 0)],
            },
        )
    });
    part.params.as_mut().unwrap().dwellings = Some(DwellingParams {
        guard_level,
        guard_factor: NonZeroU16::new(3).unwrap(),
        free_level: 1,
    });
    let part = AdventureRuleset::compile(&core, &[part]).unwrap();
    let battle = battle_part(&core, 7);
    GameRuleset::new(
        Arc::clone(&core),
        Arc::new(part),
        Arc::new(battle),
        no_towns(&core),
    )
    .unwrap()
}

fn hut_type(rules: &GameRuleset) -> ObjectTypeId {
    rules.adventure().object_type_by_key("t:hut").unwrap()
}

/// [`setup`] with a neutral `t:hut` at [`ANCHOR`]: red's hero (9 imps in slot 1, 2 army slots) at
/// (0, 0), blue's at (2, 2).
fn game(rules: &GameRuleset) -> GameState {
    game_with(rules, None, 9)
}

/// [`game`] with the hut `owner`'s or neutral, red's hero with `imps` imps in slot 1.
fn game_with(rules: &GameRuleset, owner: Option<PlayerId>, imps: u32) -> GameState {
    let mut setup = setup();
    setup.heroes[0].army = HeroArmy::Custom(vec![imp(1, imps)]);
    setup.map.objects = vec![ObjectSetup {
        scenery: Default::default(),
        anchor: ANCHOR,
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: Some(hut_type(rules)),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds: Holds::Nothing,
        owner,
    }];
    GameState::new(rules, &setup, 7).unwrap()
}

// covers: ADV-DWL-004, ADV-HIR-001, GAM-DEC-001
#[test]
fn a_dwelling_s_hire_question_opens_and_accepted_gives_its_free_creatures() {
    let rules = rules();
    let ty = hut_type(&rules);
    let mut state = game(&rules);
    // Flagged for red, then the hire question naming both creatures, without a price: it opens.
    let events = after_steps(play(&mut state, &rules, RED, move_hero(0, &[ANCHOR])).unwrap());
    assert!(matches!(
        events[0],
        GameEvent::Adventure(AdventureEvent {
            kind: AdventureEventKind::ObjectFlagged { owner: RED, .. },
            ..
        })
    ));
    assert_eq!(
        events.last(),
        Some(&GameEvent::OfferOpened {
            player: RED,
            hero: RED_HERO,
            object_type: ty,
            anchor: ANCHOR,
        })
    );
    assert_eq!(
        state.decision(RED).map(|offer| offer.kind.clone()),
        Some(OfferKind::Hire {
            creatures: vec![IMP, GOG],
        })
    );
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(RED)),
        [ACCEPT, DECLINE]
    );
    // The open question is hashed with its creatures: declined, the state hashes otherwise, and
    // nothing more happens.
    let mut declined = state.clone();
    let events = play(&mut declined, &rules, RED, DECLINE).unwrap();
    assert_eq!(
        events,
        [GameEvent::OfferAnswered {
            player: RED,
            object_type: ty,
            anchor: ANCHOR,
            accepted: false,
        }]
    );
    assert_ne!(state.state_hash(), declined.state_hash());
    // Accepted: the 3 imps (level 1) join red's stack of them, told to red alone; the gogs
    // (level 2) are the hire dialog's (T95b).
    let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
    assert_eq!(
        events,
        [
            GameEvent::OfferAnswered {
                player: RED,
                object_type: ty,
                anchor: ANCHOR,
                accepted: true,
            },
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::CreaturesJoined {
                    hero: RED_HERO,
                    creature: IMP,
                    count: 3,
                },
                seen_by: Viewers::NONE.with(RED),
            }),
        ]
    );
    let army = &state.adventure().hero(RED_HERO).unwrap().army;
    assert_eq!(
        army[..],
        [ArmyStack {
            slot: 1,
            creature: IMP,
            count: 12,
        }]
    );
    assert_eq!(state.adventure().dwelling_counts(0), Some(&[0, 2][..]));
    assert_eq!(state.adventure().owner(0), Some(RED));
    assert_eq!(state.decision(BLUE), None);
    assert_ne!(state.state_hash(), declined.state_hash());
}

/// The guards' question of the hut at [`ANCHOR`] to red's hero: its 6 gogs.
fn guarded(rules: &GameRuleset) -> OfferKind {
    let words = rules.adventure().params().monsters().unwrap();
    OfferKind::Guarded {
        creature: GOG,
        count_word: words.count_word(6),
    }
}

// covers: ADV-DWL-004, GAM-BAT-001, GAM-DEC-007, ADV-OWN-001
#[test]
fn another_s_guarded_dwelling_is_fought_by_the_system_and_a_win_flags_it_then_asks() {
    let rules = rules_with(2);
    let ty = hut_type(&rules);
    let mut state = game_with(&rules, Some(BLUE), 300);
    let gogs = vec![ArmyStack {
        slot: 0,
        creature: GOG,
        count: 6,
    }];
    assert_eq!(state.adventure().dwelling_guards(0), gogs);
    // Blue's hut, its guards standing: the guards' question, nothing flagged.
    let events = after_steps(play(&mut state, &rules, RED, move_hero(0, &[ANCHOR])).unwrap());
    assert_eq!(
        events,
        [GameEvent::OfferOpened {
            player: RED,
            hero: RED_HERO,
            object_type: ty,
            anchor: ANCHOR,
        }]
    );
    assert_eq!(
        state.decision(RED).map(|offer| offer.kind.clone()),
        Some(guarded(&rules))
    );
    // The open question is hashed (its tag, creature and word): declined, the state hashes
    // otherwise; nothing more happens, the hero stays on the entrance.
    let mut declined = state.clone();
    play(&mut declined, &rules, RED, DECLINE).unwrap();
    assert_ne!(state.state_hash(), declined.state_hash());
    assert_eq!(declined.adventure().owner(0), Some(BLUE));
    assert_eq!(declined.adventure().hero(RED_HERO).unwrap().pos, ANCHOR);
    assert_eq!(declined.adventure().dwelling_guards(0), gogs);
    // Accepted: the battle of the unowned guards, unsplit, the system plays (not blue).
    let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::BattleStarted { seen_by, .. } if *seen_by == Viewers::NONE.with(RED)
    )));
    let battle = state.battle(RED).unwrap();
    assert_eq!(battle.owners(), [Some(RED), None]);
    assert_eq!(battle.heroes(), [Some(RED_HERO), None]);
    assert_eq!(battle.tile(), ANCHOR);
    assert!(state.battle(BLUE).is_none());
    let defending: Vec<(u8, CreatureTypeId, u32)> = battle
        .state()
        .stacks()
        .iter()
        .filter(|s| s.side == BattleSide::Defender)
        .map(|s| (s.slot, s.creature, s.count))
        .collect();
    assert_eq!(defending, [(0, GOG, 6)]);
    // Won by red's 300 imps: the guards gone; after the levels, the flag (told to both) and the
    // hire question.
    let mut events = fight(&mut state, &rules);
    while state.decision(RED).is_none() {
        let answer = default_answer(&state, &rules, RED).unwrap();
        events.extend(play(&mut state, &rules, RED, GameCommand::Answer(answer)).unwrap());
    }
    let ended = events
        .iter()
        .position(|e| matches!(e, GameEvent::BattleEnded { .. }))
        .unwrap();
    let flagged = events
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
    assert!(ended < flagged);
    let told = adventure(&events);
    assert!(told.contains(&AdventureEvent {
        kind: AdventureEventKind::ObjectFlagged {
            object_type: ty,
            anchor: ANCHOR,
            owner: RED,
        },
        seen_by: Viewers::NONE.with(RED).with(BLUE),
    }));
    assert_eq!(
        events.last(),
        Some(&GameEvent::OfferOpened {
            player: RED,
            hero: RED_HERO,
            object_type: ty,
            anchor: ANCHOR,
        })
    );
    assert_eq!(
        state.decision(RED).map(|offer| offer.kind.clone()),
        Some(OfferKind::Hire {
            creatures: vec![IMP, GOG],
        })
    );
    let adventure = state.adventure();
    assert_eq!(adventure.owner(0), Some(RED));
    assert!(adventure.dwelling_guards(0).is_empty());
    assert_eq!(adventure.hero(RED_HERO).unwrap().pos, ANCHOR);
}

// covers: ADV-DWL-004, GAM-BAT-003
#[test]
fn a_lost_guards_battle_takes_the_hero_off_the_map_and_keeps_the_survivors() {
    let rules = rules_with(2);
    let mut state = game_with(&rules, None, 1);
    play(&mut state, &rules, RED, move_hero(0, &[ANCHOR])).unwrap();
    assert_eq!(
        state.decision(RED).map(|offer| offer.kind.clone()),
        Some(guarded(&rules))
    );
    play(&mut state, &rules, RED, ACCEPT).unwrap();
    let events = fight(&mut state, &rules);
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::Adventure(AdventureEvent {
            kind: AdventureEventKind::HeroRemoved { .. },
            ..
        })
    )));
    assert!(state.adventure().hero(RED_HERO).is_none());
    assert_eq!(state.decision(RED), None);
    // The survivors stay as its guards, in their slot; still neutral, nothing told of them.
    let guards = state.adventure().dwelling_guards(0);
    assert_eq!(guards.len(), 1);
    assert_eq!((guards[0].slot, guards[0].creature), (0, GOG));
    assert!((1..=6).contains(&guards[0].count), "{guards:?}");
    assert_eq!(state.adventure().owner(0), None);
    assert!(!adventure(&events).iter().any(|e| matches!(
        e.kind,
        AdventureEventKind::ObjectFlagged { .. } | AdventureEventKind::GuardsChanged { .. }
    )));
}
