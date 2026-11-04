//! The town's kind in the ruleset part (E09 T8a1, the town epic's design note §8): compiled,
//! ownable and entered, without a visit until T8a2 (`TWN-STUB-4`), and in the hash.

use super::*;

/// [`packs`] with `t`'s town type `t:town`, visited as `visit`.
fn town_packs(visit: Visit) -> Vec<AdventurePack> {
    edit(|p| {
        pack(p, "t")
            .object_types
            .push(object("t:town", visit, false, ObjectKindDef::Town));
    })
}

// covers: ADV-OBJ-002
#[test]
fn the_town_is_an_ownable_kind_entered_without_a_visit_yet() {
    let part = compile(&town_packs(Visit::Enter)).unwrap();
    let ty = part
        .object_type(part.object_type_by_key("t:town").unwrap())
        .unwrap();
    assert_eq!(ty.kind, ObjectKind::Town);
    assert_eq!((ty.visit, ty.from_above), (Visit::Enter, false));
    assert!(ty.kind.ownable());
    assert!(!ty.kind.weekly());
    assert_eq!(ty.kind.effect(), None);
    assert_eq!(ty.kind.battle_terrain(), None);
    assert_eq!(ty.kind.dwelling_footprint(), None);
    // An ownable object is entered.
    assert_eq!(
        compile_err(&town_packs(Visit::Neighbour)),
        AdventureCompileError::InvalidObjectType {
            key: "t:town".into(),
            reason: "an ownable object is not entered",
        }
    );
    // The kind is in the hash (part v17): a town and a Lighthouse of its key differ.
    let other = edit(|p| {
        pack(p, "t").object_types.push(object(
            "t:town",
            Visit::Enter,
            false,
            ObjectKindDef::Lighthouse { sea_points: 500 },
        ));
    });
    assert_ne!(compile(&other).unwrap().content_hash(), part.content_hash());
}
