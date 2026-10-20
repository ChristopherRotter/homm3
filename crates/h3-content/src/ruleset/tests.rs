extern crate std;

use std::string::ToString;
use std::vec;
use std::vec::Vec;

use super::*;
use crate::flags::CreatureFlag;
use crate::pack::{
    CreatureDef, CreatureInfoDef, CreatureOverride, FactionDef, PackManifest, ProfileDef,
    ResourceDef, TerrainDef,
};
use crate::quirks::QuirkRegistry;

pub(crate) fn key(s: &str) -> ContentKey {
    ContentKey::parse(s).unwrap()
}

pub(crate) fn creature(k: &str) -> CreatureDef {
    CreatureDef {
        key: key(k),
        speed: 5,
        info: CreatureInfoDef::default(),
    }
}

pub(crate) fn manifest(id: &str, depends: &[&str]) -> PackManifest {
    PackManifest {
        id: id.into(),
        version: "1".into(),
        depends: depends.iter().map(|d| (*d).into()).collect(),
    }
}

pub(crate) fn resource(k: &str) -> ResourceDef {
    ResourceDef {
        key: key(k),
        name: "R".into(),
    }
}

pub(crate) fn faction(k: &str) -> FactionDef {
    FactionDef {
        key: key(k),
        name: "F".into(),
        native_terrain: None,
    }
}

fn base() -> ContentPack {
    ContentPack {
        manifest: PackManifest {
            id: "base".into(),
            version: "1".into(),
            depends: vec![],
        },
        resources: vec![resource("base:wood"), resource("base:gold")],
        factions: vec![FactionDef {
            native_terrain: Some(key("base:grass")),
            ..faction("base:castle")
        }],
        terrains: vec![
            TerrainDef {
                key: key("base:grass"),
                name: "Grass".into(),
            },
            TerrainDef {
                key: key("base:lava"),
                name: "Lava".into(),
            },
        ],
        creatures: vec![creature("base:a"), creature("base:b")],
        creature_overrides: vec![],
        profiles: vec![
            ProfileDef {
                key: key("base:faithful"),
                quirks: vec![],
            },
            ProfileDef {
                key: key("base:quirky"),
                quirks: vec![("quirk.battle.x".into(), true)],
            },
        ],
        ..ContentPack::new(manifest("base", &[]))
    }
}

fn ext() -> ContentPack {
    ContentPack {
        manifest: PackManifest {
            id: "ext".into(),
            version: "2".into(),
            depends: vec!["base".into()],
        },
        creatures: vec![creature("ext:c")],
        ..ContentPack::new(manifest("ext", &[]))
    }
}

const TEST_REGISTRY: QuirkRegistry = QuirkRegistry::new(&["quirk.battle.x"]);

#[test]
fn compiles_packs_in_order_with_dense_handles() {
    let r = Ruleset::compile(&[base(), ext()], "base:faithful").unwrap();
    assert_eq!(r.creature_count(), 3);
    let c = r.creature_by_key("ext:c").unwrap();
    assert_eq!(c.raw(), 2);
    assert_eq!(r.creature_key(c).unwrap().as_str(), "ext:c");
    assert_eq!(r.creature(c).unwrap().speed, 5);
    assert_eq!(r.creature(CreatureTypeId::from_raw(3)), None);
    assert_eq!(r.creature_key(CreatureTypeId::from_raw(3)), None);
    assert_eq!(r.creature_by_key("ext:zzz"), None);
    assert_eq!(
        r.packs(),
        &[("base".into(), "1".into()), ("ext".into(), "2".into())]
    );
    assert_eq!(r.profile().as_str(), "base:faithful");
    assert_eq!(r.quirks().bits(), 0);
    assert_eq!(r.creature_info(c), r.creature(c));
}

