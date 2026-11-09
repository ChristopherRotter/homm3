//! The town's kind in the ruleset part (E09 T8a1, the town epic's design note §8): compiled,
//! ownable and entered, its step's effect since T8a2 (TWN-VIS-001), and in the hash; since E09
//! T18a its boat offsets (TWN-SET-011, design note §28).

use super::*;

/// [`packs`] with `t`'s town type `t:town`, visited as `visit`, its boat offsets `boat_offsets`.
fn town_packs(visit: Visit, boat_offsets: &[(i8, i8)]) -> Vec<AdventurePack> {
    edit(|p| {
        pack(p, "t").object_types.push(object(
            "t:town",
            visit,
            false,
            ObjectKindDef::Town {
                boat_offsets: boat_offsets.to_vec(),
            },
        ));
    })
}

// covers: ADV-OBJ-002
#[test]
fn the_town_is_an_ownable_kind_entered_with_its_step() {
    let part = compile(&town_packs(Visit::Enter, &[])).unwrap();
    let ty = part
        .object_type(part.object_type_by_key("t:town").unwrap())
        .unwrap();
    assert_eq!(
        ty.kind,
        ObjectKind::Town {
            boat_offsets: Vec::new()
        }
    );
    assert_eq!((ty.visit, ty.from_above), (Visit::Enter, false));
    assert!(ty.kind.ownable());
    assert!(!ty.kind.weekly());
    assert_eq!(ty.kind.effect(), Some(Effect::Town));
    assert_eq!(ty.kind.battle_terrain(), None);
    assert_eq!(ty.kind.dwelling_footprint(), None);
    // An ownable object is entered.
    assert_eq!(
        compile_err(&town_packs(Visit::Neighbour, &[])),
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

// covers: TWN-SET-011
#[test]
fn a_town_s_boat_offsets_are_distinct_never_its_entrance_and_hashed_in_order() {
    let offsets = [(-1, 2), (1, 2)];
    let part = compile(&town_packs(Visit::Enter, &offsets)).unwrap();
    let ty = part
        .object_type(part.object_type_by_key("t:town").unwrap())
        .unwrap();
    assert_eq!(
        ty.kind,
        ObjectKind::Town {
            boat_offsets: offsets.to_vec()
        }
    );
    // Refused: an offset twice, the entrance itself.
    let refused = |boat_offsets: &[(i8, i8)], reason| {
        assert_eq!(
            compile_err(&town_packs(Visit::Enter, boat_offsets)),
            AdventureCompileError::InvalidObjectType {
                key: "t:town".into(),
                reason,
            }
        );
    };
    refused(&[(-1, 2), (-1, 2)], "a town names a boat offset twice");
    refused(&[(1, 2), (0, 0)], "a town's boat offset is its entrance");
    // In the hash (part v19): the offsets, their order and their count.
    let hash = |boat_offsets: &[(i8, i8)]| {
        compile(&town_packs(Visit::Enter, boat_offsets))
            .unwrap()
            .content_hash()
    };
    let hashes = [
        hash(&[]),
        hash(&offsets),
        hash(&[(1, 2), (-1, 2)]),
        hash(&[(-1, 2)]),
        hash(&[(-1, 2), (1, 3)]),
    ];
    assert_eq!(hashes.iter().collect::<BTreeSet<_>>().len(), hashes.len());
}
