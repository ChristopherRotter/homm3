//! Tests of the wandering monsters' offers through the game (E08 T20b1; ADR-0017 points 6 and
//! 10): a monster's answer opens as its visitor's owner's decision, a declined offer to join
//! goes on to the next in the same command, a price to join the treasury cannot pay counts as
//! declined, an accepted join pays and gains the treasure, and a guard's visit waits for the
//! answer to the offer of the tile's own object; a monster's battle (E08 T20b2): its split, the
//! system actor playing its side, its outcome; a guarded pile's question and battle (E08 T21a).

use h3_adventure::{
    AdventureEvent, AdventureEventKind, Guards, MonsterSetup, ObjectSetup, ObjectTypeId, Offer,
    OfferKind, TileMask, TileRand, Viewers, split,
};
use h3_contracts::BattleSide;

use super::decisions::{ACCEPT, DECLINE, GOLD, RED, after_steps, move_hero, play};
use super::*;

const GOG: CreatureTypeId = CreatureTypeId::from_raw(1);

/// An object of type `key` at `anchor`, blocking and visited there.
fn object(
    rules: &GameRuleset,
    key: &str,
    anchor: Pos,
    monster: Option<MonsterSetup>,
) -> ObjectSetup {
    ObjectSetup {
        scenery: Default::default(),
        anchor,
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.adventure().object_type_by_key(key),
        amount: None,
        drawn: TileMask::default(),
        monster,
        guards: None,
        holds: Default::default(),
    }
}

/// `count` gogs of `disposition` (5: aggression 3) with a treasure of 100 gold.
fn gogs(count: u32, disposition: u8) -> Option<MonsterSetup> {
    Some(MonsterSetup {
        creature: Some(GOG),
        count,
        disposition,
        never_flees: false,
        no_growth: false,
        treasure: vec![(GOLD, 100)],
        artifact: None,
    })
}

/// [`setup`] at `difficulty` (0: 3 000 gold, Easy; 1: 2 000) with these objects: red's hero 0
/// at (0, 0) with 9 imps in slot 1 of two.
fn game(rules: &GameRuleset, difficulty: u8, objects: Vec<ObjectSetup>) -> GameState {
    let mut setup = setup();
    setup.difficulty = difficulty;
    setup.map.objects = objects;
    GameState::new(rules, &setup, 7).unwrap()
}

fn monster_type(rules: &GameRuleset) -> ObjectTypeId {
    rules.adventure().object_type_by_key("t:monster").unwrap()
}

fn opened(rules: &GameRuleset, anchor: Pos) -> GameEvent {
    GameEvent::OfferOpened {
        player: RED,
        hero: HeroId::new(0),
        object_type: monster_type(rules),
        anchor,
    }
}

fn answered(rules: &GameRuleset, anchor: Pos, accepted: bool) -> GameEvent {
    GameEvent::OfferAnswered {
        player: RED,
        object_type: monster_type(rules),
        anchor,
        accepted,
    }
}

fn offer(rules: &GameRuleset, anchor: Pos, kind: OfferKind) -> Offer {
    Offer {
        hero: HeroId::new(0),
        object_type: monster_type(rules),
        anchor,
        kind,
    }
}

