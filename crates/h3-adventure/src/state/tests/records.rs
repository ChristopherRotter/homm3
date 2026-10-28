//! Tests of Pandora's Box, the Pyramid and the reward record (E08 T62b1; design §11, §11.1,
//! §11.2): the questions, the guards' battles, the record's parts (the hero's half by `grant`, the
//! resources gained and lost, the creatures and the group without room), the removal, a looted
//! Pyramid's luck, its spell after a win, and what its visitor's owner learns.

use std::boxed::Box;
use std::sync::Arc;
use std::vec;
use std::vec::Vec;

use h3_content::SpellId;
use h3_heroes::{HeroChange, HeroDetails, HeroReward, grant};

use super::draws::object;
use super::rewards::{
    AT, BLUE, DOOM, GOLD, GUST, HERO, RED, RING, Rules, WOOD, draw, gives, hero, map_of, one,
    rules, store, type_of, visit, with_book,
};
use super::*;
use crate::command::{Defender, HeroGains, NoRoom};
use crate::observe::{Known, KnownObject};
use crate::setup::{Guards, Holds, RewardRecord};
use crate::state::Resolved;
use crate::testing::{LOGISTICS, PEASANT, PIKEMAN, hero_at, started, world};

/// A box at [`AT`] holding `record`, guarded by `guards` when given.
fn boxed(rules: &Rules, record: RewardRecord, guards: Option<Vec<ArmyStack>>) -> AdventureState {
    let mut box_ = object(rules, AT.x, "t:box", Holds::Record(Box::new(record)));
    box_.guards = guards.map(|army| Guards {
        army,
        question: false,
    });
    map_of(rules, vec![box_])
}

/// `count` of `creature` in `slot`.
fn stack(slot: u8, creature: CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

/// The offer of `kind` the object of type `k` at [`AT`] makes hero 1.
fn offer(rules: &Rules, k: &str, kind: OfferKind) -> Offer {
    Offer {
        hero: HERO,
        object_type: type_of(rules, k),
        anchor: AT,
        kind,
    }
}

/// Red accepts `offer`: the outcome and the events.
fn accept(
    rules: &Rules,
    state: &mut AdventureState,
    heroes: &[Arc<HeroDetails>],
    offer: &Offer,
) -> (AdventureOutcome, Vec<AdventureEvent>) {
    let mut events = Vec::new();
    let outcome = state.accept_offer((&rules.0, &rules.1, heroes), RED, offer, &mut events);
    (outcome, events)
}

/// A record giving experience, primary skills (a negative one skipped), Logistics, morale, the
/// ring, the gust, 500 gold, taking 5 wood, and 5 peasants in slot 0 and 7 pikemen in slot 3.
fn record() -> RewardRecord {
    RewardRecord {
        hero: HeroReward {
            experience: 1000,
            primary: [1, 0, 0, -1],
            skills: vec![(LOGISTICS, 2)],
            mana: 0,
            morale: 1,
            luck: 0,
            artifacts: vec![RING],
            spells: vec![GUST],
        },
        resources: vec![(WOOD, -5), (GOLD, 500)],
        creatures: vec![stack(0, PEASANT, 5), stack(3, PIKEMAN, 7)],
    }
}

/// The events told of creatures joining hero 1.
fn joined(events: &[AdventureEvent]) -> Vec<(CreatureTypeId, u32)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            AdventureEventKind::CreaturesJoined {
                hero,
                creature,
                count,
            } => {
                assert_eq!((hero, e.seen_by), (HERO, Viewers::NONE.with(RED)));
                Some((creature, count))
            }
            _ => None,
        })
        .collect()
}

