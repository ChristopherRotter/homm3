//! Tests of the hire dialog's map side (E08 T95b; increment 10's design note §8, §13): the source's
//! stock (a dwelling's creatures and counts, a camp's, none for another object), the room's bound
//! (a free slot, the room on the stacks of the creature, none), the price in the core's resource
//! order, a purchase at a dwelling of two creatures and at a camp (the count lowered, the creatures
//! joined where ADV-ARM-007's room is, the owner told) and each refusal in order, leaving the state
//! as it was.

use std::string::ToString;

use h3_content::ResourceId;

use super::dwelling_visits::{
    BLUE, ENTRANCE, HERO, RED, accept, at_entrance, creature, game, hire as question, peasant,
    stack, to_red, visit,
};
use super::dwellings::{Rules, rules, type_of};
use super::*;
use crate::command::{HireDialog, HireError, HireSource, HireTarget};
use crate::testing::{MAX_STACK, PEASANT};

const WOOD: ResourceId = ResourceId::from_raw(0);
const GOLD: ResourceId = ResourceId::from_raw(1);

/// The dialog of `hero` for the object of type `k` at object index 0 on [`ENTRANCE`].
fn dialog(rules: &Rules, hero: HeroId, k: &str) -> HireDialog {
    HireDialog {
        target: HireTarget::Hero(hero),
        source: HireSource::Object {
            index: 0,
            object_type: type_of(rules, k),
            anchor: ENTRANCE,
        },
    }
}

/// A purchase's price, or why it was refused.
type Bought = Result<Vec<(ResourceId, u32)>, HireError>;

/// A purchase of `amount` of `k` in `dialog`: the result and the events.
fn buy(
    rules: &Rules,
    state: &mut AdventureState,
    dialog: &HireDialog,
    (k, amount): (&str, u32),
) -> (Bought, Vec<AdventureEvent>) {
    let mut events = Vec::new();
    let bought = state
        .check_hire((&rules.0, &rules.1), dialog, (creature(rules, k), amount))
        .map(|purchase| {
            let price = purchase.price().to_vec();
            state.hire(purchase, &mut events);
            price
        });
    (bought, events)
}

/// Seven full slots of `t:f2`, `k`'s stacks of these counts in the first slots.
fn full_army(rules: &Rules, k: &str, counts: &[u32]) -> Vec<ArmyStack> {
    (0..7)
        .map(|slot| match counts.get(usize::from(slot)) {
            Some(&count) => stack(slot, creature(rules, k), count),
            None => stack(slot, creature(rules, "t:f2"), 1),
        })
        .collect()
}

// covers: ADV-HIR-002, ADV-HIR-001, ADV-RFG-001
#[test]
fn an_accepted_question_opens_the_dialog_above_free_level_and_at_a_camp() {
    let rules = rules();
    // Red's own level-5 dwelling of two creatures: nothing free, the dialog for its hero.
    let mut s = game(&rules, at_entrance(&rules, "t:d5", Some(RED)), peasant());
    visit(&rules, &mut s);
    let (outcome, events) = accept(&rules, &mut s, &question(&rules, "t:d5", &["t:c5", "t:w5"]));
    assert_eq!(
        outcome,
        AdventureOutcome {
            hire: Some(dialog(&rules, HERO, "t:d5")),
            ..AdventureOutcome::default()
        }
    );
    assert!(events.is_empty());
    // A level-1 one: its free creatures, no dialog (no class-17 dwelling of level 1 opens it).
    let mut s = game(&rules, at_entrance(&rules, "t:d1", Some(RED)), peasant());
    visit(&rules, &mut s);
    let (outcome, events) = accept(&rules, &mut s, &question(&rules, "t:d1", &["t:c1", "t:w1"]));
    assert_eq!(outcome.hire, None);
    assert_eq!(events.len(), 2);
    // Blue's when red accepts: no dialog.
    let mut s = game(&rules, at_entrance(&rules, "t:d5", Some(BLUE)), peasant());
    let (outcome, _) = accept(&rules, &mut s, &question(&rules, "t:d5", &["t:c5", "t:w5"]));
    assert_eq!(outcome, AdventureOutcome::default());
    // A camp: the dialog, nothing free.
    let mut s = game(&rules, at_entrance(&rules, "t:camp", None), peasant());
    let (drawn, _) = s.camp(0).unwrap();
    let before = s.clone();
    let (outcome, events) = accept(&rules, &mut s, &question_of_camp(&rules, drawn));
    assert_eq!(outcome.hire, Some(dialog(&rules, HERO, "t:camp")));
    assert!(events.is_empty());
    assert_eq!(s, before);
}

/// The camp's hire question naming `drawn`.
fn question_of_camp(rules: &Rules, drawn: CreatureTypeId) -> Offer {
    Offer {
        kind: OfferKind::Hire {
            creatures: vec![drawn],
        },
        ..question(rules, "t:camp", &[])
    }
}

