//! Tests of the guarded piles (E08 T21a, ADV-PIK-004): their guards in the setup and the hash,
//! the question or the battle at once, the battle's end and the pickup after it.

use h3_content::{ContentPack, CreatureDef, CreatureInfoDef};

use std::string::ToString;

use super::*;
use crate::command::{AdventureCommand, AdventureEvent};
use crate::setup::MonsterSetup;
use crate::testing::{
    ANY_PILE, FIRE, GNOLL, GOLD_PILE, MONSTER, PEASANT, PIKEMAN, hero_at, key,
    movement_rules_edited, world,
};

const RED: PlayerId = PlayerId::new(0);
const HERO: HeroId = HeroId::new(1);
const GOLD: ResourceId = ResourceId::from_raw(1);
const CART: CreatureTypeId = CreatureTypeId::from_raw(4);
const TILE: Pos = Pos::new(2, 0, 0);

type Rules = (Ruleset, AdventureRuleset);

/// [`crate::testing::movement_rules`] (faithful) with a war machine `t:cart`.
fn rules() -> Rules {
    movement_rules_edited(true, 16, |core: &mut ContentPack| {
        core.creatures.push(CreatureDef {
            key: key("t:cart"),
            speed: 0,
            info: CreatureInfoDef {
                flags: vec!["war_machine".to_string()],
                ..CreatureInfoDef::default()
            },
        });
    })
}

