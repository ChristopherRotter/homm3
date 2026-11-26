//! Tests of the Artifact Merchants' stock in the adventure state (TWN-STK-001, -002; E09 T57, the
//! town epic's design note §108): the draws on the marks the build left, every entry anew with the
//! old ones' marks kept, the restart and the widening of ADV-OBJ-006's picker, an entry left empty
//! where nothing can be drawn (`TWN-QUIRK-21`), and the block in the state's hash and forks.

extern crate std;

use std::collections::BTreeSet;
use std::vec;
use std::vec::Vec;

use h3_content::{
    ArtifactClass, ArtifactDef, ArtifactSlotDef, ArtifactTypeId, ContentPack, Ruleset,
};
use h3_kernel::hash::hash_of;
use h3_kernel::{Rng, StreamId};

use crate::ruleset::{AdventurePack, AdventureRuleset, ObjectKindDef, ObjectTypeDef, Visit};
use crate::setup::{Bans, Holds, ObjectSetup, Pos, TileMask};
use crate::state::{AdventureState, Resolved};
use crate::testing::{PEASANT, hero_at, key, movement_rules_with, world};

type Rules = (Ruleset, AdventureRuleset);

const RING: ArtifactTypeId = ArtifactTypeId::from_raw(0);
const CHARM: ArtifactTypeId = ArtifactTypeId::from_raw(1);
const AMULET: ArtifactTypeId = ArtifactTypeId::from_raw(2);
const CLOAK: ArtifactTypeId = ArtifactTypeId::from_raw(3);
const CAPE: ArtifactTypeId = ArtifactTypeId::from_raw(4);
const SHAWL: ArtifactTypeId = ArtifactTypeId::from_raw(5);
const HELM: ArtifactTypeId = ArtifactTypeId::from_raw(6);
const CROWN: ArtifactTypeId = ArtifactTypeId::from_raw(7);
const BOOK: ArtifactTypeId = ArtifactTypeId::from_raw(8);

const TREASURES: [ArtifactTypeId; 3] = [RING, CHARM, AMULET];
const MINORS: [ArtifactTypeId; 3] = [CLOAK, CAPE, SHAWL];

/// The movement rules with the treasures ring, charm and amulet, the minors cloak, cape and
/// shawl, the major helm, the relic crown and the special book, and the object types `t:artifact`
/// (a placed artifact) and `t:random_treasure`.
fn rules() -> Rules {
    movement_rules_with(
        true,
        16,
        |core: &mut ContentPack| {
            core.artifact_slots = vec![ArtifactSlotDef {
                key: key("t:finger"),
                name: "Finger".into(),
                count: 1,
                backpack: true,
            }];
            let artifact = |k: &str, class| ArtifactDef {
                key: key(k),
                name: "Artifact".into(),
                class,
                cost: vec![],
                slot: Some(key("t:finger")),
                war_machine: None,
                needs_skill: None,
                effects: vec![],
                parts: vec![],
            };
            let (t, n) = (ArtifactClass::Treasure, ArtifactClass::Minor);
            core.artifacts = vec![
                artifact("t:ring", t),
                artifact("t:charm", t),
                artifact("t:amulet", t),
                artifact("t:cloak", n),
                artifact("t:cape", n),
                artifact("t:shawl", n),
                artifact("t:helm", ArtifactClass::Major),
                artifact("t:crown", ArtifactClass::Relic),
                artifact("t:book", ArtifactClass::Special),
            ];
        },
        |part: &mut AdventurePack| {
            let ty = |k: &str, kind| ObjectTypeDef {
                key: key(k),
                name: "Object".into(),
                visit: Visit::Neighbour,
                from_above: true,
                kind_mark: false,
                kind,
            };
            part.object_types.extend([
                ty("t:artifact", ObjectKindDef::Artifact { classes: None }),
                ty(
                    "t:random_treasure",
                    ObjectKindDef::Artifact {
                        classes: Some(vec![ArtifactClass::Treasure]),
                    },
                ),
            ]);
        },
    )
}

/// An object of type `k` at `(x, 0)` holding `holds`.
fn object(rules: &Rules, x: u16, k: &str, holds: Holds) -> ObjectSetup {
    ObjectSetup {
        scenery: Default::default(),
        anchor: Pos::new(x, 0, 0),
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: Some(rules.1.object_type_by_key(k).unwrap()),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds,
        owner: None,
    }
}

/// A grass map of 16 × 2 with `objects` on its first row under `bans`, red's hero 1 at (1, 1),
/// built from the setup stream of `seed`.
fn built(rules: &Rules, objects: Vec<ObjectSetup>, bans: &Bans, seed: u64) -> AdventureState {
    let mut setup = world(
        &["gggggggggggggggg", "gggggggggggggggg"],
        1,
        vec![hero_at(1, 0, Pos::new(1, 1, 0), &[PEASANT])],
    );
    setup.map.objects = objects;
    setup.bans = bans.clone();
    let mut draws = Rng::for_stream(seed, StreamId::SETUP);
    AdventureState::new(&rules.0, &rules.1, &setup, &mut draws).unwrap()
}

/// A stream of `seed`'s for the merchants' draws.
fn towns(seed: u64) -> Rng {
    Rng::for_stream(seed, StreamId::TOWNS)
}

/// The stock's artifacts, each entry's (none for an empty one).
fn stock(state: &AdventureState) -> Vec<Option<ArtifactTypeId>> {
    state.merchants_stock().to_vec()
}