// covers: ADV-PIK-013, ADV-RWD-001, ADV-OBJ-005
#[test]
fn a_box_asks_first_and_its_record_is_given_then_it_leaves() {
    for faithful in [true, false] {
        let rules = rules(faithful);
        let open = offer(&rules, "t:box", OfferKind::OpenBox);
        let details = with_book(hero());
        let heroes = store(details.clone());
        let mut s = boxed(&rules, record(), None);
        // The question, at every visit; declined, nothing changes.
        assert_eq!(
            visit(&rules, &mut s, &heroes),
            AdventureOutcome {
                offer: Some(open.clone()),
                ..AdventureOutcome::default()
            }
        );
        let before = s.clone();
        let mut events = Vec::new();
        let declined = s.decline_offer((&rules.0, &rules.1, &heroes), &open, &mut events);
        assert_eq!((declined, events.len()), (AdventureOutcome::default(), 0));
        assert_eq!(s, before);
        // Accepted: the hero's half by `grant` (the experience for the game to give last), the
        // gold a gain and the wood a loss, the creatures onto the army, the box gone.
        let (outcome, events) = accept(&rules, &mut s, &heroes, &open);
        let granted = grant(&details, &rules.0, &record().hero);
        assert!(
            granted
                .changes
                .contains(&HeroChange::AddArtifact(RING.into()))
        );
        assert_eq!(
            outcome,
            AdventureOutcome {
                gains: vec![(GOLD, 500)],
                losses: vec![(WOOD, 5)],
                hero: Some(HeroGains {
                    hero: HERO,
                    changes: granted.changes,
                    experience: granted.experience,
                }),
                ..AdventureOutcome::default()
            }
        );
        assert_eq!(joined(&events), [(PEASANT, 5), (PIKEMAN, 7)]);
        let army = &s.hero(HERO).unwrap().army;
        assert_eq!(army, &[stack(0, PEASANT, 6), stack(1, PIKEMAN, 7)]);
        assert!(s.is_removed(0));
        assert!(matches!(
            events.last().unwrap().kind,
            AdventureEventKind::ObjectRemoved { anchor: AT, .. }
        ));
        // Gone: an accepted question gives nothing.
        assert_eq!(
            accept(&rules, &mut s, &heroes, &open).0,
            AdventureOutcome::default()
        );
    }
}

// covers: ADV-RWD-001, GAM-DEC-004
#[test]
fn a_records_creatures_without_room_are_one_group_at_their_slots() {
    let rules = rules(true);
    // A full army of seven pikemen's stacks: the record's pikemen join the first, its peasants
    // and its other creature find no room.
    let mut setup = world(
        &["gggggggggggggggg", "gggggggggggggggg"],
        1,
        vec![hero_at(1, 0, Pos::new(1, 1, 0), &[PIKEMAN; 7])],
    );
    let crowded = RewardRecord {
        creatures: vec![
            stack(1, PEASANT, 5),
            stack(2, PIKEMAN, 3),
            stack(6, crate::testing::WOLF, 2),
        ],
        ..RewardRecord::default()
    };
    setup.map.objects.push(object(
        &rules,
        AT.x,
        "t:box",
        Holds::Record(Box::new(crowded)),
    ));
    let mut s = started(&rules, &setup).unwrap();
    let open = offer(&rules, "t:box", OfferKind::OpenBox);
    let (outcome, events) = accept(&rules, &mut s, &store(hero()), &open);
    assert_eq!(joined(&events), [(PIKEMAN, 3)]);
    assert_eq!(s.hero(HERO).unwrap().army[0], stack(0, PIKEMAN, 4));
    assert_eq!(
        outcome,
        AdventureOutcome {
            no_room: Some(NoRoom {
                hero: HERO,
                group: vec![stack(1, PEASANT, 5), stack(6, crate::testing::WOLF, 2)],
            }),
            ..AdventureOutcome::default()
        }
    );
    // An empty record gives nothing; the box still leaves.
    let mut s = boxed(&rules, RewardRecord::default(), None);
    let (outcome, _) = accept(&rules, &mut s, &store(hero()), &open);
    assert_eq!(outcome, AdventureOutcome::default());
    assert!(s.is_removed(0));
    // A hero without details gets no hero half, its creatures still.
    let mut s = boxed(&rules, record(), None);
    let (outcome, events) = accept(&rules, &mut s, &[], &open);
    assert_eq!((outcome.hero, joined(&events).len()), (None, 2));
    // A hero no longer on the map gets no creature; the gains still go to the player.
    let mut s = boxed(&rules, record(), None);
    let gone = Offer {
        hero: HeroId::new(9),
        ..open.clone()
    };
    let (outcome, events) = accept(&rules, &mut s, &[], &gone);
    assert_eq!(
        (outcome.gains, joined(&events).len()),
        (vec![(GOLD, 500)], 0)
    );
    assert!(s.is_removed(0));
    // A box without its record (the setup gives every box one) gives nothing and leaves.
    let mut s = boxed(&rules, record(), None);
    Arc::make_mut(&mut s.map).objects[0].holds = Holds::Nothing;
    assert_eq!(
        accept(&rules, &mut s, &[], &open).0,
        AdventureOutcome::default()
    );
    assert!(s.is_removed(0));
}

