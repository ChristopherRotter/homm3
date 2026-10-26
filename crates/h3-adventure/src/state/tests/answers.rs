//! Tests of the wandering monsters' visits (E08 T20b1): the monster's answer as an offer or a
//! fight (ADV-MON-007), the offers' answers (ADV-MON-008: joining, the flight) and the guard's
//! visit after a step onto a guarded tile (ADV-MON-006).

use h3_content::{
    ContentPack, CreatureDef, CreatureInfoDef, HeroEffectDef, HeroTypeId, SecondarySkillDef,
    SkillId,
};

use std::format;

use super::*;
use crate::command::{AdventureCommand, AdventureEvent, NoRoom};
use crate::setup::MonsterSetup;
use crate::testing::{
    CHART, GNOLL, MAX_STACK, MONSTER, PEASANT, PIKEMAN, TOWER, WOLF, hero_at, key,
    movement_rules_edited, world,
};

const RED: PlayerId = PlayerId::new(0);
const HERO: HeroId = HeroId::new(1);
const GOLD: ResourceId = ResourceId::from_raw(1);
const DIPLOMACY: SkillId = SkillId::from_raw(0);

type Rules = (Ruleset, AdventureRuleset);

/// [`crate::testing::movement_rules`] (faithful, seven slots) with AI values and costs: the
/// pikeman 100 for 20 gold, the gnoll 50 for 3 wood, the peasant 10 for 5 gold, the wolf 0;
/// five more creatures `t:c0` to `t:c4` to fill an army; Diplomacy's join bonus 1, 2, 3.
fn rules() -> Rules {
    movement_rules_edited(true, 16, |core: &mut ContentPack| {
        let set = |c: &mut CreatureDef, ai_value, cost: Vec<(&str, u32)>| {
            c.info.ai_value = ai_value;
            c.info.cost = cost.into_iter().map(|(k, a)| (key(k), a)).collect();
        };
        set(&mut core.creatures[0], 100, vec![("t:gold", 20)]);
        set(&mut core.creatures[1], 50, vec![("t:wood", 3)]);
        set(&mut core.creatures[2], 10, vec![("t:gold", 5)]);
        for i in 0..5 {
            core.creatures.push(CreatureDef {
                key: key(&format!("t:c{i}")),
                speed: 5,
                info: CreatureInfoDef {
                    ai_value: 1,
                    ..CreatureInfoDef::default()
                },
            });
        }
        core.skills.push(SecondarySkillDef {
            key: key("t:diplomacy"),
            name: "Diplomacy".into(),
            levels: [1, 2, 3].map(|level| vec![HeroEffectDef::JoinBonus { level }]),
        });
    })
}

/// A monster of `count` `creature` at `(x, y)`, blocking and visited there, with a treasure of
/// 500 gold.
fn monster_at(x: u16, y: u16, creature: CreatureTypeId, count: u32) -> ObjectSetup {
    ObjectSetup {
        scenery: Default::default(),
        anchor: Pos::new(x, y, 0),
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: Some(MONSTER),
        amount: None,
        drawn: TileMask::default(),
        monster: Some(MonsterSetup {
            creature: Some(creature),
            count,
            disposition: 2,
            never_flees: false,
            no_growth: false,
            treasure: vec![(GOLD, 500)],
        }),
        guards: None,
    }
}

/// A state of `rows` with these objects after the rows' own, red's hero 1 at `(x, y)` with
/// `army` (by slot from 0), on difficulty `difficulty`.
fn state(
    rules: &Rules,
    rows: &[&str],
    objects: Vec<ObjectSetup>,
    (x, y): (u16, u16),
    army: &[(CreatureTypeId, u32)],
    difficulty: u8,
) -> AdventureState {
    let mut setup = world(rows, 1, vec![hero_at(1, 0, Pos::new(x, y, 0), &[PEASANT])]);
    setup.heroes[0].army = army
        .iter()
        .zip(0..)
        .map(|(&(creature, count), slot)| ArmyStack {
            slot,
            creature,
            count,
        })
        .collect();
    setup.map.objects.extend(objects);
    setup.difficulty = difficulty;
    crate::testing::started(rules, &setup).unwrap()
}