// covers: ADV-MON-007, ADV-MON-008, GAM-DEC-001
#[test]
fn a_monsters_offer_to_join_opens_and_its_acceptance_adds_the_group_and_the_treasure() {
    let rules = rules();
    let at = Pos::new(1, 1, 0);
    let mut state = game(&rules, 1, vec![object(&rules, "t:monster", at, gogs(4, 0))]);
    let events = play(&mut state, &rules, RED, move_hero(0, &[at])).unwrap();
    assert_eq!(after_steps(events), [opened(&rules, at)]);
    assert_eq!(
        state.decision(RED),
        Some(&offer(&rules, at, OfferKind::Join))
    );
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(RED)),
        [ACCEPT, DECLINE]
    );
    let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
    let both = Viewers::NONE.with(RED).with(PlayerId::new(1));
    assert_eq!(
        events,
        [
            answered(&rules, at, true),
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::CreaturesJoined {
                    hero: HeroId::new(0),
                    creature: GOG,
                    count: 4,
                },
                seen_by: Viewers::NONE.with(RED),
            }),
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::ObjectRemoved {
                    object_type: monster_type(&rules),
                    anchor: at,
                },
                seen_by: both,
            }),
            GameEvent::ResourceGained {
                player: RED,
                resource: GOLD,
                amount: 100,
            },
        ]
    );
    assert_eq!(state.treasury(RED).unwrap()[0], 2100);
    let army = &state.adventure().hero(HeroId::new(0)).unwrap().army;
    assert_eq!((army[0].slot, army[0].creature, army[0].count), (0, GOG, 4));
    assert_eq!(state.decision(RED), None);
}

// covers: ADV-MON-007, ADV-MON-008, GAM-DEC-001
#[test]
fn a_declined_join_opens_the_flight_in_the_same_command_and_a_declined_flight_lets_it_go() {
    let rules = rules();
    let at = Pos::new(1, 1, 0);
    let mut state = game(&rules, 1, vec![object(&rules, "t:monster", at, gogs(4, 0))]);
    play(&mut state, &rules, RED, move_hero(0, &[at])).unwrap();
    let events = play(&mut state, &rules, RED, DECLINE).unwrap();
    assert_eq!(events, [answered(&rules, at, false), opened(&rules, at)]);
    assert_eq!(
        state.decision(RED),
        Some(&offer(&rules, at, OfferKind::Pursue))
    );
    let events = play(&mut state, &rules, RED, DECLINE).unwrap();
    assert_eq!(events.len(), 2);
    assert!(matches!(
        events[1],
        GameEvent::Adventure(AdventureEvent {
            kind: AdventureEventKind::ObjectRemoved { .. },
            ..
        })
    ));
    assert_eq!(state.decision(RED), None);
    assert_eq!(state.treasury(RED).unwrap()[0], 2000);
    assert_eq!(
        state
            .adventure()
            .guard_of(rules.adventure(), Pos::new(0, 0, 0)),
        None
    );
}

// covers: ADV-MON-007, ADV-MON-008
#[test]
fn a_price_to_join_opens_when_the_treasury_can_pay_and_else_counts_as_declined() {
    let rules = rules();
    let at = Pos::new(1, 1, 0);
    // On Easy (L 1) aggression 3 asks a price: the gogs' 100 gold each.
    let mut state = game(&rules, 0, vec![object(&rules, "t:monster", at, gogs(4, 5))]);
    play(&mut state, &rules, RED, move_hero(0, &[at])).unwrap();
    let price = OfferKind::JoinForGold {
        price: vec![(GOLD, 400)],
    };
    assert_eq!(state.decision(RED), Some(&offer(&rules, at, price)));
    let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
    assert_eq!(
        events[1],
        GameEvent::ResourcePaid {
            player: RED,
            resource: GOLD,
            amount: 400,
        }
    );
    assert_eq!(state.treasury(RED).unwrap()[0], 3000 - 400 + 100);
    // 40 gogs cost 4 000: no offer, no refusal; the flight opens instead.
    let mut state = game(
        &rules,
        0,
        vec![object(&rules, "t:monster", at, gogs(40, 5))],
    );
    let events = play(&mut state, &rules, RED, move_hero(0, &[at])).unwrap();
    assert_eq!(after_steps(events), [opened(&rules, at)]);
    assert_eq!(
        state.decision(RED),
        Some(&offer(&rules, at, OfferKind::Pursue))
    );
}

