//! Increment 7's kinds in the ruleset part (E08 T61a; design §8): their parameters compiled,
//! every check, the references and the hash.

use h3_content::{ArtifactClass, SecondarySkillDef};

use super::*;
use crate::PrimarySkill;

fn resources(amount: [u32; 2], list: &[&str]) -> FindContentDef {
    FindContentDef::Resources {
        amount,
        resources: keys(list),
    }
}

/// [`core`]'s packs with three secondary skills in `t` (`t:archery`, `t:wisdom`, `t:luck`).
fn skill_core() -> Ruleset {
    let skill = |k: &str| SecondarySkillDef {
        key: key(k),
        name: "Skill".into(),
        levels: Default::default(),
    };
    let t = ContentPack {
        terrains: vec![terrain("t:grass"), terrain("t:sea"), terrain("t:rock")],
        resources: vec![resource("t:wood"), resource("t:gold")],
        creatures: vec![
            creature("t:imp", 1, false),
            creature("t:gog", 2, false),
            creature("t:cart", 1, true),
        ],
        skills: vec![skill("t:archery"), skill("t:wisdom"), skill("t:luck")],
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

fn keys(list: &[&str]) -> Vec<ContentKey> {
    list.iter().map(|k| key(k)).collect()
}

fn chest() -> ChestDef {
    ChestDef {
        resource: key("t:gold"),
        gold: vec![(32, 1000), (32, 1500), (31, 2000)],
        artifact: vec![ArtifactClass::Treasure],
        full_backpack: 1000,
        experience_offset: 500,
    }
}

fn gift() -> GiftDef {
    GiftDef {
        gives: Gives::Choice {
            skills: [PrimarySkill::Power, PrimarySkill::Knowledge],
            amount: 1,
        },
        prices: vec![
            vec![],
            vec![(key("t:gold"), 1000)],
            vec![(key("u:gem"), 10)],
        ],
        admits: Some(10),
        leave: true,
    }
}

fn find() -> FindDef {
    FindDef {
        draws: vec![
            (10, FindContentDef::Nothing),
            (
                40,
                FindContentDef::Artifact(vec![ArtifactClass::Treasure, ArtifactClass::Minor]),
            ),
            (
                50,
                FindContentDef::Resources {
                    amount: [2, 5],
                    resources: keys(&["t:wood", "u:gem"]),
                },
            ),
        ],
        full_backpack: Some(TurnDef::Drawn),
        question: true,
        bonus: Some(OnceBonus {
            morale: -3,
            luck: 0,
            mark: 0,
        }),
        leaves: false,
    }
}

fn pyramid() -> PyramidDef {
    PyramidDef {
        split: (key("t:imp"), 40),
        fixed: (key("u:wisp"), vec![10, 10]),
        level: 5,
        looted: OnceBonus {
            morale: 0,
            luck: -2,
            mark: 1,
        },
    }
}

/// Every kind of increment 7, valid, in `u` (which reaches `t`'s skills and creatures).
fn reward_kinds() -> Vec<ObjectKindDef> {
    vec![
        ObjectKindDef::Chest(chest()),
        ObjectKindDef::Artifact { classes: None },
        ObjectKindDef::Artifact {
            classes: Some(vec![ArtifactClass::Relic]),
        },
        ObjectKindDef::Scroll,
        ObjectKindDef::Gift(gift()),
        ObjectKindDef::Shrine { level: 3 },
        ObjectKindDef::WitchHut {
            roe: keys(&["t:wisdom"]),
            roe_fixed: keys(&["t:luck", "t:archery"]),
        },
        ObjectKindDef::Scholar,
        ObjectKindDef::University {
            offers: 4,
            price: vec![(key("t:gold"), 2000)],
        },
        ObjectKindDef::Find(find()),
        ObjectKindDef::PandoraBox,
        ObjectKindDef::Event,
        ObjectKindDef::Pyramid(pyramid()),
    ]
}

/// [`packs`] with the reward kinds as `u:reward_<n>`.
fn reward_packs() -> Vec<AdventurePack> {
    edit(|packs| {
        let u = pack(packs, "u");
        for (n, kind) in reward_kinds().into_iter().enumerate() {
            u.object_types
                .push(object(&format!("u:reward_{n}"), Visit::Enter, false, kind));
        }
    })
}

fn compile_rewards(packs: &[AdventurePack]) -> Result<AdventureRuleset, AdventureCompileError> {
    AdventureRuleset::compile(&skill_core(), packs)
}

/// `reward_packs` with the `n`th reward kind changed by `f`.
fn with_kind(n: usize, f: impl FnOnce(&mut ObjectKindDef)) -> Vec<AdventurePack> {
    let mut packs = reward_packs();
    let u = pack(&mut packs, "u");
    let ty = u
        .object_types
        .iter_mut()
        .find(|t| t.key.as_str() == format!("u:reward_{n}"))
        .unwrap();
    f(&mut ty.kind);
    packs
}

// covers: ADV-OBJ-002
#[test]
fn compiles_the_reward_kinds() {
    let core = skill_core();
    let part = compile_rewards(&reward_packs()).unwrap();
    let kind = |n: usize| {
        let id = part.object_type_by_key(&format!("u:reward_{n}")).unwrap();
        part.object_type(id).unwrap().kind.clone()
    };
    let resource = |k: &str| {
        let index = core
            .resources()
            .iter()
            .position(|r| r.key.as_str() == k)
            .unwrap();
        ResourceId::from_index(index).unwrap()
    };
    let skill = |k: &str| core.skill_by_key(k).unwrap();
    let creature = |k: &str| core.creature_by_key(k).unwrap();
    let (gold, wood, gem) = (resource("t:gold"), resource("t:wood"), resource("u:gem"));
    assert_eq!(
        kind(0),
        ObjectKind::Chest(Chest {
            resource: gold,
            gold: vec![(32, 1000), (32, 1500), (31, 2000)],
            artifact: vec![ArtifactClass::Treasure],
            full_backpack: 1000,
            experience_offset: 500,
        })
    );
    assert_eq!(kind(1), ObjectKind::Artifact { classes: None });
    assert_eq!(
        kind(2),
        ObjectKind::Artifact {
            classes: Some(vec![ArtifactClass::Relic])
        }
    );
    assert_eq!(kind(3), ObjectKind::Scroll);
    assert_eq!(
        kind(4),
        ObjectKind::Gift(Gift {
            gives: Gives::Choice {
                skills: [PrimarySkill::Power, PrimarySkill::Knowledge],
                amount: 1,
            },
            prices: vec![vec![], vec![(gold, 1000)], vec![(gem, 10)]],
            admits: Some(10),
            leave: true,
        })
    );
    assert_eq!(kind(5), ObjectKind::Shrine { level: 3 });
    assert_eq!(
        kind(6),
        ObjectKind::WitchHut {
            roe: vec![skill("t:wisdom")],
            roe_fixed: vec![skill("t:luck"), skill("t:archery")],
        }
    );
    assert_eq!(kind(7), ObjectKind::Scholar);
    assert_eq!(
        kind(8),
        ObjectKind::University {
            offers: 4,
            price: vec![(gold, 2000)]
        }
    );
    assert_eq!(
        kind(9),
        ObjectKind::Find(Find {
            draws: vec![
                (10, FindContent::Nothing),
                (
                    40,
                    FindContent::Artifact(vec![ArtifactClass::Treasure, ArtifactClass::Minor])
                ),
                (
                    50,
                    FindContent::Resources {
                        amount: [2, 5],
                        resources: vec![wood, gem],
                    }
                ),
            ],
            full_backpack: Some(Turn::Drawn),
            question: true,
            bonus: Some(OnceBonus {
                morale: -3,
                luck: 0,
                mark: 0
            }),
            leaves: false,
        })
    );
    assert_eq!(kind(10), ObjectKind::PandoraBox);
    assert_eq!(kind(11), ObjectKind::Event);
    assert_eq!(
        kind(12),
        ObjectKind::Pyramid(Pyramid {
            split: (creature("t:imp"), 40),
            fixed: (creature("u:wisp"), vec![10, 10]),
            level: 5,
            looted: OnceBonus {
                morale: 0,
                luck: -2,
                mark: 1
            },
        })
    );
    // Their visits (E08 T62a, T62b1): every kind but the event, which a step fires (T62b2).
    for n in 0..reward_kinds().len() {
        let fired = matches!(kind(n), ObjectKind::Event);
        assert_eq!(kind(n).effect().is_none(), fired, "{n}");
    }
    // The other gifts' shapes, a chest of gold only, a find's other turns.
    for gives in [
        Gives::Primary([0, 1, 0, 0]),
        Gives::Experience(1000),
        Gives::Level,
    ] {
        let packs = with_kind(4, |k| {
            *k = ObjectKindDef::Gift(GiftDef {
                gives,
                prices: vec![],
                admits: None,
                leave: false,
            });
        });
        assert!(compile_rewards(&packs).is_ok(), "{gives:?}");
    }
    let packs = with_kind(0, |k| {
        *k = ObjectKindDef::Chest(ChestDef {
            gold: vec![(100, 500)],
            artifact: vec![],
            ..chest()
        });
    });
    assert!(compile_rewards(&packs).is_ok());
    for turn in [TurnDef::Stays, TurnDef::Give(vec![(key("t:gold"), 1000)])] {
        let packs = with_kind(9, |k| {
            *k = ObjectKindDef::Find(FindDef {
                full_backpack: Some(turn.clone()),
                bonus: None,
                ..find()
            });
        });
        assert!(compile_rewards(&packs).is_ok(), "{turn:?}");
    }
}

// covers: ADV-OBJ-002
#[test]
fn the_reward_kinds_parameters_are_checked() {
    let invalid = |n: usize, reason| AdventureCompileError::InvalidObjectType {
        key: format!("u:reward_{n}"),
        reason,
    };
    let check = |n: usize, f: &dyn Fn(&mut ObjectKindDef), reason: &'static str| {
        let packs = with_kind(n, f);
        assert_eq!(
            compile_rewards(&packs).unwrap_err(),
            invalid(n, reason),
            "{reason}"
        );
    };
    let chest_with = |f: fn(&mut ChestDef)| {
        move |k: &mut ObjectKindDef| {
            let mut def = chest();
            f(&mut def);
            *k = ObjectKindDef::Chest(def);
        }
    };
    check(0, &chest_with(|c| c.gold[0].0 = 0), "a chance is 0");
    check(
        0,
        &chest_with(|c| c.gold[0].0 = 40),
        "the chances do not add up to 100",
    );
    check(0, &chest_with(|c| c.gold.clear()), "a chest has no gold");
    let gold = "a chest's gold is 0, below the experience offset or above MAX_GAIN";
    check(0, &chest_with(|c| c.gold[1].1 = 400), gold);
    check(0, &chest_with(|c| c.full_backpack = MAX_GAIN + 1), gold);
    check(
        0,
        &chest_with(|c| {
            c.experience_offset = 0;
            c.full_backpack = 0;
        }),
        gold,
    );
    check(
        0,
        &chest_with(|c| c.artifact.clear()),
        "artifact classes are empty",
    );
    check(
        0,
        &chest_with(|c| c.artifact.push(ArtifactClass::Treasure)),
        "artifact classes name a class twice",
    );
    check(
        0,
        &chest_with(|c| c.artifact = vec![ArtifactClass::Special]),
        "artifact classes name Special",
    );
    check(
        0,
        &chest_with(|c| c.gold[2].0 = 36),
        "a chest's chances leave no room for its artifact",
    );
    check(
        2,
        &|k| {
            *k = ObjectKindDef::Artifact {
                classes: Some(vec![]),
            }
        },
        "artifact classes are empty",
    );
    let gift_with = |f: fn(&mut GiftDef)| {
        move |k: &mut ObjectKindDef| {
            let mut def = gift();
            f(&mut def);
            *k = ObjectKindDef::Gift(def);
        }
    };
    check(
        4,
        &gift_with(|g| g.prices[1].push((key("t:gold"), 5))),
        "a price or gift names a resource twice",
    );
    check(
        4,
        &gift_with(|g| g.prices[2][0].1 = 0),
        "a price or gift is 0 or above MAX_GAIN",
    );
    check(
        4,
        &gift_with(|g| g.gives = Gives::Primary([0; 4])),
        "a gift's primary gains are all 0",
    );
    check(
        4,
        &gift_with(|g| {
            g.gives = Gives::Choice {
                skills: [PrimarySkill::Attack; 2],
                amount: 1,
            }
        }),
        "a gift's choice names one skill twice",
    );
    check(
        4,
        &gift_with(|g| {
            g.gives = Gives::Choice {
                skills: [PrimarySkill::Attack, PrimarySkill::Defense],
                amount: 0,
            }
        }),
        "a gift's choice gives 0",
    );
    for amount in [0, MAX_GAIN + 1] {
        let packs = with_kind(4, |k| {
            *k = ObjectKindDef::Gift(GiftDef {
                gives: Gives::Experience(amount),
                ..gift()
            });
        });
        assert_eq!(
            compile_rewards(&packs).unwrap_err(),
            invalid(4, "a gift's experience is 0 or above MAX_GAIN")
        );
    }
    check(
        4,
        &gift_with(|g| g.admits = Some(0)),
        "a gift's condition is 0",
    );
    check(
        5,
        &|k| *k = ObjectKindDef::Shrine { level: 0 },
        "a Shrine's level is 0",
    );
    let twice = "skills name a skill twice";
    check(
        6,
        &|k| {
            *k = ObjectKindDef::WitchHut {
                roe: keys(&["t:luck", "t:luck"]),
                roe_fixed: vec![],
            }
        },
        twice,
    );
    check(
        6,
        &|k| {
            *k = ObjectKindDef::WitchHut {
                roe: vec![],
                roe_fixed: keys(&["t:luck", "t:luck"]),
            }
        },
        twice,
    );
    check(
        8,
        &|k| {
            *k = ObjectKindDef::University {
                offers: 0,
                price: vec![],
            }
        },
        "a University offers nothing",
    );
    check(
        8,
        &|k| {
            *k = ObjectKindDef::University {
                offers: 1,
                price: vec![(key("t:gold"), MAX_GAIN + 1)],
            }
        },
        "a price or gift is 0 or above MAX_GAIN",
    );
    let find_with = |f: fn(&mut FindDef)| {
        move |k: &mut ObjectKindDef| {
            let mut def = find();
            f(&mut def);
            *k = ObjectKindDef::Find(def);
        }
    };
    check(
        9,
        &find_with(|f| f.draws[1].1 = FindContentDef::Artifact(vec![])),
        "artifact classes are empty",
    );
    let amount = "a find's amount is not 1 <= low <= high <= MAX_GAIN";
    for bounds in [[0, 1], [3, 2], [1, MAX_GAIN + 1]] {
        let packs = with_kind(9, |k| {
            let mut def = find();
            def.draws[2].1 = resources(bounds, &["t:wood"]);
            *k = ObjectKindDef::Find(def);
        });
        assert_eq!(compile_rewards(&packs).unwrap_err(), invalid(9, amount));
    }
    check(
        9,
        &find_with(|f| f.draws[2].1 = resources([1, 2], &[])),
        "a find's resources are empty",
    );
    check(
        9,
        &find_with(|f| f.draws[2].1 = resources([1, 2], &["t:wood", "t:wood"])),
        "a find's resources name a resource twice",
    );
    check(
        9,
        &find_with(|f| f.full_backpack = Some(TurnDef::Give(vec![]))),
        "a find's turn gives nothing",
    );
    check(
        9,
        &find_with(|f| {
            f.full_backpack = Some(TurnDef::Give(vec![(key("t:gold"), 0)]));
        }),
        "a price or gift is 0 or above MAX_GAIN",
    );
    check(9, &find_with(|f| f.draws[0].0 = 0), "a chance is 0");
    check(
        9,
        &find_with(|f| f.draws[0].0 = 11),
        "the chances do not add up to 100",
    );
    check(
        9,
        &find_with(|f| f.draws[0].0 = 9),
        "the chances do not add up to 100",
    );
    check(
        9,
        &find_with(|f| f.draws[0].1 = resources([1, 2], &["t:wood"])),
        "a find has two resources entries",
    );
    let turn = "a find has a full backpack's turn exactly when it may hold an artifact";
    check(9, &find_with(|f| f.full_backpack = None), turn);
    check(
        9,
        &find_with(|f| f.draws[1].1 = FindContentDef::Nothing),
        turn,
    );
    check(
        9,
        &find_with(|f| f.draws[2].1 = FindContentDef::Nothing),
        "a find's drawn turn has no resources entry",
    );
    let mark = "a bonus's mark is not below BattleBonuses::MARKS";
    check(
        9,
        &find_with(|f| {
            f.bonus = Some(OnceBonus {
                morale: 1,
                luck: 1,
                mark: 8,
            })
        }),
        mark,
    );
    let pyramid_with = |f: fn(&mut PyramidDef)| {
        move |k: &mut ObjectKindDef| {
            let mut def = pyramid();
            f(&mut def);
            *k = ObjectKindDef::Pyramid(def);
        }
    };
    check(
        12,
        &pyramid_with(|p| p.split.0 = key("t:cart")),
        "a Pyramid's guard is a war machine",
    );
    check(
        12,
        &pyramid_with(|p| p.fixed.0 = key("t:cart")),
        "a Pyramid's guard is a war machine",
    );
    let count = "a Pyramid's count is 0 or above MAX_MONSTER_COUNT";
    check(12, &pyramid_with(|p| p.split.1 = 0), count);
    check(
        12,
        &pyramid_with(|p| p.fixed.1[1] = MAX_MONSTER_COUNT + 1),
        count,
    );
    check(
        12,
        &pyramid_with(|p| p.fixed.1.clear()),
        "a Pyramid's second creature has no stack",
    );
    check(
        12,
        &pyramid_with(|p| p.level = 0),
        "a Pyramid's spell level is 0",
    );
    check(12, &pyramid_with(|p| p.looted.mark = 9), mark);
    // Once the army's slots are known: the second creature leaves the first a slot.
    check(
        12,
        &pyramid_with(|p| p.fixed.1 = vec![1; 7]),
        "a Pyramid's second creature fills every army slot",
    );
    assert!(compile_rewards(&with_kind(12, pyramid_with(|p| p.fixed.1 = vec![1; 6]))).is_ok());
}

// covers: ADV-OBJ-002
#[test]
fn the_reward_kinds_reference_known_content() {
    let error =
        |n: usize, f: &dyn Fn(&mut ObjectKindDef)| compile_rewards(&with_kind(n, f)).unwrap_err();
    let unknown_resource = AdventureCompileError::UnknownResource("t:ruby".into());
    let ruby = || vec![(key("t:ruby"), 1)];
    assert_eq!(
        error(0, &|k| {
            *k = ObjectKindDef::Chest(ChestDef {
                resource: key("t:ruby"),
                ..chest()
            })
        }),
        unknown_resource
    );
    assert_eq!(
        error(4, &|k| {
            *k = ObjectKindDef::Gift(GiftDef {
                prices: vec![ruby()],
                ..gift()
            })
        }),
        unknown_resource
    );
    assert_eq!(
        error(8, &|k| {
            *k = ObjectKindDef::University {
                offers: 1,
                price: ruby(),
            }
        }),
        unknown_resource
    );
    assert_eq!(
        error(9, &|k| {
            let mut def = find();
            def.draws[2].1 = FindContentDef::Resources {
                amount: [1, 2],
                resources: keys(&["t:ruby"]),
            };
            *k = ObjectKindDef::Find(def);
        }),
        unknown_resource
    );
    assert_eq!(
        error(9, &|k| {
            *k = ObjectKindDef::Find(FindDef {
                full_backpack: Some(TurnDef::Give(ruby())),
                ..find()
            })
        }),
        unknown_resource
    );
    let unknown_skill = AdventureCompileError::UnknownSkill("t:magic".into());
    assert_eq!(
        error(6, &|k| {
            *k = ObjectKindDef::WitchHut {
                roe: keys(&["t:magic"]),
                roe_fixed: vec![],
            }
        }),
        unknown_skill
    );
    assert_eq!(
        error(6, &|k| {
            *k = ObjectKindDef::WitchHut {
                roe: vec![],
                roe_fixed: keys(&["t:magic"]),
            }
        }),
        unknown_skill
    );
    let unknown_creature = AdventureCompileError::UnknownCreature("t:ghost".into());
    assert_eq!(
        error(12, &|k| {
            *k = ObjectKindDef::Pyramid(PyramidDef {
                split: (key("t:ghost"), 1),
                ..pyramid()
            })
        }),
        unknown_creature
    );
    assert_eq!(
        error(12, &|k| {
            *k = ObjectKindDef::Pyramid(PyramidDef {
                fixed: (key("t:ghost"), vec![1]),
                ..pyramid()
            })
        }),
        unknown_creature
    );
    // A reference outside the pack's closure: `u` does not reach `v`.
    assert_eq!(
        error(6, &|k| {
            *k = ObjectKindDef::WitchHut {
                roe: keys(&["v:luck"]),
                roe_fixed: vec![],
            }
        }),
        AdventureCompileError::ReferenceOutsideDependencies {
            key: "v:luck".into(),
            pack: "u".into(),
        }
    );
    assert_eq!(
        error(12, &|k| {
            *k = ObjectKindDef::Pyramid(PyramidDef {
                split: (key("v:imp"), 1),
                ..pyramid()
            })
        }),
        AdventureCompileError::ReferenceOutsideDependencies {
            key: "v:imp".into(),
            pack: "u".into(),
        }
    );
    assert_eq!(
        AdventureCompileError::UnknownSkill("t:magic".into()).to_string(),
        "unknown skill `t:magic`"
    );
}

// covers: ADV-OBJ-002
#[test]
fn the_hash_sees_every_reward_parameter() {
    let base = hash_of(&compile_rewards(&reward_packs()).unwrap());
    let changes: Vec<(usize, ObjectKindDef)> = vec![
        (
            0,
            ObjectKindDef::Chest(ChestDef {
                gold: vec![(32, 1000), (32, 1500), (31, 2500)],
                ..chest()
            }),
        ),
        (
            0,
            ObjectKindDef::Chest(ChestDef {
                gold: vec![(32, 1000), (33, 1500), (31, 2000)],
                ..chest()
            }),
        ),
        (
            0,
            ObjectKindDef::Chest(ChestDef {
                resource: key("t:wood"),
                ..chest()
            }),
        ),
        (
            0,
            ObjectKindDef::Chest(ChestDef {
                artifact: vec![ArtifactClass::Minor],
                ..chest()
            }),
        ),
        (
            0,
            ObjectKindDef::Chest(ChestDef {
                full_backpack: 1001,
                ..chest()
            }),
        ),
        (
            0,
            ObjectKindDef::Chest(ChestDef {
                experience_offset: 400,
                ..chest()
            }),
        ),
        (
            1,
            ObjectKindDef::Artifact {
                classes: Some(vec![ArtifactClass::Treasure]),
            },
        ),
        (
            2,
            ObjectKindDef::Artifact {
                classes: Some(vec![ArtifactClass::Major]),
            },
        ),
        (3, ObjectKindDef::Scholar),
        (
            4,
            ObjectKindDef::Gift(GiftDef {
                gives: Gives::Choice {
                    skills: [PrimarySkill::Knowledge, PrimarySkill::Power],
                    amount: 1,
                },
                ..gift()
            }),
        ),
        (
            4,
            ObjectKindDef::Gift(GiftDef {
                gives: Gives::Choice {
                    skills: [PrimarySkill::Power, PrimarySkill::Knowledge],
                    amount: 2,
                },
                ..gift()
            }),
        ),
        (
            4,
            ObjectKindDef::Gift(GiftDef {
                gives: Gives::Primary([1, 0, 0, 0]),
                ..gift()
            }),
        ),
        (
            4,
            ObjectKindDef::Gift(GiftDef {
                gives: Gives::Experience(1000),
                ..gift()
            }),
        ),
        (
            4,
            ObjectKindDef::Gift(GiftDef {
                gives: Gives::Level,
                ..gift()
            }),
        ),
        (
            4,
            ObjectKindDef::Gift(GiftDef {
                prices: vec![],
                ..gift()
            }),
        ),
        (
            4,
            ObjectKindDef::Gift(GiftDef {
                prices: vec![vec![(key("t:gold"), 1)]],
                ..gift()
            }),
        ),
        (
            4,
            ObjectKindDef::Gift(GiftDef {
                admits: None,
                ..gift()
            }),
        ),
        (
            4,
            ObjectKindDef::Gift(GiftDef {
                admits: Some(8),
                ..gift()
            }),
        ),
        (
            4,
            ObjectKindDef::Gift(GiftDef {
                leave: false,
                ..gift()
            }),
        ),
        (5, ObjectKindDef::Shrine { level: 2 }),
        (
            6,
            ObjectKindDef::WitchHut {
                roe: keys(&["t:luck"]),
                roe_fixed: keys(&["t:luck", "t:archery"]),
            },
        ),
        (
            6,
            ObjectKindDef::WitchHut {
                roe: keys(&["t:wisdom"]),
                roe_fixed: keys(&["t:archery", "t:luck"]),
            },
        ),
        (7, ObjectKindDef::PandoraBox),
        (
            8,
            ObjectKindDef::University {
                offers: 3,
                price: vec![(key("t:gold"), 2000)],
            },
        ),
        (
            8,
            ObjectKindDef::University {
                offers: 4,
                price: vec![(key("t:gold"), 2001)],
            },
        ),
        (
            9,
            ObjectKindDef::Find(FindDef {
                question: false,
                ..find()
            }),
        ),
        (
            9,
            ObjectKindDef::Find(FindDef {
                bonus: None,
                ..find()
            }),
        ),
        (
            9,
            ObjectKindDef::Find(FindDef {
                bonus: Some(OnceBonus {
                    morale: -2,
                    luck: 0,
                    mark: 0,
                }),
                ..find()
            }),
        ),
        (
            9,
            ObjectKindDef::Find(FindDef {
                full_backpack: Some(TurnDef::Stays),
                ..find()
            }),
        ),
        (
            9,
            ObjectKindDef::Find(FindDef {
                full_backpack: Some(TurnDef::Give(vec![(key("t:gold"), 1)])),
                ..find()
            }),
        ),
        (
            9,
            ObjectKindDef::Find(FindDef {
                draws: vec![
                    (10, FindContentDef::Nothing),
                    (40, FindContentDef::Artifact(vec![ArtifactClass::Minor])),
                    (
                        50,
                        FindContentDef::Resources {
                            amount: [2, 5],
                            resources: keys(&["t:wood", "u:gem"]),
                        },
                    ),
                ],
                ..find()
            }),
        ),
        (
            9,
            ObjectKindDef::Find(FindDef {
                draws: vec![
                    (10, FindContentDef::Nothing),
                    (
                        40,
                        FindContentDef::Artifact(vec![
                            ArtifactClass::Treasure,
                            ArtifactClass::Minor,
                        ]),
                    ),
                    (
                        50,
                        FindContentDef::Resources {
                            amount: [2, 6],
                            resources: keys(&["t:wood", "u:gem"]),
                        },
                    ),
                ],
                ..find()
            }),
        ),
        (
            9,
            ObjectKindDef::Find(FindDef {
                draws: vec![
                    (10, FindContentDef::Nothing),
                    (
                        40,
                        FindContentDef::Artifact(vec![
                            ArtifactClass::Treasure,
                            ArtifactClass::Minor,
                        ]),
                    ),
                    (
                        50,
                        FindContentDef::Resources {
                            amount: [2, 5],
                            resources: keys(&["u:gem", "t:wood"]),
                        },
                    ),
                ],
                ..find()
            }),
        ),
        (
            9,
            ObjectKindDef::Find(FindDef {
                draws: vec![
                    (11, FindContentDef::Nothing),
                    (
                        39,
                        FindContentDef::Artifact(vec![
                            ArtifactClass::Treasure,
                            ArtifactClass::Minor,
                        ]),
                    ),
                    (
                        50,
                        FindContentDef::Resources {
                            amount: [2, 5],
                            resources: keys(&["t:wood", "u:gem"]),
                        },
                    ),
                ],
                ..find()
            }),
        ),
        (10, ObjectKindDef::Event),
        (11, ObjectKindDef::Scroll),
        (
            12,
            ObjectKindDef::Pyramid(PyramidDef {
                split: (key("t:gog"), 40),
                ..pyramid()
            }),
        ),
        (
            12,
            ObjectKindDef::Pyramid(PyramidDef {
                split: (key("t:imp"), 41),
                ..pyramid()
            }),
        ),
        (
            12,
            ObjectKindDef::Pyramid(PyramidDef {
                fixed: (key("t:gog"), vec![10, 10]),
                ..pyramid()
            }),
        ),
        (
            12,
            ObjectKindDef::Pyramid(PyramidDef {
                fixed: (key("u:wisp"), vec![10, 11]),
                ..pyramid()
            }),
        ),
        (
            12,
            ObjectKindDef::Pyramid(PyramidDef {
                level: 4,
                ..pyramid()
            }),
        ),
        (
            12,
            ObjectKindDef::Pyramid(PyramidDef {
                looted: OnceBonus {
                    morale: 1,
                    luck: -2,
                    mark: 1,
                },
                ..pyramid()
            }),
        ),
        (
            12,
            ObjectKindDef::Pyramid(PyramidDef {
                looted: OnceBonus {
                    morale: 0,
                    luck: -1,
                    mark: 1,
                },
                ..pyramid()
            }),
        ),
        (
            12,
            ObjectKindDef::Pyramid(PyramidDef {
                looted: OnceBonus {
                    morale: 0,
                    luck: -2,
                    mark: 2,
                },
                ..pyramid()
            }),
        ),
    ];
    for (n, kind) in changes {
        let packs = with_kind(n, |k| *k = kind.clone());
        let hash = hash_of(&compile_rewards(&packs).unwrap());
        assert_ne!(hash, base, "{n}: {kind:?}");
    }
}