#[test]
fn quirk_flags_come_from_the_profile() {
    let r = Ruleset::compile_with(&[base()], "base:quirky", &TEST_REGISTRY).unwrap();
    let flag = TEST_REGISTRY.lookup("quirk.battle.x").unwrap();
    assert!(r.quirks().is_enabled(flag));
    let err = Ruleset::compile(&[base()], "base:quirky").unwrap_err();
    assert_eq!(
        err,
        CompileError::UnknownQuirk {
            profile: "base:quirky".into(),
            flag: "quirk.battle.x".into()
        }
    );
    // A retired flag has no name in the engine's registry (ADR-0007: its position stays, its
    // name goes): a profile that still names it, on or off, does not compile, nor does one that
    // names its entry's empty name. Retired on 2026-10-12: BAT-QUIRK-2 (BAT-Q30) and BAT-QUIRK-9
    // (BAT-Q29).
    for flag in [
        "quirk.battle.attack_from_seen_pit",
        "quirk.battle.creature_mirror_untested",
        "",
    ] {
        for on in [true, false] {
            let mut pack = base();
            pack.profiles[1].quirks = vec![(flag.into(), on)];
            assert_eq!(
                Ruleset::compile(&[pack], "base:quirky").unwrap_err(),
                CompileError::UnknownQuirk {
                    profile: "base:quirky".into(),
                    flag: flag.into()
                },
                "{flag:?} {on}"
            );
        }
    }
}

#[test]
fn hash_is_stable_and_sensitive() {
    let a = Ruleset::compile(&[base()], "base:faithful").unwrap();
    let b = Ruleset::compile(&[base()], "base:faithful").unwrap();
    assert_eq!(a.content_hash(), b.content_hash());
    assert_eq!(a, b);
    let mut changed = base();
    changed.creatures[1].speed = 6;
    let c = Ruleset::compile(&[changed], "base:faithful").unwrap();
    assert_ne!(a.content_hash(), c.content_hash());
    let quirky = Ruleset::compile_with(&[base()], "base:quirky", &TEST_REGISTRY).unwrap();
    assert_ne!(a.content_hash(), quirky.content_hash());
    // Golden value: pins the canonical encoding (bump RULESET_HASH_VERSION when it changes).
    assert_eq!(
        a.content_hash(),
        GOLDEN_BASE_HASH,
        "got {:#018x}",
        a.content_hash()
    );
}

const GOLDEN_BASE_HASH: u64 = 0xd9c3_a262_c496_cd27;

fn expect(packs: &[ContentPack], profile: &str) -> CompileError {
    Ruleset::compile(packs, profile).unwrap_err()
}

#[test]
fn pack_level_errors() {
    let mut bad_id = base();
    bad_id.manifest.id = "Bad".into();
    assert_eq!(
        expect(&[bad_id], "base:faithful"),
        CompileError::InvalidPackId("Bad".into())
    );
    assert_eq!(
        expect(&[base(), base()], "base:faithful"),
        CompileError::DuplicatePack("base".into())
    );
    assert_eq!(
        expect(&[ext()], "base:faithful"),
        CompileError::MissingDependency {
            pack: "ext".into(),
            depends: "base".into()
        }
    );
    let mut outside = ext();
    outside.creatures.push(creature("base:z"));
    assert_eq!(
        expect(&[base(), outside], "base:faithful"),
        CompileError::KeyOutsidePack {
            key: "base:z".into(),
            pack: "ext".into()
        }
    );
    let mut dup = base();
    dup.creatures.push(creature("base:a"));
    assert_eq!(
        expect(&[dup], "base:faithful"),
        CompileError::DuplicateKey("base:a".into())
    );
    let mut dup_profile = base();
    dup_profile.profiles.push(ProfileDef {
        key: key("base:faithful"),
        quirks: vec![],
    });
    assert_eq!(
        expect(&[dup_profile], "base:faithful"),
        CompileError::DuplicateKey("base:faithful".into())
    );
    let mut profile_outside = ext();
    profile_outside.profiles.push(ProfileDef {
        key: key("base:p"),
        quirks: vec![],
    });
    assert_eq!(
        expect(&[base(), profile_outside], "base:faithful"),
        CompileError::KeyOutsidePack {
            key: "base:p".into(),
            pack: "ext".into()
        }
    );
    assert_eq!(
        expect(&[base()], "base:nope"),
        CompileError::UnknownProfile("base:nope".into())
    );
}
#[test]
fn too_many_creatures() {
    let mut big = base();
    big.creatures = (0..=u32::from(u16::MAX) + 1)
        .map(|i| creature(&std::format!("base:c{i}")))
        .collect();
    let result = Ruleset::compile(&[big], "base:faithful");
    assert!(matches!(result, Err(CompileError::TooManyCreatures)));
}

