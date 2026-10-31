//! Increment 9's kinds and parameters in the ruleset part (E08 T83a; increment 9's design note
//! §5): the seven kinds compiled, every check, the calendar, the growth, the months and the hash.

use super::*;

/// [`core`]'s packs with adventure counts: `t:imp` 20 to 50, `t:gog` 10 to 20; `t:shade` and
/// `u:wisp` none.
fn economy_core() -> Ruleset {
    let counted = |k: &str, level: u8, count: [u16; 2]| {
        let mut c = creature(k, level, false);
        c.info.adventure_count = count;
        c
    };
    let t = ContentPack {
        terrains: vec![terrain("t:grass"), terrain("t:sea"), terrain("t:rock")],
        resources: vec![resource("t:wood"), resource("t:gold")],
        creatures: vec![
            counted("t:imp", 1, [20, 50]),
            counted("t:gog", 2, [10, 20]),
            creature("t:cart", 1, true),
            creature("t:shade", 3, false),
        ],
        profiles: vec![ProfileDef {
            key: key("t:faithful"),
            quirks: vec![],
        }],
        ..ContentPack::new(manifest("t", &[]))
    };
    let u = ContentPack {
        terrains: vec![terrain("u:swamp")],
        resources: vec![resource("u:gem")],
        creatures: vec![creature("u:wisp", 1, false)],
        ..ContentPack::new(manifest("u", &["t"]))
    };
    let v = ContentPack::new(manifest("v", &[]));
    Ruleset::compile(&[t, u, v], "t:faithful").unwrap()
}

/// The economy's kinds, valid, in `u` (which reaches `t`'s resources, terrains and creatures), by
/// key; all entered, none from above.
fn economy_kinds() -> Vec<(&'static str, ObjectKindDef)> {
    let mine = |resource: &str, daily, battle_terrain: Option<&str>| ObjectKindDef::Mine {
        resource: key(resource),
        daily,
        battle_terrain: battle_terrain.map(key),
    };
    vec![
        ("u:sawmill", mine("t:wood", 2, None)),
        ("u:gold_mine", mine("t:gold", 1000, Some("t:rock"))),
        ("u:gem_pond", mine("u:gem", 1, None)),
        (
            "u:abandoned",
            ObjectKindDef::AbandonedMine {
                guards: key("t:imp"),
                count: [100, 200],
                battle_terrain: Some(key("u:swamp")),
            },
        ),
        (
            "u:lighthouse",
            ObjectKindDef::Lighthouse { sea_points: 500 },
        ),
        (
            "u:shipyard",
            ObjectKindDef::Shipyard {
                price: vec![(key("t:gold"), 1000), (key("t:wood"), 10)],
                boat_type: 1,
                offsets: vec![(-2, 0), (2, 0), (0, -1)],
            },
        ),
        (
            "u:windmill",
            ObjectKindDef::Windmill {
                resources: vec![key("u:gem"), key("t:wood")],
                amount: [3, 6],
            },
        ),
        (
            "u:water_wheel",
            ObjectKindDef::WaterWheel {
                resource: key("t:gold"),
                first: 500,
                later: 1000,
            },
        ),
        (
            "u:garden",
            ObjectKindDef::Garden {
                gifts: vec![(key("u:gem"), 5), (key("t:gold"), 500)],
            },
        ),
    ]
}

/// The months of the original's shape on `t`'s creatures, its monster `t:monster`.
fn months() -> MonthParamsDef {
    MonthParamsDef {
        odds: [5, 4, 1],
        creatures: vec![key("t:gog"), key("t:imp")],
        monster: key("t:monster"),
        spawn_one_in: NonZeroU16::new(200).unwrap(),
        spawn_factor: NonZeroU8::new(2).unwrap(),
        spawn_aggression: [1, 10],
    }
}

/// [`monster_packs`] with the economy's kinds in `u` and the months in `t`'s parameters.
fn economy_packs() -> Vec<AdventurePack> {
    let mut p = monster_packs();
    for (k, kind) in economy_kinds() {
        pack(&mut p, "u")
            .object_types
            .push(object(k, Visit::Enter, false, kind));
    }
    params_of(&mut p).months = Some(months());
    p
}

