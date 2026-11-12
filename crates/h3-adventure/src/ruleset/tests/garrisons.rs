//! Increment 12's kinds in the ruleset part (E08 T113a, increment 12's design note §1 to §3): the
//! garrison with its anti-magic flag, the six bonus kinds with their parameters, every type's kind
//! mark, compiled, checked and in the hash (part v20). Their visits come with T114a and T115b:
//! until then none has an effect.

use super::*;

/// [`packs`] with `t`'s type `t:obj` of `kind`, visited as `visit`, with `kind_mark`.
fn kind_packs(visit: Visit, kind_mark: bool, kind: ObjectKindDef) -> Vec<AdventurePack> {
    edit(|p| {
        pack(p, "t").object_types.push(ObjectTypeDef {
            kind_mark,
            ..object("t:obj", visit, false, kind)
        });
    })
}

/// The compiled `t:obj` of `kind`, entered, with no kind mark.
fn compiled(kind: ObjectKindDef) -> ObjectType {
    let part = compile(&kind_packs(Visit::Enter, false, kind)).unwrap();
    part.object_type(part.object_type_by_key("t:obj").unwrap())
        .unwrap()
        .clone()
}

/// `t:obj`'s compile error with `kind`.
fn refused(kind: ObjectKindDef) -> &'static str {
    match compile_err(&kind_packs(Visit::Enter, false, kind)) {
        AdventureCompileError::InvalidObjectType { key, reason } => {
            assert_eq!(key, "t:obj");
            reason
        }
        other => panic!("{other:?}"),
    }
}

/// A bonus of these days, points and drain, mark 2.
fn bonus(days: [(i8, i8); BONUS_DAYS], points: u32, drain: bool) -> ObjectKindDef {
    ObjectKindDef::Bonus {
        mark: 2,
        days,
        points,
        drain,
    }
}

/// The Stables' kind upgrading `from` to `to`, 400 points.
fn stables(from: &str, to: &str) -> ObjectKindDef {
    ObjectKindDef::Stables {
        points: 400,
        from: key(from),
        to: key(to),
    }
}

/// One definition of each of the seven kinds.
fn kinds() -> [ObjectKindDef; 8] {
    [
        ObjectKindDef::Garrison { anti_magic: false },
        ObjectKindDef::Garrison { anti_magic: true },
        bonus([(1, 0); BONUS_DAYS], 400, false),
        ObjectKindDef::FountainOfFortune {
            mark: 5,
            values: vec![-1, 1, 2, 3],
        },
        ObjectKindDef::MagicWell,
        ObjectKindDef::MagicSpring,
        stables("t:imp", "t:gog"),
        ObjectKindDef::Sirens {
            mark: 13,
            keep: (7, 10),
        },
    ]
}

// covers: ADV-GAR-001, ADV-BON-001
#[test]
fn the_garrisons_and_the_bonus_kinds_compile_with_no_visit_yet() {
    let core = core();
    let creature = |k: &str| core.creature_by_key(k).unwrap();
    let expected = [
        ObjectKind::Garrison { anti_magic: false },
        ObjectKind::Garrison { anti_magic: true },
        ObjectKind::Bonus {
            mark: 2,
            days: [(1, 0); BONUS_DAYS],
            points: 400,
            drain: false,
        },
        ObjectKind::FountainOfFortune {
            mark: 5,
            values: vec![-1, 1, 2, 3],
        },
        ObjectKind::MagicWell,
        ObjectKind::MagicSpring,
        ObjectKind::Stables {
            points: 400,
            from: creature("t:imp"),
            to: creature("t:gog"),
        },
        ObjectKind::Sirens {
            mark: 13,
            keep: (7, 10),
        },
    ];
    for (def, kind) in kinds().into_iter().zip(expected) {
        let ty = compiled(def);
        assert_eq!(ty.kind, kind);
        // No visit yet (T114a, T115b): a step onto one is refused as a blocker's.
        assert_eq!(ty.kind.effect(), None);
        // Only a garrison is ownable; none is weekly.
        assert_eq!(
            ty.kind.ownable(),
            matches!(kind, ObjectKind::Garrison { .. })
        );
        assert!(!ty.kind.weekly());
        assert_eq!(ty.kind.battle_terrain(), None);
        assert_eq!(ty.kind.dwelling_footprint(), None);
        assert_eq!(ty.kind.reveal_terrain(), None);
        assert!(!ty.kind_mark);
    }
    // The kind mark is the type's, as the pack writes it.
    let part = compile(&kind_packs(Visit::Neighbour, true, kinds()[2].clone())).unwrap();
    let ty = part
        .object_type(part.object_type_by_key("t:obj").unwrap())
        .unwrap();
    assert!(ty.kind_mark);
    assert_eq!(ty.visit, Visit::Neighbour);
    // A garrison is entered, as every ownable kind.
    assert_eq!(
        compile_err(&kind_packs(Visit::Neighbour, false, kinds()[0].clone())),
        AdventureCompileError::InvalidObjectType {
            key: "t:obj".into(),
            reason: "an ownable object is not entered",
        }
    );
}