#[test]
fn creature_errors() {
    type Edit = fn(&mut CreatureDef);
    const SPEED: &str = "speed must be 0 for war machines and at least 1 otherwise";
    let cases: [(Edit, &str); 2] = [
        (|c| c.speed = 0, SPEED),
        // War machines never move (BAT-WAR-001).
        (|c| c.info.flags = vec!["war_machine".into()], SPEED),
    ];
    for (edit, reason) in cases {
        let mut p = base();
        edit(&mut p.creatures[0]);
        assert_eq!(
            expect(&[p], "base:faithful"),
            CompileError::InvalidCreature {
                key: "base:a".into(),
                reason
            }
        );
    }
    let mut machine = base();
    machine.creatures[0].speed = 0;
    machine.creatures[0].info.flags = vec!["war_machine".into()];
    assert!(Ruleset::compile(&[machine], "base:faithful").is_ok());
}
#[test]
fn error_messages() {
    let cases = [
        (
            CompileError::InvalidPackId("X".into()),
            "invalid pack id `X`",
        ),
        (
            CompileError::DuplicatePack("p".into()),
            "pack `p` is loaded twice",
        ),
        (
            CompileError::MissingDependency {
                pack: "a".into(),
                depends: "b".into(),
            },
            "pack `a` depends on `b`, which is not enabled",
        ),
        (
            CompileError::KeyOutsidePack {
                key: "b:x".into(),
                pack: "a".into(),
            },
            "pack `a` defines `b:x` outside its namespace",
        ),
        (
            CompileError::DuplicateKey("a:x".into()),
            "`a:x` is defined twice",
        ),
        (
            CompileError::InvalidCreature {
                key: "a:x".into(),
                reason: "bad",
            },
            "creature `a:x`: bad",
        ),
        (CompileError::TooManyCreatures, "too many creature types"),
        (
            CompileError::DependencyCycle("a".into()),
            "pack dependencies form a cycle through `a`",
        ),
        (
            CompileError::TooManyEntries("resource"),
            "too many resource entries",
        ),
        (
            CompileError::UnknownReference {
                kind: "faction",
                key: "a:f".into(),
            },
            "unknown faction `a:f`",
        ),
        (
            CompileError::ReferenceOutsideDependencies {
                key: "b:x".into(),
                pack: "a".into(),
            },
            "pack `a` refers to `b:x`, which is not in its dependencies",
        ),
        (
            CompileError::UnknownFlag("swims".into()),
            "unknown creature flag `swims`",
        ),
        (
            CompileError::DuplicateHeroParams("a".into()),
            "pack `a` provides hero parameters a second time",
        ),
        (
            CompileError::InvalidHeroParams("bad"),
            "hero parameters: bad",
        ),
        (
            CompileError::InvalidContent {
                kind: "spell",
                key: "a:s".into(),
                reason: "bad",
            },
            "spell `a:s`: bad",
        ),
        (
            CompileError::UnknownProfile("a:p".into()),
            "unknown profile `a:p`",
        ),
        (
            CompileError::UnknownQuirk {
                profile: "a:p".into(),
                flag: "f".into(),
            },
            "profile `a:p` sets unknown quirk flag `f`",
        ),
        (
            CompileError::NotADataQuirk("quirk.battle.x".into()),
            "an override fixes `quirk.battle.x`, which is no registered data quirk flag",
        ),
    ];
    for (err, text) in cases {
        assert_eq!(err.to_string(), text);
    }
}