fn params_of(p: &mut [AdventurePack]) -> &mut AdventureParamsDef {
    pack(p, "t").params.as_mut().unwrap()
}

/// [`economy_packs`] with the kind of `u`'s type `k` changed by `f`.
fn with_economy(k: &str, f: impl FnOnce(&mut ObjectTypeDef)) -> Vec<AdventurePack> {
    let mut p = economy_packs();
    let ty = pack(&mut p, "u")
        .object_types
        .iter_mut()
        .find(|t| t.key.as_str() == k)
        .unwrap();
    f(ty);
    p
}

fn compile_economy(packs: &[AdventurePack]) -> Result<AdventureRuleset, AdventureCompileError> {
    AdventureRuleset::compile(&economy_core(), packs)
}

fn invalid(k: &str, reason: &'static str) -> AdventureCompileError {
    AdventureCompileError::InvalidObjectType {
        key: k.into(),
        reason,
    }
}

// covers: ADV-OBJ-002
#[test]
fn compiles_the_economy_kinds() {
    let core = economy_core();
    let part = compile_economy(&economy_packs()).unwrap();
    let kind = |k: &str| {
        part.object_type(part.object_type_by_key(k).unwrap())
            .unwrap()
            .kind
            .clone()
    };
    let resource = |k: &str| {
        let index = core
            .resources()
            .iter()
            .position(|r| r.key.as_str() == k)
            .unwrap();
        ResourceId::from_index(index).unwrap()
    };
    let (wood, gold, gem) = (resource("t:wood"), resource("t:gold"), resource("u:gem"));
    let terrain = |k: &str| core.terrain_by_key(k);
    assert_eq!(
        kind("u:sawmill"),
        ObjectKind::Mine {
            resource: wood,
            daily: 2,
            battle_terrain: None,
        }
    );
    assert_eq!(
        kind("u:gold_mine"),
        ObjectKind::Mine {
            resource: gold,
            daily: 1000,
            battle_terrain: terrain("t:rock"),
        }
    );
    assert_eq!(
        kind("u:abandoned"),
        ObjectKind::AbandonedMine {
            guards: core.creature_by_key("t:imp").unwrap(),
            count: [100, 200],
            battle_terrain: terrain("u:swamp"),
        }
    );
    assert_eq!(
        kind("u:lighthouse"),
        ObjectKind::Lighthouse { sea_points: 500 }
    );
    // A price sorted by resource, as every price; the offsets in the pack's order.
    assert_eq!(
        kind("u:shipyard"),
        ObjectKind::Shipyard {
            price: vec![(wood, 10), (gold, 1000)],
            boat_type: 1,
            offsets: vec![(-2, 0), (2, 0), (0, -1)],
        }
    );
    // The Windmill's resources in the core's order; the Garden's gifts in the pack's.
    assert_eq!(
        kind("u:windmill"),
        ObjectKind::Windmill {
            resources: vec![wood, gem],
            amount: [3, 6],
        }
    );
    assert_eq!(
        kind("u:water_wheel"),
        ObjectKind::WaterWheel {
            resource: gold,
            first: 500,
            later: 1000,
        }
    );
    assert_eq!(
        kind("u:garden"),
        ObjectKind::Garden {
            gifts: vec![(gem, 5), (gold, 500)],
        }
    );
    // Typed: the mines' visits built (T84a), the others' not yet (T84b); four ownable, three
    // weekly; a mine's battle terrain its own.
    for (k, _) in economy_kinds() {
        let kind = kind(k);
        let mine = matches!(
            kind,
            ObjectKind::Mine { .. } | ObjectKind::AbandonedMine { .. }
        );
        let effect = mine.then_some(Effect::Mine);
        assert_eq!(kind.effect(), effect, "{k}");
        if !mine {
            assert_eq!(kind.battle_terrain(), None, "{k}");
        }
    }
    assert_eq!(
        ["u:sawmill", "u:gold_mine", "u:abandoned"].map(|k| kind(k).battle_terrain()),
        [None, terrain("t:rock"), terrain("u:swamp")]
    );
    for (k, _) in economy_kinds() {
        let kind = kind(k);
        let ownable = !matches!(
            kind,
            ObjectKind::Windmill { .. } | ObjectKind::WaterWheel { .. } | ObjectKind::Garden { .. }
        );
        assert_eq!((kind.ownable(), kind.weekly()), (ownable, !ownable), "{k}");
    }
    assert!(!kind("t:monster").ownable() && !kind("t:monster").weekly());
    // Each resource's mine's daily amount (an abandoned mine's by its drawn resource).
    assert_eq!(
        [wood, gold, gem].map(|r| part.mine_daily(r)),
        [Some(2), Some(1000), Some(1)]
    );
    assert_eq!(compile(&packs()).unwrap().mine_daily(wood), None);
    // The parameters.
    let params = part.params();
    let calendar = params.calendar();
    assert_eq!(
        (calendar.week_days.get(), calendar.month_weeks.get()),
        (7, 4)
    );
    let monsters = params.monsters().unwrap();
    assert_eq!(
        (
            monsters.growth_bp(),
            monsters.growth_unit(),
            monsters.growth_cap()
        ),
        (1000, 16, 4000)
    );
    let creature = |k: &str| core.creature_by_key(k).unwrap();
    assert_eq!(
        params.months(),
        Some(&MonthParams {
            odds: [5, 4, 1],
            creatures: vec![creature("t:gog"), creature("t:imp")],
            monster: part.object_type_by_key("t:monster").unwrap(),
            spawn_one_in: NonZeroU16::new(200).unwrap(),
            spawn_factor: NonZeroU8::new(2).unwrap(),
            spawn_aggression: [1, 10],
        })
    );
    // Without months every month is normal; without creatures, a creature's month needs none.
    let mut p = economy_packs();
    params_of(&mut p).months = None;
    assert_eq!(compile_economy(&p).unwrap().params().months(), None);
    let mut p = economy_packs();
    let m = params_of(&mut p).months.as_mut().unwrap();
    m.odds = [1, 0, 1];
    m.creatures.clear();
    assert!(compile_economy(&p).is_ok());
}