// covers: ADV-MON-006, GAM-DEC-001
#[test]
fn a_guards_visit_waits_for_the_answer_to_the_tiles_own_offer() {
    let rules = rules();
    let chart = rules.adventure().object_type_by_key("t:chart").unwrap();
    let (tile, guard) = (Pos::new(1, 0, 0), Pos::new(2, 0, 0));
    let objects = |rules: &GameRuleset| {
        vec![
            object(rules, "t:chart", tile, None),
            object(rules, "t:monster", guard, gogs(4, 0)),
        ]
    };
    // On Easy the Cartographer's offer opens; the guard's comes once it is answered.
    let mut state = game(&rules, 0, objects(&rules));
    let events = play(&mut state, &rules, RED, move_hero(0, &[tile])).unwrap();
    let chart_opened = GameEvent::OfferOpened {
        player: RED,
        hero: HeroId::new(0),
        object_type: chart,
        anchor: tile,
    };
    assert_eq!(after_steps(events), [chart_opened]);
    let hash = state.state_hash();
    let events = play(&mut state, &rules, RED, DECLINE).unwrap();
    let chart_declined = GameEvent::OfferAnswered {
        player: RED,
        object_type: chart,
        anchor: tile,
        accepted: false,
    };
    assert_eq!(events, [chart_declined, opened(&rules, guard)]);
    assert_eq!(
        state.decision(RED),
        Some(&offer(&rules, guard, OfferKind::Join))
    );
    assert_ne!(state.state_hash(), hash);
    // On Normal it cannot open: refused, and the guard's visit follows at once.
    let mut state = game(&rules, 1, objects(&rules));
    let events = play(&mut state, &rules, RED, move_hero(0, &[tile])).unwrap();
    let refused = GameEvent::OfferRefused {
        player: RED,
        hero: HeroId::new(0),
        object_type: chart,
        anchor: tile,
    };
    assert_eq!(after_steps(events), [refused, opened(&rules, guard)]);
}

// covers: GAM-DEC-001
#[test]
fn the_hash_sees_each_kind_of_a_monsters_offer() {
    let rules = rules();
    let at = Pos::new(1, 1, 0);
    let open = |difficulty, count, disposition, answers: &[GameCommand]| {
        let monster = object(&rules, "t:monster", at, gogs(count, disposition));
        let mut state = game(&rules, difficulty, vec![monster]);
        play(&mut state, &rules, RED, move_hero(0, &[at])).unwrap();
        for answer in answers {
            play(&mut state, &rules, RED, answer.clone()).unwrap();
        }
        state.state_hash()
    };
    // Join, join for gold, the flight after a declined join: three hashes.
    let hashes = [
        open(0, 4, 0, &[]),
        open(0, 4, 5, &[]),
        open(0, 4, 0, &[DECLINE]),
    ];
    assert_ne!(hashes[0], hashes[1]);
    assert_ne!(hashes[0], hashes[2]);
    assert_ne!(hashes[1], hashes[2]);
}

/// The rules with AI values (the imp 50, the gog 100: E08 T20b2) and the profile `profile`
/// (`t:faithful`, or `t:split` with ADV-QUIRK-6).
pub(super) fn valued(profile: &str) -> GameRuleset {
    let mut pack = core_pack(false);
    pack.creatures[0].info.ai_value = 50;
    pack.creatures[1].info.ai_value = 100;
    let core = Arc::new(Ruleset::compile(&[pack], profile).unwrap());
    let part = part(&core);
    let battle = battle_part(&core, 7);
    GameRuleset::new(core, Arc::new(part), Arc::new(battle)).unwrap()
}

/// Plays red's battle to its end: the system actor for the monster's side, red for its own, each
/// the last legal command that stays in the battle; the events of the last command.
pub(super) fn fight_out(state: &mut GameState, rules: &GameRuleset) -> Vec<GameEvent> {
    for _ in 0..2000 {
        let actor = if state.unowned_battle().is_some() {
            Actor::System
        } else {
            Actor::Player(RED)
        };
        let listed = legal_commands(state, rules, actor);
        let fights = |c: &&GameCommand| !matches!(c, GameCommand::Battle(c) if c.leaves_battle());
        let command = listed.iter().rfind(fights).unwrap().clone();
        let mut events = Vec::new();
        apply(state, rules, actor, command, &mut events).unwrap();
        if state.battle(RED).is_none() {
            return events;
        }
    }
    panic!("the battle did not end");
}