// covers: ADV-HIR-002
#[test]
fn the_stock_is_a_dwelling_s_creatures_or_a_camp_s_and_none_for_another_object() {
    let rules = rules();
    let (c5, w5) = (creature(&rules, "t:c5"), creature(&rules, "t:w5"));
    let mut s = game(&rules, at_entrance(&rules, "t:d5", Some(RED)), peasant());
    let source = dialog(&rules, HERO, "t:d5").source;
    assert_eq!(
        s.hire_stock(&rules.1, &source),
        Some(vec![(c5, 2), (w5, 2)])
    );
    s.set_dwelling(0, &[2, 1], &[]);
    assert_eq!(
        s.hire_stock(&rules.1, &source),
        Some(vec![(c5, 2), (w5, 1)])
    );
    // Another anchor, another type, an index past the objects: none.
    let HireSource::Object {
        index, object_type, ..
    } = source;
    let moved = HireSource::Object {
        index,
        object_type,
        anchor: Pos::new(3, 7, 0),
    };
    let typed = dialog(&rules, HERO, "t:d1").source;
    let past = HireSource::Object {
        index: 5,
        object_type,
        anchor: ENTRANCE,
    };
    for other in [moved, typed, past] {
        assert_eq!(s.hire_stock(&rules.1, &other), None);
    }
    // A camp's creature and count; a Windmill sells nothing.
    let s = game(&rules, at_entrance(&rules, "t:camp", None), peasant());
    let source = dialog(&rules, HERO, "t:camp").source;
    assert_eq!(
        s.hire_stock(&rules.1, &source),
        Some(vec![s.camp(0).unwrap()])
    );
    let s = game(&rules, at_entrance(&rules, "t:windmill", None), peasant());
    let source = dialog(&rules, HERO, "t:windmill").source;
    assert_eq!(s.hire_stock(&rules.1, &source), None);
}

// covers: ADV-ARM-007, ADV-HIR-002
#[test]
fn the_room_s_bound_is_a_stack_s_most_with_a_slot_free_else_the_room_on_its_stacks() {
    let rules = rules();
    let c5 = creature(&rules, "t:c5");
    let object = || at_entrance(&rules, "t:d5", Some(RED));
    let room = |s: &AdventureState, hero| s.hire_room(&rules.1, &HireTarget::Hero(hero), c5);
    // A slot free: a stack's most, whatever stack of the creature it has.
    let s = game(
        &rules,
        object(),
        vec![stack(0, PEASANT, 1), stack(4, c5, 9)],
    );
    assert_eq!(room(&s, HERO), MAX_STACK);
    // Seven slots full: the most less the smallest stack of the creature, else 0.
    let s = game(
        &rules,
        object(),
        full_army(&rules, "t:c5", &[100, 7, MAX_STACK]),
    );
    assert_eq!(room(&s, HERO), MAX_STACK - 7);
    let s = game(&rules, object(), full_army(&rules, "t:c5", &[MAX_STACK]));
    assert_eq!(room(&s, HERO), 0);
    let s = game(&rules, object(), full_army(&rules, "t:c5", &[]));
    assert_eq!(room(&s, HERO), 0);
    // A hero not on the map has none.
    assert_eq!(room(&s, HeroId::new(9)), 0);
}

// covers: ADV-HIR-002, ADV-ARM-007
#[test]
fn a_purchase_lowers_the_count_joins_the_army_and_tells_the_owner_alone() {
    let rules = rules();
    let (c5, w5) = (creature(&rules, "t:c5"), creature(&rules, "t:w5"));
    let mut s = game(&rules, at_entrance(&rules, "t:d5", Some(RED)), peasant());
    visit(&rules, &mut s);
    let (outcome, _) = accept(&rules, &mut s, &question(&rules, "t:d5", &["t:c5", "t:w5"]));
    let open = outcome.hire.unwrap();
    let visited = s.clone();
    // The price by resource in the core's order (wood, then gold), whatever the pack's order.
    let check = |s: &AdventureState, k, amount| {
        s.check_hire((&rules.0, &rules.1), &open, (creature(&rules, k), amount))
            .map(|purchase| purchase.price().to_vec())
    };
    assert_eq!(check(&s, "t:w5", 2), Ok(vec![(WOOD, 8), (GOLD, 100)]));
    assert_eq!(check(&s, "t:c5", 1), Ok(vec![(GOLD, 50)]));
    // The second creature of two: into the first free slot, its count lowered, red told.
    let (bought, events) = buy(&rules, &mut s, &open, ("t:w5", 1));
    assert_eq!(bought, Ok(vec![(WOOD, 4), (GOLD, 50)]));
    assert_eq!(
        events,
        [to_red(AdventureEventKind::CreaturesJoined {
            hero: HERO,
            creature: w5,
            count: 1,
        })]
    );
    assert_eq!(
        s.hero(HERO).unwrap().army,
        [stack(0, PEASANT, 1), stack(1, w5, 1)]
    );
    assert_eq!(s.dwelling_counts(0), Some(&[2, 1][..]));
    assert!(s.check_dwellings());
    assert_ne!(s.state_hash(), visited.state_hash());
    // The fork made before keeps the dwelling full: the write copied the block.
    assert!(visited.dwellings.is_none());
    // Again: onto the stack of the creature; the last of it bought, then none to buy.
    buy(&rules, &mut s, &open, ("t:w5", 1)).0.unwrap();
    assert_eq!(
        s.hero(HERO).unwrap().army,
        [stack(0, PEASANT, 1), stack(1, w5, 2)]
    );
    assert_eq!(
        s.hire_stock(&rules.1, &open.source),
        Some(vec![(c5, 2), (w5, 0)])
    );
    assert_eq!(check(&s, "t:w5", 1), Err(HireError::Amount(1)));
    // A creature costing 0 of a resource: that resource left out of the price.
    let level_1 = dialog(&rules, HERO, "t:d1");
    let s = game(&rules, at_entrance(&rules, "t:d1", Some(RED)), peasant());
    let price = s.check_hire(
        (&rules.0, &rules.1),
        &level_1,
        (creature(&rules, "t:w1"), 2),
    );
    assert_eq!(price.map(|p| p.price().to_vec()), Ok(vec![(GOLD, 20)]));
    assert_eq!(
        hire_price(&rules.0, creature(&rules, "t:w1"), 1),
        [(GOLD, 10)]
    );
}

