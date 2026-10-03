extern crate std;

use std::vec;
use std::vec::Vec;

use super::*;
use crate::effect::{Affects, CreatureSelector, CreatureStat, HeroEffect, HeroEffectDef};
use crate::pack::{
    ArtifactDef, ArtifactSlotDef, ArtifactSlotOverride, ProfileDef, SecondarySkillDef,
};
use crate::ruleset::Ruleset;
use crate::ruleset::tests::{battle_def, creature, faction, key, manifest, resource};

fn slot(k: &str, count: u8) -> ArtifactSlotDef {
    ArtifactSlotDef {
        key: key(k),
        name: "Slot".into(),
        count,
        backpack: true,
    }
}

fn artifact(k: &str, slot: Option<&str>, effects: Vec<HeroEffectDef>) -> ArtifactDef {
    ArtifactDef {
        key: key(k),
        name: "Artifact".into(),
        class: ArtifactClass::Treasure,
        cost: vec![(key("t:gold"), 1000)],
        slot: slot.map(key),
        war_machine: None,
        needs_skill: None,
        effects,
        parts: Vec::new(),
    }
}

fn luck(amount: i8) -> HeroEffectDef {
    HeroEffect::Luck {
        amount,
        affects: Affects::Own,
    }
}

/// A pack with slots, ordinary artifacts, a war machine, a backpack-only artifact, an artifact
/// that needs a skill and a combination (`t:circle` = band + loop + helm, worn as a ring).
fn pack() -> ContentPack {
    let mut cart = creature("t:cart");
    cart.info.flags = vec!["war_machine".into()];
    let mut kit = artifact("t:kit", Some("t:machine"), vec![]);
    kit.war_machine = Some(key("t:cart"));
    kit.class = ArtifactClass::Special;
    let mut bow = artifact(
        "t:bow",
        Some("t:misc"),
        vec![HeroEffect::RangedDamage { bp: 500 }],
    );
    bow.needs_skill = Some(key("t:archery"));
    let mut grail = artifact("t:grail", None, vec![]);
    grail.cost = Vec::new();
    // A forward reference: the combination is defined before its parts.
    let mut circle = artifact(
        "t:circle",
        Some("t:ring"),
        vec![HeroEffect::AlignmentGroup {
            factions: vec![key("t:town"), key("t:wild")],
        }],
    );
    circle.parts = vec![key("t:band"), key("t:loop"), key("t:helm")];
    circle.class = ArtifactClass::Relic;
    ContentPack {
        resources: vec![resource("t:gold"), resource("t:gem")],
        factions: vec![faction("t:town"), faction("t:wild")],
        creatures: vec![creature("t:peasant"), cart],
        skills: vec![SecondarySkillDef {
            key: key("t:archery"),
            name: "Archery".into(),
            levels: [
                vec![HeroEffect::RangedDamage { bp: 1000 }],
                vec![HeroEffect::RangedDamage { bp: 2500 }],
                vec![HeroEffect::RangedDamage { bp: 5000 }],
            ],
        }],
        artifact_slots: vec![
            slot("t:head", 1),
            slot("t:ring", 2),
            slot("t:misc", 3),
            ArtifactSlotDef {
                backpack: false,
                ..slot("t:machine", 1)
            },
        ],
        artifacts: vec![
            circle,
            artifact(
                "t:helm",
                Some("t:head"),
                vec![HeroEffect::PrimarySkills {
                    attack: 2,
                    defense: -1,
                    power: 0,
                    knowledge: 1,
                }],
            ),
            artifact("t:band", Some("t:ring"), vec![luck(1)]),
            artifact(
                "t:loop",
                Some("t:ring"),
                vec![HeroEffect::Morale {
                    amount: 1,
                    affects: Affects::Own,
                }],
            ),
            bow,
            kit,
            grail,
        ],
        profiles: vec![ProfileDef {
            key: key("t:faithful"),
            quirks: vec![],
        }],
        battle: Some(battle_def()),
        ..ContentPack::new(manifest("t", &[]))
    }
}

fn compile(packs: &[ContentPack]) -> Result<Ruleset, CompileError> {
    Ruleset::compile(packs, "t:faithful")
}

fn invalid(kind: &'static str, k: &str, reason: &'static str) -> CompileError {
    CompileError::InvalidContent {
        kind,
        key: k.into(),
        reason,
    }
}

fn find<'a>(p: &'a mut ContentPack, k: &str) -> &'a mut ArtifactDef {
    p.artifacts
        .iter_mut()
        .find(|a| a.key.as_str() == k)
        .unwrap()
}