// covers: ADV-MON-009, GAM-BAT-001, GAM-BAT-010, GAM-BAT-011, GAM-BAT-012
#[test]
fn a_monster_fights_split_by_its_tile_and_the_system_plays_its_side() {
    let rules = valued("t:split");
    let at = Pos::new(1, 1, 0);
    // A savage (aggression 10) group of 12 gogs against 9 imps: C = −1, a fight.
    let mut state = game(
        &rules,
        1,
        vec![object(&rules, "t:monster", at, gogs(12, 4))],
    );
    let events = play(&mut state, &rules, RED, move_hero(0, &[at])).unwrap();
    assert_eq!(
        after_steps(events)[0],
        GameEvent::BattleStarted {
            heroes: [Some(HeroId::new(0)), None],
            tile: at,
            seen_by: Viewers::NONE.with(RED),
        }
    );
    let battle = state.battle(RED).unwrap();
    assert_eq!(battle.owners(), [Some(RED), None]);
    assert_eq!(battle.heroes(), [Some(HeroId::new(0)), None]);
    // The monsters' side has no player: not human, no colour (GAM-BAT-010, GAM-BAT-012); nothing
    // stands around (GAM-BAT-011).
    let seen = h3_battle::observe(battle.state(), rules.battle(), BattleSide::Attacker);
    assert_eq!(seen.human, [true, false]);
    assert_eq!(seen.colours, [Some(0), None]);
    assert_eq!(seen.background, Some(2));
    assert_eq!(seen.difficulty, state.adventure().difficulty());
    // Under ADV-QUIRK-6 its stacks are the split with the tile's draws.
    let army = state.adventure().hero(HeroId::new(0)).unwrap().army.clone();
    let mut tile = TileRand::new(at);
    let expected: Vec<(u8, CreatureTypeId, u32)> =
        split(rules.core(), &army, (GOG, 12), || tile.percent())
            .iter()
            .map(|s| (s.slot, s.creature, s.count))
            .collect();
    let defending: Vec<(u8, CreatureTypeId, u32)> = battle
        .state()
        .stacks()
        .iter()
        .filter(|s| s.side == BattleSide::Defender)
        .map(|s| (s.slot, s.creature, s.count))
        .collect();
    assert_eq!(defending, expected);
    assert!(expected.len() > 1);
    // The imps act first; the system actor has nothing to command, then the gogs' side.
    assert_eq!(legal_commands(&state, &rules, Actor::System), []);
    let mut probe = state.clone();
    assert_eq!(
        apply(
            &mut probe,
            &rules,
            Actor::System,
            GameCommand::EndTurn,
            &mut NoopSink
        ),
        Err(GameCommandError::NotForSystem)
    );
    assert_eq!(
        legal_moves(&state, &rules, Actor::System, HeroId::new(0)),
        None
    );
    let wait = legal_commands(&state, &rules, Actor::Player(RED))
        .into_iter()
        .rfind(|c| !matches!(c, GameCommand::Battle(c) if c.leaves_battle()))
        .unwrap();
    assert_eq!(
        apply(
            &mut probe,
            &rules,
            Actor::System,
            wait.clone(),
            &mut NoopSink
        ),
        Err(GameCommandError::NoBattle)
    );
    apply(&mut state, &rules, Actor::Player(RED), wait, &mut NoopSink).unwrap();
    if state.unowned_battle().is_some() {
        let listed = legal_commands(&state, &rules, Actor::System);
        assert!(!listed.is_empty());
        let mut probe = state.clone();
        assert_eq!(
            apply(
                &mut probe,
                &rules,
                Actor::Player(RED),
                listed[0].clone(),
                &mut NoopSink
            ),
            Err(GameCommandError::NotYourSide)
        );
    }
}