/// Sets the aggression of the monster at object index `index` (drawn by its disposition when
/// the game was built).
fn set_aggression(state: &mut AdventureState, index: usize, aggression: i8) {
    let map = Arc::make_mut(&mut state.map);
    map.objects[index].monster.as_mut().unwrap().aggression = aggression;
}

/// Sets whether the monster at object index `index` never flees.
fn set_never_flees(state: &mut AdventureState, index: usize) {
    let map = Arc::make_mut(&mut state.map);
    map.objects[index].monster.as_mut().unwrap().never_flees = true;
}

/// Details with Diplomacy at `level` (none for 0).
fn diplomat(level: u8) -> Arc<HeroDetails> {
    Arc::new(HeroDetails {
        hero_type: HeroTypeId::from_raw(0),
        level: 1,
        experience: 0,
        offered: Default::default(),
        primary: [0, 0, 1, 1],
        skills: (level > 0)
            .then_some((DIPLOMACY, level))
            .into_iter()
            .collect(),
        worn: vec![],
        backpack: vec![],
        spells: vec![],
        mana: 0,
    })
}

/// The store of [`HERO`] with these details (the store is by id: hero 0's details unused).
fn store(details: Arc<HeroDetails>) -> Vec<Arc<HeroDetails>> {
    vec![diplomat(0), details]
}

/// Moves [`HERO`] along `path` with the store `heroes`: the outcome and the events.
fn walk(
    rules: &Rules,
    state: &mut AdventureState,
    heroes: &[Arc<HeroDetails>],
    path: &[(u16, u16)],
) -> (AdventureOutcome, Vec<AdventureEvent>) {
    let mut events = Vec::new();
    let step = AdventureCommand::MoveHero {
        hero: HERO,
        path: path.iter().map(|&(x, y)| Pos::new(x, y, 0)).collect(),
    };
    let outcome = state
        .apply(&rules.0, &rules.1, heroes, RED, &step, &mut events)
        .unwrap();
    (outcome, events)
}

/// The offer of the monster at `(x, y)`, of this kind.
fn offer(x: u16, y: u16, kind: OfferKind) -> Option<Offer> {
    Some(Offer {
        hero: HERO,
        object_type: MONSTER,
        anchor: Pos::new(x, y, 0),
        kind,
    })
}

const ROWS: [&str; 3] = ["gggggg", "gggggg", "gggggg"];

/// Ten pikemen at (2, 0), the only object (index 0), with this aggression, and the hero at (1, 1)
/// with `army`, on Normal (difficulty 1): a strength of 1 000 against the army's value.
fn pikemen(rules: &Rules, army: &[(CreatureTypeId, u32)], aggression: i8) -> AdventureState {
    let mut s = state(
        rules,
        &ROWS,
        vec![monster_at(2, 0, PIKEMAN, 10)],
        (1, 1),
        army,
        1,
    );
    set_aggression(&mut s, 0, aggression);
    s
}

// covers: ADV-MON-007
#[test]
fn a_step_toward_a_monster_visits_it_from_the_tile_before() {
    let rules = rules();
    // 100 peasants (power 1 000): C = 0; the monster's M = −4 offers to join.
    let mut s = pikemen(&rules, &[(PEASANT, 100)], -4);
    let before = s.hero(HERO).unwrap().movement;
    let reach = s.reach(&rules.0, &rules.1, &[], HERO).unwrap();
    assert!(reach.points_left(Pos::new(2, 0, 0)).is_some());
    let (outcome, events) = walk(&rules, &mut s, &[], &[(2, 0)]);
    assert_eq!(outcome.offer, offer(2, 0, OfferKind::Join));
    assert!(!outcome.guard_after);
    // The hero stays, pays the step and moves nowhere: no step event.
    let hero = s.hero(HERO).unwrap();
    assert_eq!(hero.pos, Pos::new(1, 1, 0));
    assert!(hero.movement < before);
    assert!(events.is_empty());
}