#[test]
fn class_names_round_trip() {
    for c in ArtifactClass::ALL {
        assert_eq!(ArtifactClass::from_name(c.name()), Some(c));
    }
    assert_eq!(ArtifactClass::from_name("legendary"), None);
}

#[test]
fn compiles_slots_artifacts_and_combinations() {
    let r = compile(&[pack()]).unwrap();
    assert_eq!(r.artifact_slots().len(), 4);
    let ring = r.artifact_slot_by_key("t:ring").unwrap();
    let head = r.artifact_slot_by_key("t:head").unwrap();
    assert_eq!(r.artifact_slot(ring).unwrap().count, 2);
    let machine = r.artifact_slot_by_key("t:machine").unwrap();
    assert!(r.artifact_slot(ring).unwrap().backpack);
    assert!(!r.artifact_slot(machine).unwrap().backpack);
    assert!(r.artifact_slot(Handle::from_raw(9)).is_none());

    assert_eq!(r.artifacts().len(), 7);
    let circle_id = r.artifact_by_key("t:circle").unwrap();
    let circle = r.artifact(circle_id).unwrap();
    let parts: Vec<ArtifactTypeId> = ["t:band", "t:loop", "t:helm"]
        .iter()
        .map(|k| r.artifact_by_key(k).unwrap())
        .collect();
    assert!(circle.is_combination());
    assert_eq!(circle.parts, parts);
    // Worn as a ring: one ring position is its own, the other ring and the head are locked.
    assert_eq!(
        (circle.slot, circle.locks.clone()),
        (Some(ring), vec![ring, head])
    );
    assert_eq!(circle.class, ArtifactClass::Relic);
    assert_eq!(
        circle.effects,
        vec![HeroEffect::AlignmentGroup {
            factions: vec![Handle::from_raw(0), Handle::from_raw(1)]
        }]
    );
    for p in &parts {
        assert_eq!(r.combination_of(*p), Some(circle_id));
    }
    assert_eq!(r.combination_of(circle_id), None);

    let helm = r.artifact(parts[2]).unwrap();
    assert!(!helm.is_combination() && helm.locks.is_empty());
    assert_eq!(helm.cost, vec![(Handle::from_raw(0), 1000)]);
    let kit = r.artifact(r.artifact_by_key("t:kit").unwrap()).unwrap();
    assert_eq!(kit.war_machine, r.creature_by_key("t:cart"));
    let bow = r.artifact(r.artifact_by_key("t:bow").unwrap()).unwrap();
    assert_eq!(bow.needs_skill, r.skill_by_key("t:archery"));
    let grail = r.artifact(r.artifact_by_key("t:grail").unwrap()).unwrap();
    assert_eq!((grail.slot, grail.cost.len()), (None, 0));
    assert!(r.artifact(Handle::from_raw(99)).is_none());
}

#[test]
fn later_packs_override_slots_and_the_hash_sees_all_artifact_content() {
    let base = compile(&[pack()]).unwrap().content_hash();
    assert_eq!(compile(&[pack()]).unwrap().content_hash(), base);
    let changes: [fn(&mut ContentPack); 11] = [
        |p| p.artifact_slots[0].name = "Crown".into(),
        |p| p.artifact_slots[3].backpack = true,
        |p| p.artifact_slots[2].count = 4,
        |p| find(p, "t:helm").name = "Cap".into(),
        |p| find(p, "t:helm").class = ArtifactClass::Major,
        |p| find(p, "t:helm").cost = vec![(key("t:gem"), 1)],
        |p| find(p, "t:helm").slot = Some(key("t:misc")),
        |p| find(p, "t:bow").needs_skill = None,
        |p| find(p, "t:kit").war_machine = None,
        |p| find(p, "t:band").effects = vec![luck(2)],
        |p| find(p, "t:circle").parts.truncate(2),
    ];
    for change in changes {
        let mut p = pack();
        change(&mut p);
        assert_ne!(compile(&[p]).unwrap().content_hash(), base);
    }

    // An expansion changes a slot's name and count, then only its count.
    let mut ext = ContentPack {
        artifact_slot_overrides: vec![
            ArtifactSlotOverride {
                target: key("t:misc"),
                name: Some("Pocket".into()),
                count: None,
            },
            ArtifactSlotOverride {
                target: key("t:misc"),
                name: None,
                count: Some(5),
            },
        ],
        artifact_slots: vec![slot("u:belt", 1)],
        artifacts: vec![artifact("u:buckle", Some("u:belt"), vec![luck(1)])],
        ..ContentPack::new(manifest("u", &["t"]))
    };
    let r = compile(&[pack(), ext.clone()]).unwrap();
    let misc = r
        .artifact_slot(r.artifact_slot_by_key("t:misc").unwrap())
        .unwrap();
    assert_eq!((misc.name.as_str(), misc.count), ("Pocket", 5));
    assert_eq!(r.artifacts().len(), 8);
    // A later override can make a combination impossible.
    ext.artifact_slot_overrides = vec![ArtifactSlotOverride {
        target: key("t:ring"),
        name: None,
        count: Some(1),
    }];
    assert_eq!(
        compile(&[pack(), ext]).unwrap_err(),
        invalid(
            "artifact",
            "t:circle",
            "the parts need more positions of a slot than the slot has"
        )
    );
}