// covers: ADV-PIK-013, ADV-PIK-004
#[test]
fn a_guarded_box_fights_at_once_and_gives_its_record_after_a_win() {
    let rules = rules(true);
    let guards = vec![stack(0, PIKEMAN, 4), stack(2, PEASANT, 9)];
    let open = offer(&rules, "t:box", OfferKind::OpenBox);
    let heroes = store(with_book(hero()));
    let mut s = boxed(&rules, record(), Some(guards.clone()));
    // Still the question first; accepted, the battle at the box's tile against the guards.
    assert_eq!(visit(&rules, &mut s, &heroes).offer, Some(open.clone()));
    let (outcome, events) = accept(&rules, &mut s, &heroes, &open);
    assert!(events.is_empty());
    let battle = outcome.battle.unwrap();
    assert_eq!(
        (battle.defender, battle.tile, battle.attacker.hero),
        (Defender::Guards { army: guards }, AT, HERO)
    );
    let ended = |s: &mut AdventureState, survivors: &[ArmyStack], won| {
        let mut events = Vec::new();
        let rules = (&rules.0, &rules.1, &heroes[..]);
        let outcome = s.end_unowned_battle(rules, (HERO, AT), (survivors, won), &mut events);
        (outcome, events)
    };
    // Lost: the survivors stay as its guards, the box with them.
    let left = [stack(2, PEASANT, 3)];
    let (outcome, _) = ended(&mut s, &left, false);
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!((s.guards_left(0), s.is_removed(0)), (&left[..], false));
    // Won: after the winner's levels (E08 T63b, GAM-DEC-007: `give_won`) the record, the box
    // gone.
    let mut won = s.clone();
    let (outcome, events) = ended(&mut won, &[], true);
    assert_eq!((outcome.reward_after, events.len()), (Some((HERO, AT)), 0));
    assert!(!won.is_removed(0));
    let mut events = Vec::new();
    let context = (&rules.0, &rules.1, &heroes[..]);
    let outcome = won.give_won(context, (HERO, AT), &mut events);
    assert_eq!(outcome.gains, [(GOLD, 500)]);
    assert_eq!(joined(&events).len(), 2);
    assert!(won.is_removed(0));
    // Nothing visited at the battle's tile: nothing.
    let mut events = Vec::new();
    let elsewhere = (HERO, Pos::new(5, 0, 0));
    let context = (&rules.0, &rules.1, &heroes[..]);
    let none = s.end_unowned_battle(context, elsewhere, (&[], true), &mut events);
    assert_eq!((none, events.len()), (AdventureOutcome::default(), 0));
    // A draw leaves none: the next accepted question opens it without a battle.
    let (_, _) = ended(&mut s, &[], false);
    let (outcome, _) = accept(&rules, &mut s, &heroes, &open);
    assert_eq!((outcome.battle, outcome.losses), (None, vec![(WOOD, 5)]));
    assert!(s.is_removed(0));
}

/// The Pyramid at [`AT`] holding `spell`.
fn pyramid(rules: &Rules, spell: Option<SpellId>) -> AdventureState {
    let mut s = one(rules, "t:pyramid", Holds::Nothing);
    let resolved = spell.map_or(Resolved::Nothing, Resolved::Spell);
    draw(&mut s, 0, resolved, &[]);
    s
}

// covers: ADV-TCH-008, ADV-MON-012, ADV-VIS-006
#[test]
fn the_pyramid_asks_at_every_visit_and_fights_with_a_second_creature() {
    let rules = rules(true);
    let search = offer(&rules, "t:pyramid", OfferKind::SearchPyramid);
    // The test rules teach spells up to level 2 to a hero with a book.
    let heroes = store(with_book(hero()));
    let mut s = pyramid(&rules, Some(GUST));
    assert_eq!(visit(&rules, &mut s, &heroes).offer, Some(search.clone()));
    // Declined: nothing, and red does not learn it.
    let mut events = Vec::new();
    let declined = s.decline_offer((&rules.0, &rules.1, &heroes), &search, &mut events);
    assert_eq!(declined, AdventureOutcome::default());
    assert!(!s.knows(RED, 0));
    // Accepted: red learns it; the battle against its first creature, split, and its second's
    // stacks, at its tile.
    let (outcome, _) = accept(&rules, &mut s, &heroes, &search);
    assert!(s.knows(RED, 0) && !s.knows(BLUE, 0));
    let battle = outcome.battle.unwrap();
    assert_eq!(
        (battle.defender, battle.tile),
        (
            Defender::Monsters {
                split: (PIKEMAN, 40),
                fixed: (PEASANT, vec![10, 10]),
            },
            AT
        )
    );
    // Lost: nothing changes (its losses are not kept); asked again at the next visit.
    let mut events = Vec::new();
    let context = (&rules.0, &rules.1, &heroes[..]);
    let survivors = [stack(0, PIKEMAN, 3)];
    let lost = s.end_unowned_battle(context, (HERO, AT), (&survivors, false), &mut events);
    assert_eq!(
        (lost, s.is_emptied(0)),
        (AdventureOutcome::default(), false)
    );
    assert_eq!(s.army_left(0), None);
    assert_eq!(visit(&rules, &mut s, &heroes).offer, Some(search.clone()));
    // Won: after the winner's levels (E08 T63b, GAM-DEC-007: `give_won`) looted, and its spell
    // taught (a book, its level within the hero's); a second `give_won` finds it looted.
    let won = s.end_unowned_battle(context, (HERO, AT), (&[], true), &mut events);
    assert_eq!(
        (won.reward_after, s.is_emptied(0)),
        (Some((HERO, AT)), false)
    );
    let won = s.give_won(context, (HERO, AT), &mut events);
    assert_eq!(won, gives(&[HeroChange::LearnSpell(GUST)]));
    assert!(s.is_emptied(0) && !s.is_removed(0));
    let again = s.give_won(context, (HERO, AT), &mut events);
    assert_eq!((again, events.len()), (AdventureOutcome::default(), 0));
    // Looted: still asked; accepted, −2 luck once by its mark (the hero model gates it).
    assert_eq!(visit(&rules, &mut s, &heroes).offer, Some(search.clone()));
    let looted = HeroChange::Bonus {
        morale: 0,
        luck: -2,
        mark: Some(1),
    };
    assert_eq!(accept(&rules, &mut s, &heroes, &search).0, gives(&[looted]));
    // Its owner sees it looted.
    let seen = s.observe(&rules.0, &rules.1, RED).known;
    assert_eq!(
        seen,
        [KnownObject {
            object_type: search.object_type,
            anchor: AT,
            known: Known::Emptied(true),
        }]
    );
}