// ---- E02 pack format v2: load order, references, creature info, overrides ----

#[test]
fn load_order_is_topological_with_pack_id_tie_break() {
    let first = ContentPack::new(manifest("aaa", &[]));
    let r = Ruleset::compile(&[ext(), base(), first], "base:faithful").unwrap();
    let ids: Vec<&str> = r.packs().iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(ids, ["aaa", "base", "ext"]);
    // The handle order follows the load order, not the argument order.
    assert_eq!(r.creature_by_key("base:a").unwrap().raw(), 0);
    assert_eq!(r.creature_by_key("ext:c").unwrap().raw(), 2);
    // A dependency chain: `z` depends on `ext`, which depends on `base`.
    let z = ContentPack::new(manifest("z", &["ext"]));
    let r = Ruleset::compile(&[z, ext(), base()], "base:faithful").unwrap();
    let ids: Vec<&str> = r.packs().iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(ids, ["base", "ext", "z"]);
}

#[test]
fn dependency_cycles_are_rejected() {
    let a = ContentPack::new(manifest("a", &["b"]));
    let b = ContentPack::new(manifest("b", &["a"]));
    assert_eq!(
        expect(&[base(), a, b], "base:faithful"),
        CompileError::DependencyCycle("a".into())
    );
    let selfish = ContentPack::new(manifest("s", &["s"]));
    assert_eq!(
        expect(&[base(), selfish], "base:faithful"),
        CompileError::DependencyCycle("s".into())
    );
}

fn rich_base() -> ContentPack {
    let mut p = base();
    let info = &mut p.creatures[0].info;
    info.name = "Alpha".into();
    info.plural = "Alphas".into();
    info.faction = Some(key("base:castle"));
    info.level = 3;
    info.cost = vec![(key("base:gold"), 60), (key("base:wood"), 1)];
    info.growth = 14;
    info.horde_growth = 4;
    info.fight_value = 100;
    info.ai_value = 80;
    info.adventure_count = [20, 50];
    info.flags = vec!["flying".into(), "two_hex".into()];
    p
}

#[test]
fn resources_factions_and_creature_info_compile() {
    let r = Ruleset::compile(&[rich_base()], "base:faithful").unwrap();
    assert_eq!(r.resources().len(), 2);
    assert_eq!(r.resources()[1].key.as_str(), "base:gold");
    assert_eq!(r.resources()[1].name, "R");
    assert_eq!(r.factions()[0].key.as_str(), "base:castle");
    assert_eq!(r.factions()[0].name, "F");
    let a = r.creature_by_key("base:a").unwrap();
    let info = r.creature_info(a).unwrap();
    assert_eq!(
        (info.name.as_str(), info.plural.as_str()),
        ("Alpha", "Alphas")
    );
    assert_eq!(info.faction, Some(FactionId::from_raw(0)));
    assert_eq!(info.level, 3);
    assert_eq!(
        info.cost,
        vec![(ResourceId::from_raw(1), 60), (ResourceId::from_raw(0), 1)]
    );
    assert_eq!((info.growth, info.horde_growth), (14, 4));
    assert_eq!((info.fight_value, info.ai_value), (100, 80));
    assert_eq!((info.speed, info.adventure_count), (5, [20, 50]));
    let stats = r.creature(a).unwrap();
    assert!(stats.flags.contains(CreatureFlag::Flying));
    assert!(stats.flags.contains(CreatureFlag::TwoHex));
    assert!(!stats.flags.contains(CreatureFlag::Undead));
    let b = r
        .creature_info(r.creature_by_key("base:b").unwrap())
        .unwrap();
    assert_eq!(b.faction, None);
    assert_eq!(b.level, 1);
    assert_eq!(r.creature_info(CreatureTypeId::from_raw(9)), None);
    // Info fields are part of the content hash.
    let plain = Ruleset::compile(&[base()], "base:faithful").unwrap();
    assert_ne!(plain.content_hash(), r.content_hash());
}