// covers: ADV-HIR-002, ADV-RFG-001
#[test]
fn a_camp_s_purchase_lowers_its_count() {
    let rules = rules();
    let c3 = creature(&rules, "t:c3");
    let mut s = game(&rules, at_entrance(&rules, "t:camp", None), peasant());
    s.set_camp(0, c3, 2);
    let before = s.clone();
    let open = dialog(&rules, HERO, "t:camp");
    let (bought, events) = buy(&rules, &mut s, &open, ("t:c3", 2));
    assert_eq!(bought, Ok(vec![(GOLD, 60)]));
    assert_eq!(
        events,
        [to_red(AdventureEventKind::CreaturesJoined {
            hero: HERO,
            creature: c3,
            count: 2,
        })]
    );
    assert_eq!(s.camp(0), Some((c3, 0)));
    assert_eq!(before.camp(0), Some((c3, 2)));
    assert_eq!(
        s.hero(HERO).unwrap().army,
        [stack(0, PEASANT, 1), stack(1, c3, 2)]
    );
    assert!(s.check_camps(&rules.0, &rules.1));
    assert_ne!(s.state_hash(), before.state_hash());
}

// covers: ADV-HIR-002, ADV-ARM-007
#[test]
fn each_refused_purchase_leaves_the_state_as_it_was() {
    let rules = rules();
    let open = dialog(&rules, HERO, "t:d5");
    let HireSource::Object {
        index, object_type, ..
    } = open.source;
    let moved = HireDialog {
        source: HireSource::Object {
            index,
            object_type,
            anchor: Pos::new(3, 7, 0),
        },
        ..open
    };
    // A stack of the castle's level 5 one short of a stack's most, every slot taken.
    let mut army = full_army(&rules, "t:c5", &[MAX_STACK - 1]);
    let s = game(&rules, at_entrance(&rules, "t:d5", Some(RED)), army.clone());
    let c1 = creature(&rules, "t:c1");
    for (dialog, (k, amount), error) in [
        (
            dialog(&rules, HeroId::new(9), "t:d5"),
            ("t:c5", 1),
            HireError::NoSource,
        ),
        (moved, ("t:c5", 1), HireError::NoSource),
        // The creature before the amount, the amount before the room.
        (open, ("t:c1", 0), HireError::NotOffered(c1)),
        (open, ("t:c5", 0), HireError::Amount(0)),
        (open, ("t:w5", 3), HireError::Amount(3)),
        (open, ("t:c5", 2), HireError::NoRoom),
        (open, ("t:w5", 1), HireError::NoRoom),
    ] {
        let check = s.check_hire((&rules.0, &rules.1), &dialog, (creature(&rules, k), amount));
        assert_eq!(check, Err(error));
        let mut refused = s.clone();
        let (bought, events) = buy(&rules, &mut refused, &dialog, (k, amount));
        assert_eq!(bought, Err(error));
        assert!(events.is_empty());
        assert_eq!(refused, s);
        assert_eq!(refused.state_hash(), s.state_hash());
    }
    // One fits the stack: bought, the stack at its most.
    let mut s = s;
    buy(&rules, &mut s, &open, ("t:c5", 1)).0.unwrap();
    army[0].count = MAX_STACK;
    assert_eq!(s.hero(HERO).unwrap().army, army);
    // Each refusal reads.
    let texts = [
        HireError::NoSource,
        HireError::NotOffered(c1),
        HireError::Amount(0),
        HireError::NoRoom,
    ]
    .map(|error| error.to_string());
    assert_eq!(
        texts,
        [
            "the hire dialog's hero or source is not on the map".to_string(),
            std::format!("creature {} is not the source's", c1.raw()),
            "0 is not an amount the source holds".to_string(),
            "the army has no room for the creatures".to_string(),
        ]
    );
}
