extern crate std;

use std::string::ToString;
use std::vec;
use std::vec::Vec;

use super::*;
use crate::flags::CreatureFlag;
use crate::pack::{
    CreatureDef, CreatureInfoDef, CreatureOverride, FactionDef, ObstacleDef, PackManifest,
    ProfileDef, ResourceDef, TerrainDef,
};
use crate::quirks::QuirkRegistry;

pub(crate) fn key(s: &str) -> ContentKey {
    ContentKey::parse(s).unwrap()
}

pub(crate) fn creature(k: &str) -> CreatureDef {
    CreatureDef {
        key: key(k),
        attack: 5,
        defense: 4,
        damage_min: 1,
        damage_max: 3,
        hit_points: 10,
        speed: 5,
        info: CreatureInfoDef::default(),
    }
}

pub(crate) fn battle_def() -> BattleParamsDef {
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
        ranged_full_damage_distance: 10,
        ranged_penalty_percent: 50,
        melee_penalty_percent: 50,
        morale_dice: 24,
        good_morale_chances: vec![1, 2, 3],
        bad_morale_chances: vec![2, 4, 6],
        luck_dice: 24,
        good_luck_chances: vec![1, 2, 3],
        war_machine_positions: vec![[1, 3], [1, 1], [1, 9]],
        loose_formation: vec![vec![5], vec![3, 7], vec![2, 5, 8]],
        tight_formation: vec![vec![5], vec![4, 6], vec![4, 5, 6]],
        tactics_zone_columns: vec![3, 5, 7],
        surrender: None,
        hero_casts_per_round: 1,
        side_stack_limit: 20,
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

fn obstacle(k: &str, terrains: &[&str], blocked: &[(i8, i8)], absolute: bool) -> ObstacleDef {
    ObstacleDef {
        key: key(k),
        terrains: terrains.iter().map(|t| key(t)).collect(),
        width: 2,
        height: 1,
        blocked: blocked.to_vec(),
        absolute,
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
        obstacles: vec![
            obstacle(
                "base:rock",
                &["base:grass", "base:lava"],
                &[(0, 0), (1, -1)],
                false,
            ),
            obstacle("base:cliff", &["base:lava"], &[(5, 4), (6, 4)], true),
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
        battle: Some(battle_def()),
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
    assert_eq!(r.battle().field.width, 17);
    assert_eq!(r.battle().army_slots, 3);
}

#[test]
fn formation_rows() {
    let r = Ruleset::compile(&[base()], "base:faithful").unwrap();
    let b = r.battle();
    assert_eq!(b.formation_row(Formation::Loose, 1, 0), Some(5));
    assert_eq!(b.formation_row(Formation::Loose, 3, 2), Some(8));
    assert_eq!(b.formation_row(Formation::Tight, 3, 2), Some(6));
    assert_eq!(b.formation_row(Formation::Tight, 2, 0), Some(4));
    for f in [Formation::Loose, Formation::Tight] {
        assert_eq!(b.formation_row(f, 3, 3), None);
        assert_eq!(b.formation_row(f, 4, 0), None);
        assert_eq!(b.formation_row(f, 0, 0), None);
    }
    assert_eq!(Formation::default(), Formation::Loose);
    assert_eq!(Formation::Tight.index(), 1);
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
    let mut tight = base();
    tight.battle.as_mut().unwrap().tight_formation[1] = vec![3, 7];
    let tight = Ruleset::compile(&[tight], "base:faithful").unwrap();
    assert_ne!(a.content_hash(), tight.content_hash());
    let mut tactics = base();
    tactics.battle.as_mut().unwrap().tactics_zone_columns[2] = 15;
    let tactics = Ruleset::compile(&[tactics], "base:faithful").unwrap();
    assert_ne!(a.content_hash(), tactics.content_hash());
    // The surrender's resource is resolved like any reference (BAT-FLE-002).
    let mut surrender = base();
    surrender.battle.as_mut().unwrap().surrender = Some(crate::SurrenderDef {
        resource: key("base:gold"),
        cost_percent: 50,
    });
    let priced = Ruleset::compile(&[surrender.clone()], "base:faithful").unwrap();
    assert_ne!(a.content_hash(), priced.content_hash());
    let gold = priced
        .resources()
        .iter()
        .position(|r| r.key.as_str() == "base:gold");
    assert_eq!(
        priced
            .battle()
            .surrender
            .map(|s| (s.resource.index(), s.cost_percent)),
        Some((gold.unwrap(), 50))
    );
    surrender
        .battle
        .as_mut()
        .unwrap()
        .surrender
        .as_mut()
        .unwrap()
        .resource = key("base:silver");
    assert_eq!(
        Ruleset::compile(&[surrender], "base:faithful").unwrap_err(),
        CompileError::UnknownReference {
            kind: "resource",
            key: "base:silver".into(),
        }
    );
    // Golden value: pins the canonical encoding (bump RULESET_HASH_VERSION when it changes).
    assert_eq!(
        a.content_hash(),
        GOLDEN_BASE_HASH,
        "got {:#018x}",
        a.content_hash()
    );
}

const GOLDEN_BASE_HASH: u64 = 0xf0f5_5aab_6d57_54e3;

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
    const DAMAGE: &str = "damage_min must be at least 1 (war machines: or damage [0, 0])";
    const SPEED: &str = "speed must be 0 for war machines and at least 1 otherwise";
    let cases: [(Edit, &str); 9] = [
        (|c| c.damage_min = 0, DAMAGE),
        (
            |c| c.damage_max = 0,
            "damage_max must not be below damage_min",
        ),
        (|c| c.hit_points = 0, "hit_points must be at least 1"),
        (|c| c.speed = 0, SPEED),
        // War machines never move (BAT-WAR-001).
        (|c| c.info.flags = vec!["war_machine".into()], SPEED),
        // A war machine may have no damage, but not a damage range starting at 0.
        (
            |c| {
                c.info.flags = vec!["war_machine".into()];
                c.damage_min = 0;
            },
            DAMAGE,
        ),
        (
            |c| {
                c.info.flags = vec!["war_machine".into()];
                c.damage_min = 0;
                c.damage_max = 0;
                c.speed = 0;
                c.hit_points = 0;
            },
            "hit_points must be at least 1",
        ),
        // Only war machines act automatically.
        (
            |c| c.info.abilities = vec![AbilityDef::AmmoSupply],
            "ammo_supply and first_aid require flag war_machine",
        ),
        (
            |c| {
                c.info.abilities = vec![AbilityDef::FirstAid {
                    min_hp: 1,
                    max_hp: 2,
                }]
            },
            "ammo_supply and first_aid require flag war_machine",
        ),
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
            |b| b.ranged_penalty_percent = 101,
            "ranged and melee penalties must not exceed 100 percent",
        ),
        (
            |b| b.melee_penalty_percent = 101,
            "ranged and melee penalties must not exceed 100 percent",
        ),
        (
            |b| b.morale_dice = 0,
            "morale_dice and luck_dice must be at least 1",
        ),
        (
            |b| b.luck_dice = 0,
            "morale_dice and luck_dice must be at least 1",
        ),
        (
            |b| b.bad_morale_chances = vec![1, 24],
            "bad morale chances must be below morale_dice",
        ),
        (
            |b| b.bad_morale_chances = vec![1; 9],
            "morale and luck chances: at most 8 levels, each at most the die's sides",
        ),
        (
            |b| b.good_luck_chances = vec![25],
            "morale and luck chances: at most 8 levels, each at most the die's sides",
        ),
        (
            |b| b.war_machine_positions = vec![[1, 3]],
            "war_machine_positions needs 3 hexes [x, y] on the field",
        ),
        (
            |b| b.tactics_zone_columns.truncate(2),
            "tactics_zone_columns needs one width per skill level, each between 1 and the standable columns",
        ),
        (
            |b| b.side_stack_limit = 22,
            "side_stack_limit must be at most 21",
        ),
        (
            |b| b.tactics_zone_columns[0] = 0,
            "tactics_zone_columns needs one width per skill level, each between 1 and the standable columns",
        ),
        (
            |b| b.tactics_zone_columns[2] = 16,
            "tactics_zone_columns needs one width per skill level, each between 1 and the standable columns",
        ),
        (
            |b| b.war_machine_positions[2] = [17, 9],
            "war_machine_positions needs 3 hexes [x, y] on the field",
        ),
        (
            |b| b.war_machine_positions[2] = [1, 11],
            "war_machine_positions needs 3 hexes [x, y] on the field",
        ),
        // covers: BAT-WAR-001
        (
            |b| b.war_machine_positions[0] = [0, 3],
            "war machine positions need room for a two-hex machine behind them",
        ),
        (
            |b| b.war_machine_positions[1] = [2, 3],
            "war machine positions overlap (as two-hex footprints)",
        ),
        (
            |b| b.war_machine_positions[2] = [1, 3],
            "war machine positions overlap (as two-hex footprints)",
        ),
        (
            |b| b.war_machine_positions[2] = [8, 5],
            "war machine positions overlap (as two-hex footprints)",
        ),
        (
            |b| drop(b.loose_formation.pop()),
            "formations need one entry per army size 1..=army_slots",
        ),
        (
            |b| drop(b.tight_formation.pop()),
            "formations need one entry per army size 1..=army_slots",
        ),
        (
            |b| b.loose_formation[1] = vec![1],
            "formation entry n must list n rows",
        ),
        (
            |b| b.tight_formation[1] = vec![1, 11],
            "formation row outside the field",
        ),
        (
            |b| b.loose_formation[2] = vec![1, 2, 1],
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
    b.loose_formation = (1..=16u8).map(|n| (0..n).collect()).collect();
    b.tight_formation = (1..=16u8).map(|n| (0..n).rev().collect()).collect();
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
            CompileError::InvalidObstacle {
                key: "a:o".into(),
                reason: "bad",
            },
            "obstacle `a:o`: bad",
        ),
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
    info.shots = 12;
    info.casts = 1;
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
    assert_eq!((info.casts, info.adventure_count), (1, [20, 50]));
    let stats = r.creature(a).unwrap();
    assert_eq!(stats.shots, 12);
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
    let mut p = base();
    p.obstacles = (0..=n)
        .map(|i| {
            obstacle(
                &std::format!("base:o{i}"),
                &["base:grass"],
                &[(0, 0)],
                false,
            )
        })
        .collect();
    assert_eq!(
        expect(&[p], "base:faithful"),
        CompileError::TooManyEntries("obstacle")
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
        attack: Some(9),
        defense: Some(8),
        damage_min: Some(2),
        damage_max: Some(7),
        hit_points: Some(30),
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
        shots: Some(24),
        casts: Some(3),
        adventure_count: Some([4, 10]),
        flags: Some(vec!["undead".into()]),
        abilities: Some(vec![AbilityDef::Hate {
            target: key("ext:c"),
            percent: 50,
        }]),
        upgrades: Some(vec![key("ext:c")]),
    }
}

fn overriding(target: &str, patch: CreaturePatch) -> ContentPack {
    let mut p = ext();
    p.resources.push(resource("ext:gems"));
    p.creature_overrides.push(CreatureOverride {
        target: key(target),
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
    let stats = r.creature(a).unwrap();
    assert_eq!(
        (
            stats.attack,
            stats.defense,
            stats.damage_min,
            stats.damage_max
        ),
        (9, 8, 2, 7)
    );
    assert_eq!((stats.hit_points, stats.speed), (30, 11));
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
    assert_eq!((info.casts, info.adventure_count), (3, [4, 10]));
    assert_eq!(stats.shots, 24);
    assert!(stats.flags.contains(CreatureFlag::Undead));
    assert!(!stats.flags.contains(CreatureFlag::Flying));
    assert_eq!(
        r.abilities(a),
        [CompiledAbility::Hate {
            target: r.creature_by_key("ext:c").unwrap(),
            percent: 50
        }]
    );
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
        hit_points: Some(0),
        ..CreaturePatch::default()
    };
    assert_eq!(
        expect(&[base(), overriding("base:a", broken)], "base:faithful"),
        CompileError::InvalidCreature {
            key: "base:a".into(),
            reason: "hit_points must be at least 1"
        }
    );
}

fn with_abilities(abilities: Vec<AbilityDef>) -> ContentPack {
    let mut p = base();
    p.creatures[0].info.abilities = abilities;
    p
}

#[test]
fn abilities_compile_with_forward_references() {
    use crate::ability::{AreaShot, AuraTarget};
    let mut p = with_abilities(vec![
        // `base:b` is defined after `base:a`.
        AbilityDef::Hate {
            target: key("base:b"),
            percent: 50,
        },
        AbilityDef::MoraleAura {
            amount: 1,
            affects: AuraTarget::Allies,
        },
    ]);
    p.creatures[1].info.flags = vec!["area_attack".into(), "multi_headed".into()];
    p.creatures[1].info.abilities = vec![
        AbilityDef::AreaShot {
            effect: AreaShot::Fireball,
        },
        AbilityDef::ThreeHeadedAttack,
    ];
    let r = Ruleset::compile(&[p], "base:faithful").unwrap();
    let a = r.creature_by_key("base:a").unwrap();
    let b = r.creature_by_key("base:b").unwrap();
    assert_eq!(
        r.abilities(a),
        [
            CompiledAbility::Hate {
                target: b,
                percent: 50
            },
            CompiledAbility::MoraleAura {
                amount: 1,
                affects: AuraTarget::Allies
            }
        ]
    );
    assert_eq!(r.abilities(b).len(), 2);
    assert!(r.abilities(CreatureTypeId::from_raw(99)).is_empty());
    // Abilities are part of the content hash.
    let plain = Ruleset::compile(&[with_abilities(vec![])], "base:faithful").unwrap();
    let hated = Ruleset::compile(
        &[with_abilities(vec![AbilityDef::Teleports])],
        "base:faithful",
    )
    .unwrap();
    assert_ne!(plain.content_hash(), hated.content_hash());
}

#[test]
fn ability_errors() {
    let invalid = |reason| CompileError::InvalidCreature {
        key: "base:a".into(),
        reason,
    };
    assert_eq!(
        expect(
            &[with_abilities(vec![AbilityDef::Hate {
                target: key("base:zzz"),
                percent: 50
            }])],
            "base:faithful"
        ),
        CompileError::UnknownReference {
            kind: "creature",
            key: "base:zzz".into()
        }
    );
    // A base creature cannot refer to a creature of a pack it does not depend on.
    assert_eq!(
        expect(
            &[
                with_abilities(vec![AbilityDef::Hate {
                    target: key("ext:c"),
                    percent: 50
                }]),
                ext()
            ],
            "base:faithful"
        ),
        CompileError::ReferenceOutsideDependencies {
            key: "ext:c".into(),
            pack: "base".into()
        }
    );
    assert_eq!(
        expect(
            &[with_abilities(vec![AbilityDef::Jousting {
                percent_per_hex: 0
            }])],
            "base:faithful"
        ),
        invalid("jousting percent_per_hex must be at least 1")
    );
    assert_eq!(
        expect(
            &[with_abilities(vec![AbilityDef::AttacksAllAdjacent])],
            "base:faithful"
        ),
        invalid("flag multi_headed requires attacks_all_adjacent or three_headed_attack")
    );
    let mut flagged = base();
    flagged.creatures[0].info.flags = vec!["area_attack".into()];
    assert_eq!(
        expect(&[flagged], "base:faithful"),
        invalid("flag area_attack requires an area_shot ability (and vice versa)")
    );
    // Overrides resolve their abilities in the overriding pack's scope and are validated.
    let bad = CreaturePatch {
        abilities: Some(vec![AbilityDef::Hate {
            target: key("ext:nobody"),
            percent: 50,
        }]),
        ..CreaturePatch::default()
    };
    assert!(matches!(
        expect(&[base(), overriding("base:a", bad)], "base:faithful"),
        CompileError::UnknownReference {
            kind: "creature",
            ..
        }
    ));
    let twice = CreaturePatch {
        abilities: Some(vec![AbilityDef::Teleports, AbilityDef::Teleports]),
        ..CreaturePatch::default()
    };
    assert_eq!(
        expect(&[base(), overriding("base:a", twice)], "base:faithful"),
        invalid("an ability is listed twice")
    );
}

#[test]
fn roll_tables_index_levels_from_one_and_cap_at_the_top() {
    let r = Ruleset::compile(&[base()], "base:faithful").unwrap();
    let b = r.battle();
    assert_eq!((b.morale_dice, b.luck_dice), (24, 24));
    assert_eq!(b.good_morale.levels(), 3);
    assert_eq!(
        (0..=4).map(|l| b.bad_morale.chance(l)).collect::<Vec<_>>(),
        vec![0, 2, 4, 6, 6]
    );
    assert_eq!(b.good_luck.chance(1), 1);
    assert_eq!(RollTable::default().chance(1), 0);
}

#[test]
fn aura_values_are_keyed_and_limited() {
    use crate::ability::AuraTarget;
    let morale = |amount, affects| AbilityDef::MoraleAura { amount, affects };
    let k = |a: AbilityDef| a.aura_key();
    assert_eq!(
        k(morale(1, AuraTarget::Allies)),
        k(morale(1, AuraTarget::Allies))
    );
    assert_ne!(
        k(morale(1, AuraTarget::Allies)),
        k(morale(1, AuraTarget::Enemies))
    );
    assert_ne!(
        k(morale(-1, AuraTarget::Enemies)),
        k(morale(1, AuraTarget::Enemies))
    );
    assert_ne!(
        k(morale(-1, AuraTarget::Enemies)),
        k(AbilityDef::LuckAura {
            amount: -1,
            affects: AuraTarget::Enemies
        })
    );
    assert_eq!(k(AbilityDef::Regeneration), None);
    let with_auras = |n: i8| {
        let mut p = base();
        p.creatures = (0..n)
            .map(|i| {
                let mut c = creature(&std::format!("base:c{i}"));
                c.info.abilities = vec![morale(i + 1, AuraTarget::Allies)];
                c
            })
            .collect();
        p
    };
    Ruleset::compile(&[with_auras(8)], "base:faithful").unwrap();
    assert_eq!(
        expect(&[with_auras(9)], "base:faithful"),
        CompileError::TooManyEntries("distinct aura")
    );
}

#[test]
fn terrains_native_terrains_and_obstacles() {
    let r = Ruleset::compile(&[base()], "base:faithful").unwrap();
    let grass = r.terrain_by_key("base:grass").unwrap();
    let lava = r.terrain_by_key("base:lava").unwrap();
    assert_eq!(r.terrains()[lava.index()].name, "Lava");
    assert_eq!(r.terrain_by_key("base:snow"), None);
    assert_eq!(r.factions()[0].native_terrain, Some(grass));
    let o = &r.obstacles()[0];
    assert_eq!(
        (o.terrains.as_slice(), o.width, o.absolute),
        (&[grass, lava][..], 2, false)
    );
    assert_eq!(r.obstacles()[1].blocked, vec![(5, 4), (6, 4)]);
    // Errors: unknown terrain, duplicate key, invalid shapes.
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
    p.obstacles.push(p.obstacles[0].clone());
    assert_eq!(
        expect(&[p], "base:faithful"),
        CompileError::DuplicateKey("base:rock".into())
    );
    let mut p = base();
    p.obstacles[0].terrains.push(key("base:snow"));
    assert!(matches!(
        expect(&[p], "base:faithful"),
        CompileError::UnknownReference {
            kind: "terrain",
            ..
        }
    ));
    let mut p = base();
    p.terrains[0].key = key("ext:grass");
    assert!(matches!(
        expect(&[p], "base:faithful"),
        CompileError::KeyOutsidePack { .. }
    ));
    let mut p = base();
    p.obstacles[0].key = key("ext:rock");
    assert!(matches!(
        expect(&[p], "base:faithful"),
        CompileError::KeyOutsidePack { .. }
    ));
    let shapes: [(ObstacleDef, &str); 5] = [
        (
            obstacle("base:o", &[], &[(0, 0)], false),
            "an obstacle needs at least one terrain and one blocked hex",
        ),
        (
            obstacle("base:o", &["base:grass"], &[], false),
            "an obstacle needs at least one terrain and one blocked hex",
        ),
        (
            obstacle("base:o", &["base:grass"], &[(0, 1)], false),
            "usual obstacles need width and height >= 1 and blocked dy <= 0",
        ),
        (
            ObstacleDef {
                height: 0,
                ..obstacle("base:o", &["base:grass"], &[(0, 0)], false)
            },
            "usual obstacles need width and height >= 1 and blocked dy <= 0",
        ),
        (
            obstacle("base:o", &["base:grass"], &[(0, 4)], true),
            "absolute obstacles must block standable hexes of the field",
        ),
    ];
    for (o, reason) in shapes {
        let mut p = base();
        p.obstacles.push(o);
        assert_eq!(
            expect(&[p], "base:faithful"),
            CompileError::InvalidObstacle {
                key: "base:o".into(),
                reason
            }
        );
    }
    for (x, y) in [(16, 4), (5, -1), (5, 11)] {
        let mut p = base();
        p.obstacles
            .push(obstacle("base:o", &["base:grass"], &[(x, y)], true));
        assert!(matches!(
            expect(&[p], "base:faithful"),
            CompileError::InvalidObstacle { .. }
        ));
    }
    let mut p = base();
    p.obstacles.push(ObstacleDef {
        width: 0,
        ..obstacle("base:o", &["base:grass"], &[(0, 0)], false)
    });
    assert!(matches!(
        expect(&[p], "base:faithful"),
        CompileError::InvalidObstacle { .. }
    ));
}