// covers: TWN-STK-001, ADV-OBJ-006
#[test]
fn the_stock_draws_on_the_marks_the_build_left() {
    let rules = rules();
    let (t, n, j) = (
        ArtifactClass::Treasure,
        ArtifactClass::Minor,
        ArtifactClass::Major,
    );
    for seed in 0..20 {
        // The placed ring and the random treasure's draw mark two treasures: the stock's first
        // treasure is the third, its second finds the class out and restarts it from the bans.
        let objects = vec![
            object(&rules, 0, "t:artifact", Holds::Artifact(RING.into())),
            object(&rules, 2, "t:random_treasure", Holds::Nothing),
        ];
        let mut state = built(&rules, objects, &Bans::default(), seed);
        assert!(state.merchants_stock().is_empty(), "no stock before a draw");
        let Resolved::Artifact(random) = state.map().objects()[1].resolved else {
            panic!("the random treasure draws one");
        };
        state.draw_merchants(&rules.0, &[t, t, n, n, j], &mut towns(seed));
        let drawn = stock(&state);
        let third = TREASURES.into_iter().find(|&a| a != RING && a != random);
        assert_eq!(drawn[0], third);
        assert!(drawn[1].is_some_and(|a| TREASURES.contains(&a)));
        let minors: BTreeSet<_> = drawn[2..4].iter().flatten().copied().collect();
        assert_eq!(minors.len(), 2, "no repeat while the class lasts");
        assert!(minors.iter().all(|a| MINORS.contains(a)));
        assert_eq!(drawn[4], Some(HELM));
    }
}

// covers: TWN-STK-002, ADV-OBJ-006
#[test]
fn each_draw_replaces_every_entry_and_keeps_the_old_ones_marked() {
    let rules = rules();
    let mut restarted = BTreeSet::new();
    for seed in 0..20 {
        let mut state = built(&rules, vec![], &Bans::default(), seed);
        let mut draws = towns(seed);
        // One minor an entry: three draws give the three minors once each (an unsold artifact
        // stays marked), the fourth restarts the class from the bans.
        let months: Vec<_> = (0..4)
            .map(|_| {
                state.draw_merchants(&rules.0, &[ArtifactClass::Minor], &mut draws);
                stock(&state)[0].unwrap()
            })
            .collect();
        assert_eq!(
            BTreeSet::from_iter(months[..3].iter().copied()),
            MINORS.into()
        );
        restarted.insert(months[3]);
        // A draw of other classes replaces the stock whole: its length is theirs.
        let classes = [ArtifactClass::Major, ArtifactClass::Major];
        state.draw_merchants(&rules.0, &classes, &mut draws);
        assert_eq!(stock(&state), [Some(HELM), Some(HELM)]);
        state.draw_merchants(&rules.0, &[], &mut draws);
        assert!(state.merchants_stock().is_empty());
    }
    assert_eq!(restarted, MINORS.into(), "after the restart any minor");
}

// covers: TWN-STK-001, ADV-OBJ-006
#[test]
fn a_class_banned_whole_widens_and_nothing_left_leaves_the_entry_empty() {
    let rules = rules();
    let treasures = Bans {
        artifacts: TREASURES.to_vec(),
        ..Bans::default()
    };
    let mut widened = BTreeSet::new();
    for seed in 0..20 {
        let mut state = built(&rules, vec![], &treasures, seed);
        let classes = [ArtifactClass::Treasure; 4];
        state.draw_merchants(&rules.0, &classes, &mut towns(seed));
        for entry in stock(&state) {
            let artifact = entry.unwrap();
            assert!(!TREASURES.contains(&artifact) && artifact != BOOK);
            widened.insert(artifact);
        }
    }
    assert!(widened.contains(&CROWN), "the relics through the widening");
    // TWN-QUIRK-21: every artifact but the special book banned: the entries stay empty (the
    // original's picker loops for ever).
    let all = Bans {
        artifacts: vec![RING, CHARM, AMULET, CLOAK, CAPE, SHAWL, HELM, CROWN],
        ..Bans::default()
    };
    let mut state = built(&rules, vec![], &all, 0);
    let classes = [ArtifactClass::Treasure, ArtifactClass::Major];
    state.draw_merchants(&rules.0, &classes, &mut towns(0));
    assert_eq!(stock(&state), [None, None]);
}

// covers: TWN-STK-001, TWN-STK-002, ADV-OBJ-006
#[test]
fn the_stock_and_the_marks_are_in_the_hash_and_a_fork_shares_them() {
    let rules = rules();
    let built_state = built(&rules, vec![], &Bans::default(), 0);
    let mut state = built_state.clone();
    let major = [ArtifactClass::Major];
    state.draw_merchants(&rules.0, &major, &mut towns(0));
    assert_eq!(stock(&state), [Some(HELM)]);
    assert_ne!(
        hash_of(&state),
        hash_of(&built_state),
        "the stock is hashed"
    );
    // A fork is its parent's; a draw on the fork leaves the parent as it was.
    let parent = state.clone();
    let mut fork = state.clone();
    assert_eq!((hash_of(&fork), &fork), (hash_of(&parent), &parent));
    fork.draw_merchants(&rules.0, &[ArtifactClass::Minor], &mut towns(0));
    assert_eq!(state, parent);
    assert_ne!(fork, parent);
    // The helm again, its class restarted alone: the minor drawn stays marked, so the same stock
    // on other marks hashes otherwise.
    fork.draw_merchants(&rules.0, &major, &mut towns(0));
    assert_eq!(fork.merchants_stock(), parent.merchants_stock());
    assert_ne!(hash_of(&fork), hash_of(&parent), "the marks are hashed");
}
