//! Tests of what the map puts in increment 7's objects (E08 T61a; increment 7's design note §7):
//! each kind's shape checked when the game is built, the guards they take, a monster's treasure
//! artifact, and the static map's hash.

use h3_content::{
    ArtifactClass, ArtifactDef, ArtifactSlotDef, ArtifactTypeId, ContentPack, CreatureDef,
    CreatureInfoDef, HeroEffect, SkillId, SpellCategory, SpellDef, SpellId, SpellSchoolDef,
};
use h3_heroes::{ArtifactInstance, HeroReward, PrimarySkill, ScholarGift};
use h3_kernel::hash::hash_of;

use std::boxed::Box;
use std::string::ToString;

use super::*;
use crate::ruleset::{AdventurePack, ChestDef, ObjectKindDef, ObjectTypeDef, Visit};
use crate::setup::{EventSetup, Holds, MonsterSetup, RewardRecord};
use crate::testing::{
    GOLD_PILE, LOGISTICS, PEASANT, PIKEMAN, SCOUTING, hero_at, hero_skills, key,
    movement_rules_with, world,
};

type Rules = (Ruleset, AdventureRuleset);

const RING: ArtifactTypeId = ArtifactTypeId::from_raw(0);
const SCROLL: ArtifactTypeId = ArtifactTypeId::from_raw(1);
const GUST: SpellId = SpellId::from_raw(0);
const SPIT: SpellId = SpellId::from_raw(1);
const CART: CreatureTypeId = CreatureTypeId::from_raw(4);

/// [`crate::testing::movement_rules`] (faithful) with a war machine `t:cart`, a ring and the
/// scroll, a combat spell `t:gust` and a creature ability `t:spit`, [`hero_skills`], and one
/// object type of each kind of increment 7 that holds something, a chest and a monster.
fn rules() -> Rules {
    movement_rules_with(
        true,
        16,
        |core: &mut ContentPack| {
            core.creatures.push(CreatureDef {
                key: key("t:cart"),
                speed: 0,
                info: CreatureInfoDef {
                    flags: vec!["war_machine".to_string()],
                    ..CreatureInfoDef::default()
                },
            });
            core.artifact_slots = vec![ArtifactSlotDef {
                key: key("t:finger"),
                name: "Finger".into(),
                count: 1,
                backpack: true,
            }];
            let artifact = |k: &str, slot: Option<&str>, effects| ArtifactDef {
                key: key(k),
                name: "Artifact".into(),
                class: ArtifactClass::Treasure,
                cost: vec![],
                slot: slot.map(key),
                war_machine: None,
                needs_skill: None,
                effects,
                parts: vec![],
            };
            core.artifacts = vec![
                artifact("t:ring", Some("t:finger"), vec![]),
                artifact("t:scroll", None, vec![HeroEffect::Scroll]),
            ];
            core.spell_schools = vec![SpellSchoolDef {
                key: key("t:air"),
                name: "Air".into(),
                protection_rank: 0,
            }];
            let spell = |k: &str, category, schools: Vec<_>| SpellDef {
                key: key(k),
                name: "Spell".into(),
                category,
                level: 1,
                schools,
                cost: [5, 4, 4, 4],
                power: 10,
                effect: [10, 10, 20, 30],
                granted_by_name_only: false,
            };
            core.spells = vec![
                spell("t:gust", SpellCategory::Combat, vec![key("t:air")]),
                spell("t:spit", SpellCategory::Ability, vec![]),
            ];
            core.skills = hero_skills();
        },
        |part: &mut AdventurePack| {
            let ty = |k: &str, kind| ObjectTypeDef {
                key: key(k),
                name: "Object".into(),
                visit: Visit::Neighbour,
                from_above: true,
                kind,
            };
            part.object_types.extend([
                ty("t:artifact", ObjectKindDef::Artifact { classes: None }),
                ty(
                    "t:random_artifact",
                    ObjectKindDef::Artifact {
                        classes: Some(vec![ArtifactClass::Treasure]),
                    },
                ),
                ty("t:spell_scroll", ObjectKindDef::Scroll),
                ty("t:shrine", ObjectKindDef::Shrine { level: 1 }),
                ty(
                    "t:hut",
                    ObjectKindDef::WitchHut {
                        roe: vec![key("t:logistics")],
                        roe_fixed: vec![key("t:scouting")],
                    },
                ),
                ty("t:scholar", ObjectKindDef::Scholar),
                ty("t:box", ObjectKindDef::PandoraBox),
                ty("t:event", ObjectKindDef::Event),
                // E08 T83a: increment 9's ownable kinds, entered.
                ObjectTypeDef {
                    visit: Visit::Enter,
                    ..ty(
                        "t:sawmill",
                        ObjectKindDef::Mine {
                            resource: key("t:wood"),
                            daily: 2,
                            battle_terrain: None,
                        },
                    )
                },
                ObjectTypeDef {
                    visit: Visit::Enter,
                    ..ty(
                        "t:gold_mine",
                        ObjectKindDef::Mine {
                            resource: key("t:gold"),
                            daily: 1000,
                            battle_terrain: None,
                        },
                    )
                },
                ObjectTypeDef {
                    visit: Visit::Enter,
                    ..ty(
                        "t:abandoned",
                        ObjectKindDef::AbandonedMine {
                            guards: key("t:peasant"),
                            count: [100, 200],
                            battle_terrain: None,
                        },
                    )
                },
                ObjectTypeDef {
                    visit: Visit::Enter,
                    ..ty(
                        "t:lighthouse",
                        ObjectKindDef::Lighthouse { sea_points: 500 },
                    )
                },
                ty(
                    "t:chest",
                    ObjectKindDef::Chest(ChestDef {
                        resource: key("t:gold"),
                        gold: vec![(100, 1000)],
                        artifact: vec![],
                        full_backpack: 1000,
                        experience_offset: 500,
                    }),
                ),
            ]);
        },
    )
}