#[test]
fn reference_errors() {
    type Edit = fn(&mut CreatureInfoDef);
    let cases: [(Edit, CompileError); 4] = [
        (
            |i| i.faction = Some(key("base:rampart")),
            CompileError::UnknownReference {
                kind: "faction",
                key: "base:rampart".into(),
            },
        ),
        (
            |i| i.cost = vec![(key("base:ore"), 1)],
            CompileError::UnknownReference {
                kind: "resource",
                key: "base:ore".into(),
            },
        ),
        (
            |i| i.cost = vec![(key("base:gold"), 1), (key("base:gold"), 2)],
            CompileError::DuplicateKey("base:gold".into()),
        ),
        (
            |i| i.flags = vec!["flying".into(), "swims".into()],
            CompileError::UnknownFlag("swims".into()),
        ),
    ];
    for (edit, err) in cases {
        let mut p = base();
        edit(&mut p.creatures[0].info);
        assert_eq!(expect(&[p], "base:faithful"), err);
    }
    // A pack may only refer to content of its dependency closure, even if the other pack is
    // enabled and loaded before it.
    let mut stranger = ContentPack::new(manifest("stranger", &[]));
    let mut c = creature("stranger:x");
    c.info.faction = Some(key("base:castle"));
    stranger.creatures.push(c);
    assert_eq!(
        expect(&[base(), stranger.clone()], "base:faithful"),
        CompileError::ReferenceOutsideDependencies {
            key: "base:castle".into(),
            pack: "stranger".into()
        }
    );
    stranger.creatures[0].info.faction = None;
    stranger.creatures[0].info.cost = vec![(key("base:gold"), 1)];
    assert_eq!(
        expect(&[base(), stranger], "base:faithful"),
        CompileError::ReferenceOutsideDependencies {
            key: "base:gold".into(),
            pack: "stranger".into()
        }
    );
}

#[test]
fn resource_and_faction_keys_are_checked() {
    let mut p = base();
    p.resources.push(resource("base:gold"));
    assert_eq!(
        expect(&[p], "base:faithful"),
        CompileError::DuplicateKey("base:gold".into())
    );
    let mut p = base();
    p.factions.push(faction("base:castle"));
    assert_eq!(
        expect(&[p], "base:faithful"),
        CompileError::DuplicateKey("base:castle".into())
    );
    let mut p = ext();
    p.resources.push(resource("base:ore"));
    assert_eq!(
        expect(&[base(), p], "base:faithful"),
        CompileError::KeyOutsidePack {
            key: "base:ore".into(),
            pack: "ext".into()
        }
    );
    let mut p = ext();
    p.factions.push(faction("base:tower"));
    assert_eq!(
        expect(&[base(), p], "base:faithful"),
        CompileError::KeyOutsidePack {
            key: "base:tower".into(),
            pack: "ext".into()
        }
    );
}

#[test]
fn too_many_resources_and_factions() {
    let n = u32::from(u16::MAX) + 1;
    let mut p = base();
    p.resources = (0..=n)
        .map(|i| resource(&std::format!("base:r{i}")))
        .collect();
    assert_eq!(
        expect(&[p], "base:faithful"),
        CompileError::TooManyEntries("resource")
    );
    let mut p = base();
    p.factions = (0..=n)
        .map(|i| faction(&std::format!("base:f{i}")))
        .collect();
    assert_eq!(
        expect(&[p], "base:faithful"),
        CompileError::TooManyEntries("faction")
    );
    let mut p = base();
    p.terrains = (0..=n)
        .map(|i| TerrainDef {
            key: key(&std::format!("base:t{i}")),
            name: "T".into(),
        })
        .collect();
    assert_eq!(
        expect(&[p], "base:faithful"),
        CompileError::TooManyEntries("terrain")
    );
}
#[test]
fn info_validation() {
    let mut p = base();
    p.creatures[0].info.level = 0;
    assert_eq!(
        expect(&[p], "base:faithful"),
        CompileError::InvalidCreature {
            key: "base:a".into(),
            reason: "level must be at least 1"
        }
    );
    let mut p = base();
    p.creatures[0].info.adventure_count = [5, 4];
    assert_eq!(
        expect(&[p], "base:faithful"),
        CompileError::InvalidCreature {
            key: "base:a".into(),
            reason: "adventure_count high must not be below low"
        }
    );
}