// covers: ADV-OBJ-002
#[test]
fn the_economy_kinds_are_checked() {
    let kind = |k: &'static str, f: &dyn Fn(&mut ObjectKindDef)| {
        compile_economy(&with_economy(k, |t| f(&mut t.kind))).unwrap_err()
    };
    let daily = |d: &mut ObjectKindDef, value| {
        if let ObjectKindDef::Mine { daily, .. } = d {
            *daily = value;
        }
    };
    let bad_daily = invalid("u:sawmill", "a mine's daily amount is 0 or above MAX_GAIN");
    assert_eq!(kind("u:sawmill", &|d| daily(d, 0)), bad_daily);
    assert_eq!(kind("u:sawmill", &|d| daily(d, MAX_GAIN + 1)), bad_daily);
    let count = |d: &mut ObjectKindDef, value: [u32; 2]| {
        if let ObjectKindDef::AbandonedMine { count, .. } = d {
            *count = value;
        }
    };
    let bad_count = invalid(
        "u:abandoned",
        "an abandoned mine's count is not 1 <= low <= high",
    );
    assert_eq!(kind("u:abandoned", &|d| count(d, [0, 1])), bad_count);
    assert_eq!(kind("u:abandoned", &|d| count(d, [2, 1])), bad_count);
    assert_eq!(
        kind("u:abandoned", &|d| count(d, [1, 1_000_001])),
        invalid(
            "u:abandoned",
            "an abandoned mine's count is above max_stack_count"
        )
    );
    assert_eq!(
        kind("u:abandoned", &|d| {
            if let ObjectKindDef::AbandonedMine { guards, .. } = d {
                *guards = key("t:cart");
            }
        }),
        invalid(
            "u:abandoned",
            "an abandoned mine's guards are a war machine"
        )
    );
    assert_eq!(
        kind("u:lighthouse", &|d| *d =
            ObjectKindDef::Lighthouse { sea_points: 0 }),
        invalid("u:lighthouse", "a Lighthouse's sea points are 0")
    );
    type Edit = fn(&mut Vec<(i8, i8)>, &mut Vec<(ContentKey, u32)>);
    let shipyard = |d: &mut ObjectKindDef, f: Edit| {
        if let ObjectKindDef::Shipyard { offsets, price, .. } = d {
            f(offsets, price);
        }
    };
    let cases: [(Edit, &str); 4] = [
        (|o, _| o.clear(), "a Shipyard has no offsets"),
        (|o, _| o.push((2, 0)), "a Shipyard names an offset twice"),
        (|o, _| o.push((0, 0)), "a Shipyard's offset is its entrance"),
        (|_, p| p[0].1 = 0, "a price or gift is 0 or above MAX_GAIN"),
    ];
    for (f, reason) in cases {
        let p = with_economy("u:shipyard", |t| shipyard(&mut t.kind, f));
        assert_eq!(
            compile_economy(&p).unwrap_err(),
            invalid("u:shipyard", reason)
        );
    }
    let windmill = |d: &mut ObjectKindDef, f: Mill| {
        if let ObjectKindDef::Windmill { resources, amount } = d {
            f(resources, amount);
        }
    };
    let bad_amount = "a Windmill's amount is not 1 <= low <= high <= MAX_GAIN";
    type Mill = fn(&mut Vec<ContentKey>, &mut [u32; 2]);
    let cases: [(Mill, &str); 5] = [
        (|r, _| r.clear(), "a Windmill's resources are empty"),
        (
            |r, _| r.push(key("t:wood")),
            "a Windmill names a resource twice",
        ),
        (|_, a| *a = [0, 1], bad_amount),
        (|_, a| *a = [3, 2], bad_amount),
        (|_, a| *a = [1, MAX_GAIN + 1], bad_amount),
    ];
    for (f, reason) in cases {
        let p = with_economy("u:windmill", |t| windmill(&mut t.kind, f));
        assert_eq!(
            compile_economy(&p).unwrap_err(),
            invalid("u:windmill", reason)
        );
    }
    let wheel = |first, later| ObjectKindDef::WaterWheel {
        resource: key("t:gold"),
        first,
        later,
    };
    for (first, later) in [(0, 1000), (500, 0), (MAX_GAIN + 1, 1)] {
        let p = with_economy("u:water_wheel", |t| t.kind = wheel(first, later));
        assert_eq!(
            compile_economy(&p).unwrap_err(),
            invalid(
                "u:water_wheel",
                "a Water Wheel's amount is 0 or above MAX_GAIN"
            )
        );
    }
    assert_eq!(
        kind("u:garden", &|d| *d =
            ObjectKindDef::Garden { gifts: vec![] }),
        invalid("u:garden", "a Garden has no gifts")
    );
    for amount in [0, MAX_GAIN + 1] {
        let p = with_economy("u:garden", |t| {
            t.kind = ObjectKindDef::Garden {
                gifts: vec![(key("t:gold"), amount)],
            }
        });
        assert_eq!(
            compile_economy(&p).unwrap_err(),
            invalid("u:garden", "a Garden's gift is 0 or above MAX_GAIN")
        );
    }
    // The ownable four are entered; a weekly object may be visited from a neighbour.
    for k in ["u:sawmill", "u:abandoned", "u:lighthouse", "u:shipyard"] {
        let p = with_economy(k, |t| t.visit = Visit::Neighbour);
        assert_eq!(
            compile_economy(&p).unwrap_err(),
            invalid(k, "an ownable object is not entered")
        );
    }
    let p = with_economy("u:garden", |t| t.visit = Visit::Neighbour);
    assert!(compile_economy(&p).is_ok());
    // One mine a resource; with an abandoned mine, one for every resource.
    let p = with_economy("u:gem_pond", |t| {
        t.kind = ObjectKindDef::Mine {
            resource: key("t:wood"),
            daily: 1,
            battle_terrain: None,
        }
    });
    assert_eq!(
        compile_economy(&p).unwrap_err(),
        invalid("u:gem_pond", "a second mine produces its resource")
    );
    let p = with_economy("u:gem_pond", |t| t.kind = ObjectKindDef::Scroll);
    assert_eq!(
        compile_economy(&p).unwrap_err(),
        invalid("u:abandoned", "a core resource has no mine")
    );
    let mut p = economy_packs();
    pack(&mut p, "u").object_types.retain(|t| {
        !matches!(
            t.kind,
            ObjectKindDef::AbandonedMine { .. } | ObjectKindDef::Mine { .. }
        )
    });
    pack(&mut p, "u").object_types.push(object(
        "u:only",
        Visit::Enter,
        false,
        ObjectKindDef::Mine {
            resource: key("u:gem"),
            daily: 1,
            battle_terrain: None,
        },
    ));
    assert!(compile_economy(&p).is_ok());
    // References: unknown, and outside the pack's dependencies.
    let unknown = |k: &'static str, f: fn(&mut ObjectKindDef)| {
        compile_economy(&with_economy(k, |t| f(&mut t.kind))).unwrap_err()
    };
    assert_eq!(
        unknown("u:sawmill", |d| {
            if let ObjectKindDef::Mine { resource, .. } = d {
                *resource = key("t:stone");
            }
        }),
        AdventureCompileError::UnknownResource("t:stone".into())
    );
    assert_eq!(
        unknown("u:gold_mine", |d| {
            if let ObjectKindDef::Mine { battle_terrain, .. } = d {
                *battle_terrain = Some(key("t:lava"));
            }
        }),
        AdventureCompileError::UnknownTerrain("t:lava".into())
    );
    assert_eq!(
        unknown("u:abandoned", |d| {
            if let ObjectKindDef::AbandonedMine { guards, .. } = d {
                *guards = key("t:ghost");
            }
        }),
        AdventureCompileError::UnknownCreature("t:ghost".into())
    );
    assert_eq!(
        unknown("u:abandoned", |d| {
            if let ObjectKindDef::AbandonedMine { battle_terrain, .. } = d {
                *battle_terrain = Some(key("v:void"));
            }
        }),
        AdventureCompileError::ReferenceOutsideDependencies {
            key: "v:void".into(),
            pack: "u".into(),
        }
    );
    assert_eq!(
        unknown("u:windmill", |d| *d = ObjectKindDef::Windmill {
            resources: vec![key("t:stone")],
            amount: [1, 2],
        }),
        AdventureCompileError::UnknownResource("t:stone".into())
    );
    assert_eq!(
        unknown("u:water_wheel", |d| *d = ObjectKindDef::WaterWheel {
            resource: key("t:stone"),
            first: 1,
            later: 1,
        }),
        AdventureCompileError::UnknownResource("t:stone".into())
    );
    assert_eq!(
        unknown("u:garden", |d| *d = ObjectKindDef::Garden {
            gifts: vec![(key("t:stone"), 1)],
        }),
        AdventureCompileError::UnknownResource("t:stone".into())
    );
    assert_eq!(
        unknown("u:shipyard", |d| *d = ObjectKindDef::Shipyard {
            price: vec![(key("t:stone"), 1)],
            boat_type: 0,
            offsets: vec![(1, 1)],
        }),
        AdventureCompileError::UnknownResource("t:stone".into())
    );
}

