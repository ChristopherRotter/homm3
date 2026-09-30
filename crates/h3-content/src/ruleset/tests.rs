extern crate std;

use std::string::ToString;
use std::vec;
use std::vec::Vec;

use super::*;
use crate::pack::{PackManifest, ProfileDef};
use crate::quirks::QuirkRegistry;

fn key(s: &str) -> ContentKey {
    ContentKey::parse(s).unwrap()
}

fn creature(k: &str) -> CreatureDef {
    CreatureDef {
        key: key(k),
        attack: 5,
        defense: 4,
        damage_min: 1,
        damage_max: 3,
        hit_points: 10,
        speed: 5,
    }
}

fn battle_def() -> BattleParamsDef {
    BattleParamsDef {
        field_width: 17,
        field_height: 11,
        side_columns: 1,
        army_slots: 3,
        damage_roll_samples: 10,
        attack_bonus_per_point_percent: 5,
        attack_bonus_cap_percent: 300,
        defense_reduction_per_point_permille: 25,
        defense_reduction_cap_permille: 700,
        defend_bonus_percent: 20,
        defend_bonus_min: 1,
        formation: vec![vec![5], vec![3, 7], vec![2, 5, 8]],
    }
}

fn base() -> ContentPack {
    ContentPack {
        manifest: PackManifest {
            id: "base".into(),
            version: "1".into(),
            depends: vec![],
        },
        creatures: vec![creature("base:a"), creature("base:b")],
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
        battle: Some(battle_def()),
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
        profiles: vec![],
        battle: None,
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
    assert_eq!(r.battle().field.width, 17);
    assert_eq!(r.battle().army_slots, 3);
}

#[test]
fn formation_rows() {
    let r = Ruleset::compile(&[base()], "base:faithful").unwrap();
    let b = r.battle();
    assert_eq!(b.formation_row(1, 0), Some(5));
    assert_eq!(b.formation_row(3, 2), Some(8));
    assert_eq!(b.formation_row(3, 3), None);
    assert_eq!(b.formation_row(4, 0), None);
    assert_eq!(b.formation_row(0, 0), None);
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

const GOLDEN_BASE_HASH: u64 = 0x6ca4_e8e6_7843_ae40;

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
        expect(&[ext(), base()], "base:faithful"),
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
    let mut no_battle = base();
    no_battle.battle = None;
    assert_eq!(
        expect(&[no_battle], "base:faithful"),
        CompileError::MissingBattleParams
    );
    let mut second = ext();
    second.battle = Some(battle_def());
    assert_eq!(
        expect(&[base(), second], "base:faithful"),
        CompileError::DuplicateBattleParams("ext".into())
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
    let cases: [(Edit, &str); 4] = [
        (|c| c.damage_min = 0, "damage_min must be at least 1"),
        (
            |c| c.damage_max = 0,
            "damage_max must not be below damage_min",
        ),
        (|c| c.hit_points = 0, "hit_points must be at least 1"),
        (|c| c.speed = 0, "speed must be at least 1"),
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
}

#[test]
fn battle_param_errors() {
    type Edit = fn(&mut BattleParamsDef);
    let cases: Vec<(Edit, &str)> = vec![
        (
            |b| b.field_height = 0,
            "the field needs at least one row and two standable columns",
        ),
        (
            |b| b.field_width = 3,
            "the field needs at least one row and two standable columns",
        ),
        (|b| b.field_height = 16, "the field has more than 255 hexes"),
        (
            |b| b.army_slots = 0,
            "army_slots must be between 1 and min(16, field_height)",
        ),
        (
            |b| b.army_slots = 17,
            "army_slots must be between 1 and min(16, field_height)",
        ),
        (
            |b| b.army_slots = 12,
            "army_slots must be between 1 and min(16, field_height)",
        ),
        (
            |b| b.damage_roll_samples = 0,
            "damage_roll_samples must be at least 1",
        ),
        (
            |b| b.defense_reduction_cap_permille = 1001,
            "defense_reduction_cap_permille must not exceed 1000",
        ),
        (
            |b| drop(b.formation.pop()),
            "formation needs one entry per army size 1..=army_slots",
        ),
        (
            |b| b.formation[1] = vec![1],
            "formation entry n must list n rows",
        ),
        (
            |b| b.formation[1] = vec![1, 11],
            "formation row outside the field",
        ),
        (
            |b| b.formation[2] = vec![1, 2, 1],
            "formation rows must be distinct",
        ),
    ];
    for (edit, reason) in cases {
        let mut p = base();
        edit(p.battle.as_mut().unwrap());
        assert_eq!(
            expect(&[p], "base:faithful"),
            CompileError::InvalidBattleParams(reason),
            "{reason}"
        );
    }
    // Wide fields with many rows are fine as long as they fit in 255 hexes; large slot counts too.
    let mut ok = base();
    let b = ok.battle.as_mut().unwrap();
    b.field_width = 15;
    b.field_height = 17;
    b.army_slots = 16;
    b.formation = (1..=16u8).map(|n| (0..n).collect()).collect();
    assert!(Ruleset::compile(&[ok], "base:faithful").is_ok());
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
            "pack `a` depends on `b`, which is not loaded before it",
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
            CompileError::MissingBattleParams,
            "no pack provides battle parameters",
        ),
        (
            CompileError::DuplicateBattleParams("a".into()),
            "pack `a` provides battle parameters a second time",
        ),
        (
            CompileError::InvalidBattleParams("bad"),
            "battle parameters: bad",
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
    ];
    for (err, text) in cases {
        assert_eq!(err.to_string(), text);
    }
}
