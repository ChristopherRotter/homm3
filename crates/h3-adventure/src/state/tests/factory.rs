//! Tests of the War Machine Factories' visits and the hire dialog's factory side (E08 T134b2;
//! increment 14's design note §5, §10, §11, §16; survey `E08-trades.md` §14): the hire question
//! naming the machines at every visit and its yes opening the dialog of them (ADV-WMF-001), the
//! stock of one of each machine the hero lacks, the room of a machine's artifact, and the purchase
//! of one machine as its artifact with each refusal in order (ADV-HIR-002 amended).

use std::sync::Arc;
use std::vec;
use std::vec::Vec;

use h3_content::{ArtifactClass, ArtifactDef, ArtifactSlotDef, CreatureDef, CreatureInfoDef};
use h3_content::{ArtifactTypeId, ContentPack, CreatureTypeId};
use h3_contracts::{HeroId, TownId};
use h3_heroes::HeroDetails;

use super::draws::{object, rules_with};
use super::rewards::{AT, GOLD, HERO, RED, Rules, map_of, store, type_of};
use super::*;
use crate::command::{HireDialog, HireError, HireSource, HireTarget, Offer, OfferKind};
use crate::ruleset::{ObjectKindDef, ObjectTypeDef, Visit, WarMachineFactoryDef};
use crate::setup::Holds;
use crate::testing::{PEASANT, hero_details, key, stream};

/// The war machines this file adds, by key: each one's gold cost and its artifact's slot (the
/// cart's shared with the wheel, an artifact of no machine).
const MACHINES: [(&str, u32, &str); 3] = [
    ("t:ballista", 2500, "t:ballista_slot"),
    ("t:tent", 750, "t:tent_slot"),
    ("t:cart", 1000, "t:wagon"),
];

/// The draws' rules with the three machines of [`MACHINES`] (war machine creatures, each with
/// its artifact `<key>_art` in its own position, the cart's in the wagon's, which the wheel also
/// takes) and an entered `t:factory` selling them in that order, no kind mark.
fn rules() -> Rules {
    rules_with(
        true,
        |core: &mut ContentPack| {
            for slot in ["t:ballista_slot", "t:tent_slot", "t:wagon"] {
                core.artifact_slots.push(ArtifactSlotDef {
                    key: key(slot),
                    name: "Machine".into(),
                    count: 1,
                    backpack: false,
                });
            }
            let artifact = |k: &str, slot: &str, machine: Option<&str>| ArtifactDef {
                key: key(k),
                name: "Machine".into(),
                class: ArtifactClass::Special,
                cost: vec![],
                slot: Some(key(slot)),
                war_machine: machine.map(key),
                needs_skill: None,
                effects: vec![],
                parts: vec![],
            };
            for (k, gold, slot) in MACHINES {
                core.creatures.push(CreatureDef {
                    key: key(k),
                    speed: 0,
                    info: CreatureInfoDef {
                        level: 1,
                        cost: vec![(key("t:gold"), gold)],
                        flags: vec!["war_machine".into()],
                        ..CreatureInfoDef::default()
                    },
                });
                let art = std::format!("{k}_art");
                core.artifacts.push(artifact(&art, slot, Some(k)));
            }
            core.artifacts.push(artifact("t:wheel", "t:wagon", None));
        },
        |types| {
            types.push(ObjectTypeDef {
                key: key("t:factory"),
                name: "War Machine Factory".into(),
                visit: Visit::Enter,
                from_above: false,
                kind_mark: false,
                kind: ObjectKindDef::WarMachineFactory(WarMachineFactoryDef {
                    machines: MACHINES.iter().map(|&(k, ..)| key(k)).collect(),
                }),
            });
        },
    )
}

/// The creature of key `k`.
fn creature(rules: &Rules, k: &str) -> CreatureTypeId {
    rules.0.creature_by_key(k).unwrap()
}

/// The artifact of key `k`.
fn artifact(rules: &Rules, k: &str) -> ArtifactTypeId {
    rules.0.artifact_by_key(k).unwrap()
}

/// The three machines in the factory's order.
fn machines(rules: &Rules) -> [CreatureTypeId; 3] {
    MACHINES.map(|(k, ..)| creature(rules, k))
}