// covers: ADV-MON-009, ADV-MON-010, GAM-BAT-002
#[test]
fn a_won_battle_removes_the_monster_and_gives_its_treasure() {
    // Without ADV-QUIRK-6 the split draws from red's stream.
    let rules = valued("t:faithful");
    let at = Pos::new(1, 1, 0);
    let mut state = game(&rules, 1, vec![object(&rules, "t:monster", at, gogs(1, 4))]);
    play(&mut state, &rules, RED, move_hero(0, &[at])).unwrap();
    let defending = state
        .battle(RED)
        .unwrap()
        .state()
        .stacks()
        .iter()
        .filter(|s| s.side == BattleSide::Defender)
        .count();
    assert_eq!(defending, 1);
    let events = fight_out(&mut state, &rules);
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::Adventure(AdventureEvent {
            kind: AdventureEventKind::ObjectRemoved { anchor, .. },
            ..
        }) if *anchor == at
    )));
    assert_eq!(
        events.last(),
        Some(&GameEvent::ResourceGained {
            player: RED,
            resource: GOLD,
            amount: 100,
        })
    );
    assert_eq!(state.treasury(RED).unwrap()[0], 2100);
    assert!(state.adventure().hero(HeroId::new(0)).is_some());
    assert!(state.adventure().is_removed(0));
}

// covers: ADV-MON-010, GAM-BAT-003
#[test]
fn a_lost_battle_takes_the_hero_off_the_map_and_leaves_the_survivors() {
    let rules = valued("t:split");
    let at = Pos::new(1, 1, 0);
    let mut state = game(
        &rules,
        1,
        vec![object(&rules, "t:monster", at, gogs(40, 4))],
    );
    play(&mut state, &rules, RED, move_hero(0, &[at])).unwrap();
    let events = fight_out(&mut state, &rules);
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::Adventure(AdventureEvent {
            kind: AdventureEventKind::HeroRemoved { .. },
            ..
        })
    )));
    assert!(state.adventure().hero(HeroId::new(0)).is_none());
    assert!(!state.adventure().is_removed(0));
    let left = state.adventure().monster_count(0).unwrap();
    assert!((1..=40).contains(&left), "{left}");
    assert_eq!(state.treasury(RED).unwrap()[0], 2000);
}

/// The gold pile `t:pile` of 5 hundreds at `anchor`, guarded by these stacks, asking first when
/// `question` (E08 T21a).
fn guarded_pile(
    rules: &GameRuleset,
    anchor: Pos,
    army: Vec<ArmyStack>,
    question: bool,
) -> ObjectSetup {
    ObjectSetup {
        scenery: Default::default(),
        amount: Some(5),
        guards: Some(Guards { army, question }),
        ..object(rules, "t:pile", anchor, None)
    }
}

/// `count` gogs in slot `slot`.
fn gogs_in(slot: u8, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature: GOG,
        count,
    }
}

// covers: ADV-PIK-004, ADV-FOG-006
#[test]
fn a_guarded_pile_looks_like_any_pile_to_every_viewer() {
    // Its guards and whether it asks first are hidden (`docs/06-security.md`, E08 T21b's audit):
    // each player sees the same game as with the pile unguarded, the pile in sight.
    let rules = valued("t:split");
    let at = Pos::new(1, 1, 0);
    let plain = ObjectSetup {
        scenery: Default::default(),
        amount: Some(5),
        ..object(&rules, "t:pile", at, None)
    };
    let guarded = game(
        &rules,
        1,
        vec![guarded_pile(&rules, at, vec![gogs_in(1, 9)], true)],
    );
    let plain = game(&rules, 1, vec![plain]);
    for viewer in [RED, PlayerId::new(1)].map(Viewer::Player) {
        let seen = observe(&guarded, &rules, viewer);
        assert_eq!(seen, observe(&plain, &rules, viewer));
        assert!(seen.adventure.objects.iter().any(|o| o.anchor == at));
    }
}

