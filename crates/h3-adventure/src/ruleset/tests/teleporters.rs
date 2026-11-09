//! Increment 11's kinds in the ruleset part (E08 T103a, increment 11's design note §1): the
//! monoliths with their channel and way, the Subterranean Gate and the Whirlpool with its divisor,
//! compiled, entered and in the hash (part v18); the monoliths' and the gates' visit since E08
//! T105a1 (their jump), the Whirlpool's since T105a2 (its loss and the jump).

use super::*;

/// [`packs`] with `t`'s type `t:tele` of `kind`, visited as `visit`.
fn teleporter_packs(visit: Visit, kind: ObjectKindDef) -> Vec<AdventurePack> {
    edit(|p| {
        pack(p, "t")
            .object_types
            .push(object("t:tele", visit, false, kind));
    })
}

/// The compiled `t:tele` of `kind`, entered.
fn teleporter(kind: ObjectKindDef) -> ObjectType {
    let part = compile(&teleporter_packs(Visit::Enter, kind)).unwrap();
    part.object_type(part.object_type_by_key("t:tele").unwrap())
        .unwrap()
        .clone()
}

/// The three kinds' definitions, one of each way for the monoliths.
fn kinds() -> [ObjectKindDef; 5] {
    [
        ObjectKindDef::Monolith {
            channel: 0,
            way: MonolithWay::Entrance,
        },
        ObjectKindDef::Monolith {
            channel: 0,
            way: MonolithWay::Exit,
        },
        ObjectKindDef::Monolith {
            channel: 255,
            way: MonolithWay::TwoWay,
        },
        ObjectKindDef::SubterraneanGate,
        ObjectKindDef::Whirlpool { divisor: 2 },
    ]
}

// covers: ADV-TEL-001
#[test]
fn the_teleporters_are_entered_kinds() {
    assert_eq!(
        teleporter(kinds()[2].clone()).kind,
        ObjectKind::Monolith {
            channel: 255,
            way: MonolithWay::TwoWay
        }
    );
    assert_eq!(
        teleporter(ObjectKindDef::SubterraneanGate).kind,
        ObjectKind::SubterraneanGate
    );
    assert_eq!(
        teleporter(ObjectKindDef::Whirlpool { divisor: 7 }).kind,
        ObjectKind::Whirlpool { divisor: 7 }
    );
    // The monoliths' and the gates' visit is their jump (E08 T105a1); the Whirlpool's its loss
    // and the jump (T105a2).
    let types = kinds().map(teleporter);
    assert_eq!(
        types[2].kind.effect(),
        Some(Effect::Monolith {
            channel: 255,
            way: MonolithWay::TwoWay
        })
    );
    assert_eq!(types[3].kind.effect(), Some(Effect::SubterraneanGate));
    assert_eq!(types[4].kind.effect(), Some(Effect::Whirlpool(2)));
    for def in kinds() {
        let ty = teleporter(def.clone());
        assert_eq!(ty.visit, Visit::Enter);
        // Neither ownable nor weekly.
        assert!(!ty.kind.ownable() && !ty.kind.weekly());
        assert_eq!(ty.kind.battle_terrain(), None);
        assert_eq!(ty.kind.dwelling_footprint(), None);
        assert_eq!(ty.kind.reveal_terrain(), None);
        // A teleporter is entered.
        assert_eq!(
            compile_err(&teleporter_packs(Visit::Neighbour, def)),
            AdventureCompileError::InvalidObjectType {
                key: "t:tele".into(),
                reason: "a teleporter is not entered",
            }
        );
    }
    // A Whirlpool's divisor is at least 2.
    for divisor in [0, 1] {
        assert_eq!(
            compile_err(&teleporter_packs(
                Visit::Enter,
                ObjectKindDef::Whirlpool { divisor }
            )),
            AdventureCompileError::InvalidObjectType {
                key: "t:tele".into(),
                reason: "a whirlpool's divisor is below 2",
            }
        );
    }
}

// covers: ADV-TEL-001
#[test]
fn the_teleporters_parameters_are_in_the_hash() {
    let hash = |def: ObjectKindDef| {
        compile(&teleporter_packs(Visit::Enter, def))
            .unwrap()
            .content_hash()
    };
    let mut defs = kinds().to_vec();
    defs.extend([
        ObjectKindDef::Monolith {
            channel: 1,
            way: MonolithWay::Entrance,
        },
        ObjectKindDef::Monolith {
            channel: 255,
            way: MonolithWay::Exit,
        },
        ObjectKindDef::Whirlpool { divisor: 3 },
        ObjectKindDef::Town,
    ]);
    let hashes = defs.into_iter().map(hash).collect::<BTreeSet<_>>();
    assert_eq!(hashes.len(), 9);
}