// covers: ADV-MON-007
#[test]
fn the_answer_follows_the_aggression_against_the_charisma() {
    let rules = rules();
    let visit = |s: &mut AdventureState, heroes: &[Arc<HeroDetails>]| {
        walk(&rules, s, heroes, &[(2, 0)]).0.offer.map(|o| o.kind)
    };
    // C = 0 (St 0, Sy 0, L 0): M = 1 fights (a battle, no offer), M = 0 = C offers to join.
    assert_eq!(visit(&mut pikemen(&rules, &[(PEASANT, 100)], 1), &[]), None);
    assert_eq!(
        visit(&mut pikemen(&rules, &[(PEASANT, 100)], 0), &[]),
        Some(OfferKind::Join)
    );
    // Power 3 000: St 4, so C = 4 with Sy 0; M = 1 ≤ Sy + L + 1 joins; M = 2 above Sy + 2L + 1
    // = 1 and below C flees, or fights when it never flees; M = 4 = C fights.
    assert_eq!(
        visit(&mut pikemen(&rules, &[(PEASANT, 300)], 1), &[]),
        Some(OfferKind::Join)
    );
    assert_eq!(
        visit(&mut pikemen(&rules, &[(PEASANT, 300)], 2), &[]),
        Some(OfferKind::Pursue)
    );
    assert_eq!(visit(&mut pikemen(&rules, &[(PEASANT, 300)], 4), &[]), None);
    let mut s = pikemen(&rules, &[(PEASANT, 300)], 2);
    set_never_flees(&mut s, 0);
    assert_eq!(visit(&mut s, &[]), None);
    // Power 2 000 (St 2) with Basic Diplomacy (L 1): C = 3, and M = 3 ≤ Sy + 2L + 1 joins for its
    // price, 20 gold × 10.
    let basic = store(diplomat(1));
    let mut s = pikemen(&rules, &[(PEASANT, 200)], 3);
    assert_eq!(
        visit(&mut s, &basic),
        Some(OfferKind::JoinForGold {
            price: vec![(GOLD, 200)]
        })
    );
    // 20 pikemen, the monster's own type: St 2, Sy 2, so C = 4 and M = 3 joins for free.
    let mut s = pikemen(&rules, &[(PIKEMAN, 20)], 3);
    assert_eq!(visit(&mut s, &[]), Some(OfferKind::Join));
}

// covers: ADV-MON-007
#[test]
fn easy_raises_diplomacy_and_a_creature_without_the_price_resource_joins_for_nothing() {
    let rules = rules();
    // 20 gnolls (AI 50, priced in wood: a strength of 1 000) against 200 peasants (2 000) on
    // Easy: L 1 without Diplomacy.
    let mut s = state(
        &rules,
        &ROWS,
        vec![monster_at(2, 0, GNOLL, 20)],
        (1, 1),
        &[(PEASANT, 200)],
        0,
    );
    set_aggression(&mut s, 0, 3);
    let (outcome, _) = walk(&rules, &mut s, &[], &[(2, 0)]);
    // C = 2 + 0 + 1 = 3, M = 3 = Sy + 2L + 1: the price is the gnolls' gold cost, none.
    assert_eq!(
        outcome.offer,
        offer(2, 0, OfferKind::JoinForGold { price: vec![] })
    );
}

// covers: ADV-MON-007, ADV-MON-008, ADV-ARM-007
#[test]
fn a_join_is_offered_whatever_room_the_army_has() {
    let rules = rules();
    let c = |i: u16| CreatureTypeId::from_raw(4 + i);
    // Seven stacks, none of pikemen, or one that cannot take the group's 10 within a stack's
    // most (E08 decision 115): no room, the offer to join is made all the same (E08 T51b: the
    // army dialog follows its acceptance, GAM-DEC-004); with a free slot or room in the pikemen
    // too.
    let mut full = [
        (PEASANT, 100),
        (GNOLL, 1),
        (WOLF, 1),
        (c(0), 1),
        (c(1), 1),
        (c(2), 1),
        (c(3), 1),
    ];
    for last in [(c(3), 1), (PIKEMAN, 1), (PIKEMAN, MAX_STACK - 9)] {
        full[6] = last;
        for army in [&full[..], &full[1..]] {
            let mut s = pikemen(&rules, army, -4);
            let (outcome, _) = walk(&rules, &mut s, &[], &[(2, 0)]);
            assert_eq!(outcome.offer, offer(2, 0, OfferKind::Join));
        }
    }
}

