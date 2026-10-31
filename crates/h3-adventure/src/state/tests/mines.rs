//! Tests of the flags and the mines' visits (E08 T84a; increment 9's design note §7, §13): a visit
//! flags a mine without guards and reveals around it; an own mine opens its Leave Guards dialog,
//! whose group the game writes back; another's guards are asked, then fought as a monster, as
//! guards or as their owner's side, on the mine's battle terrain; the battle's end keeps the
//! survivors or flags the mine after the levels.

use super::economy::{
    GRASS_ROWS, OFFSETS, Rules, abandoned, object, owned, rules, rules_with, setup,
};
use super::*;
use crate::command::{BattleStart, Defender, Garrison};
use crate::testing::{PEASANT, PIKEMAN, started};

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const HERO: HeroId = HeroId::new(1);
/// The mines' entrance in these tests, two steps right of red's hero at (0, 7).
const ENTRANCE: Pos = Pos::new(2, 7, 0);

fn stack(slot: u8, creature: CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

/// The game of [`setup`]'s grass map with `mine` at [`ENTRANCE`].
fn game(rules: &Rules, mine: ObjectSetup) -> AdventureState {
    started(rules, &setup(&GRASS_ROWS, vec![mine])).unwrap()
}

/// A mine of type `k` at [`ENTRANCE`], neutral or `owner`'s.
fn mine(rules: &Rules, k: &str, owner: Option<u8>) -> ObjectSetup {
    match owner {
        Some(owner) => owned(rules, ENTRANCE.x, ENTRANCE.y, k, owner),
        None => object(rules, ENTRANCE.x, ENTRANCE.y, k),
    }
}

/// Red's hero's walk onto [`ENTRANCE`]: the outcome and the events.
fn visit(rules: &Rules, state: &mut AdventureState) -> (AdventureOutcome, Vec<AdventureEvent>) {
    let mut events = Vec::new();
    let walk = AdventureCommand::MoveHero {
        hero: HERO,
        path: vec![Pos::new(1, 7, 0), ENTRANCE],
    };
    let outcome = state
        .apply(&rules.0, &rules.1, &[], RED, &walk, &mut events)
        .unwrap();
    (outcome, events)
}

/// The mine's question to red's hero.
fn question(rules: &Rules, k: &str) -> Offer {
    Offer {
        hero: HERO,
        object_type: rules.1.object_type_by_key(k).unwrap(),
        anchor: ENTRANCE,
        kind: OfferKind::Fight,
    }
}

/// The events after the walk's steps.
fn after_steps(events: &[AdventureEvent]) -> Vec<AdventureEventKind> {
    events
        .iter()
        .filter(|e| !matches!(e.kind, AdventureEventKind::HeroStepped { .. }))
        .map(|e| e.kind)
        .collect()
}

// covers: ADV-OWN-001
#[test]
fn a_visit_flags_a_mine_without_guards_and_reveals_around_it() {
    // A flag radius past the hero's own (5), so the flag reveals something.
    let rules = rules_with(&OFFSETS, 8);
    let sawmill = rules.1.object_type_by_key("t:sawmill").unwrap();
    let mut s = game(&rules, mine(&rules, "t:sawmill", None));
    let before = s.clone();
    let (outcome, events) = visit(&rules, &mut s);
    assert_eq!(outcome, AdventureOutcome::default());
    // Entered: the walk ends on the entrance; the owner written, told, then the reveal of radius
    // 8 around the entrance, told to red with the number of tiles it had not explored.
    assert_eq!(s.hero(HERO).unwrap().pos, ENTRANCE);
    assert_eq!(s.owner(0), Some(RED));
    let told = after_steps(&events);
    assert_eq!(
        told[0],
        AdventureEventKind::ObjectFlagged {
            object_type: sawmill,
            anchor: ENTRANCE,
            owner: RED,
        }
    );
    let AdventureEventKind::TilesRevealed { object_type, count } = told[1] else {
        panic!("{told:?}");
    };
    assert_eq!((object_type, told.len()), (sawmill, 2));
    assert!(count > 0);
    let steps = events.len() - 2;
    assert_eq!(events[steps].seen_by, Viewers::NONE.with(RED));
    assert_eq!(events[steps + 1].seen_by, Viewers::NONE.with(RED));
    assert!(s.is_explored(RED, Pos::new(10, 7, 0)));
    assert!(!before.is_explored(RED, Pos::new(10, 7, 0)));
    assert!(!s.is_explored(RED, Pos::new(11, 7, 0)));
    assert!(s.check_owners(&rules.1));
    assert_ne!(s.state_hash(), before.state_hash());
    // Where blue has explored the mine, it is told the flag too, not the reveal.
    let mut seen = before;
    seen.explore_all();
    let (_, events) = visit(&rules, &mut seen);
    let steps = events.len() - 2;
    assert_eq!(events[steps].seen_by, Viewers::NONE.with(RED).with(BLUE));
    assert_eq!(
        events[steps + 1].kind,
        AdventureEventKind::TilesRevealed {
            object_type: sawmill,
            count: 0,
        }
    );
}

// covers: ADV-OWN-001, ADV-MIN-002
#[test]
fn another_player_s_mine_without_guards_is_flagged_and_an_own_one_opens_leave_guards() {
    let rules = rules();
    let sawmill = rules.1.object_type_by_key("t:sawmill").unwrap();
    // Blue's: flagged for red at once, blue told as it sees its own mine.
    let mut s = game(&rules, mine(&rules, "t:sawmill", Some(1)));
    let (outcome, events) = visit(&rules, &mut s);
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(s.owner(0), Some(RED));
    let flagged = events
        .iter()
        .find(|e| matches!(e.kind, AdventureEventKind::ObjectFlagged { .. }))
        .unwrap();
    assert_eq!(flagged.seen_by, Viewers::NONE.with(RED).with(BLUE));
    // Red's own: its Leave Guards dialog, the guards as they stand (none), nothing told.
    let mut own = game(&rules, mine(&rules, "t:sawmill", Some(0)));
    let (outcome, events) = visit(&rules, &mut own);
    let dialog = Garrison {
        hero: HERO,
        mine: 0,
        object_type: sawmill,
        anchor: ENTRANCE,
        guards: vec![],
    };
    assert_eq!(
        outcome,
        AdventureOutcome {
            garrison: Some(dialog.clone()),
            ..AdventureOutcome::default()
        }
    );
    assert!(after_steps(&events).is_empty());
    assert_eq!(own.owner(0), Some(RED));
    // With guards, its group.
    let guards = vec![stack(1, PIKEMAN, 3), stack(4, PEASANT, 9)];
    let mut guarded = game(&rules, mine(&rules, "t:sawmill", Some(0)));
    guarded.set_army_left(0, guards.clone());
    let (outcome, _) = visit(&rules, &mut guarded);
    assert_eq!(
        outcome.garrison,
        Some(Garrison {
            guards: guards.clone(),
            ..dialog
        })
    );
}

// covers: ADV-MIN-002, ADV-OWN-003
#[test]
fn the_leave_guards_dialog_s_group_is_written_back_and_its_hint_change_told() {
    let rules = rules();
    let sawmill = rules.1.object_type_by_key("t:sawmill").unwrap();
    let mut s = game(&rules, mine(&rules, "t:sawmill", Some(0)));
    s.explore_all();
    let parts = (&rules.0, &rules.1);
    let mut events = Vec::new();
    // Nine pikemen left: the hint appears, told to every player that sees the mine.
    s.set_guards(parts, 0, vec![stack(2, PIKEMAN, 9)], &mut events);
    assert_eq!(s.mine_guards(&rules.1, 0), [stack(2, PIKEMAN, 9)]);
    assert_eq!(
        events,
        [AdventureEvent {
            kind: AdventureEventKind::GuardsChanged {
                object_type: sawmill,
                anchor: ENTRANCE,
                creature: Some(PIKEMAN),
                count_word: Some(1),
            },
            seen_by: Viewers::NONE.with(RED).with(BLUE),
        }]
    );
    // One more in another slot: the word stays (5 to 9 is one word), nothing told.
    events.clear();
    let two = vec![stack(0, PIKEMAN, 1), stack(2, PIKEMAN, 8)];
    s.set_guards(parts, 0, two.clone(), &mut events);
    assert_eq!(s.mine_guards(&rules.1, 0), two);
    assert!(events.is_empty());
    // A peasant beside them: no creature named any more.
    s.set_guards(
        parts,
        0,
        vec![stack(0, PEASANT, 1), stack(2, PIKEMAN, 8)],
        &mut events,
    );
    assert!(matches!(
        events[0].kind,
        AdventureEventKind::GuardsChanged {
            creature: None,
            count_word: Some(1),
            ..
        }
    ));
    // All taken back: none left.
    events.clear();
    s.set_guards(parts, 0, vec![], &mut events);
    assert!(s.mine_guards(&rules.1, 0).is_empty());
    assert!(matches!(
        events[0].kind,
        AdventureEventKind::GuardsChanged {
            creature: None,
            count_word: None,
            ..
        }
    ));
    assert!(s.check_owners(&rules.1));
    // Nothing for an object that is no mine.
    let mut windmill = started(
        &rules,
        &setup(&GRASS_ROWS, vec![object(&rules, 2, 7, "t:windmill")]),
    )
    .unwrap();
    let before = windmill.clone();
    windmill.set_guards(parts, 0, vec![stack(0, PIKEMAN, 1)], &mut events);
    assert_eq!(windmill, before);
}

/// The battle `outcome` opens.
fn battle(outcome: &AdventureOutcome) -> &BattleStart {
    outcome.battle.as_ref().unwrap()
}

// covers: ADV-MIN-002, GAM-BAT-001
#[test]
fn another_s_guards_are_asked_then_fought_as_a_monster_guards_or_their_owner_s_side() {
    let rules = rules();
    let parts = (&rules.0, &rules.1, &[][..]);
    let swamp = rules.0.terrain_by_key("t:swamp").unwrap();
    // A neutral abandoned mine: its drawn Troglodytes (here peasants) asked first.
    let mut s = game(
        &rules,
        ObjectSetup {
            anchor: ENTRANCE,
            ..abandoned(&rules, ENTRANCE.x, vec![ResourceId::from_raw(0)])
        },
    );
    let guards = s.mine_guards(&rules.1, 0);
    let (outcome, events) = visit(&rules, &mut s);
    let ask = question(&rules, "t:abandoned");
    assert_eq!(
        outcome,
        AdventureOutcome {
            offer: Some(ask.clone()),
            ..AdventureOutcome::default()
        }
    );
    assert!(after_steps(&events).is_empty());
    assert!(s.offer_stands(&ask));
    // Accepted: a single stack fights as a wandering monster, on the mine's terrain.
    let mut events = Vec::new();
    let accepted = s.accept_offer(parts, RED, &ask, &mut events);
    let start = battle(&accepted);
    assert_eq!(
        start.defender,
        Defender::Monster {
            creature: PEASANT,
            count: guards[0].count,
        }
    );
    assert_eq!((start.tile, start.terrain), (ENTRANCE, swamp));
    assert_eq!(start.battlefield, None);
    assert_eq!(start.attacker.hero, HERO);
    assert!(events.is_empty());
    // A neutral sawmill with two stacks: guards, unsplit, on the tile's ground.
    let mut two = game(&rules, mine(&rules, "t:sawmill", None));
    let army = vec![stack(1, PIKEMAN, 3), stack(4, PEASANT, 9)];
    two.set_army_left(0, army.clone());
    visit(&rules, &mut two);
    let accepted = two.accept_offer(parts, RED, &question(&rules, "t:sawmill"), &mut events);
    let start = battle(&accepted);
    assert_eq!(start.defender, Defender::Guards { army: army.clone() });
    assert_eq!(
        (start.terrain, start.battlefield.clone()),
        two.battle_ground(&rules.1, ENTRANCE)
    );
    // Blue's gold mine: its owner's side, a single stack too, on the swamp.
    let mut blue = game(&rules, mine(&rules, "t:gold_mine", Some(1)));
    let one = vec![stack(3, PIKEMAN, 2)];
    blue.set_army_left(0, one.clone());
    let (outcome, _) = visit(&rules, &mut blue);
    let ask = question(&rules, "t:gold_mine");
    assert_eq!(outcome.offer, Some(ask.clone()));
    let accepted = blue.accept_offer(parts, RED, &ask, &mut events);
    let start = battle(&accepted);
    assert_eq!(
        start.defender,
        Defender::Garrison {
            owner: BLUE,
            army: one,
        }
    );
    assert_eq!(start.defender.owner(), Some(BLUE));
    assert_eq!(start.terrain, swamp);
    assert_eq!(blue.owner(0), Some(BLUE));
    // The guards gone while the question was open: accepted, the mine is flagged at once.
    blue.set_army_left(0, vec![]);
    let accepted = blue.accept_offer(parts, RED, &ask, &mut events);
    assert_eq!(accepted, AdventureOutcome::default());
    assert_eq!(blue.owner(0), Some(RED));
    assert!(
        events
            .iter()
            .any(|e| matches!(e.kind, AdventureEventKind::ObjectFlagged { owner: RED, .. }))
    );
    // A defender's owner: a hero's, none for unowned guards.
    assert_eq!(Defender::Guards { army: vec![] }.owner(), None);
    let hero = Defender::Hero(blue.hero(HERO).unwrap().combatant());
    assert_eq!(hero.owner(), Some(RED));
}

// covers: ADV-MIN-002
#[test]
fn a_mine_battle_s_end_keeps_the_survivors_or_flags_the_mine_after_the_levels() {
    let rules = rules();
    let parts = (&rules.0, &rules.1, &[][..]);
    let gold = rules.1.object_type_by_key("t:gold_mine").unwrap();
    // A neutral single stack in slot 2 (A-29 fixed): its survivors, every one counted as its
    // creature, summed into its slot; the word's change told.
    let mut s = game(&rules, mine(&rules, "t:gold_mine", None));
    s.explore_all();
    s.set_army_left(0, vec![stack(2, PEASANT, 150)]);
    visit(&rules, &mut s);
    let left = [stack(0, PEASANT, 30), stack(1, PIKEMAN, 20)];
    let mut events = Vec::new();
    let ended = s.end_unowned_battle(parts, (HERO, ENTRANCE), (&left, false), &mut events);
    assert_eq!(ended, AdventureOutcome::default());
    assert_eq!(s.mine_guards(&rules.1, 0), [stack(2, PEASANT, 50)]);
    assert_eq!(
        events,
        [AdventureEvent {
            kind: AdventureEventKind::GuardsChanged {
                object_type: gold,
                anchor: ENTRANCE,
                creature: Some(PEASANT),
                count_word: Some(4),
            },
            seen_by: Viewers::NONE.with(RED).with(BLUE),
        }]
    );
    assert_eq!(s.owner(0), None);
    // Fewer within the word: nothing told.
    events.clear();
    s.end_unowned_battle(
        parts,
        (HERO, ENTRANCE),
        (&[stack(0, PEASANT, 55)], false),
        &mut events,
    );
    assert_eq!(s.mine_guards(&rules.1, 0), [stack(2, PEASANT, 55)]);
    assert!(events.is_empty());
    // None left without a win (a draw): no guards, no flag; the next visit flags it.
    let mut draw = s.clone();
    let ended = draw.end_unowned_battle(parts, (HERO, ENTRANCE), (&[], false), &mut events);
    assert_eq!(ended, AdventureOutcome::default());
    assert!(draw.mine_guards(&rules.1, 0).is_empty());
    assert_eq!(draw.owner(0), None);
    // Won: the guards gone, the flag after the winner's levels (`give_won`).
    events.clear();
    let ended = s.end_unowned_battle(parts, (HERO, ENTRANCE), (&left, true), &mut events);
    assert_eq!(
        ended,
        AdventureOutcome {
            reward_after: Some((HERO, ENTRANCE)),
            ..AdventureOutcome::default()
        }
    );
    assert!(s.mine_guards(&rules.1, 0).is_empty());
    assert_eq!(s.owner(0), None);
    assert!(matches!(
        events[..],
        [AdventureEvent {
            kind: AdventureEventKind::GuardsChanged {
                count_word: None,
                ..
            },
            ..
        }]
    ));
    events.clear();
    let given = s.give_won(parts, (HERO, ENTRANCE), &mut events);
    assert_eq!(given, AdventureOutcome::default());
    assert_eq!(s.owner(0), Some(RED));
    assert!(matches!(
        events[0].kind,
        AdventureEventKind::ObjectFlagged { owner: RED, .. }
    ));
    assert!(s.check_owners(&rules.1));
    // A hero gone from the map flags nothing.
    let mut gone = game(&rules, mine(&rules, "t:gold_mine", None));
    let before = gone.clone();
    gone.give_won(parts, (HeroId::new(9), ENTRANCE), &mut events);
    assert_eq!(gone, before);
    // An owned mine's guards: their survivors by slot as they stand.
    let mut blue = game(&rules, mine(&rules, "t:gold_mine", Some(1)));
    blue.set_army_left(0, vec![stack(1, PIKEMAN, 3), stack(4, PEASANT, 9)]);
    visit(&rules, &mut blue);
    let survivors = [stack(4, PEASANT, 2)];
    blue.end_unowned_battle(parts, (HERO, ENTRANCE), (&survivors, false), &mut events);
    assert_eq!(blue.mine_guards(&rules.1, 0), survivors);
    assert_eq!(blue.owner(0), Some(BLUE));
    assert!(blue.check_owners(&rules.1));
}