// covers: ADV-BON-001
#[test]
fn the_bonus_kinds_parameters_are_checked() {
    assert_eq!(
        refused(bonus([(0, 2); BONUS_DAYS], 1, true)),
        "a bonus both adds points and drains them"
    );
    // Drained alone (the Swan Pond), or neither, compiles.
    compiled(bonus([(0, 2); BONUS_DAYS], 0, true));
    compiled(bonus([(0, 0); BONUS_DAYS], 0, false));
    let fountain = |values: &[i8]| ObjectKindDef::FountainOfFortune {
        mark: 5,
        values: values.to_vec(),
    };
    assert_eq!(
        refused(fountain(&[])),
        "a fountain of fortune has no values"
    );
    assert_eq!(
        refused(fountain(&[1, 0])),
        "a fountain of fortune's value is 0"
    );
    assert_eq!(
        refused(fountain(&[2, -1, 2])),
        "a fountain of fortune names a value twice"
    );
    assert_eq!(
        refused(ObjectKindDef::Stables {
            points: 0,
            from: key("t:imp"),
            to: key("t:gog"),
        }),
        "a Stables' points are 0"
    );
    assert_eq!(
        refused(stables("t:imp", "t:imp")),
        "a Stables upgrades a creature to itself"
    );
    for (from, to) in [("t:cart", "t:imp"), ("t:imp", "t:cart")] {
        assert_eq!(
            refused(stables(from, to)),
            "a Stables' creature is a war machine"
        );
    }
    for (from, to) in [("t:nobody", "t:imp"), ("t:imp", "t:nobody")] {
        assert_eq!(
            compile_err(&kind_packs(Visit::Enter, false, stables(from, to))),
            AdventureCompileError::UnknownCreature("t:nobody".into())
        );
    }
    for keep in [(1, 0), (11, 10)] {
        assert_eq!(
            refused(ObjectKindDef::Sirens { mark: 13, keep }),
            "a Sirens' keep is not numerator <= denominator, denominator from 1"
        );
    }
    // All of a count kept, or none, compiles.
    for keep in [(10, 10), (0, 1)] {
        compiled(ObjectKindDef::Sirens { mark: 13, keep });
    }
    // A bonus names seven days: a longer week is refused, a shorter one's days are its first.
    let week = |days: u8| {
        let mut packs = kind_packs(Visit::Enter, false, kinds()[2].clone());
        pack(&mut packs, "t")
            .params
            .as_mut()
            .unwrap()
            .calendar
            .week_days = NonZeroU8::new(days).unwrap();
        compile(&packs)
    };
    assert_eq!(
        week(8).unwrap_err(),
        AdventureCompileError::InvalidObjectType {
            key: "t:obj".into(),
            reason: "a bonus's days are fewer than the week's",
        }
    );
    week(5).unwrap();
}

// covers: ADV-GAR-001, ADV-BON-001
#[test]
fn the_new_kinds_parameters_and_the_kind_mark_are_in_the_hash() {
    let hash = |kind_mark: bool, def: ObjectKindDef| {
        compile(&kind_packs(Visit::Enter, kind_mark, def))
            .unwrap()
            .content_hash()
    };
    let mut days = [(1, 0); BONUS_DAYS];
    days[6] = (2, 0);
    let mut defs = kinds().to_vec();
    defs.extend([
        ObjectKindDef::Bonus {
            mark: 3,
            days: [(1, 0); BONUS_DAYS],
            points: 400,
            drain: false,
        },
        bonus(days, 400, false),
        bonus([(1, 0); BONUS_DAYS], 800, false),
        bonus([(1, 0); BONUS_DAYS], 0, true),
        ObjectKindDef::FountainOfFortune {
            mark: 6,
            values: vec![-1, 1, 2, 3],
        },
        ObjectKindDef::FountainOfFortune {
            mark: 5,
            values: vec![-1, 1, 2],
        },
        ObjectKindDef::Stables {
            points: 401,
            from: key("t:imp"),
            to: key("t:gog"),
        },
        stables("t:gog", "t:imp"),
        ObjectKindDef::Sirens {
            mark: 14,
            keep: (7, 10),
        },
        ObjectKindDef::Sirens {
            mark: 13,
            keep: (7, 11),
        },
        ObjectKindDef::Sirens {
            mark: 13,
            keep: (6, 10),
        },
        ObjectKindDef::SubterraneanGate,
    ]);
    let count = defs.len();
    let mut hashes = defs
        .into_iter()
        .map(|def| hash(false, def))
        .collect::<BTreeSet<_>>();
    assert_eq!(hashes.len(), count);
    // The kind mark of the same kind.
    hashes.insert(hash(true, ObjectKindDef::MagicWell));
    assert_eq!(hashes.len(), count + 1);
}