// covers: ADV-OBJ-002
#[test]
fn the_calendar_growth_and_months_are_checked() {
    let params = |f: fn(&mut AdventureParamsDef)| {
        let mut p = economy_packs();
        f(params_of(&mut p));
        compile_economy(&p)
    };
    let reason = |f: fn(&mut AdventureParamsDef)| match params(f) {
        Err(AdventureCompileError::InvalidParams(reason)) => reason,
        other => panic!("{other:?}"),
    };
    assert_eq!(
        reason(|p| p.monsters.as_mut().unwrap().growth_bp = 10_001),
        "monsters.growth_bp is above 10 000"
    );
    assert!(params(|p| p.monsters.as_mut().unwrap().growth_bp = 10_000).is_ok());
    assert_eq!(
        reason(|p| p.monsters.as_mut().unwrap().growth_unit = NonZeroU16::new(257).unwrap()),
        "monsters.growth_unit is above 256"
    );
    assert!(
        params(|p| p.monsters.as_mut().unwrap().growth_unit = NonZeroU16::new(256).unwrap())
            .is_ok()
    );
    assert_eq!(
        reason(|p| p.monsters.as_mut().unwrap().growth_cap =
            NonZeroU32::new(MAX_MONSTER_COUNT + 1).unwrap()),
        "monsters.growth_cap is above MAX_MONSTER_COUNT"
    );
    let month_reason = |f: &dyn Fn(&mut MonthParamsDef)| {
        let mut p = economy_packs();
        f(params_of(&mut p).months.as_mut().unwrap());
        match compile_economy(&p) {
            Err(AdventureCompileError::InvalidParams(reason)) => reason,
            other => panic!("{other:?}"),
        }
    };
    assert_eq!(
        month_reason(&|m| m.odds = [0, 0, 0]),
        "months.odds sum to 0"
    );
    assert_eq!(
        month_reason(&|m| m.creatures.push(key("t:imp"))),
        "months.creatures names a creature twice"
    );
    assert_eq!(
        month_reason(&|m| m.creatures.clear()),
        "months.creatures is empty with a creature's month's weight"
    );
    assert_eq!(
        month_reason(&|m| m.spawn_aggression = [10, 1]),
        "months.spawn_aggression's low bound is above its high one"
    );
    assert_eq!(
        month_reason(&|m| m.creatures = vec![key("t:cart")]),
        "a months.creatures entry is a war machine"
    );
    assert_eq!(
        month_reason(&|m| m.creatures = vec![key("t:shade")]),
        "a months.creatures entry has no adventure count"
    );
    assert_eq!(
        month_reason(&|m| m.spawn_factor = NonZeroU8::new(82).unwrap()),
        "a months.creatures entry's count times spawn_factor is above MAX_MONSTER_COUNT"
    );
    assert!(
        params(|p| p.months.as_mut().unwrap().spawn_factor = NonZeroU8::new(81).unwrap()).is_ok()
    );
    assert_eq!(
        month_reason(&|m| m.monster = key("t:random")),
        "months.monster is not a monster whose creature the map names"
    );
    assert_eq!(
        month_reason(&|m| m.monster = key("t:wood_pile")),
        "months.monster is not a monster whose creature the map names"
    );
    assert_eq!(
        reason(|p| p.monsters = None),
        "months without the monsters' parameters"
    );
    // References: an unknown type or creature, a creature without an adventure count, a key
    // outside the parameters' pack's dependencies.
    let error = |f: &dyn Fn(&mut MonthParamsDef)| {
        let mut p = economy_packs();
        f(params_of(&mut p).months.as_mut().unwrap());
        compile_economy(&p).unwrap_err()
    };
    assert_eq!(
        error(&|m| m.monster = key("t:dragon")),
        AdventureCompileError::UnknownObjectType("t:dragon".into())
    );
    assert_eq!(
        error(&|m| m.creatures = vec![key("t:ghost")]),
        AdventureCompileError::UnknownCreature("t:ghost".into())
    );
    assert_eq!(
        error(&|m| m.creatures = vec![key("u:wisp")]),
        AdventureCompileError::ReferenceOutsideDependencies {
            key: "u:wisp".into(),
            pack: "t".into(),
        }
    );
    assert_eq!(
        error(&|m| m.monster = key("u:sawmill")),
        AdventureCompileError::ReferenceOutsideDependencies {
            key: "u:sawmill".into(),
            pack: "t".into(),
        }
    );
    // Another monster whose creature the map names serves as well.
    let mut p = economy_packs();
    pack(&mut p, "t").object_types.push(object(
        "t:empty",
        Visit::Neighbour,
        true,
        ObjectKindDef::Monster { levels: None },
    ));
    params_of(&mut p).months.as_mut().unwrap().monster = key("t:empty");
    assert!(compile_economy(&p).is_ok());
}

