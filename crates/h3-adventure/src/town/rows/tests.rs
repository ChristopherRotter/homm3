//! Tests of a town's two rows (TWN-ROW-001 to -004; E09 T28a, the town epic's design note §52,
//! §53): the garrison hero's record and its three moves, the absorb, the capture's release
//! (`TWN-STUB-14`), the town's bars and a garrison hero in the army commands, the hire's town
//! target with a garrison hero, the owner's view and the hash.

extern crate std;

use std::vec;
use std::vec::Vec;

use h3_content::{CreatureDef, CreatureInfoDef, CreatureTypeId, FactionId, Formation, Ruleset};
use h3_contracts::{ArmyStack, HeroId, PlayerId, TownId};
use h3_kernel::hash::hash_of;

use super::{TownHeroError, TownHeroMove, absorb};
use crate::army::{ArmyCommand, ArmyCommandError, ArmyRef, ArmySlot};
use crate::command::{
    AdventureEvent, AdventureEventKind, HireTarget, OwnHeroes, Viewers,
};
use crate::ruleset::{AdventurePack, AdventureRuleset, ObjectKindDef, ObjectTypeDef, Visit};
use crate::setup::{AdventureSetup, Holds, ObjectSetup, Pos, Scenery, TileMask};
use crate::state::AdventureState;
use crate::testing::{
    GNOLL, MAX_STACK, PEASANT, PIKEMAN, WOLF, hero_at, key, movement_rules_with, started, world,
};

type Rules = (Ruleset, AdventureRuleset);

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const VISITOR: HeroId = HeroId::new(1);
const OTHER: HeroId = HeroId::new(2);
const TOWN: TownId = TownId::new(0);
const ENTRANCE: Pos = Pos::new(8, 3, 0);
/// The most heroes on the map the tests pass (the original's 8).
const MOST: u8 = 8;

/// [`movement_rules_with`] with a town type `t:town` and four more creatures, ids 4 to 7.
fn rules() -> Rules {
    movement_rules_with(
        true,
        16,
        |core| {
            core.creatures
                .extend(["t:ra", "t:rb", "t:rc", "t:rd"].map(|k| CreatureDef {
                    key: key(k),
                    speed: 5,
                    info: CreatureInfoDef::default(),
                }));
        },
        |part: &mut AdventurePack| {
            part.object_types.push(ObjectTypeDef {
                key: key("t:town"),
                name: "Town".into(),
                visit: Visit::Enter,
                from_above: false,
                kind_mark: false,
                kind: ObjectKindDef::Town {
                    boat_offsets: Vec::new(),
                },
            });
        },
    )
}

/// The creature of id `n`.
const fn c(n: u16) -> CreatureTypeId {
    CreatureTypeId::from_raw(n)
}