/// Hero 1's details wearing `worn` (every position empty but these) and carrying `carried`.
fn details(rules: &Rules, worn: &[&str], carried: &[&str]) -> Vec<Arc<HeroDetails>> {
    let positions = h3_heroes::positions(&rules.0);
    let mut details = HeroDetails {
        worn: vec![None; positions.len()],
        backpack: carried.iter().map(|&k| artifact(rules, k).into()).collect(),
        ..hero_details(&[])
    };
    for &k in worn {
        let at = h3_heroes::first_position(&details, &rules.0, artifact(rules, k)).unwrap();
        details.worn[at.0] = Some(artifact(rules, k).into());
    }
    store(details)
}

/// The factory's dialog at object index 0 and [`AT`] for `target`.
fn dialog(rules: &Rules, target: HireTarget) -> HireDialog {
    HireDialog {
        target,
        source: HireSource::Object {
            index: 0,
            object_type: type_of(rules, "t:factory"),
            anchor: AT,
        },
    }
}

/// [`map_of`] with the factory at [`AT`].
fn at_factory(rules: &Rules) -> AdventureState {
    map_of(
        rules,
        vec![object(rules, AT.x, "t:factory", Holds::Nothing)],
    )
}

// covers: ADV-WMF-001, ADV-OBJ-003, ADV-TEL-008, ADV-HIR-002
#[test]
fn a_factorys_step_asks_the_hire_question_of_its_machines_at_every_visit() {
    let rules = rules();
    let factory = type_of(&rules, "t:factory");
    let mut state = at_factory(&rules);
    let heroes = details(&rules, &[], &[]);
    let apply = |state: &mut AdventureState, command: &AdventureCommand| {
        let mut events = Vec::new();
        let turn = (RED, Weekday::FIRST, &mut stream());
        let outcome = state.apply(&rules.0, &rules.1, &heroes, turn, command, &mut events);
        (outcome.unwrap(), events)
    };
    let step = AdventureCommand::MoveHero {
        hero: HERO,
        path: vec![AT],
    };
    let (outcome, events) = apply(&mut state, &step);
    let offer = Offer {
        hero: HERO,
        object_type: factory,
        anchor: AT,
        kind: OfferKind::Hire {
            creatures: machines(&rules).to_vec(),
        },
    };
    let asked = AdventureOutcome {
        offer: Some(offer.clone()),
        ..AdventureOutcome::default()
    };
    assert_eq!(outcome, asked);
    // Entered: the move ends on the entrance; no mark, so nothing learnt or written.
    assert_eq!(state.hero(HERO).unwrap().pos, AT);
    assert!(events.iter().all(|e| e.kind
        != AdventureEventKind::KindKnown {
            object_type: factory
        }));
    assert!(!state.knows_kind(RED, factory));
    // Every visit asks again.
    let (again, _) = apply(&mut state, &AdventureCommand::Revisit { hero: HERO });
    assert_eq!(again, asked);
    // Its yes opens the dialog of the machines for the hero, nothing free, nothing told.
    let before = state.state_hash();
    let mut events = Vec::new();
    let rules3 = (&rules.0, &rules.1, &heroes[..]);
    let accepted = state.accept_offer(rules3, RED, &offer, &mut events);
    let opened = AdventureOutcome {
        hire: Some(dialog(&rules, HireTarget::Hero(HERO))),
        ..AdventureOutcome::default()
    };
    assert_eq!((accepted, events), (opened, vec![]));
    assert_eq!(state.state_hash(), before);
}