/// The rules' tuple for `decline_offer` and `visit_guard`.
fn triple<'a>(
    rules: &'a Rules,
    heroes: &'a [Arc<HeroDetails>],
) -> (&'a Ruleset, &'a AdventureRuleset, &'a [Arc<HeroDetails>]) {
    (&rules.0, &rules.1, heroes)
}

// covers: ADV-MON-007, ADV-MON-008
#[test]
fn a_declined_join_goes_on_to_the_flight_or_a_fight() {
    let rules = rules();
    let mut events: Vec<AdventureEvent> = Vec::new();
    // M = −4, C = 0: declined, the monster offers to flee.
    let mut s = pikemen(&rules, &[(PEASANT, 100)], -4);
    let join = offer(2, 0, OfferKind::Join).unwrap();
    let next = s.decline_offer(triple(&rules, &[]), &join, &mut events);
    assert_eq!(next.offer, offer(2, 0, OfferKind::Pursue));
    // M = C = 0: insulted, it fights; so does one that never flees.
    let mut s = pikemen(&rules, &[(PEASANT, 100)], 0);
    let next = s.decline_offer(triple(&rules, &[]), &join, &mut events);
    assert_eq!((next.offer, next.battle.is_some()), (None, true));
    let mut s = pikemen(&rules, &[(PEASANT, 100)], -4);
    set_never_flees(&mut s, 0);
    let gold = offer(2, 0, OfferKind::JoinForGold { price: vec![] }).unwrap();
    let next = s.decline_offer(triple(&rules, &[]), &gold, &mut events);
    assert_eq!((next.offer, next.battle.is_some()), (None, true));
    assert!(events.is_empty());
    assert!(!s.is_removed(0));
}

// covers: ADV-MON-008
#[test]
fn a_declined_flight_lets_the_monster_go_without_its_treasure() {
    let rules = rules();
    let mut s = pikemen(&rules, &[(PEASANT, 100)], -4);
    let mut events: Vec<AdventureEvent> = Vec::new();
    let pursue = offer(2, 0, OfferKind::Pursue).unwrap();
    let next = s.decline_offer(triple(&rules, &[]), &pursue, &mut events);
    assert_eq!(next, AdventureOutcome::default());
    assert!(s.is_removed(0));
    assert_eq!(s.guard_of(&rules.1, Pos::new(1, 1, 0)), None);
    assert_eq!(
        events,
        [AdventureEvent {
            kind: AdventureEventKind::ObjectRemoved {
                object_type: MONSTER,
                anchor: Pos::new(2, 0, 0),
            },
            seen_by: Viewers::NONE.with(RED),
        }]
    );
    // Accepted, the hero pursues and fights it: the battle, the monster still on the map.
    let mut s = pikemen(&rules, &[(PEASANT, 100)], -4);
    let before = s.clone();
    let outcome = s.accept_offer(&rules.0, &rules.1, RED, &pursue, &mut events);
    assert_eq!(
        outcome.battle.map(|b| (b.tile, b.defender)),
        Some((
            Pos::new(2, 0, 0),
            Defender::Monster {
                creature: PIKEMAN,
                count: 10
            }
        ))
    );
    assert_eq!(s, before);
}

