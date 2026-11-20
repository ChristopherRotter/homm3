//! Tests of the Hill Forts' visits and the adventure's checks of their upgrades (E08 T134b1;
//! increment 14's design note §4, §10, §11, §15; survey `E08-trades.md` §13): the kind mark and
//! the window at every visit (ADV-HFT-001, ADV-BON-009), each stack's row by the ruleset's
//! upgrades and its price (ADV-HFT-002: the difference times the count, the gold alone scaled by
//! the level and truncated once, a level-1 stack free; D-74 the original's), the checks of one row
//! and of all, the change, and the errors' texts.

use std::string::ToString;
use std::sync::Arc;
use std::vec;
use std::vec::Vec;

use h3_content::{ContentPack, CreatureDef, CreatureInfoDef, CreatureTypeId, ResourceId};
use h3_contracts::ArmyStack;

use super::draws::{object, rules_with};
use super::rewards::{AT, GOLD, HERO, RED, Rules, WOOD, map_of, type_of};
use super::*;
use crate::army::{ArmyRef, ArmySlot};
use crate::command::{HillFortError, HillFortVisit, ServiceVisit};
use crate::ruleset::{HillFort, ObjectKindDef, ObjectTypeDef, Visit};
use crate::setup::Holds;
use crate::state::HillFortRow;
use crate::testing::{PEASANT, PIKEMAN, hero_details, key, stream};

/// The creatures this file adds, by key: a castle creature and its upgrade of each level the
/// survey's rows read (E14-12), with gold and wood costs (wood the core's first resource).
const ADDED: [(&str, u8, u32, u32, Option<&str>); 11] = [
    ("t:halberdier", 1, 75, 0, None),
    ("t:archer", 2, 100, 0, Some("t:marksman")),
    ("t:marksman", 2, 150, 2, None),
    ("t:elf", 3, 200, 0, Some("t:grand_elf")),
    ("t:grand_elf", 3, 225, 0, None),
    ("t:pegasus", 4, 250, 0, Some("t:silver_pegasus")),
    ("t:silver_pegasus", 4, 275, 0, None),
    ("t:angel", 7, 3000, 1, Some("t:archangel")),
    ("t:archangel", 7, 5000, 3, None),
    // An upgrade cheaper in wood (none of it paid) and one whose price saturates.
    ("t:giant", 7, 0, 5, Some("t:titan")),
    ("t:titan", 7, 4_000_000_000, 1, None),
];

/// The draws' rules with the pikeman of level 1 (60 gold) upgrading to the halberdier, the
/// creatures of [`ADDED`], and an entered `t:fort` (the base pack's percents 0, 25, 50, 75,
/// 100, 100, 100; its kind marked).
fn rules() -> Rules {
    rules_with(
        true,
        |core: &mut ContentPack| {
            let pikeman = &mut core.creatures[PIKEMAN.index()].info;
            pikeman.level = 1;
            pikeman.cost = vec![(key("t:gold"), 60)];
            pikeman.upgrades = vec![key("t:halberdier")];
            core.creatures
                .extend(
                    ADDED
                        .iter()
                        .map(|&(k, level, gold, wood, upgrade)| CreatureDef {
                            key: key(k),
                            speed: 4,
                            info: CreatureInfoDef {
                                faction: Some(key("t:castle")),
                                level,
                                cost: vec![(key("t:gold"), gold), (key("t:wood"), wood)],
                                upgrades: upgrade.into_iter().map(key).collect(),
                                ..CreatureInfoDef::default()
                            },
                        }),
                );
        },
        |types| {
            types.push(ObjectTypeDef {
                key: key("t:fort"),
                name: "Hill Fort".into(),
                visit: Visit::Enter,
                from_above: false,
                kind_mark: true,
                kind: ObjectKindDef::HillFort(HillFort {
                    gold: [0, 25, 50, 75, 100, 100, 100],
                }),
            });
        },
    )
}

/// The creature of key `k`.
fn creature(rules: &Rules, k: &str) -> CreatureTypeId {
    rules.0.creature_by_key(k).unwrap()
}