fn stack(slot: u8, creature: CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

/// Guards of `army` that ask first when `question`.
fn guards(army: Vec<ArmyStack>, question: bool) -> Option<Guards> {
    Some(Guards { army, question })
}

/// An object of `object_type` at `(x, 0)`, blocking and visited there, a gold pile's amount 7
/// (700), with these guards.
fn pile_at(x: u16, object_type: ObjectTypeId, guards: Option<Guards>) -> ObjectSetup {
    ObjectSetup {
        scenery: Default::default(),
        anchor: Pos::new(x, 0, 0),
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: Some(object_type),
        amount: (object_type == GOLD_PILE).then_some(7),
        drawn: TileMask::default(),
        monster: None,
        guards,
        holds: Default::default(),
    }
}

/// The setup of three grass rows with these objects, red's hero 1 at (1, 1) with 100 peasants.
fn setup(objects: Vec<ObjectSetup>) -> AdventureSetup {
    let mut setup = world(
        &["gggggg", "gggggg", "gggggg"],
        1,
        vec![hero_at(1, 0, Pos::new(1, 1, 0), &[PEASANT])],
    );
    setup.heroes[0].army = vec![stack(0, PEASANT, 100)];
    setup.map.objects.extend(objects);
    setup
}

/// The state of [`setup`].
fn state(rules: &Rules, objects: Vec<ObjectSetup>) -> AdventureState {
    crate::testing::started(rules, &setup(objects)).unwrap()
}

/// A gold pile at [`TILE`] guarded by two pikemen in slot 1 and five gnolls in slot 4.
fn guarded(rules: &Rules, question: bool) -> AdventureState {
    let army = vec![stack(4, GNOLL, 5), stack(1, PIKEMAN, 2)];
    state(rules, vec![pile_at(2, GOLD_PILE, guards(army, question))])
}

/// The guards of [`guarded`] in slot order.
fn the_guards() -> Vec<ArmyStack> {
    vec![stack(1, PIKEMAN, 2), stack(4, GNOLL, 5)]
}

/// [`HERO`]'s step toward the pile: the outcome and the events.
fn visit(rules: &Rules, state: &mut AdventureState) -> (AdventureOutcome, Vec<AdventureEvent>) {
    let mut events = Vec::new();
    let step = AdventureCommand::MoveHero {
        hero: HERO,
        path: vec![TILE],
    };
    let outcome = state
        .apply(&rules.0, &rules.1, &[], RED, &step, &mut events)
        .unwrap();
    (outcome, events)
}

/// The pile's question to [`HERO`].
fn question() -> Offer {
    Offer {
        hero: HERO,
        object_type: GOLD_PILE,
        anchor: TILE,
        kind: OfferKind::Fight,
    }
}

// covers: ADV-PIK-004
#[test]
fn guards_stand_on_piles_only_as_a_valid_army() {
    let rules = rules();
    // Kept by slot, on a pile or a random pile.
    let s = guarded(&rules, true);
    assert_eq!(s.map().objects()[0].guards, guards(the_guards(), true));
    assert_eq!(s.guards_left(0), &the_guards()[..]);
    let random = state(
        &rules,
        vec![pile_at(2, ANY_PILE, guards(the_guards(), false))],
    );
    assert_eq!(random.guards_left(0), &the_guards()[..]);
    // Refused: on another kind, or an invalid army (as a hero's), or a war machine among them.
    let refused = |object: ObjectSetup| {
        let result = crate::testing::started(&rules, &setup(vec![object]));
        assert_eq!(result.err(), Some(SetupError::Guards(0)));
    };
    refused(pile_at(2, FIRE, guards(the_guards(), true)));
    refused(ObjectSetup {
        scenery: Default::default(),
        monster: Some(MonsterSetup {
            creature: Some(PIKEMAN),
            count: 5,
            disposition: 2,
            never_flees: false,
            no_growth: false,
            treasure: vec![],
            artifact: None,
        }),
        ..pile_at(2, MONSTER, guards(the_guards(), true))
    });
    for army in [
        vec![],
        vec![stack(7, PIKEMAN, 1)],
        vec![stack(1, PIKEMAN, 1), stack(1, GNOLL, 1)],
        vec![stack(0, PIKEMAN, 0)],
        vec![stack(0, CreatureTypeId::from_raw(99), 1)],
        vec![stack(0, PIKEMAN, 1), stack(1, CART, 1)],
    ] {
        refused(pile_at(2, GOLD_PILE, guards(army, false)));
    }
    assert_eq!(
        SetupError::Guards(1).to_string(),
        "object 1 has invalid guards"
    );
    // The hash covers them: their question and their stacks.
    let asks = guarded(&rules, true);
    let fights = guarded(&rules, false);
    let fewer = state(
        &rules,
        vec![pile_at(
            2,
            GOLD_PILE,
            guards(vec![stack(1, PIKEMAN, 1)], true),
        )],
    );
    let plain = state(&rules, vec![pile_at(2, GOLD_PILE, None)]);
    let hashes = [&asks, &fights, &fewer, &plain].map(|s| s.map().content_hash());
    for (i, a) in hashes.iter().enumerate() {
        assert!(hashes[i + 1..].iter().all(|b| b != a), "{hashes:?}");
    }
    assert_ne!(asks.state_hash(), fights.state_hash());
}

// covers: ADV-PIK-004
#[test]
fn a_guarded_pile_asks_first_and_a_declined_question_changes_nothing() {
    let rules = rules();
    let mut s = guarded(&rules, true);
    let before = s.hero(HERO).unwrap().movement;
    let (outcome, events) = visit(&rules, &mut s);
    assert_eq!(
        outcome,
        AdventureOutcome {
            offer: Some(question()),
            ..AdventureOutcome::default()
        }
    );
    // Visited from the neighbouring tile: the hero stays and pays the step.
    let hero = s.hero(HERO).unwrap();
    assert_eq!(hero.pos, Pos::new(1, 1, 0));
    assert!(hero.movement < before);
    assert!(events.is_empty());
    assert!(s.offer_stands(&question()));
    // Declined: nothing happens.
    let after = s.clone();
    let mut events = Vec::new();
    let declined = s.decline_offer((&rules.0, &rules.1, &[]), &question(), &mut events);
    assert_eq!(declined, AdventureOutcome::default());
    assert_eq!((s, events), (after, vec![]));
}

// covers: ADV-PIK-004
#[test]
fn an_accepted_question_or_no_question_fights_the_unsplit_guards_at_the_piles_tile() {
    let rules = rules();
    let battle = |outcome: AdventureOutcome, s: &AdventureState| {
        assert_eq!((outcome.offer, outcome.gains), (None, vec![]));
        let battle = outcome.battle.unwrap();
        assert_eq!(
            battle.attacker,
            Combatant {
                hero: HERO,
                owner: RED,
                army: vec![stack(0, PEASANT, 100)],
                formation: Formation::Loose,
                tactics: true,
            }
        );
        assert_eq!(battle.defender, Defender::Guards { army: the_guards() });
        assert_eq!(battle.tile, TILE);
        assert_eq!(
            (battle.terrain, battle.battlefield),
            s.battle_ground(&rules.1, TILE)
        );
    };
    let mut s = guarded(&rules, true);
    visit(&rules, &mut s);
    let mut events = Vec::new();
    let accepted = s.accept_offer(&rules.0, &rules.1, RED, &question(), &mut events);
    battle(accepted, &s);
    assert!(events.is_empty());
    let mut s = guarded(&rules, false);
    let (outcome, mut events) = visit(&rules, &mut s);
    battle(outcome, &s);
    assert!(events.is_empty());
    assert!(!s.is_removed(0));
    // A hero no longer on the map starts none.
    let stranger = Offer {
        hero: HeroId::new(9),
        ..question()
    };
    let outcome = s.accept_offer(&rules.0, &rules.1, RED, &stranger, &mut events);
    assert_eq!(outcome, AdventureOutcome::default());
}

// covers: ADV-PIK-004
#[test]
fn the_guards_battle_leaves_their_survivors_or_gives_the_pile() {
    let rules = rules();
    let parts = (&rules.0, &rules.1);
    let mut events: Vec<AdventureEvent> = Vec::new();
    // Lost or retreated: the survivors stay as the guards, unseen; the next visit fights them.
    let mut s = guarded(&rules, false);
    let left = [stack(4, GNOLL, 3)];
    let gains = s.end_unowned_battle(parts, TILE, (&left, false), &mut events);
    assert!(gains.is_empty() && events.is_empty());
    assert_eq!(s.guards_left(0), &left[..]);
    assert_eq!(s.army_left(0), Some(&left[..]));
    assert!(!s.is_removed(0));
    let (outcome, _) = visit(&rules, &mut s);
    assert_eq!(
        outcome.battle.map(|b| b.defender),
        Some(Defender::Guards {
            army: left.to_vec()
        })
    );
    // Won: the pile is picked up, told to the players that see it.
    let gains = s.end_unowned_battle(parts, TILE, (&[], true), &mut events);
    assert_eq!(gains, [(GOLD, 700)]);
    assert!(s.is_removed(0));
    assert_eq!(s.army_left(0), None);
    assert_eq!(
        events,
        [AdventureEvent {
            kind: AdventureEventKind::ObjectRemoved {
                object_type: GOLD_PILE,
                anchor: TILE,
            },
            seen_by: Viewers::NONE.with(RED),
        }]
    );
    events.clear();
    // Both sides dead: the pile stays without guards; a question made before it is a pickup
    // when accepted, and so is the next visit (no question while none stand).
    let mut s = guarded(&rules, true);
    let (outcome, _) = visit(&rules, &mut s);
    assert_eq!(outcome.offer, Some(question()));
    let gains = s.end_unowned_battle(parts, TILE, (&[], false), &mut events);
    assert!(gains.is_empty() && events.is_empty());
    assert!(s.guards_left(0).is_empty());
    let mut again = s.clone();
    let accepted = s.accept_offer(&rules.0, &rules.1, RED, &question(), &mut events);
    assert_eq!(accepted.gains, [(GOLD, 700)]);
    assert!(accepted.battle.is_none() && s.is_removed(0));
    let (outcome, _) = visit(&rules, &mut again);
    assert_eq!((outcome.gains, outcome.offer), (vec![(GOLD, 700)], None));
    assert!(again.is_removed(0));
    // Nothing visited at the tile, or a pile without guards: nothing. A monster's ends as its
    // own (ADV-MON-010).
    let mut s = state(
        &rules,
        vec![
            pile_at(2, GOLD_PILE, None),
            ObjectSetup {
                scenery: Default::default(),
                monster: Some(MonsterSetup {
                    creature: Some(PIKEMAN),
                    count: 5,
                    disposition: 2,
                    never_flees: false,
                    no_growth: false,
                    treasure: vec![(GOLD, 50)],
                    artifact: None,
                }),
                ..pile_at(4, MONSTER, None)
            },
        ],
    );
    let before = s.clone();
    for at in [Pos::new(0, 2, 0), TILE] {
        assert!(
            s.end_unowned_battle(parts, at, (&[], true), &mut events)
                .is_empty()
        );
    }
    assert_eq!(s, before);
    let gains = s.end_unowned_battle(parts, Pos::new(4, 0, 0), (&[], true), &mut events);
    assert_eq!(gains, [(GOLD, 50)]);
    assert!(s.is_removed(1));
}