// covers: ADV-MON-008
#[test]
fn an_accepted_join_adds_the_group_gives_the_treasure_and_removes_the_monster() {
    let rules = rules();
    let mut events: Vec<AdventureEvent> = Vec::new();
    let join = offer(2, 0, OfferKind::Join).unwrap();
    // Into the stack of its creature.
    let mut s = pikemen(&rules, &[(PEASANT, 100), (PIKEMAN, 3)], -4);
    let outcome = s.accept_offer(&rules.0, &rules.1, RED, &join, &mut events);
    assert_eq!(outcome.gains, [(GOLD, 500)]);
    assert_eq!(outcome.offer, None);
    let army = &s.hero(HERO).unwrap().army;
    assert_eq!(army[1].creature, PIKEMAN);
    assert_eq!(army[1].count, 13);
    assert!(s.is_removed(0));
    assert_eq!(
        events,
        [
            AdventureEvent {
                kind: AdventureEventKind::CreaturesJoined {
                    hero: HERO,
                    creature: PIKEMAN,
                    count: 10,
                },
                seen_by: Viewers::NONE.with(RED),
            },
            AdventureEvent {
                kind: AdventureEventKind::ObjectRemoved {
                    object_type: MONSTER,
                    anchor: Pos::new(2, 0, 0),
                },
                seen_by: Viewers::NONE.with(RED),
            },
        ]
    );
    // Else into the first free slot; for its price the same (the game pays).
    let mut s = pikemen(&rules, &[(PEASANT, 100), (GNOLL, 1)], -4);
    let gold = offer(
        2,
        0,
        OfferKind::JoinForGold {
            price: vec![(GOLD, 200)],
        },
    )
    .unwrap();
    let outcome = s.accept_offer(&rules.0, &rules.1, RED, &gold, &mut events);
    assert_eq!(outcome.gains, [(GOLD, 500)]);
    let army = &s.hero(HERO).unwrap().army;
    assert_eq!(
        (army[2].slot, army[2].creature, army[2].count),
        (2, PIKEMAN, 10)
    );
}

// covers: ADV-MON-008, ADV-ARM-007
#[test]
fn an_offer_whose_monster_is_gone_does_nothing_and_a_join_without_room_is_reported() {
    let rules = rules();
    let mut events: Vec<AdventureEvent> = Vec::new();
    let join = offer(2, 0, OfferKind::Join).unwrap();
    // The army filled up since the offer (only in a test until the game's army dialog, T51): the
    // group joins nothing and is reported; its treasure is given and the monster leaves as for
    // any join (E08 T50).
    let c = |i: u16| CreatureTypeId::from_raw(4 + i);
    let six = [
        (PEASANT, 100),
        (GNOLL, 1),
        (WOLF, 1),
        (c(0), 1),
        (c(1), 1),
        (c(2), 1),
    ];
    let mut s = pikemen(&rules, &six, -4);
    s.raise(&rules.0, &rules.1, HERO, (c(3), 1));
    let army = s.hero(HERO).unwrap().army.clone();
    let outcome = s.accept_offer(&rules.0, &rules.1, RED, &join, &mut events);
    let group = NoRoom {
        hero: HERO,
        creature: PIKEMAN,
        count: 10,
    };
    assert_eq!(
        outcome,
        AdventureOutcome {
            gains: vec![(GOLD, 500)],
            no_room: Some(group),
            ..AdventureOutcome::default()
        }
    );
    assert_eq!(s.hero(HERO).unwrap().army, army);
    assert!(s.is_removed(0));
    assert!(
        events
            .iter()
            .all(|e| !matches!(e.kind, AdventureEventKind::CreaturesJoined { .. }))
    );
    events.clear();
    // The monster left the map: nothing, accepted or declined.
    let mut s = pikemen(&rules, &[(PEASANT, 100)], -4);
    s.remove_object(0);
    let before = s.clone();
    for kind in [OfferKind::Join, OfferKind::Pursue] {
        let gone = offer(2, 0, kind).unwrap();
        assert_eq!(
            s.accept_offer(&rules.0, &rules.1, RED, &gone, &mut events),
            AdventureOutcome::default()
        );
        assert_eq!(
            s.decline_offer(triple(&rules, &[]), &gone, &mut events),
            AdventureOutcome::default()
        );
    }
    assert_eq!(s, before);
    // The hero left the map: the declined join has no answer to go on with.
    let mut s = pikemen(&rules, &[(PEASANT, 100)], -4);
    let stranger = Offer {
        hero: HeroId::new(9),
        ..join.clone()
    };
    assert_eq!(
        s.decline_offer(triple(&rules, &[]), &stranger, &mut events),
        AdventureOutcome::default()
    );
    assert_eq!(
        s.accept_offer(&rules.0, &rules.1, RED, &stranger, &mut events),
        AdventureOutcome::default()
    );
    // An offer naming an object that is no monster: no answer either.
    let mut s = state(
        &rules,
        &["gggggg", "gOgggg", "gggggg"],
        vec![],
        (0, 0),
        &[(PEASANT, 1)],
        1,
    );
    let odd = Offer {
        object_type: TOWER,
        anchor: Pos::new(1, 1, 0),
        ..join.clone()
    };
    assert_eq!(
        s.decline_offer(triple(&rules, &[]), &odd, &mut events),
        AdventureOutcome::default()
    );
    assert!(events.is_empty());
    // A Cartographer's or a border guard's decline changes nothing.
    let chart = Offer {
        object_type: CHART,
        kind: OfferKind::Pass,
        ..join
    };
    assert_eq!(
        s.decline_offer(triple(&rules, &[]), &chart, &mut events),
        AdventureOutcome::default()
    );
}