/// The Hill Fort's window at object index 0 and [`AT`] for hero 1.
fn window(rules: &Rules) -> HillFortVisit {
    HillFortVisit {
        hero: HERO,
        index: 0,
        object_type: type_of(rules, "t:fort"),
        anchor: AT,
    }
}

/// A stack of `count` creatures of key `k` in `slot`.
fn stack(rules: &Rules, slot: u8, k: &str, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature: creature(rules, k),
        count,
    }
}

/// [`map_of`] with the Hill Fort at [`AT`] and hero 1's army `army`.
fn at_fort(rules: &Rules, army: Vec<ArmyStack>) -> AdventureState {
    let mut state = map_of(rules, vec![object(rules, AT.x, "t:fort", Holds::Nothing)]);
    state.hero_mut(HERO).unwrap().army = army;
    state
}

// covers: ADV-HFT-001, ADV-BON-009, ADV-OBJ-003, ADV-TEL-008
#[test]
fn a_hill_forts_step_marks_its_kind_and_opens_its_window_at_every_visit() {
    let rules = rules();
    let fort = type_of(&rules, "t:fort");
    let mut state = at_fort(
        &rules,
        vec![ArmyStack {
            slot: 0,
            creature: PEASANT,
            count: 1,
        }],
    );
    let heroes: Vec<_> = (0..3).map(|_| Arc::new(hero_details(&[]))).collect();
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
    let opened = AdventureOutcome {
        service: Some(ServiceVisit::HillFort(window(&rules))),
        ..AdventureOutcome::default()
    };
    assert_eq!(outcome, opened);
    // Entered: the move ends on the entrance; the team learns the kind once.
    assert_eq!(state.hero(HERO).unwrap().pos, AT);
    let known = AdventureEventKind::KindKnown { object_type: fort };
    assert!(events.iter().any(|e| e.kind == known));
    assert!(state.knows_kind(RED, fort));
    // Every visit opens it again, whatever the army holds (a peasant upgrades to nothing).
    let revisit = AdventureCommand::Revisit { hero: HERO };
    let (again, events) = apply(&mut state, &revisit);
    assert_eq!(again, opened);
    assert!(events.iter().all(|e| e.kind != known));
}

// covers: ADV-HFT-002
#[test]
fn a_rows_price_is_the_difference_times_the_count_with_the_gold_scaled_by_level_once() {
    let rules = rules();
    let (gold, wood) = (GOLD, WOOD);
    let army = vec![
        ArmyStack {
            slot: 0,
            creature: PIKEMAN,
            count: 20,
        },
        stack(&rules, 1, "t:archer", 10),
        stack(&rules, 2, "t:elf", 3),
        stack(&rules, 3, "t:pegasus", 3),
        stack(&rules, 4, "t:marksman", 5),
        stack(&rules, 5, "t:angel", 1),
        ArmyStack {
            slot: 6,
            creature: PEASANT,
            count: 9,
        },
    ];
    let state = at_fort(&rules, army.clone());
    let rows = state
        .hill_fort_rows((&rules.0, &rules.1), &window(&rules))
        .unwrap();
    let row = |at: usize, upgrade: Option<&str>, price: Vec<(ResourceId, u32)>| HillFortRow {
        slot: army[at].slot,
        creature: army[at].creature,
        count: army[at].count,
        upgrade: upgrade.map(|k| creature(&rules, k)),
        price,
    };
    let expected = vec![
        // Level 1 free whole (the original computes no price): 20 × 15 gold not paid.
        row(0, Some("t:halberdier"), vec![]),
        // 10 × 50 gold at 25 %: 125; the wood not scaled: 10 × 2.
        row(1, Some("t:marksman"), vec![(wood, 20), (gold, 125)]),
        // 3 × 25 gold at 50 %: 37.5, truncated once for the stack (D-74), not 3 × 12.
        row(2, Some("t:grand_elf"), vec![(gold, 37)]),
        // 3 × 25 at 75 %: 56.25.
        row(3, Some("t:silver_pegasus"), vec![(gold, 56)]),
        // An upgraded creature and a neutral one have no upgrade.
        row(4, None, vec![]),
        // Level 7 whole: 2 000 gold and 2 wood.
        row(5, Some("t:archangel"), vec![(wood, 2), (gold, 2000)]),
        row(6, None, vec![]),
    ];
    assert_eq!(rows, expected);
    // A resource the upgrade costs less of is not paid; a price past 32 bits saturates.
    let state = at_fort(&rules, vec![stack(&rules, 0, "t:giant", 2)]);
    let rows = state
        .hill_fort_rows((&rules.0, &rules.1), &window(&rules))
        .unwrap();
    assert_eq!(rows[0].price, vec![(gold, u32::MAX)]);
}