fn type_of(rules: &Rules, k: &str) -> ObjectTypeId {
    rules.1.object_type_by_key(k).unwrap()
}

/// An object of type `k` at `(x, 0)`, blocking and visited there, holding `holds`.
fn object(rules: &Rules, x: u16, k: &str, holds: Holds) -> ObjectSetup {
    ObjectSetup {
        scenery: Default::default(),
        anchor: Pos::new(x, 0, 0),
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: Some(type_of(rules, k)),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds,
        owner: None,
    }
}

/// A record giving a skill, an artifact, a spell, resources and creatures.
fn record() -> RewardRecord {
    RewardRecord {
        hero: HeroReward {
            experience: 1000,
            primary: [1, -1, 0, 2],
            skills: vec![(SCOUTING, 3)],
            mana: -5,
            morale: 1,
            luck: -3,
            artifacts: vec![RING, SCROLL],
            spells: vec![GUST],
        },
        resources: vec![
            (ResourceId::from_raw(1), -100),
            (ResourceId::from_raw(0), 5),
        ],
        creatures: vec![stack(2, PIKEMAN, 4), stack(0, PEASANT, 9)],
    }
}

fn stack(slot: u8, creature: CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

/// Every holding of increment 7's kinds, valid, one object each in a row.
fn valid(rules: &Rules) -> Vec<ObjectSetup> {
    vec![
        object(rules, 0, "t:artifact", Holds::Artifact(RING.into())),
        object(rules, 1, "t:random_artifact", Holds::Nothing),
        object(
            rules,
            2,
            "t:spell_scroll",
            Holds::Artifact(ArtifactInstance::scroll(SCROLL, GUST)),
        ),
        object(rules, 3, "t:shrine", Holds::Spell(Some(GUST))),
        object(rules, 4, "t:shrine", Holds::Spell(None)),
        object(
            rules,
            5,
            "t:hut",
            Holds::Skills(Some(vec![SCOUTING, LOGISTICS])),
        ),
        object(rules, 6, "t:hut", Holds::Skills(None)),
        object(
            rules,
            7,
            "t:scholar",
            Holds::Gift(Some(ScholarGift::Primary(PrimarySkill::Power))),
        ),
        object(
            rules,
            8,
            "t:scholar",
            Holds::Gift(Some(ScholarGift::Skill(LOGISTICS))),
        ),
        object(
            rules,
            9,
            "t:scholar",
            Holds::Gift(Some(ScholarGift::Spell(GUST))),
        ),
        object(rules, 10, "t:scholar", Holds::Gift(None)),
        object(rules, 11, "t:box", Holds::Record(Box::new(record()))),
        object(
            rules,
            12,
            "t:event",
            Holds::Event(Box::new(EventSetup {
                record: record(),
                colours: 0b101,
                computer: true,
                remove: true,
            })),
        ),
        object(rules, 13, "t:chest", Holds::Nothing),
        object(rules, 14, "t:box", Holds::Record(Box::default())),
    ]
}

/// The setup of a grass row of 16 tiles with these objects, red's hero 1 below at (1, 1).
fn setup(objects: Vec<ObjectSetup>) -> AdventureSetup {
    let mut setup = world(
        &["gggggggggggggggg", "gggggggggggggggg"],
        1,
        vec![hero_at(1, 0, Pos::new(1, 1, 0), &[PEASANT])],
    );
    setup.map.objects.extend(objects);
    setup
}

fn built(rules: &Rules, objects: Vec<ObjectSetup>) -> Result<AdventureState, SetupError> {
    crate::testing::started(rules, &setup(objects))
}

// covers: ADV-OBJ-002
#[test]
fn objects_hold_what_their_kind_takes() {
    let rules = rules();
    let objects = valid(&rules);
    let state = built(&rules, objects.clone()).unwrap();
    let held: Vec<&Holds> = state.map().objects().iter().map(|o| &o.holds).collect();
    let given: Vec<&Holds> = objects.iter().map(|o| &o.holds).collect();
    assert_eq!(held, given);
    // Each holding against another kind, or an invalid one: refused at its index.
    let refused = |x: u16, k: &str, holds: Holds| {
        let object = object(&rules, x, k, holds);
        assert_eq!(
            built(&rules, vec![object.clone()]).err(),
            Some(SetupError::Holds(0)),
            "{k}: {:?}",
            object.holds
        );
    };
    let unknown = ArtifactTypeId::from_raw(9);
    refused(0, "t:artifact", Holds::Nothing);
    refused(0, "t:artifact", Holds::Artifact(unknown.into()));
    refused(0, "t:artifact", Holds::Artifact(SCROLL.into()));
    refused(
        0,
        "t:artifact",
        Holds::Artifact(ArtifactInstance::scroll(SCROLL, GUST)),
    );
    refused(0, "t:artifact", Holds::Spell(None));
    refused(0, "t:random_artifact", Holds::Artifact(RING.into()));
    refused(0, "t:spell_scroll", Holds::Artifact(RING.into()));
    refused(
        0,
        "t:spell_scroll",
        Holds::Artifact(ArtifactInstance::scroll(SCROLL, SPIT)),
    );
    refused(0, "t:spell_scroll", Holds::Nothing);
    refused(0, "t:shrine", Holds::Spell(Some(SPIT)));
    refused(0, "t:shrine", Holds::Spell(Some(SpellId::from_raw(9))));
    refused(0, "t:shrine", Holds::Skills(None));
    refused(0, "t:hut", Holds::Skills(Some(vec![SCOUTING, SCOUTING])));
    refused(0, "t:hut", Holds::Skills(Some(vec![SkillId::from_raw(40)])));
    refused(0, "t:hut", Holds::Nothing);
    refused(
        0,
        "t:scholar",
        Holds::Gift(Some(ScholarGift::Skill(SkillId::from_raw(40)))),
    );
    refused(0, "t:scholar", Holds::Gift(Some(ScholarGift::Spell(SPIT))));
    refused(0, "t:scholar", Holds::Nothing);
    refused(0, "t:box", Holds::Nothing);
    refused(0, "t:event", Holds::Record(Box::new(record())));
    refused(0, "t:chest", Holds::Spell(None));
    // A blocker and the older kinds hold nothing.
    let mut blocker = object(&rules, 0, "t:chest", Holds::Spell(None));
    blocker.object_type = None;
    assert_eq!(
        built(&rules, vec![blocker]).err(),
        Some(SetupError::Holds(0))
    );
    let mut pile = object(&rules, 0, "t:chest", Holds::Artifact(RING.into()));
    pile.object_type = Some(GOLD_PILE);
    assert_eq!(built(&rules, vec![pile]).err(), Some(SetupError::Holds(0)));
    assert_eq!(
        SetupError::Holds(3).to_string(),
        "object 3 holds what its type does not take"
    );
}

// covers: ADV-OBJ-002
#[test]
fn a_reward_record_names_valid_content() {
    let rules = rules();
    let refused = |edit: fn(&mut RewardRecord)| {
        let mut bad = record();
        edit(&mut bad);
        let event = Holds::Event(Box::new(EventSetup {
            record: bad.clone(),
            ..EventSetup::default()
        }));
        for (k, holds) in [
            ("t:box", Holds::Record(Box::new(bad.clone()))),
            ("t:event", event),
        ] {
            assert_eq!(
                built(&rules, vec![object(&rules, 0, k, holds)]).err(),
                Some(SetupError::Holds(0)),
                "{k}: {bad:?}"
            );
        }
    };
    refused(|r| r.hero.skills[0].1 = 0);
    refused(|r| r.hero.skills[0].1 = 4);
    refused(|r| r.hero.skills[0].0 = SkillId::from_raw(40));
    refused(|r| r.hero.artifacts.push(ArtifactTypeId::from_raw(9)));
    refused(|r| r.hero.spells.push(SPIT));
    refused(|r| r.resources.push((ResourceId::from_raw(1), 3)));
    refused(|r| r.resources[0].1 = 0);
    refused(|r| r.resources.push((ResourceId::from_raw(7), 3)));
    refused(|r| r.creatures.push(stack(7, PIKEMAN, 1)));
    refused(|r| r.creatures.push(stack(0, PIKEMAN, 1)));
    refused(|r| r.creatures.push(stack(1, CART, 1)));
    refused(|r| r.creatures[0].count = 0);
    // A Spell Scroll without a spell is a valid gift: the hero cannot receive it and loses it
    // (HER-CHG-003).
    let state = built(&rules, valid(&rules)).unwrap();
    assert!(state.map().objects().len() > 11);
}

// covers: ADV-PIK-004
#[test]
fn artifacts_scrolls_boxes_and_events_take_guards() {
    let rules = rules();
    let guards = Some(Guards {
        army: vec![stack(1, PIKEMAN, 2)],
        question: true,
    });
    for (x, mut object) in (0..).zip(valid(&rules)) {
        object.guards.clone_from(&guards);
        let kind = &rules
            .1
            .object_type(object.object_type.unwrap())
            .unwrap()
            .kind;
        let takes = matches!(
            kind,
            ObjectKind::Artifact { .. }
                | ObjectKind::Scroll
                | ObjectKind::PandoraBox
                | ObjectKind::Event
        );
        let result = built(&rules, vec![object.clone()]);
        if takes {
            let state = result.unwrap();
            assert_eq!(state.map().objects()[0].guards, guards, "{x}");
        } else {
            assert_eq!(result.err(), Some(SetupError::Guards(0)), "{x}");
        }
    }
}

// covers: ADV-MON-001
#[test]
fn a_monster_s_treasure_holds_an_artifact_a_hero_may_hold() {
    let rules = rules();
    let monster = |artifact| {
        let mut object = object(&rules, 2, "t:chest", Holds::Nothing);
        object.object_type = Some(crate::testing::MONSTER);
        object.monster = Some(MonsterSetup {
            creature: Some(PIKEMAN),
            count: 5,
            disposition: 0,
            never_flees: false,
            no_growth: false,
            treasure: vec![],
            artifact,
        });
        object
    };
    let state = built(&rules, vec![monster(Some(RING))]).unwrap();
    assert_eq!(
        state.map().objects()[0].monster.as_ref().unwrap().artifact,
        Some(RING)
    );
    for artifact in [SCROLL, ArtifactTypeId::from_raw(9)] {
        assert_eq!(
            built(&rules, vec![monster(Some(artifact))]).err(),
            Some(SetupError::Monster(0))
        );
    }
    // The hash sees it.
    let none = built(&rules, vec![monster(None)]).unwrap();
    assert_ne!(hash_of(&state), hash_of(&none));
}

// covers: ADV-MIN-003, ADV-OWN-002
#[test]
fn an_abandoned_mine_holds_its_resources_and_an_ownable_its_owner() {
    // E08 T83a: checked when the game is built; the owners kept by the state from T83b.
    let rules = rules();
    let (wood, gold) = (ResourceId::from_raw(0), ResourceId::from_raw(1));
    let abandoned = |holds| object(&rules, 3, "t:abandoned", holds);
    let state = built(&rules, vec![abandoned(Holds::Resources(vec![wood, gold]))]).unwrap();
    assert_eq!(
        state.map().objects()[0].holds,
        Holds::Resources(vec![wood, gold])
    );
    for holds in [
        Holds::Resources(vec![]),
        Holds::Resources(vec![gold, wood]),
        Holds::Resources(vec![wood, wood]),
        Holds::Resources(vec![ResourceId::from_raw(2)]),
        Holds::Nothing,
    ] {
        assert_eq!(
            built(&rules, vec![abandoned(holds.clone())]).err(),
            Some(SetupError::Holds(0)),
            "{holds:?}"
        );
    }
    assert_eq!(
        built(
            &rules,
            vec![object(
                &rules,
                3,
                "t:lighthouse",
                Holds::Resources(vec![wood])
            )]
        )
        .err(),
        Some(SetupError::Holds(0))
    );
    // The hash sees the set.
    let hash = |set| hash_of(&built(&rules, vec![abandoned(Holds::Resources(set))]).unwrap());
    assert_ne!(hash(vec![wood]), hash(vec![gold]));
    // An owner only on an ownable object, a player of the game (two here).
    let owned = |k: &str, owner: u8, holds| ObjectSetup {
        owner: Some(PlayerId::new(owner)),
        ..object(&rules, 5, k, holds)
    };
    for k in ["t:sawmill", "t:lighthouse"] {
        assert!(built(&rules, vec![owned(k, 1, Holds::Nothing)]).is_ok());
        assert_eq!(
            built(&rules, vec![owned(k, 2, Holds::Nothing)]).err(),
            Some(SetupError::ObjectOwner(0))
        );
    }
    assert!(
        built(
            &rules,
            vec![owned("t:abandoned", 0, Holds::Resources(vec![gold]))]
        )
        .is_ok()
    );
    assert_eq!(
        built(&rules, vec![owned("t:chest", 0, Holds::Nothing)]).err(),
        Some(SetupError::ObjectOwner(0))
    );
    let blocker = ObjectSetup {
        object_type: None,
        ..owned("t:chest", 0, Holds::Nothing)
    };
    assert_eq!(
        built(&rules, vec![blocker]).err(),
        Some(SetupError::ObjectOwner(0))
    );
}

// covers: ADV-OBJ-002
#[test]
fn the_hash_sees_what_objects_hold() {
    let rules = rules();
    let base = valid(&rules);
    let hash = |objects: Vec<ObjectSetup>| hash_of(&built(&rules, objects).unwrap());
    let reference = hash(base.clone());
    let changes: Vec<(usize, Holds)> = vec![
        (0, Holds::Artifact(SCROLL.into())),
        (2, Holds::Artifact(ArtifactInstance::scroll(SCROLL, GUST))),
        (3, Holds::Spell(None)),
        (5, Holds::Skills(Some(vec![LOGISTICS, SCOUTING]))),
        (5, Holds::Skills(None)),
        (
            7,
            Holds::Gift(Some(ScholarGift::Primary(PrimarySkill::Attack))),
        ),
        (8, Holds::Gift(Some(ScholarGift::Skill(SCOUTING)))),
        (9, Holds::Gift(None)),
        (11, Holds::Record(Box::default())),
    ];
    let mut checked = 0;
    for (index, holds) in changes {
        let mut objects = base.clone();
        if objects[index].holds == holds {
            continue;
        }
        objects[index].holds = holds;
        if built(&rules, objects.clone()).is_ok() {
            assert_ne!(hash(objects), reference, "{index}");
            checked += 1;
        }
    }
    assert!(checked >= 7, "{checked}");
    // Every part of a record and an event.
    let edits: Vec<fn(&mut RewardRecord)> = vec![
        |r| r.hero.experience += 1,
        |r| r.hero.primary[3] = 3,
        |r| r.hero.skills[0].1 = 2,
        |r| r.hero.mana = 4,
        |r| r.hero.morale = 2,
        |r| r.hero.luck = 0,
        |r| r.hero.artifacts.pop().map_or((), drop),
        |r| r.hero.spells.clear(),
        |r| r.resources[0].1 = -99,
        |r| r.creatures[0].count = 5,
    ];
    for edit in edits {
        let mut objects = base.clone();
        let mut changed = record();
        edit(&mut changed);
        objects[11].holds = Holds::Record(Box::new(changed));
        assert_ne!(hash(objects), reference);
    }
    let event = |colours, computer, remove| {
        let mut objects = base.clone();
        objects[12].holds = Holds::Event(Box::new(EventSetup {
            record: record(),
            colours,
            computer,
            remove,
        }));
        hash(objects)
    };
    assert_eq!(event(0b101, true, true), reference);
    for (colours, computer, remove) in [(1, true, true), (0b101, false, true), (0b101, true, false)]
    {
        assert_ne!(event(colours, computer, remove), reference);
    }
}