#[test]
fn the_hash_sees_the_economy() {
    let base = compile_economy(&economy_packs()).unwrap();
    let mut changes: Vec<Vec<AdventurePack>> = Vec::new();
    let mut kind = |k: &str, kind: ObjectKindDef| {
        changes.push(with_economy(k, |t| t.kind = kind));
    };
    let mine = |daily, battle_terrain: Option<&str>| ObjectKindDef::Mine {
        resource: key("t:gold"),
        daily,
        battle_terrain: battle_terrain.map(key),
    };
    kind("u:gold_mine", mine(999, Some("t:rock")));
    kind("u:gold_mine", mine(1000, None));
    kind("u:gold_mine", mine(1000, Some("t:grass")));
    let abandoned =
        |guards: &str, count, battle_terrain: Option<&str>| ObjectKindDef::AbandonedMine {
            guards: key(guards),
            count,
            battle_terrain: battle_terrain.map(key),
        };
    kind(
        "u:abandoned",
        abandoned("t:gog", [100, 200], Some("u:swamp")),
    );
    kind(
        "u:abandoned",
        abandoned("t:imp", [101, 200], Some("u:swamp")),
    );
    kind(
        "u:abandoned",
        abandoned("t:imp", [100, 201], Some("u:swamp")),
    );
    kind("u:abandoned", abandoned("t:imp", [100, 200], None));
    kind(
        "u:lighthouse",
        ObjectKindDef::Lighthouse { sea_points: 501 },
    );
    let shipyard = |price: u32, boat_type, offsets: Vec<(i8, i8)>| ObjectKindDef::Shipyard {
        price: vec![(key("t:gold"), price), (key("t:wood"), 10)],
        boat_type,
        offsets,
    };
    kind(
        "u:shipyard",
        shipyard(999, 1, vec![(-2, 0), (2, 0), (0, -1)]),
    );
    kind(
        "u:shipyard",
        shipyard(1000, 2, vec![(-2, 0), (2, 0), (0, -1)]),
    );
    kind(
        "u:shipyard",
        shipyard(1000, 1, vec![(2, 0), (-2, 0), (0, -1)]),
    );
    kind("u:shipyard", shipyard(1000, 1, vec![(-2, 0), (2, 0)]));
    let windmill = |resources: &[&str], amount| ObjectKindDef::Windmill {
        resources: resources.iter().map(|k| key(k)).collect(),
        amount,
    };
    kind("u:windmill", windmill(&["u:gem"], [3, 6]));
    kind("u:windmill", windmill(&["u:gem", "t:wood"], [2, 6]));
    kind("u:windmill", windmill(&["u:gem", "t:wood"], [3, 7]));
    let wheel = |resource: &str, first, later| ObjectKindDef::WaterWheel {
        resource: key(resource),
        first,
        later,
    };
    kind("u:water_wheel", wheel("t:wood", 500, 1000));
    kind("u:water_wheel", wheel("t:gold", 400, 1000));
    kind("u:water_wheel", wheel("t:gold", 500, 900));
    kind(
        "u:garden",
        ObjectKindDef::Garden {
            gifts: vec![(key("t:gold"), 500), (key("u:gem"), 5)],
        },
    );
    kind(
        "u:garden",
        ObjectKindDef::Garden {
            gifts: vec![(key("u:gem"), 5)],
        },
    );
    let mut params = |f: fn(&mut AdventureParamsDef)| {
        let mut p = economy_packs();
        f(params_of(&mut p));
        changes.push(p);
    };
    params(|p| p.calendar.week_days = NonZeroU8::new(6).unwrap());
    params(|p| p.calendar.month_weeks = NonZeroU8::new(5).unwrap());
    params(|p| p.monsters.as_mut().unwrap().growth_bp = 999);
    params(|p| p.monsters.as_mut().unwrap().growth_unit = NonZeroU16::new(15).unwrap());
    params(|p| p.monsters.as_mut().unwrap().growth_cap = NonZeroU32::new(3999).unwrap());
    params(|p| p.months = None);
    params(|p| p.months.as_mut().unwrap().odds = [5, 4, 2]);
    params(|p| p.months.as_mut().unwrap().creatures.reverse());
    params(|p| p.months.as_mut().unwrap().monster = key("t:empty_monster"));
    params(|p| p.months.as_mut().unwrap().spawn_one_in = NonZeroU16::new(199).unwrap());
    params(|p| p.months.as_mut().unwrap().spawn_factor = NonZeroU8::new(3).unwrap());
    params(|p| p.months.as_mut().unwrap().spawn_aggression = [2, 10]);
    params(|p| p.months.as_mut().unwrap().spawn_aggression = [1, 9]);
    let extra = object(
        "t:empty_monster",
        Visit::Neighbour,
        true,
        ObjectKindDef::Monster { levels: None },
    );
    for (i, mut packs) in changes.into_iter().enumerate() {
        pack(&mut packs, "t").object_types.push(extra.clone());
        let mut with_extra = economy_packs();
        pack(&mut with_extra, "t").object_types.push(extra.clone());
        let expected = compile_economy(&with_extra).unwrap().content_hash();
        assert_ne!(
            compile_economy(&packs).unwrap().content_hash(),
            expected,
            "change {i}"
        );
    }
    assert_ne!(
        base.content_hash(),
        compile(&monster_packs()).unwrap().content_hash()
    );
}