/// A change to the test pack and the error it causes.
type Case = (fn(&mut ContentPack), CompileError);

#[test]
fn references_are_resolved_in_the_closure() {
    let unknown = |kind: &'static str, k: &str| CompileError::UnknownReference {
        kind,
        key: k.into(),
    };
    let cases: [Case; 13] = [
        (
            |p| p.artifact_slots.push(slot("x:hat", 1)),
            CompileError::KeyOutsidePack {
                key: "x:hat".into(),
                pack: "t".into(),
            },
        ),
        (
            |p| p.artifacts.push(artifact("x:hat", None, vec![])),
            CompileError::KeyOutsidePack {
                key: "x:hat".into(),
                pack: "t".into(),
            },
        ),
        (
            |p| {
                p.artifact_slot_overrides.push(ArtifactSlotOverride {
                    target: key("t:belt"),
                    name: None,
                    count: Some(1),
                });
            },
            unknown("artifact slot", "t:belt"),
        ),
        (
            |p| find(p, "t:helm").slot = Some(key("t:belt")),
            unknown("artifact slot", "t:belt"),
        ),
        (
            |p| find(p, "t:kit").war_machine = Some(key("t:ballista")),
            unknown("creature", "t:ballista"),
        ),
        (
            |p| find(p, "t:bow").needs_skill = Some(key("t:offense")),
            unknown("secondary skill", "t:offense"),
        ),
        (
            |p| find(p, "t:helm").cost = vec![(key("t:wood"), 1)],
            unknown("resource", "t:wood"),
        ),
        (
            |p| find(p, "t:helm").cost = vec![(key("t:gold"), 1), (key("t:gold"), 2)],
            CompileError::DuplicateKey("t:gold".into()),
        ),
        (
            |p| {
                find(p, "t:helm").effects = vec![HeroEffect::Regeneration {
                    creatures: CreatureSelector::Creature(key("t:troll")),
                    hit_points: 50,
                }];
            },
            unknown("creature", "t:troll"),
        ),
        (
            |p| find(p, "t:circle").parts.push(key("t:ring_of_x")),
            unknown("artifact", "t:ring_of_x"),
        ),
        (
            |p| find(p, "t:circle").parts.push(key("t:band")),
            CompileError::DuplicateKey("t:band".into()),
        ),
        (
            |p| {
                find(p, "t:circle").effects = vec![HeroEffect::AlignmentGroup {
                    factions: vec![key("t:town"), key("t:lost")],
                }];
            },
            unknown("faction", "t:lost"),
        ),
        (
            |p| {
                find(p, "t:circle").effects = vec![HeroEffect::AlignmentGroup {
                    factions: vec![key("t:town"), key("t:town")],
                }];
            },
            CompileError::DuplicateKey("t:town".into()),
        ),
    ];
    for (change, error) in cases {
        let mut p = pack();
        change(&mut p);
        assert_eq!(compile(&[p]).unwrap_err(), error);
    }
    // A pack may not refer to slots outside its dependency closure.
    let other = ContentPack {
        artifact_slots: vec![slot("v:cape", 1)],
        ..ContentPack::new(manifest("v", &[]))
    };
    let ext = ContentPack {
        artifacts: vec![artifact("u:cloak", Some("v:cape"), vec![])],
        ..ContentPack::new(manifest("u", &["t"]))
    };
    assert_eq!(
        compile(&[pack(), other, ext]).unwrap_err(),
        CompileError::ReferenceOutsideDependencies {
            key: "v:cape".into(),
            pack: "u".into(),
        }
    );
}

