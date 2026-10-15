//! Tests of the wandering monsters' offers through the game (E08 T20b1; ADR-0017 points 6 and
//! 10): a monster's answer opens as its visitor's owner's decision, a declined offer to join
//! goes on to the next in the same command, a price to join the treasury cannot pay counts as
//! declined, an accepted join pays and gains the treasure, and a guard's visit waits for the
//! answer to the offer of the tile's own object.

use h3_adventure::{
    AdventureEvent, AdventureEventKind, MonsterSetup, ObjectSetup, ObjectTypeId, Offer, OfferKind,
    TileMask, Viewers,
};

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
        anchor,
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.adventure().object_type_by_key(key),
        amount: None,
        drawn: TileMask::default(),
        monster,
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
