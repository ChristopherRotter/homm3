//! The map's Tavern and the Den of Thieves in the ruleset part (E09 T39a, the town epic's design
//! note §66, §74): two kinds without a parameter, the Tavern visited from a neighbour, the Den
//! entered, each in the hash (part v22); the Tavern's visit is its decision's, the Den's the
//! Thieves' Guild's (from T39b).

use super::*;

/// [`packs`] with `t`'s type `t:inn` of `kind`, visited as `visit`.
fn inn_packs(visit: Visit, kind: ObjectKindDef) -> Vec<AdventurePack> {
    edit(|p| {
        pack(p, "t")
            .object_types
            .push(object("t:inn", visit, false, kind));
    })
}

/// The compiled `t:inn` of `kind`, visited as `visit`.
fn inn(visit: Visit, kind: ObjectKindDef) -> ObjectType {
    let part = compile(&inn_packs(visit, kind)).unwrap();
    part.object_type(part.object_type_by_key("t:inn").unwrap())
        .unwrap()
        .clone()
}

// covers: ADV-VIS-007, ADV-VIS-008
#[test]
fn the_tavern_is_visited_from_a_neighbour_and_the_den_entered() {
    let tavern = inn(Visit::Neighbour, ObjectKindDef::Tavern);
    assert_eq!(tavern.kind, ObjectKind::Tavern);
    assert_eq!(tavern.kind.effect(), Some(Effect::Tavern));
    let den = inn(Visit::Enter, ObjectKindDef::DenOfThieves);
    assert_eq!(den.kind, ObjectKind::DenOfThieves);
    assert_eq!(den.kind.effect(), Some(Effect::DenOfThieves));
    for ty in [&tavern, &den] {
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
            compile_err(&inn_packs(visit, kind)),
            AdventureCompileError::InvalidObjectType {
                key: "t:inn".into(),
                reason,
            }
        );
    };
    refused(
        Visit::Enter,
        ObjectKindDef::Tavern,
        "a tavern is not visited from a neighbour",
    );
    refused(
        Visit::Neighbour,
        ObjectKindDef::DenOfThieves,
        "a den of thieves is not entered",
    );
}

// covers: ADV-VIS-007, ADV-VIS-008
#[test]
fn the_tavern_and_the_den_are_in_the_hash() {
    let hash = |visit, def| compile(&inn_packs(visit, def)).unwrap().content_hash();
    let hashes = [
        hash(Visit::Neighbour, ObjectKindDef::Tavern),
        hash(Visit::Enter, ObjectKindDef::DenOfThieves),
        hash(Visit::Enter, ObjectKindDef::SubterraneanGate),
        hash(Visit::Neighbour, ObjectKindDef::Scholar),
    ];
    assert_eq!(hashes.iter().collect::<BTreeSet<_>>().len(), hashes.len());
}
