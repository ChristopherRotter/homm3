//! The Prison and the Market of Time in the ruleset part (E08 T141a; increment 15's design note §1,
//! §2): two kinds without a parameter, the Prison visited from a neighbour, the empty kind
//! entered, each in the hash (part v25); the Prison's visit names its prisoner, the empty kind's
//! does nothing.

use super::*;

/// [`packs`] with `t`'s type `t:cell` of `kind`, visited as `visit`.
fn cell_packs(visit: Visit, kind: ObjectKindDef) -> Vec<AdventurePack> {
    edit(|p| {
        pack(p, "t")
            .object_types
            .push(object("t:cell", visit, false, kind));
    })
}

/// The compiled `t:cell` of `kind`, visited as `visit`.
fn cell(visit: Visit, kind: ObjectKindDef) -> ObjectType {
    let part = compile(&cell_packs(visit, kind)).unwrap();
    part.object_type(part.object_type_by_key("t:cell").unwrap())
        .unwrap()
        .clone()
}

// covers: ADV-PRI-001, ADV-OBJ-008
#[test]
fn the_prison_is_visited_from_a_neighbour_and_the_empty_kind_entered() {
    let prison = cell(Visit::Neighbour, ObjectKindDef::Prison);
    assert_eq!(prison.kind, ObjectKind::Prison);
    assert_eq!(prison.kind.effect(), Some(Effect::Prison));
    let empty = cell(Visit::Enter, ObjectKindDef::Empty);
    assert_eq!(empty.kind, ObjectKind::Empty);
    assert_eq!(empty.kind.effect(), Some(Effect::Empty));
    for ty in [&prison, &empty] {
        // Neither ownable nor weekly, no mark, no battle, no footprint.
        assert!(!ty.kind.ownable() && !ty.kind.weekly());
        assert_eq!(ty.kind.timed_mark(), None);
        assert_eq!(ty.kind.battle_terrain(), None);
        assert_eq!(ty.kind.dwelling_footprint(), None);
        assert_eq!(ty.kind.reveal_terrain(), None);
    }
    // The other way of visiting each is refused.
    let refused = |visit, kind, reason| {
        assert_eq!(
            compile_err(&cell_packs(visit, kind)),
            AdventureCompileError::InvalidObjectType {
                key: "t:cell".into(),
                reason,
            }
        );
    };
    refused(
        Visit::Enter,
        ObjectKindDef::Prison,
        "a prison is not visited from a neighbour",
    );
    refused(
        Visit::Neighbour,
        ObjectKindDef::Empty,
        "an empty object is not entered",
    );
}

// covers: ADV-PRI-001, ADV-OBJ-008
#[test]
fn the_prison_and_the_empty_kind_are_in_the_hash() {
    let hash = |visit, def| compile(&cell_packs(visit, def)).unwrap().content_hash();
    let hashes = [
        hash(Visit::Neighbour, ObjectKindDef::Prison),
        hash(Visit::Enter, ObjectKindDef::Empty),
        hash(Visit::Neighbour, ObjectKindDef::Tavern),
        hash(Visit::Enter, ObjectKindDef::DenOfThieves),
    ];
    assert_eq!(hashes.iter().collect::<BTreeSet<_>>().len(), hashes.len());
}