// covers: ADV-MON-006, ADV-MON-007
#[test]
fn the_guard_answers_after_the_step_or_after_the_tiles_offer() {
    let rules = rules();
    // The hero steps onto (2, 1), guarded by the pikemen at (2, 0): their answer.
    let mut s = pikemen(&rules, &[(PEASANT, 100)], -4);
    let (outcome, events) = walk(&rules, &mut s, &[], &[(2, 1)]);
    assert_eq!(outcome.offer, offer(2, 0, OfferKind::Join));
    assert_eq!(events.len(), 1);
    // Onto a Cartographer's tile the pikemen guard (beside it: the row above is left out, the
    // Cartographer is not visited from above): its offer first, the guard's visit waits.
    let mut s = state(
        &rules,
        &["gggggg", "ggCggg", "gggggg"],
        vec![monster_at(3, 1, PIKEMAN, 10)],
        (2, 2),
        &[(PEASANT, 100)],
        1,
    );
    set_aggression(&mut s, 1, -4);
    let (outcome, _) = walk(&rules, &mut s, &[], &[(2, 1)]);
    assert_eq!(outcome.offer.map(|o| o.object_type), Some(CHART));
    assert!(outcome.guard_after);
    let guard = s.visit_guard(triple(&rules, &[]), HERO);
    assert_eq!(guard.offer, offer(3, 1, OfferKind::Join));
    // Onto an observatory's tile: its reveal, then the guard's answer in the same outcome.
    let mut s = state(
        &rules,
        &["gggggg", "ggOggg", "gggggg"],
        vec![monster_at(3, 1, PIKEMAN, 10)],
        (2, 2),
        &[(PEASANT, 100)],
        1,
    );
    set_aggression(&mut s, 1, -4);
    let (outcome, _) = walk(&rules, &mut s, &[], &[(2, 1)]);
    assert_eq!(outcome.offer, offer(3, 1, OfferKind::Join));
    assert!(!outcome.guard_after);
    // No guard, or no hero: nothing.
    assert_eq!(
        s.visit_guard(triple(&rules, &[]), HeroId::new(9)),
        AdventureOutcome::default()
    );
    let s = state(
        &rules,
        &ROWS,
        vec![monster_at(5, 0, PIKEMAN, 10)],
        (0, 2),
        &[(PEASANT, 100)],
        1,
    );
    assert_eq!(
        s.visit_guard(triple(&rules, &[]), HERO),
        AdventureOutcome::default()
    );
}

// covers: ADV-MON-009
#[test]
fn a_fight_starts_a_battle_against_the_unsplit_monster_at_its_tile() {
    let rules = rules();
    let tile = Pos::new(2, 0, 0);
    // M = 1 above C = 0: the hero's step toward the pikemen starts the battle.
    let mut s = pikemen(&rules, &[(PEASANT, 100)], 1);
    let (outcome, _) = walk(&rules, &mut s, &[], &[(2, 0)]);
    let battle = outcome.battle.unwrap();
    assert_eq!(
        battle.attacker,
        Combatant {
            hero: HERO,
            owner: RED,
            army: s.hero(HERO).unwrap().army.clone(),
            formation: Formation::Loose,
            tactics: true,
        }
    );
    assert_eq!(
        battle.defender,
        Defender::Monster {
            creature: PIKEMAN,
            count: 10
        }
    );
    assert_eq!(battle.tile, tile);
    assert_eq!(
        (battle.terrain, battle.battlefield),
        s.battle_ground(&rules.1, tile)
    );
    // A guard's fight is at the guard's tile, not at the tile the hero stepped onto.
    let mut s = pikemen(&rules, &[(PEASANT, 100)], 1);
    let (outcome, _) = walk(&rules, &mut s, &[], &[(2, 1)]);
    assert_eq!(outcome.battle.map(|b| b.tile), Some(tile));
}