#[test]
fn invalid_artifacts_are_rejected() {
    let cases: [Case; 13] = [
        (
            |p| p.artifact_slots[0].count = 0,
            invalid("artifact slot", "t:head", "count must be 1..=8"),
        ),
        (
            |p| p.artifact_slots[0].count = MAX_SLOT_COUNT + 1,
            invalid("artifact slot", "t:head", "count must be 1..=8"),
        ),
        (
            |p| {
                for i in 0..5 {
                    p.artifact_slots.push(slot(&std::format!("t:extra{i}"), 8));
                }
            },
            CompileError::TooManyEntries("artifact slot position"),
        ),
        (
            |p| find(p, "t:helm").effects.push(luck(0)),
            invalid(
                "artifact",
                "t:helm",
                "morale and luck amounts must not be 0",
            ),
        ),
        (
            |p| find(p, "t:kit").war_machine = Some(key("t:peasant")),
            invalid(
                "artifact",
                "t:kit",
                "war_machine must have flag war_machine",
            ),
        ),
        (
            |p| find(p, "t:kit").slot = None,
            invalid(
                "artifact",
                "t:kit",
                "a war machine artifact needs a slot and cannot be a combination",
            ),
        ),
        (
            |p| find(p, "t:circle").war_machine = Some(key("t:cart")),
            invalid(
                "artifact",
                "t:circle",
                "a war machine artifact needs a slot and cannot be a combination",
            ),
        ),
        (
            |p| find(p, "t:circle").parts = vec![key("t:band")],
            invalid("artifact", "t:circle", "a combination needs 2..=16 parts"),
        ),
        (
            |p| {
                let mut ring = artifact("t:ring2", Some("t:ring"), vec![]);
                ring.parts = vec![key("t:circle"), key("t:bow")];
                p.artifacts.push(ring);
            },
            invalid(
                "artifact",
                "t:ring2",
                "combination parts cannot be combinations",
            ),
        ),
        (
            |p| find(p, "t:circle").parts.push(key("t:kit")),
            invalid(
                "artifact",
                "t:circle",
                "war machine artifacts cannot be combination parts",
            ),
        ),
        (
            |p| {
                let mut twin = artifact("t:twin", Some("t:misc"), vec![]);
                twin.parts = vec![key("t:bow"), key("t:band")];
                p.artifacts.push(twin);
            },
            invalid(
                "artifact",
                "t:band",
                "an artifact can be a part of one combination only",
            ),
        ),
        (
            |p| find(p, "t:circle").slot = None,
            invalid("artifact", "t:circle", "a combination needs a slot"),
        ),
        (
            |p| find(p, "t:helm").slot = None,
            invalid("artifact", "t:circle", "combination parts need a slot"),
        ),
    ];
    for (change, error) in cases {
        let mut p = pack();
        change(&mut p);
        assert_eq!(compile(&[p]).unwrap_err(), error);
    }
    let mut p = pack();
    find(&mut p, "t:circle").slot = Some(key("t:misc"));
    assert_eq!(
        compile(&[p]).unwrap_err(),
        invalid(
            "artifact",
            "t:circle",
            "a combination's slot must be the slot of one of its parts"
        )
    );
    // At most 16 parts.
    let mut p = pack();
    p.artifact_slots[2].count = 8;
    let mut many = artifact("t:many", Some("t:misc"), vec![]);
    for i in 0..=MAX_PARTS {
        p.artifacts.push(artifact(
            &std::format!("t:piece{i}"),
            Some("t:misc"),
            vec![],
        ));
        many.parts.push(key(&std::format!("t:piece{i}")));
    }
    p.artifacts.push(many);
    assert_eq!(
        compile(&[p]).unwrap_err(),
        invalid("artifact", "t:many", "a combination needs 2..=16 parts")
    );
}

#[test]
fn every_new_effect_kind_compiles_on_an_artifact() {
    let mut p = pack();
    let alive = || CreatureSelector::Flag(CreatureFlag::Alive);
    find(&mut p, "t:bow").effects = vec![
        HeroEffect::MoraleAtMost {
            value: 0,
            affects: Affects::Both,
        },
        HeroEffect::LuckAtMost {
            value: 0,
            affects: Affects::Both,
        },
        HeroEffect::CreatureStatPercent {
            creatures: alive(),
            stat: CreatureStat::HitPoints,
            bp: 2500,
        },
        HeroEffect::Regeneration {
            creatures: alive(),
            hit_points: 50,
        },
        HeroEffect::NoRangePenalty {
            creatures: CreatureSelector::Creature(key("t:peasant")),
        },
        HeroEffect::ShootWhenAdjacent {
            creatures: CreatureSelector::All,
        },
        HeroEffect::NoRetreat,
    ];
    let r = compile(&[p]).unwrap();
    let bow = r.artifact(r.artifact_by_key("t:bow").unwrap()).unwrap();
    assert_eq!(bow.effects.len(), 7);
    assert_eq!(
        bow.effects[4],
        HeroEffect::NoRangePenalty {
            creatures: CreatureSelector::Creature(r.creature_by_key("t:peasant").unwrap())
        }
    );
}