// covers: ADV-TCH-008
#[test]
fn a_won_pyramids_spell_is_lost_to_a_hero_who_cannot_learn_it() {
    let rules = rules(true);
    let context = |spell, heroes: &[Arc<HeroDetails>]| -> Vec<HeroChange> {
        let mut s = pyramid(&rules, Some(spell));
        let outcome = s.give_won((&rules.0, &rules.1, heroes), (HERO, AT), &mut Vec::new());
        assert!(s.is_emptied(0), "looted either way");
        outcome.hero.map(|g| g.changes).unwrap_or_default()
    };
    // A level above the hero's Wisdom allows, no book, no details: lost.
    assert!(context(DOOM, &store(with_book(hero()))).is_empty());
    assert!(context(GUST, &store(hero())).is_empty());
    assert!(context(GUST, &[]).is_empty());
    // Known already: no change (relearning changes nothing).
    let known = HeroDetails {
        spells: vec![GUST],
        ..with_book(hero())
    };
    assert!(context(GUST, &store(known)).is_empty());
    // A Pyramid without a spell (A-20 fixed): nothing.
    let mut s = pyramid(&rules, None);
    let outcome = s.give_won(
        (&rules.0, &rules.1, &store(with_book(hero()))),
        (HERO, AT),
        &mut Vec::new(),
    );
    assert_eq!(outcome, AdventureOutcome::default());
    // A question whose Pyramid is gone, or a type that is no Pyramid: nothing.
    let gone = Offer {
        anchor: Pos::new(9, 0, 0),
        ..offer(&rules, "t:pyramid", OfferKind::SearchPyramid)
    };
    let mut s = pyramid(&rules, Some(DOOM));
    assert_eq!(
        accept(&rules, &mut s, &[], &gone).0,
        AdventureOutcome::default()
    );
    let wrong = offer(&rules, "t:box", OfferKind::SearchPyramid);
    assert_eq!(
        accept(&rules, &mut s, &[], &wrong).0,
        AdventureOutcome::default()
    );
    assert!(!s.knows(RED, 0));
}

// covers: ADV-RWD-001
#[test]
fn every_box_holds_its_own_record() {
    // A record per object, with no bound on their number (A-19 not reproduced): the second box
    // gives its own record, not the first one's.
    let rules = rules(true);
    let gold = |amount| RewardRecord {
        resources: vec![(GOLD, amount)],
        ..RewardRecord::default()
    };
    let far = Pos::new(4, 0, 0);
    let mut s = map_of(
        &rules,
        vec![
            object(&rules, AT.x, "t:box", Holds::Record(Box::new(gold(100)))),
            object(&rules, far.x, "t:box", Holds::Record(Box::new(gold(200)))),
        ],
    );
    let open = offer(&rules, "t:box", OfferKind::OpenBox);
    let other = Offer {
        anchor: far,
        ..open.clone()
    };
    assert_eq!(accept(&rules, &mut s, &[], &other).0.gains, [(GOLD, 200)]);
    assert_eq!(accept(&rules, &mut s, &[], &open).0.gains, [(GOLD, 100)]);
    assert!(s.is_removed(0) && s.is_removed(1));
}