fn full_patch() -> CreaturePatch {
    CreaturePatch {
        speed: Some(11),
        name: Some("Beta".into()),
        plural: Some("Betas".into()),
        faction: Some(Some(key("base:castle"))),
        level: Some(6),
        cost: Some(vec![(key("base:gold"), 3000), (key("ext:gems"), 1)]),
        growth: Some(2),
        horde_growth: Some(1),
        fight_value: Some(5000),
        ai_value: Some(6000),
        adventure_count: Some([4, 10]),
        flags: Some(vec!["undead".into()]),
        upgrades: Some(vec![key("ext:c")]),
    }
}

fn overriding(target: &str, patch: CreaturePatch) -> ContentPack {
    let mut p = ext();
    p.resources.push(resource("ext:gems"));
    p.creature_overrides.push(CreatureOverride {
        target: key(target),
        fixes: None,
        patch,
    });
    p
}

#[test]
fn overrides_change_only_the_listed_fields() {
    let before = Ruleset::compile(&[rich_base()], "base:faithful").unwrap();
    // An empty patch changes nothing.
    let r = Ruleset::compile(
        &[rich_base(), overriding("base:a", CreaturePatch::default())],
        "base:faithful",
    )
    .unwrap();
    let a = r.creature_by_key("base:a").unwrap();
    assert_eq!(r.creature(a), before.creature(a));
    assert_eq!(r.creature_info(a), before.creature_info(a));
    // A full patch changes everything.
    let r = Ruleset::compile(
        &[rich_base(), overriding("base:a", full_patch())],
        "base:faithful",
    )
    .unwrap();
    let info = r.creature_info(a).unwrap();
    assert_eq!(
        (info.name.as_str(), info.plural.as_str()),
        ("Beta", "Betas")
    );
    assert_eq!(info.faction, Some(FactionId::from_raw(0)));
    assert_eq!(info.level, 6);
    assert_eq!(
        info.cost,
        vec![
            (ResourceId::from_raw(1), 3000),
            (ResourceId::from_raw(2), 1)
        ]
    );
    assert_eq!((info.growth, info.horde_growth), (2, 1));
    assert_eq!((info.fight_value, info.ai_value), (5000, 6000));
    assert_eq!((info.speed, info.adventure_count), (11, [4, 10]));
    assert!(info.flags.contains(CreatureFlag::Undead));
    assert!(!info.flags.contains(CreatureFlag::Flying));
    assert_eq!(info.upgrades, [r.creature_by_key("ext:c").unwrap()]);
    // `Some(None)` makes a creature neutral.
    let neutral = CreaturePatch {
        faction: Some(None),
        ..CreaturePatch::default()
    };
    let r = Ruleset::compile(
        &[rich_base(), overriding("base:a", neutral)],
        "base:faithful",
    )
    .unwrap();
    assert_eq!(r.creature_info(a).unwrap().faction, None);
}