// covers: ADV-MON-010
#[test]
fn a_monsters_battle_leaves_its_survivors_or_removes_it_with_its_treasure() {
    let rules = rules();
    let core = (&rules.0, &rules.1);
    let tile = Pos::new(2, 0, 0);
    let stack = |slot, creature, count| ArmyStack {
        slot,
        creature,
        count,
    };
    let mut events: Vec<AdventureEvent> = Vec::new();
    let word = |s: &AdventureState| {
        s.observe(&rules.0, &rules.1, RED).objects[0]
            .monster
            .unwrap()
            .count_word
    };
    // It survives: every stack it has left counts as its creature (A-6 fixed: another stack's,
    // an upgrade's in the game, too); the count and the word a viewer sees follow, the word's
    // change told to the players that see it (E08 T20b3b1).
    let mut s = pikemen(&rules, &[(PEASANT, 100)], 1);
    assert_eq!(word(&s), 2);
    let mut same = s.clone();
    let left = [stack(0, PIKEMAN, 3), stack(2, GNOLL, 2)];
    let gains = s.end_monster_battle(core, tile, (&left, false), &mut events);
    assert!(gains.is_empty());
    assert_eq!(s.monster_count(0), Some(5));
    assert_eq!(s.army_left(0), Some(&[stack(0, PIKEMAN, 5)][..]));
    assert_eq!(word(&s), 1);
    assert_eq!(
        events,
        [AdventureEvent {
            kind: AdventureEventKind::MonsterChanged {
                object_type: MONSTER,
                anchor: tile,
                count_word: 1,
            },
            seen_by: Viewers::NONE.with(RED),
        }]
    );
    events.clear();
    // Survivors of the same word: nothing to tell.
    let had = same.monster_count(0).unwrap();
    same.end_monster_battle(core, tile, (&[stack(1, PIKEMAN, had)], false), &mut events);
    assert_eq!((same.monster_count(0), word(&same)), (Some(had), 2));
    assert!(events.is_empty());
    // Its next battle is against the survivors.
    set_aggression(&mut s, 0, 10);
    let (outcome, _) = walk(&rules, &mut s, &[], &[(2, 0)]);
    assert_eq!(
        outcome.battle.map(|b| b.defender),
        Some(Defender::Monster {
            creature: PIKEMAN,
            count: 5
        })
    );
    // Beaten: it leaves the map, and the victory gives its treasure.
    let gains = s.end_monster_battle(core, tile, (&[], true), &mut events);
    assert_eq!(gains, [(GOLD, 500)]);
    assert!(s.is_removed(0));
    assert_eq!(s.army_left(0), None);
    assert_eq!(
        events,
        [AdventureEvent {
            kind: AdventureEventKind::ObjectRemoved {
                object_type: MONSTER,
                anchor: tile,
            },
            seen_by: Viewers::NONE.with(RED),
        }]
    );
    // Both sides dead: it leaves the map without its treasure.
    let mut s = pikemen(&rules, &[(PEASANT, 100)], 1);
    let gains = s.end_monster_battle(core, tile, (&[], false), &mut events);
    assert!(gains.is_empty());
    assert!(s.is_removed(0));
    // No monster visited at the tile, or another object: nothing.
    let mut s = state(
        &rules,
        &["gggggg", "ggOggg", "gggggg"],
        vec![monster_at(4, 0, PIKEMAN, 10)],
        (0, 2),
        &[(PEASANT, 100)],
        1,
    );
    let before = s.clone();
    for at in [Pos::new(0, 0, 0), Pos::new(2, 1, 0)] {
        assert!(
            s.end_monster_battle(core, at, (&[], true), &mut events)
                .is_empty()
        );
    }
    assert_eq!(s, before);
}