// covers: ADV-WMF-001, ADV-HIR-002
#[test]
fn the_stock_is_one_of_each_machine_the_hero_lacks_and_the_room_its_position() {
    let rules = rules();
    let state = at_factory(&rules);
    let [ballista, tent, cart] = machines(&rules);
    let open = dialog(&rules, HireTarget::Hero(HERO));
    let stock = |heroes: &[Arc<HeroDetails>], dialog| state.hire_stock((&rules.1, heroes), dialog);
    let room = |heroes: &[Arc<HeroDetails>], creature| {
        state.dialog_room((&rules.0, &rules.1, heroes), &open, creature)
    };
    // Nothing held: one of each, each with room.
    let bare = details(&rules, &[], &[]);
    let all = vec![(ballista, 1), (tent, 1), (cart, 1)];
    assert_eq!(stock(&bare, &open), Some(all));
    assert_eq!(machines(&rules).map(|machine| room(&bare, machine)), [1; 3]);
    // A Ballista worn: none of it (the original's E14-15), and no room for a second.
    let worn = details(&rules, &["t:ballista_art"], &[]);
    let lacking = vec![(ballista, 0), (tent, 1), (cart, 1)];
    assert_eq!(stock(&worn, &open), Some(lacking.clone()));
    assert_eq!(room(&worn, ballista), 0);
    // Held anywhere counts (a store's backpack carrying it, as a setup may give).
    assert_eq!(
        stock(&details(&rules, &[], &["t:ballista_art"]), &open),
        Some(lacking)
    );
    // The wheel in the cart's position: the cart is lacked, so counted, but finds no room.
    let wheel = details(&rules, &["t:wheel"], &[]);
    assert_eq!(
        stock(&wheel, &open),
        Some(vec![(ballista, 1), (tent, 1), (cart, 1)])
    );
    assert_eq!((room(&wheel, cart), room(&wheel, tent)), (0, 1));
    // A creature the factory does not sell has no room; a hero the store lacks or a garrison
    // has no stock.
    assert_eq!(room(&bare, PEASANT), 0);
    let gone = dialog(&rules, HireTarget::Hero(HeroId::new(7)));
    assert_eq!(stock(&bare, &gone), None);
    let garrison = dialog(&rules, HireTarget::Garrison(TownId::new(0)));
    assert_eq!(stock(&bare, &garrison), None);
    // Another source's room is the army's ([`AdventureState::hire_room`]).
    let portal = HireDialog {
        source: HireSource::Portal {
            town: TownId::new(0),
        },
        ..open
    };
    let army = state.hire_room(&rules.1, &open.target, PEASANT);
    let rules3 = (&rules.0, &rules.1, &bare[..]);
    assert_eq!(state.dialog_room(rules3, &portal, PEASANT), army);
}

// covers: ADV-HIR-002, ADV-WMF-001
#[test]
fn a_machine_is_bought_alone_as_its_artifact_and_refused_in_order() {
    let rules = rules();
    let mut state = at_factory(&rules);
    let [ballista, tent, cart] = machines(&rules);
    let open = dialog(&rules, HireTarget::Hero(HERO));
    let check = |state: &AdventureState, heroes: &[Arc<HeroDetails>], creature, amount| {
        state.check_hire((&rules.0, &rules.1, heroes), &open, (creature, amount))
    };
    let bare = details(&rules, &[], &[]);
    // One Ballista: its gold cost, its artifact for the hero.
    let purchase = check(&state, &bare, ballista, 1).unwrap();
    assert_eq!(purchase.price(), [(GOLD, 2500)]);
    let art = artifact(&rules, "t:ballista_art");
    assert_eq!(purchase.machine(), Some((HERO, art)));
    // Made: no army change, no count, nothing told (the game gives the artifact).
    let (army, before) = (state.hero(HERO).unwrap().army.clone(), state.state_hash());
    let mut events = Vec::new();
    assert_eq!(state.hire(purchase, &mut events), None);
    assert_eq!(events, []);
    assert_eq!(state.hero(HERO).unwrap().army, army);
    assert_eq!(state.state_hash(), before);
    // The refusals in order: a creature not sold, an amount but 1, a machine held, no room;
    // a hero the store lacks has no stock.
    assert_eq!(
        check(&state, &bare, PEASANT, 1),
        Err(HireError::NotOffered(PEASANT))
    );
    assert_eq!(check(&state, &bare, tent, 2), Err(HireError::Amount(2)));
    assert_eq!(check(&state, &bare, tent, 0), Err(HireError::Amount(0)));
    let worn = details(&rules, &["t:ballista_art"], &[]);
    assert_eq!(check(&state, &worn, ballista, 1), Err(HireError::Amount(1)));
    assert!(check(&state, &worn, tent, 1).is_ok());
    let wheel = details(&rules, &["t:wheel"], &[]);
    assert_eq!(check(&state, &wheel, cart, 1), Err(HireError::NoRoom));
    assert_eq!(check(&state, &[], cart, 1), Err(HireError::NoSource));
}