#[test]
fn override_errors() {
    assert_eq!(
        expect(
            &[base(), overriding("base:zzz", CreaturePatch::default())],
            "base:faithful"
        ),
        CompileError::UnknownReference {
            kind: "creature",
            key: "base:zzz".into()
        }
    );
    // `abc` is enabled and loads before `ext` (id order), but is not a dependency of `ext`.
    let mut other = ContentPack::new(manifest("abc", &[]));
    other.creatures.push(creature("abc:o"));
    assert_eq!(
        expect(
            &[base(), other, overriding("abc:o", CreaturePatch::default())],
            "base:faithful"
        ),
        CompileError::ReferenceOutsideDependencies {
            key: "abc:o".into(),
            pack: "ext".into()
        }
    );
    // References inside a patch are resolved in the overriding pack's scope.
    let bad_faction = CreaturePatch {
        faction: Some(Some(key("ext:nowhere"))),
        ..CreaturePatch::default()
    };
    assert_eq!(
        expect(
            &[base(), overriding("base:a", bad_faction)],
            "base:faithful"
        ),
        CompileError::UnknownReference {
            kind: "faction",
            key: "ext:nowhere".into()
        }
    );
    let bad_cost = CreaturePatch {
        cost: Some(vec![(key("ext:nothing"), 1)]),
        ..CreaturePatch::default()
    };
    assert!(matches!(
        expect(&[base(), overriding("base:a", bad_cost)], "base:faithful"),
        CompileError::UnknownReference {
            kind: "resource",
            ..
        }
    ));
    let bad_flag = CreaturePatch {
        flags: Some(vec!["swims".into()]),
        ..CreaturePatch::default()
    };
    assert_eq!(
        expect(&[base(), overriding("base:a", bad_flag)], "base:faithful"),
        CompileError::UnknownFlag("swims".into())
    );
    // The patched creature is validated like a definition.
    let broken = CreaturePatch {
        level: Some(0),
        ..CreaturePatch::default()
    };
    assert_eq!(
        expect(&[base(), overriding("base:a", broken)], "base:faithful"),
        CompileError::InvalidCreature {
            key: "base:a".into(),
            reason: "level must be at least 1"
        }
    );
}

#[test]
fn terrains_and_native_terrains() {
    let r = Ruleset::compile(&[base()], "base:faithful").unwrap();
    let grass = r.terrain_by_key("base:grass").unwrap();
    let lava = r.terrain_by_key("base:lava").unwrap();
    assert_eq!(r.terrains()[lava.index()].name, "Lava");
    assert_eq!(r.terrain_by_key("base:snow"), None);
    assert_eq!(r.factions()[0].native_terrain, Some(grass));
    // Errors: unknown terrain, duplicate key, a key outside the pack.
    let mut p = base();
    p.factions[0].native_terrain = Some(key("base:snow"));
    assert_eq!(
        expect(&[p], "base:faithful"),
        CompileError::UnknownReference {
            kind: "terrain",
            key: "base:snow".into()
        }
    );
    let mut p = base();
    p.terrains.push(p.terrains[0].clone());
    assert_eq!(
        expect(&[p], "base:faithful"),
        CompileError::DuplicateKey("base:grass".into())
    );
    let mut p = base();
    p.terrains[0].key = key("ext:grass");
    assert!(matches!(
        expect(&[p], "base:faithful"),
        CompileError::KeyOutsidePack { .. }
    ));
}

#[test]
fn formations_have_an_index() {
    assert_eq!(Formation::default(), Formation::Loose);
    assert_eq!([Formation::Loose.index(), Formation::Tight.index()], [0, 1]);
    assert_eq!(Formation::COUNT, 2);
}

#[test]
fn selectors_match_flags_creatures_and_upgrades() {
    use crate::effect::CreatureSelector;
    let mut p = rich_base();
    p.creatures[0].info.upgrades = vec![key("base:b")];
    let r = Ruleset::compile(&[p], "base:faithful").unwrap();
    let a = r.creature_by_key("base:a").unwrap();
    let b = r.creature_by_key("base:b").unwrap();
    let flying = CreatureSelector::Flag(CreatureFlag::Flying);
    assert!(CreatureSelector::All.matches(&r, b));
    assert!(flying.matches(&r, a) && !flying.matches(&r, b));
    // A named creature takes its direct upgrades too.
    assert!(CreatureSelector::Creature(a).matches(&r, a));
    assert!(CreatureSelector::Creature(a).matches(&r, b));
    assert!(!CreatureSelector::Creature(b).matches(&r, a));
    assert_eq!(
        r.faction_by_key("base:castle"),
        Some(FactionId::from_raw(0))
    );
    assert_eq!(r.faction_by_key("base:tower"), None);
}