fn stack(slot: u8, creature: CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

/// A grass map of 16 × 8 with `owner`'s town anchored on [`ENTRANCE`] (its gate) holding
/// `garrison`, red's hero 1 on `visitor` with `army`, red's hero 2 on (1, 1) and blue's hero 3 on
/// (14, 6).
fn setup(
    rules: &Rules,
    owner: Option<u8>,
    garrison: Vec<ArmyStack>,
    (visitor, army): (Pos, Vec<ArmyStack>),
) -> AdventureSetup {
    let mut setup = world(
        &["gggggggggggggggg"; 8],
        1,
        vec![
            hero_at(1, 0, visitor, &[PEASANT]),
            hero_at(2, 0, Pos::new(1, 1, 0), &[PEASANT]),
            hero_at(3, 1, Pos::new(14, 6, 0), &[PEASANT]),
        ],
    );
    setup.heroes[0].army = army;
    setup.map.objects = vec![ObjectSetup {
        scenery: Scenery::Ownable,
        anchor: ENTRANCE,
        blocked: TileMask::from_bits(0b11),
        visitable: TileMask::from_bits(1),
        object_type: rules.1.object_type_by_key("t:town"),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds: Holds::Town {
            formation: Formation::Loose,
        },
        owner: owner.map(PlayerId::new),
    }];
    setup.town_factions = vec![FactionId::from_raw(0)];
    setup.garrisons = vec![garrison];
    setup
}

/// Red's town with `garrison` and red's visitor with `army` on its entrance.
fn at_town(rules: &Rules, garrison: Vec<ArmyStack>, army: Vec<ArmyStack>) -> AdventureState {
    started(rules, &setup(rules, Some(0), garrison, (ENTRANCE, army))).unwrap()
}

/// Makes `player`'s move `change` at `town`, its events.
fn row(
    rules: &Rules,
    state: &mut AdventureState,
    player: PlayerId,
    (town, change): (TownId, TownHeroMove),
) -> Result<(Vec<ArmySlot>, Vec<AdventureEvent>), TownHeroError> {
    let mut events = Vec::new();
    let checked = state.check_town_hero(&rules.1, player, (town, MOST), change);
    let made = state
        .move_town_hero((&rules.0, &rules.1), player, (town, MOST), change, &mut events)
        .map(|slots| (slots, events));
    // The legal query answers as the move does.
    assert_eq!(checked, made.as_ref().map(|_| ()).map_err(|&e| e));
    made
}

/// `state` after a refused move: unchanged.
fn refused(
    rules: &Rules,
    state: &AdventureState,
    player: PlayerId,
    (town, change): (TownId, TownHeroMove),
) -> TownHeroError {
    let mut copy = state.clone();
    let error = row(rules, &mut copy, player, (town, change)).unwrap_err();
    assert_eq!(&copy, state);
    error
}

// covers: TWN-ROW-002
#[test]
fn the_visitor_enters_the_garrison_with_its_whole_record_and_the_garrison_s_creatures() {
    let rules = rules();
    let garrison = vec![stack(4, PEASANT, 5), stack(1, PIKEMAN, 3)];
    let army = vec![stack(0, PEASANT, 2), stack(2, GNOLL, 1)];
    let mut state = at_town(&rules, garrison, army);
    let mut before = state.hero(VISITOR).unwrap().clone();
    before.movement = 77;
    before.formation = Formation::Tight;
    state.hero_mut(VISITOR).unwrap().clone_from(&before);
    let hash = state.state_hash();
    let own = state.own_heroes();
    let (slots, events) = row(&rules, &mut state, RED, (TOWN, TownHeroMove::Enter)).unwrap();
    // The garrison in slot order: the pikemen into the first free slot, the peasants onto the
    // hero's peasants.
    let army = vec![stack(0, PEASANT, 7), stack(1, PIKEMAN, 3), stack(2, GNOLL, 1)];
    let held = state.town_hero(TOWN).unwrap();
    assert_eq!(held.army, army);
    assert_eq!((held.movement, held.formation), (77, Formation::Tight));
    assert_eq!(held.pos, ENTRANCE);
    assert_eq!(state.garrisoned(VISITOR), Some((TOWN, held)));
    assert_eq!(state.garrisoned(OTHER), None);
    assert_eq!(state.town_heroes().len(), 1);
    // Off the map: no hero there, the town's garrison empty.
    assert!(state.hero(VISITOR).is_none());
    assert_eq!(state.town_visitor(TOWN), None);
    assert_eq!(state.town_garrison(0), Some(&[][..]));
    assert_eq!(
        slots,
        [
            ArmySlot {
                army: ArmyRef::Hero(VISITOR),
                slot: 0
            },
            ArmySlot {
                army: ArmyRef::Hero(VISITOR),
                slot: 1
            },
            ArmySlot {
                army: ArmyRef::Garrison(TOWN),
                slot: 1
            },
            ArmySlot {
                army: ArmyRef::Garrison(TOWN),
                slot: 4
            },
        ]
    );
    // Blue sees the entrance on this map, as a hero there (ADV-FOG-004).
    let seen_by = Viewers::NONE.with(RED).with(BLUE);
    assert_eq!(state.viewers(&rules.0, RED, ENTRANCE), seen_by);
    assert_eq!(
        events,
        [AdventureEvent {
            kind: AdventureEventKind::GarrisonEntered {
                hero: VISITOR,
                town: TOWN,
                pos: ENTRANCE,
            },
            seen_by,
        }]
    );
    // The hash reads the garrison hero (v24); its owner sees it in its garrison heroes.
    assert_ne!(state.state_hash(), hash);
    assert_eq!(state.state_hash(), hash_of(&state));
    let view = state.observe(&rules.0, &rules.1, RED);
    assert_eq!(view.garrison_heroes.len(), 1);
    assert_eq!(
        (view.garrison_heroes[0].town, view.garrison_heroes[0].id),
        (TOWN, VISITOR)
    );
    assert_eq!(view.garrison_heroes[0].own.army, army);
    assert!(!view.heroes.iter().any(|h| h.id == VISITOR));
    let other = state.observe(&rules.0, &rules.1, BLUE);
    assert!(other.garrison_heroes.is_empty());
    // Its points and marks did not change: nothing more is told.
    let mut told = Vec::new();
    own.tell(&state, &mut told);
    assert_eq!(told, []);
    // A second `Enter` finds the garrison held.
    assert_eq!(
        refused(&rules, &state, RED, (TOWN, TownHeroMove::Enter)),
        TownHeroError::GarrisonHeld
    );
}

// covers: TWN-ROW-002
#[test]
fn entering_is_refused_with_a_garrison_hero_another_s_visitor_or_no_room() {
    let rules = rules();
    let mut state = at_town(&rules, vec![], vec![stack(0, PEASANT, 1)]);
    assert_eq!(
        refused(&rules, &state, RED, (TownId::new(4), TownHeroMove::Enter)),
        TownHeroError::UnknownTown
    );
    // Blue's move at red's town: blue has no visitor there (the owner is the game's check).
    assert_eq!(
        refused(&rules, &state, BLUE, (TOWN, TownHeroMove::Enter)),
        TownHeroError::NoVisitor
    );
    row(&rules, &mut state, RED, (TOWN, TownHeroMove::Enter)).unwrap();
    // Another visitor finds the garrison held.
    state.hero_mut(OTHER).unwrap().pos = ENTRANCE;
    assert_eq!(
        refused(&rules, &state, RED, (TOWN, TownHeroMove::Enter)),
        TownHeroError::GarrisonHeld
    );
    // Eight creatures in seven slots do not fit; the visitor's own two of one creature merge
    // first, so seven do.
    let full: Vec<ArmyStack> = (0..7).map(|n| stack(n, c(u16::from(n)), 1)).collect();
    let state = at_town(&rules, vec![stack(0, c(7), 9)], full.clone());
    assert_eq!(
        refused(&rules, &state, RED, (TOWN, TownHeroMove::Enter)),
        TownHeroError::ArmiesDoNotFit
    );
    let mut paired = full;
    paired[6].creature = c(0);
    let mut state = at_town(&rules, vec![stack(0, c(7), 9)], paired);
    row(&rules, &mut state, RED, (TOWN, TownHeroMove::Enter)).unwrap();
    assert_eq!(state.town_hero(TOWN).unwrap().army[6], stack(6, c(7), 9));
    // A join past a stack's most.
    let state = at_town(
        &rules,
        vec![stack(0, WOLF, 1)],
        vec![stack(0, WOLF, MAX_STACK)],
    );
    assert_eq!(
        refused(&rules, &state, RED, (TOWN, TownHeroMove::Enter)),
        TownHeroError::TooManyCreatures
    );
}

// covers: TWN-ROW-002
#[test]
fn the_absorb_joins_fills_merges_and_refuses_all_or_nothing() {
    let rules = (7, MAX_STACK);
    // A stack of the creature takes the garrison's; a free slot the next.
    let mut army = vec![stack(3, PEASANT, 1)];
    absorb(rules, &mut army, &[stack(5, GNOLL, 2), stack(1, PEASANT, 4)]).unwrap();
    assert_eq!(army, [stack(0, GNOLL, 2), stack(3, PEASANT, 5)]);
    // A full army merges its own first two stacks of one creature, the later into the earlier,
    // and the slot is tried again.
    let mut army: Vec<ArmyStack> = (0..7).map(|n| stack(n, c(u16::from(n % 6)), 1)).collect();
    absorb(rules, &mut army, &[stack(0, c(7), 3)]).unwrap();
    assert_eq!(army[0], stack(0, c(0), 2));
    assert_eq!(army[6], stack(6, c(7), 3));
    // No pair: nothing frees a slot, nothing changes.
    let distinct: Vec<ArmyStack> = (0..7).map(|n| stack(n, c(u16::from(n)), 1)).collect();
    let mut army = distinct.clone();
    assert_eq!(
        absorb(rules, &mut army, &[stack(0, c(7), 1)]),
        Err(TownHeroError::ArmiesDoNotFit)
    );
    assert_eq!(army, distinct);
    // A join or a merge past a stack's most.
    let mut army = vec![stack(0, PEASANT, MAX_STACK)];
    assert_eq!(
        absorb(rules, &mut army, &[stack(0, PEASANT, 1)]),
        Err(TownHeroError::TooManyCreatures)
    );
    let mut army: Vec<ArmyStack> = (0..7)
        .map(|n| stack(n, c(u16::from(n % 6)), if n == 6 { MAX_STACK } else { 1 }))
        .collect();
    assert_eq!(
        absorb(rules, &mut army, &[stack(0, c(7), 1)]),
        Err(TownHeroError::TooManyCreatures)
    );
    assert_eq!(army[0], stack(0, c(0), 1));
    // An empty garrison changes nothing.
    let mut army = vec![stack(2, GNOLL, 1)];
    absorb(rules, &mut army, &[]).unwrap();
    assert_eq!(army, [stack(2, GNOLL, 1)]);
}

// covers: TWN-ROW-003
#[test]
fn the_garrison_hero_leaves_onto_the_entrance_with_its_points_kept() {
    let rules = rules();
    let mut state = at_town(&rules, vec![], vec![stack(0, PEASANT, 1)]);
    let start = state.clone();
    state.hero_mut(VISITOR).unwrap().movement = 300;
    row(&rules, &mut state, RED, (TOWN, TownHeroMove::Enter)).unwrap();
    let own = state.own_heroes();
    let (slots, events) = row(&rules, &mut state, RED, (TOWN, TownHeroMove::Leave)).unwrap();
    assert_eq!(slots, []);
    let hero = state.hero(VISITOR).unwrap();
    assert_eq!((hero.pos, hero.boat, hero.movement), (ENTRANCE, None, 300));
    assert_eq!(state.town_visitor(TOWN), Some(VISITOR));
    // The block is dropped: the state is the start's with the points changed.
    assert!(state.town_heroes().is_empty());
    let mut expected = start;
    expected.hero_mut(VISITOR).unwrap().movement = 300;
    assert_eq!(state, expected);
    assert_eq!(
        events,
        [AdventureEvent {
            kind: AdventureEventKind::GarrisonLeft {
                hero: VISITOR,
                town: TOWN,
                pos: ENTRANCE,
            },
            seen_by: Viewers::NONE.with(RED).with(BLUE),
        }]
    );
    let mut told = Vec::new();
    own.tell(&state, &mut told);
    assert_eq!(told, []);
}

// covers: TWN-ROW-003
#[test]
fn leaving_is_refused_without_a_garrison_hero_a_free_entrance_room_or_an_army() {
    let rules = rules();
    let mut state = at_town(&rules, vec![], vec![stack(0, PEASANT, 1)]);
    assert_eq!(
        refused(&rules, &state, RED, (TOWN, TownHeroMove::Leave)),
        TownHeroError::NoGarrisonHero
    );
    row(&rules, &mut state, RED, (TOWN, TownHeroMove::Enter)).unwrap();
    assert_eq!(
        refused(&rules, &state, BLUE, (TOWN, TownHeroMove::Leave)),
        TownHeroError::NoGarrisonHero
    );
    // Blue's hero on the entrance holds it.
    let mut held = state.clone();
    held.hero_mut(HeroId::new(3)).unwrap().pos = ENTRANCE;
    assert_eq!(
        refused(&rules, &held, RED, (TOWN, TownHeroMove::Leave)),
        TownHeroError::EntranceHeld
    );
    // Red has one hero on the map: a most of 1 refuses.
    let mut copy = state.clone();
    assert_eq!(
        copy.check_town_hero(&rules.1, RED, (TOWN, 1), TownHeroMove::Leave),
        Err(TownHeroError::TooManyHeroes)
    );
    assert_eq!(
        copy.move_town_hero((&rules.0, &rules.1), RED, (TOWN, 1), TownHeroMove::Leave, &mut Vec::new()),
        Err(TownHeroError::TooManyHeroes)
    );
    assert_eq!(copy, state);
    // An army without a stack (T-5's case, built as a state).
    let at = state.own_town_hero(RED, VISITOR).unwrap();
    state.town_hero_at_mut(at).unwrap().army.clear();
    assert_eq!(
        refused(&rules, &state, RED, (TOWN, TownHeroMove::Leave)),
        TownHeroError::EmptyArmy
    );
    assert_eq!(state.own_town_hero(BLUE, VISITOR), None);
    assert!(state.town_hero_at_mut(1).is_none());
}

// covers: TWN-ROW-004
#[test]
fn the_visitor_and_the_garrison_hero_swap_with_their_armies() {
    let rules = rules();
    let mut state = at_town(&rules, vec![], vec![stack(0, PEASANT, 4)]);
    assert_eq!(
        refused(&rules, &state, RED, (TOWN, TownHeroMove::Swap)),
        TownHeroError::NoGarrisonHero
    );
    row(&rules, &mut state, RED, (TOWN, TownHeroMove::Enter)).unwrap();
    assert_eq!(
        refused(&rules, &state, RED, (TOWN, TownHeroMove::Swap)),
        TownHeroError::NoVisitor
    );
    // Blue's hero on the entrance moves no hero (TD-17).
    let mut blue = state.clone();
    blue.hero_mut(HeroId::new(3)).unwrap().pos = ENTRANCE;
    assert_eq!(
        refused(&rules, &blue, RED, (TOWN, TownHeroMove::Swap)),
        TownHeroError::NoVisitor
    );
    state.hero_mut(OTHER).unwrap().pos = ENTRANCE;
    state.hero_mut(OTHER).unwrap().army = vec![stack(3, GNOLL, 2)];
    let (_, events) = row(&rules, &mut state, RED, (TOWN, TownHeroMove::Swap)).unwrap();
    assert_eq!(state.town_hero(TOWN).unwrap().id, OTHER);
    assert_eq!(state.town_hero(TOWN).unwrap().army, [stack(3, GNOLL, 2)]);
    let hero = state.hero(VISITOR).unwrap();
    assert_eq!((hero.pos, &hero.army[..]), (ENTRANCE, &[stack(0, PEASANT, 4)][..]));
    let seen_by = Viewers::NONE.with(RED).with(BLUE);
    assert_eq!(
        events,
        [
            AdventureEvent {
                kind: AdventureEventKind::GarrisonEntered {
                    hero: OTHER,
                    town: TOWN,
                    pos: ENTRANCE,
                },
                seen_by,
            },
            AdventureEvent {
                kind: AdventureEventKind::GarrisonLeft {
                    hero: VISITOR,
                    town: TOWN,
                    pos: ENTRANCE,
                },
                seen_by,
            },
        ]
    );
}

// covers: TWN-ROW-001
#[test]
fn the_town_s_bars_follow_the_one_owner_rule() {
    let rules = rules();
    let part = &rules.1;
    let mut state = at_town(&rules, vec![stack(0, PIKEMAN, 6)], vec![stack(0, PEASANT, 1)]);
    let garrison = |slot| ArmySlot {
        army: ArmyRef::Garrison(TOWN),
        slot,
    };
    let hero = |id, slot| ArmySlot {
        army: ArmyRef::Hero(id),
        slot,
    };
    // The town's garrison is its owner's bar and may be emptied.
    let mut copy = state.clone();
    let split = ArmyCommand::Split {
        from: garrison(0),
        to: hero(VISITOR, 1),
        count: 2,
    };
    copy.manage_army(part, RED, &split, None).unwrap();
    assert_eq!(copy.town_garrison(0), Some(&[stack(0, PIKEMAN, 4)][..]));
    let back = ArmyCommand::Move {
        from: hero(VISITOR, 1),
        to: garrison(0),
    };
    copy.manage_army(part, RED, &back, None).unwrap();
    let out = ArmyCommand::Move {
        from: garrison(0),
        to: hero(VISITOR, 3),
    };
    copy.manage_army(part, RED, &out, None).unwrap();
    assert_eq!(copy.town_garrison(0), Some(&[][..]));
    assert_eq!(
        copy.split_range(part, RED, (hero(VISITOR, 3), garrison(2)), None),
        Some(1..=5)
    );
    // Blue's commands on red's town's garrison are refused, nothing changed.
    let dismiss = ArmyCommand::Dismiss { at: garrison(0) };
    assert_eq!(
        state.clone().manage_army(part, BLUE, &dismiss, None),
        Err(ArmyCommandError::Garrison(TOWN))
    );
    assert_eq!(
        state.army_commands(part, RED, (ArmyRef::Garrison(TOWN), None), None),
        [
            1, 2, 3, 4, 5, 6
        ]
        .map(|slot| ArmyCommand::Move {
            from: garrison(0),
            to: garrison(slot),
        })
        .into_iter()
        .chain([dismiss])
        .collect::<Vec<_>>()
    );
    // With a garrison hero, the upper bar is its army: the town's garrison is not named, the
    // hero keeps its last stack, its formation and flag are its own.
    row(&rules, &mut state, RED, (TOWN, TownHeroMove::Enter)).unwrap();
    assert_eq!(
        state.clone().manage_army(part, RED, &dismiss, None),
        Err(ArmyCommandError::Garrison(TOWN))
    );
    let last = ArmyCommand::Dismiss {
        at: hero(VISITOR, 0),
    };
    let mut copy = state.clone();
    copy.manage_army(part, RED, &last, None).unwrap();
    assert_eq!(copy.town_hero(TOWN).unwrap().army, [stack(1, PIKEMAN, 6)]);
    assert_eq!(
        copy.clone().manage_army(
            part,
            RED,
            &ArmyCommand::Dismiss {
                at: hero(VISITOR, 1)
            },
            None
        ),
        Err(ArmyCommandError::LastStack)
    );
    copy.set_formation(RED, VISITOR, Formation::Tight).unwrap();
    copy.set_tactics(RED, VISITOR, false).unwrap();
    let held = copy.town_hero(TOWN).unwrap();
    assert_eq!((held.formation, held.tactics), (Formation::Tight, false));
    assert_eq!(
        copy.set_tactics(BLUE, VISITOR, true),
        Err(ArmyCommandError::NotYourHero(VISITOR))
    );
    assert_eq!(
        state.clone().manage_army(part, BLUE, &last, None),
        Err(ArmyCommandError::NotYourHero(VISITOR))
    );
}

// covers: TWN-ROW-001
#[test]
fn a_town_s_purchase_joins_its_garrison_hero_s_army() {
    let rules = rules();
    let mut state = at_town(&rules, vec![], vec![stack(2, PEASANT, 1)]);
    row(&rules, &mut state, RED, (TOWN, TownHeroMove::Enter)).unwrap();
    let target = HireTarget::Garrison(TOWN);
    assert_eq!(state.hire_room(&rules.1, &target, GNOLL), MAX_STACK);
    let purchase = state
        .check_join((&rules.0, &rules.1), &target, (PEASANT, 3))
        .unwrap();
    let joined = state.hire(purchase, &mut Vec::new());
    assert_eq!(
        joined,
        Some(ArmySlot {
            army: ArmyRef::Hero(VISITOR),
            slot: 2
        })
    );
    let purchase = state
        .check_join((&rules.0, &rules.1), &target, (GNOLL, 1))
        .unwrap();
    let joined = state.hire(purchase, &mut Vec::new());
    assert_eq!(joined.map(|at| at.slot), Some(0));
    assert_eq!(
        state.town_hero(TOWN).unwrap().army,
        [stack(0, GNOLL, 1), stack(2, PEASANT, 4)]
    );
    assert_eq!(state.town_garrison(0), Some(&[][..]));
}

// covers: TWN-ROW-002
#[test]
fn the_garrison_heroes_keep_town_order_and_their_owner_s_snapshot() {
    let rules = rules();
    let mut setup = setup(&rules, Some(0), vec![], (ENTRANCE, vec![stack(0, PEASANT, 1)]));
    // A second town of red's, its entrance on (12, 6), with red's hero 2 visiting.
    let mut second = setup.map.objects[0].clone();
    second.anchor = Pos::new(12, 6, 0);
    setup.map.objects.push(second);
    setup.town_factions.push(FactionId::from_raw(0));
    setup.garrisons.push(vec![]);
    setup.heroes[1].pos = Pos::new(12, 6, 0);
    let mut state = started(&rules, &setup).unwrap();
    let second = TownId::new(1);
    row(&rules, &mut state, RED, (second, TownHeroMove::Enter)).unwrap();
    row(&rules, &mut state, RED, (TOWN, TownHeroMove::Enter)).unwrap();
    let order: Vec<(TownId, HeroId)> = state.town_heroes().iter().map(|(t, h)| (*t, h.id)).collect();
    assert_eq!(order, [(TOWN, VISITOR), (second, OTHER)]);
    // The snapshot holds both, by id; a garrison hero's points told as a hero's.
    let own = state.own_heroes();
    let at = state.own_town_hero(RED, OTHER).unwrap();
    state.town_hero_at_mut(at).unwrap().movement = 5;
    let mut told = Vec::new();
    own.tell(&state, &mut told);
    assert_eq!(
        told,
        [AdventureEvent {
            kind: AdventureEventKind::MovementChanged {
                hero: OTHER,
                movement: 5,
                max_movement: state.town_hero(second).unwrap().max_movement,
            },
            seen_by: Viewers::NONE.with(RED),
        }]
    );
    // One leaves, the block keeps the other.
    row(&rules, &mut state, RED, (TOWN, TownHeroMove::Leave)).unwrap();
    assert_eq!(state.town_heroes().len(), 1);
    assert_eq!(state.town_hero(second).unwrap().id, OTHER);
    let _: OwnHeroes = state.own_heroes();
}

// covers: TWN-SET-005
// stub: TWN-STUB-14
#[test]
fn a_captured_town_s_garrison_hero_leaves_the_game() {
    let rules = rules();
    let mut state = at_town(&rules, vec![], vec![stack(0, PEASANT, 1)]);
    let mut events = Vec::new();
    assert_eq!(state.release_town_hero(TOWN, &mut events), None);
    assert_eq!(state.release_town_hero(TownId::new(3), &mut events), None);
    assert_eq!(events, []);
    row(&rules, &mut state, RED, (TOWN, TownHeroMove::Enter)).unwrap();
    assert_eq!(state.release_town_hero(TOWN, &mut events), Some(VISITOR));
    assert!(state.town_heroes().is_empty());
    assert!(state.hero(VISITOR).is_none());
    assert_eq!(
        events,
        [AdventureEvent {
            kind: AdventureEventKind::HeroRemoved {
                hero: VISITOR,
                pos: ENTRANCE,
            },
            seen_by: Viewers::NONE.with(RED),
        }]
    );
}