// covers: ADV-PIK-004, GAM-DEC-001, GAM-BAT-001
#[test]
fn a_guarded_piles_question_opens_and_its_acceptance_fights_the_unsplit_guards() {
    let rules = valued("t:split");
    let at = Pos::new(1, 1, 0);
    let pile = guarded_pile(&rules, at, vec![gogs_in(1, 1)], true);
    let pile_type = pile.object_type.unwrap();
    let mut state = game(&rules, 1, vec![pile]);
    let events = play(&mut state, &rules, RED, move_hero(0, &[at])).unwrap();
    let opened = GameEvent::OfferOpened {
        player: RED,
        hero: HeroId::new(0),
        object_type: pile_type,
        anchor: at,
    };
    assert_eq!(after_steps(events), [opened]);
    let question = Offer {
        hero: HeroId::new(0),
        object_type: pile_type,
        anchor: at,
        kind: OfferKind::Fight,
    };
    assert_eq!(state.decision(RED), Some(&question));
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(RED)),
        [ACCEPT, DECLINE]
    );
    // The hash sees the offer's kind.
    let asked = state.state_hash();
    // Declined: nothing else happens.
    let events = play(&mut state, &rules, RED, DECLINE).unwrap();
    assert_eq!(
        events,
        [GameEvent::OfferAnswered {
            player: RED,
            object_type: pile_type,
            anchor: at,
            accepted: false,
        }]
    );
    assert_ne!(state.state_hash(), asked);
    assert!(!state.adventure().is_removed(0));
    assert_eq!(state.treasury(RED).unwrap()[0], 2000);
    // Asked again and accepted: the battle against the guards as the map gives them, unsplit
    // (the split's profile does not touch them), owned by red alone.
    play(&mut state, &rules, RED, move_hero(0, &[at])).unwrap();
    let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::BattleStarted { tile, .. } if *tile == at))
    );
    let battle = state.battle(RED).unwrap();
    assert_eq!(battle.owners(), [Some(RED), None]);
    assert_eq!(battle.heroes(), [Some(HeroId::new(0)), None]);
    let defending: Vec<(u8, CreatureTypeId, u32)> = battle
        .state()
        .stacks()
        .iter()
        .filter(|s| s.side == BattleSide::Defender)
        .map(|s| (s.slot, s.creature, s.count))
        .collect();
    assert_eq!(defending, [(1, GOG, 1)]);
    // Won: the pile is picked up, its gold in red's treasury.
    let events = fight_out(&mut state, &rules);
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::Adventure(AdventureEvent {
            kind: AdventureEventKind::ObjectRemoved { anchor, .. },
            ..
        }) if *anchor == at
    )));
    assert_eq!(
        events.last(),
        Some(&GameEvent::ResourceGained {
            player: RED,
            resource: GOLD,
            amount: 500,
        })
    );
    assert_eq!(state.treasury(RED).unwrap()[0], 2500);
    assert!(state.adventure().hero(HeroId::new(0)).is_some());
    assert!(state.adventure().is_removed(0));
}

// covers: ADV-PIK-004, GAM-BAT-003
#[test]
fn a_guarded_pile_without_a_question_fights_at_once_and_keeps_its_guards_survivors() {
    let rules = valued("t:faithful");
    let at = Pos::new(1, 1, 0);
    let mut state = game(
        &rules,
        1,
        vec![guarded_pile(&rules, at, vec![gogs_in(0, 40)], false)],
    );
    let events = play(&mut state, &rules, RED, move_hero(0, &[at])).unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::BattleStarted { .. }))
    );
    let events = fight_out(&mut state, &rules);
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::Adventure(AdventureEvent {
            kind: AdventureEventKind::HeroRemoved { .. },
            ..
        })
    )));
    assert!(state.adventure().hero(HeroId::new(0)).is_none());
    assert!(!state.adventure().is_removed(0));
    let left = state.adventure().guards_left(0);
    assert!(
        matches!(left, [ArmyStack { slot: 0, creature, count }]
            if *creature == GOG && (1..=40).contains(count)),
        "{left:?}"
    );
    assert_eq!(state.treasury(RED).unwrap()[0], 2000);
}