// covers: ADV-HFT-002
#[test]
fn an_upgrade_is_checked_by_its_row_and_replaces_the_creature_keeping_the_count() {
    let rules = rules();
    let army = vec![
        stack(&rules, 1, "t:archer", 10),
        stack(&rules, 3, "t:marksman", 5),
        stack(&rules, 4, "t:pegasus", 3),
    ];
    let mut state = at_fort(&rules, army);
    let fort = window(&rules);
    let checks = (&rules.0, &rules.1);
    let gold = GOLD;
    // A slot without a stack, a stack without an upgrade.
    for slot in [0, 3, 7] {
        assert_eq!(
            state.check_upgrade(checks, &fort, slot),
            Err(HillFortError::NoUpgrade(slot))
        );
    }
    // All: every row that upgrades, in slot order.
    let all = state.check_upgrade_all(checks, &fort).unwrap();
    let all: Vec<_> = all.iter().map(|u| (u.hero(), u.price().to_vec())).collect();
    let wood = WOOD;
    assert_eq!(
        all,
        vec![
            (HERO, vec![(wood, 20), (gold, 125)]),
            (HERO, vec![(gold, 56)])
        ]
    );
    // One row: its price; made, the creature replaced and the count kept.
    let upgrade = state.check_upgrade(checks, &fort, 4).unwrap();
    assert_eq!((upgrade.hero(), upgrade.price()), (HERO, &[(gold, 56)][..]));
    let at = state.upgrade(upgrade);
    assert_eq!(
        at,
        ArmySlot {
            army: ArmyRef::Hero(HERO),
            slot: 4
        }
    );
    let upgraded = stack(&rules, 4, "t:silver_pegasus", 3);
    assert_eq!(state.hero(HERO).unwrap().army[2], upgraded);
    // Then it upgrades no more; with no row left, All has nothing.
    assert_eq!(
        state.check_upgrade(checks, &fort, 4),
        Err(HillFortError::NoUpgrade(4))
    );
    state.upgrade(state.check_upgrade(checks, &fort, 1).unwrap());
    assert_eq!(
        state.check_upgrade_all(checks, &fort),
        Err(HillFortError::Nothing)
    );
    // A window whose hero is off the map or whose object is none.
    let gone = [
        HillFortVisit {
            hero: HeroId::new(9),
            ..fort
        },
        HillFortVisit { index: 7, ..fort },
    ];
    for visit in gone {
        assert_eq!(
            state.hill_fort_rows(checks, &visit),
            Err(HillFortError::Visit)
        );
        assert_eq!(
            state.check_upgrade(checks, &visit, 1),
            Err(HillFortError::Visit)
        );
        assert_eq!(
            state.check_upgrade_all(checks, &visit),
            Err(HillFortError::Visit)
        );
    }
}

#[test]
fn a_hill_forts_errors_read_as_text() {
    let texts = [
        HillFortError::Visit,
        HillFortError::NoUpgrade(3),
        HillFortError::Nothing,
    ]
    .map(|e| e.to_string());
    assert_eq!(
        texts,
        [
            "the hill fort's hero is not on the map or its object no hill fort",
            "slot 3 holds no stack that upgrades",
            "no stack of the army upgrades",
        ]
    );
}
