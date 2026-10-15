extern crate std;

use std::string::ToString;
use std::vec;
use std::vec::Vec;

use h3_content::{
    ContentPack, CreatureDef, CreatureInfoDef, CreatureOverride, CreaturePatch, FactionDef,
    Formation, PackManifest, ProfileDef, ResourceDef, SpellDef, TerrainDef,
};

use super::*;
use crate::ability::{AreaShot, AuraTarget};
use crate::pack::{
    BattleCreatureDef, BattleCreatureOverride, BattleParamsDef, BattleSpellDef, ObstacleDef,
    SurrenderDef,
};
use crate::params::{MAX_ROLL_LEVELS, RollTable};

mod battlefields;
mod sieges;
mod spells;

// ---- Test support: a core pack with its battle part (R01) ----

pub(crate) fn key(s: &str) -> ContentKey {
    ContentKey::parse(s).unwrap()
}

pub(crate) fn manifest(id: &str, depends: &[&str]) -> PackManifest {
    PackManifest {
        id: id.into(),
        version: "1".into(),
        depends: depends.iter().map(|d| (*d).into()).collect(),
    }
}

/// A core pack and its battle part, as the tests build them.
#[derive(Debug, Clone)]
pub(crate) struct Pack {
    pub(crate) core: ContentPack,
    pub(crate) battle: BattlePack,
}

/// A creature's two definitions.
#[derive(Debug, Clone)]
pub(crate) struct Creature {
    pub(crate) core: CreatureDef,
    pub(crate) battle: BattleCreatureDef,
}

/// A spell's two definitions.
#[derive(Debug, Clone)]
pub(crate) struct Spell {
    pub(crate) core: SpellDef,
    pub(crate) battle: BattleSpellDef,
}

impl Pack {
    pub(crate) fn new(manifest: PackManifest) -> Self {
        Self {
            core: ContentPack::new(manifest.clone()),
            battle: BattlePack::new(manifest),
        }
    }

    pub(crate) fn creatures(mut self, list: impl IntoIterator<Item = Creature>) -> Self {
        for c in list {
            self.add_creature(c);
        }
        self
    }

    pub(crate) fn add_creature(&mut self, c: Creature) {
        self.core.creatures.push(c.core);
        self.battle.creatures.push(c.battle);
    }

    pub(crate) fn add_spells(&mut self, list: impl IntoIterator<Item = Spell>) {
        for s in list {
            self.core.spells.push(s.core);
            self.battle.spells.push(s.battle);
        }
    }

    /// Adds an empty override of `target` to both halves; returns its index.
    pub(crate) fn add_override(&mut self, target: &str, fixes: Option<&str>) -> usize {
        self.core.creature_overrides.push(CreatureOverride {
            target: key(target),
            fixes: fixes.map(Into::into),
            patch: CreaturePatch::default(),
        });
        self.battle.creature_overrides.push(BattleCreatureOverride {
            target: key(target),
            fixes: fixes.map(Into::into),
            patch: BattleCreaturePatch::default(),
        });
        self.battle.creature_overrides.len() - 1
    }
}

pub(crate) fn creature(k: &str) -> Creature {
    Creature {
        core: CreatureDef {
            key: key(k),
            speed: 5,
            info: CreatureInfoDef::default(),
        },
        battle: BattleCreatureDef {
            key: key(k),
            attack: 5,
            defense: 4,
            damage_min: 1,
            damage_max: 3,
            hit_points: 10,
            shots: 0,
            casts: 0,
            abilities: vec![],
        },
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
        siege: None,
    }
}

pub(crate) fn compile_with(
    packs: &[Pack],
    profile: &str,
    registry: &QuirkRegistry,
) -> Result<BattleRuleset, BattleCompileError> {
    let cores: Vec<ContentPack> = packs.iter().map(|p| p.core.clone()).collect();
    let battles: Vec<BattlePack> = packs.iter().map(|p| p.battle.clone()).collect();
    let core = Ruleset::compile_with(&cores, profile, registry)?;
    BattleRuleset::compile_with(Arc::new(core), &battles, registry)
}

pub(crate) fn compile(packs: &[Pack], profile: &str) -> Result<BattleRuleset, BattleCompileError> {
    compile_with(packs, profile, &REGISTRY)
}

fn expect(packs: &[Pack], profile: &str) -> BattleCompileError {
    compile(packs, profile).unwrap_err()
}

/// The core compiler's error a failed compile raised.
pub(crate) fn content(err: BattleCompileError) -> CompileError {
    match err {
        BattleCompileError::Content(e) => e,
        other => panic!("{other:?}"),
    }
}

fn resource(k: &str) -> ResourceDef {
    ResourceDef {
        key: key(k),
        name: "R".into(),
    }
}

fn obstacle(k: &str, terrains: &[&str], blocked: &[(i8, i8)], absolute: bool) -> ObstacleDef {
    ObstacleDef {
        key: key(k),
        terrains: terrains.iter().map(|t| key(t)).collect(),
        battlefields: Vec::new(),
        width: 2,
        height: 1,
        blocked: blocked.to_vec(),
        absolute,
    }
}

fn base() -> Pack {
    let mut p =
        Pack::new(manifest("base", &[])).creatures([creature("base:a"), creature("base:b")]);
    p.core.resources = vec![resource("base:wood"), resource("base:gold")];
    p.core.factions = vec![FactionDef {
        key: key("base:castle"),
        name: "F".into(),
        native_terrain: Some(key("base:grass")),
    }];
    p.core.terrains = vec![
        TerrainDef {
            key: key("base:grass"),
            name: "Grass".into(),
        },
        TerrainDef {
            key: key("base:lava"),
            name: "Lava".into(),
        },
    ];
    p.core.profiles = vec![ProfileDef {
        key: key("base:faithful"),
        quirks: vec![],
    }];
    p.battle.obstacles = vec![
        obstacle(
            "base:rock",
            &["base:grass", "base:lava"],
            &[(0, 0), (1, -1)],
            false,
        ),
        obstacle("base:cliff", &["base:lava"], &[(5, 4), (6, 4)], true),
    ];
    p.battle.battle = Some(battle_def());
    p
}

fn ext() -> Pack {
    let mut m = manifest("ext", &["base"]);
    m.version = "2".into();
    Pack::new(m).creatures([creature("ext:c")])
}

// ---- The part's shape: one battle pack per core pack, the core's handles (R01) ----

#[test]
fn compiles_on_the_core_with_its_handles() {
    let r = compile(&[base(), ext()], "base:faithful").unwrap();
    let c = r.creature_by_key("ext:c").unwrap();
    assert_eq!(c.raw(), 2);
    let row = r.creature(c).unwrap();
    assert_eq!((row.attack, row.speed, row.hit_points), (5, 5, 10));
    assert_eq!(r.creature(CreatureTypeId::from_raw(3)), None);
    // The core's row stays reachable through the core.
    assert_eq!(r.core().creature(c).unwrap().speed, 5);
    assert_eq!(r.core_hash(), r.core().content_hash());
    assert_ne!(r.content_hash(), r.core_hash());
    assert_eq!(r.battle().field.width, 17);
    assert_eq!(r.battle().army_slots, 3);
    assert_eq!(r.packs().len(), 2);
    assert!(r.spells().is_empty());
}

#[test]
fn battle_packs_pair_with_the_core_packs() {
    let core = |packs: &[Pack]| {
        let cores: Vec<ContentPack> = packs.iter().map(|p| p.core.clone()).collect();
        Arc::new(Ruleset::compile(&cores, "base:faithful").unwrap())
    };
    let both = core(&[base(), ext()]);
    let fails =
        |battles: Vec<BattlePack>| BattleRuleset::compile(Arc::clone(&both), &battles).unwrap_err();
    assert_eq!(
        fails(vec![base().battle]),
        BattleCompileError::MissingPack("ext".into())
    );
    let mut stranger = ext().battle;
    stranger.manifest.version = "3".into();
    assert_eq!(
        fails(vec![base().battle, stranger]),
        BattleCompileError::UnknownPack("ext@3".into())
    );
    assert_eq!(
        fails(vec![base().battle, base().battle, ext().battle]),
        BattleCompileError::DuplicatePack("base".into())
    );
    let mut cyclic = base().battle;
    cyclic.manifest.depends = vec!["ext".into()];
    assert_eq!(
        fails(vec![cyclic, ext().battle]),
        BattleCompileError::UnknownDependency {
            pack: "base".into(),
            dependency: "ext".into()
        }
    );
    // The argument order does not matter: the core's load order does.
    let r = BattleRuleset::compile(both, &[ext().battle, base().battle]).unwrap();
    assert_eq!(r.creature_count(), 3);
}

#[test]
fn every_core_creature_and_spell_needs_its_battle_row() {
    let mut p = base();
    p.battle.creatures.pop();
    assert_eq!(
        expect(&[p], "base:faithful"),
        BattleCompileError::MissingCreature("base:b".into())
    );
    // An override of a creature without a battle row.
    let mut p = base();
    p.battle.creatures.pop();
    let mut x = ext();
    x.add_override("base:b", None);
    assert_eq!(
        expect(&[p, x], "base:faithful"),
        BattleCompileError::MissingCreature("base:b".into())
    );
    // A battle row for a creature of another pack, an unknown one, or a second one.
    let mut x = ext();
    x.battle.creatures[0].key = key("base:a");
    assert_eq!(
        content(expect(&[base(), x], "base:faithful")),
        CompileError::KeyOutsidePack {
            key: "base:a".into(),
            pack: "ext".into()
        }
    );
    let mut p = base();
    p.battle.creatures[1].key = key("base:zzz");
    assert_eq!(
        content(expect(&[p], "base:faithful")),
        CompileError::UnknownReference {
            kind: "creature",
            key: "base:zzz".into()
        }
    );
    let mut p = base();
    p.battle.creatures[1].key = key("base:a");
    assert_eq!(
        content(expect(&[p], "base:faithful")),
        CompileError::DuplicateKey("base:a".into())
    );
    // The core's errors come first.
    let mut p = base();
    p.core.profiles.clear();
    assert_eq!(
        content(expect(&[p], "base:faithful")),
        CompileError::UnknownProfile("base:faithful".into())
    );
}

#[test]
fn an_adventure_spell_may_have_no_battle_row() {
    let spell = |k: &str, category| Spell {
        core: SpellDef {
            key: key(k),
            name: "S".into(),
            category,
            level: u8::from(category != SpellCategory::Ability),
            schools: if category == SpellCategory::Ability {
                vec![]
            } else {
                vec![key("base:air")]
            },
            cost: [1, 2, 3, 4],
            power: 5,
            effect: [6, 7, 8, 9],
            granted_by_name_only: true,
        },
        battle: BattleSpellDef {
            key: key(k),
            polarity: SpellPolarity::Negative,
            magic: SpellMagic::default(),
            counters: vec![],
            dispels: vec![],
            spares: vec![],
            tests_as: None,
            summons: None,
        },
    };
    let mut p = base();
    p.core.spell_schools = vec![h3_content::SpellSchoolDef {
        key: key("base:air"),
        name: "Air".into(),
        protection_rank: 0,
    }];
    p.add_spells([
        spell("base:fly", SpellCategory::Adventure),
        spell("base:gaze", SpellCategory::Ability),
    ]);
    p.battle.spells.remove(0);
    let r = compile(&[p.clone()], "base:faithful").unwrap();
    let fly = r.spell_by_key("base:fly").unwrap();
    let row = r.spell(fly).unwrap();
    assert_eq!(
        (row.polarity, &row.magic, row.counters.is_empty()),
        (SpellPolarity::Neutral, &SpellMagic::default(), true)
    );
    // The copies of the core row's fields.
    let core = r.core().spell(fly).unwrap();
    assert_eq!(
        (&row.key, &row.name, row.category, row.level, &row.schools),
        (
            &core.key,
            &core.name,
            core.category,
            core.level,
            &core.schools
        )
    );
    assert_eq!(
        (row.cost, row.power, row.effect, row.granted_by_name_only),
        (
            core.cost,
            core.power,
            core.effect,
            core.granted_by_name_only
        )
    );
    assert_eq!(r.spells().len(), 2);
    assert_eq!(r.spells()[1].polarity, SpellPolarity::Negative);
    assert_eq!(r.spell(SpellId::from_raw(2)), None);
    // Other spells need one.
    p.battle.spells.clear();
    assert_eq!(
        expect(&[p.clone()], "base:faithful"),
        BattleCompileError::MissingSpell("base:gaze".into())
    );
    // A row for an unknown spell, one of another pack, a second one.
    let mut q = p.clone();
    q.battle.spells = vec![spell("base:nope", SpellCategory::Ability).battle];
    assert_eq!(
        content(expect(&[q], "base:faithful")),
        CompileError::UnknownReference {
            kind: "spell",
            key: "base:nope".into()
        }
    );
    let mut x = ext();
    x.battle.spells = vec![spell("base:gaze", SpellCategory::Ability).battle];
    assert_eq!(
        content(expect(&[p.clone(), x], "base:faithful")),
        CompileError::KeyOutsidePack {
            key: "base:gaze".into(),
            pack: "ext".into()
        }
    );
    let gaze = spell("base:gaze", SpellCategory::Ability).battle;
    p.battle.spells = vec![gaze.clone(), gaze];
    assert_eq!(
        content(expect(&[p], "base:faithful")),
        CompileError::DuplicateKey("base:gaze".into())
    );
}

#[test]
fn formation_rows() {
    let r = compile(&[base()], "base:faithful").unwrap();
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
}

#[test]
fn hash_is_stable_and_sensitive() {
    let a = compile(&[base()], "base:faithful").unwrap();
    let b = compile(&[base()], "base:faithful").unwrap();
    assert_eq!(a.content_hash(), b.content_hash());
    assert_eq!(a, b);
    let mut changed = base();
    changed.battle.creatures[1].attack = 6;
    let c = compile(&[changed], "base:faithful").unwrap();
    assert_ne!(a.content_hash(), c.content_hash());
    // A core change leaves the part's hash alone (replays record both).
    let mut faster = base();
    faster.core.creatures[1].speed = 6;
    let f = compile(&[faster], "base:faithful").unwrap();
    assert_eq!(a.content_hash(), f.content_hash());
    assert_ne!(a.core_hash(), f.core_hash());
    let mut tight = base();
    tight.battle.battle.as_mut().unwrap().tight_formation[1] = vec![3, 7];
    let tight = compile(&[tight], "base:faithful").unwrap();
    assert_ne!(a.content_hash(), tight.content_hash());
    let mut tactics = base();
    tactics.battle.battle.as_mut().unwrap().tactics_zone_columns[2] = 15;
    let tactics = compile(&[tactics], "base:faithful").unwrap();
    assert_ne!(a.content_hash(), tactics.content_hash());
    // The surrender's resource is resolved like any reference (BAT-FLE-002).
    let mut surrender = base();
    surrender.battle.battle.as_mut().unwrap().surrender = Some(SurrenderDef {
        resource: key("base:gold"),
        cost_percent: 50,
    });
    let priced = compile(&[surrender.clone()], "base:faithful").unwrap();
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
        .battle
        .as_mut()
        .unwrap()
        .surrender
        .as_mut()
        .unwrap()
        .resource = key("base:silver");
    assert_eq!(
        content(expect(&[surrender], "base:faithful")),
        CompileError::UnknownReference {
            kind: "resource",
            key: "base:silver".into(),
        }
    );
    // Golden value: pins the canonical encoding (bump BATTLE_RULESET_HASH_VERSION when it
    // changes).
    assert_eq!(
        a.content_hash(),
        GOLDEN_BASE_HASH,
        "got {:#018x}",
        a.content_hash()
    );
}

const GOLDEN_BASE_HASH: u64 = 0x7dad_6797_22dd_7bac;

#[test]
fn battle_params_are_unique_and_required() {
    let mut no_battle = base();
    no_battle.battle.battle = None;
    assert_eq!(
        expect(&[no_battle], "base:faithful"),
        BattleCompileError::MissingBattleParams
    );
    let mut second = ext();
    second.battle.battle = Some(battle_def());
    assert_eq!(
        expect(&[base(), second], "base:faithful"),
        BattleCompileError::DuplicateBattleParams("ext".into())
    );
}

#[test]
fn creature_errors() {
    type Edit = fn(&mut Creature);
    const DAMAGE: &str = "damage_min must be at least 1 (war machines: or damage [0, 0])";
    let cases: [(Edit, &str); 7] = [
        (|c| c.battle.damage_min = 0, DAMAGE),
        (
            |c| c.battle.damage_max = 0,
            "damage_max must not be below damage_min",
        ),
        (|c| c.battle.hit_points = 0, "hit_points must be at least 1"),
        // A war machine may have no damage, but not a damage range starting at 0.
        (
            |c| {
                c.core.info.flags = vec!["war_machine".into()];
                c.core.speed = 0;
                c.battle.damage_min = 0;
            },
            DAMAGE,
        ),
        (
            |c| {
                c.core.info.flags = vec!["war_machine".into()];
                c.battle.damage_min = 0;
                c.battle.damage_max = 0;
                c.core.speed = 0;
                c.battle.hit_points = 0;
            },
            "hit_points must be at least 1",
        ),
        // Only war machines act automatically.
        (
            |c| c.battle.abilities = vec![AbilityDef::AmmoSupply],
            "ammo_supply and first_aid require flag war_machine",
        ),
        (
            |c| {
                c.battle.abilities = vec![AbilityDef::FirstAid {
                    min_hp: 1,
                    max_hp: 2,
                }]
            },
            "ammo_supply and first_aid require flag war_machine",
        ),
    ];
    for (edit, reason) in cases {
        let mut c = creature("base:a");
        edit(&mut c);
        let mut p = base();
        p.core.creatures[0] = c.core;
        p.battle.creatures[0] = c.battle;
        assert_eq!(
            content(expect(&[p], "base:faithful")),
            CompileError::InvalidCreature {
                key: "base:a".into(),
                reason
            }
        );
    }
    // A war machine without damage: the copies of speed and flags are the core's.
    let mut p = base();
    p.core.creatures[0].info.flags = vec!["war_machine".into()];
    p.core.creatures[0].speed = 0;
    p.battle.creatures[0].damage_min = 0;
    p.battle.creatures[0].damage_max = 0;
    let r = compile(&[p], "base:faithful").unwrap();
    let a = r.creature(r.creature_by_key("base:a").unwrap()).unwrap();
    assert_eq!(a.speed, 0);
    assert!(a.flags.contains(CreatureFlag::WarMachine));
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
        edit(p.battle.battle.as_mut().unwrap());
        assert_eq!(
            expect(&[p], "base:faithful"),
            BattleCompileError::InvalidBattleParams(reason),
            "{reason}"
        );
    }
    // Wide fields with many rows are fine as long as they fit in 255 hexes; large slot counts too.
    let mut ok = base();
    let b = ok.battle.battle.as_mut().unwrap();
    b.field_width = 15;
    b.field_height = 17;
    b.army_slots = 16;
    b.loose_formation = (1..=16u8).map(|n| (0..n).collect()).collect();
    b.tight_formation = (1..=16u8).map(|n| (0..n).rev().collect()).collect();
    assert!(compile(&[ok], "base:faithful").is_ok());
}

#[test]
fn error_messages() {
    let cases = [
        (
            BattleCompileError::MissingPack("p".into()),
            "pack `p` has no battle part",
        ),
        (
            BattleCompileError::UnknownPack("p@1".into()),
            "battle part of `p@1`, which the core does not load",
        ),
        (
            BattleCompileError::DuplicatePack("p".into()),
            "pack `p` has two battle parts",
        ),
        (
            BattleCompileError::UnknownDependency {
                pack: "a".into(),
                dependency: "b".into(),
            },
            "pack `a` depends on `b`, which does not load before it",
        ),
        (
            BattleCompileError::MissingCreature("a:x".into()),
            "creature `a:x` has no battle row",
        ),
        (
            BattleCompileError::MissingSpell("a:s".into()),
            "spell `a:s` has no battle row",
        ),
        (
            BattleCompileError::InvalidObstacle {
                key: "a:o".into(),
                reason: "bad",
            },
            "obstacle `a:o`: bad",
        ),
        (
            BattleCompileError::MissingBattleParams,
            "no pack provides battle parameters",
        ),
        (
            BattleCompileError::DuplicateBattleParams("a".into()),
            "pack `a` provides battle parameters a second time",
        ),
        (
            BattleCompileError::InvalidBattleParams("bad"),
            "battle parameters: bad",
        ),
        (
            BattleCompileError::Content(CompileError::TooManyCreatures),
            "too many creature types",
        ),
    ];
    for (err, text) in cases {
        assert_eq!(err.to_string(), text);
    }
}

#[test]
fn rows_compile_with_their_copies_of_the_core() {
    let mut p = base();
    p.core.creatures[0].info.flags = vec!["flying".into(), "two_hex".into()];
    p.battle.creatures[0].shots = 12;
    p.battle.creatures[0].casts = 1;
    let r = compile(&[p], "base:faithful").unwrap();
    let a = r.creature_by_key("base:a").unwrap();
    let stats = r.creature(a).unwrap();
    assert_eq!((stats.shots, stats.casts), (12, 1));
    assert!(stats.flags.contains(CreatureFlag::Flying));
    assert!(stats.flags.contains(CreatureFlag::TwoHex));
    assert!(!stats.flags.contains(CreatureFlag::Undead));
    assert_eq!(stats.flags, r.core().creature(a).unwrap().flags);
    let plain = compile(&[base()], "base:faithful").unwrap();
    assert_ne!(plain.content_hash(), r.content_hash());
}

fn overriding(target: &str, patch: BattleCreaturePatch) -> Pack {
    let mut p = ext();
    let i = p.add_override(target, None);
    p.battle.creature_overrides[i].patch = patch;
    p
}

#[test]
fn overrides_change_only_the_listed_fields() {
    let before = compile(&[base()], "base:faithful").unwrap();
    // An empty patch changes nothing.
    let r = compile(
        &[base(), overriding("base:a", BattleCreaturePatch::default())],
        "base:faithful",
    )
    .unwrap();
    let a = r.creature_by_key("base:a").unwrap();
    assert_eq!(r.creature(a), before.creature(a));
    assert_eq!(r.abilities(a), before.abilities(a));
    // A full patch changes everything; the core half keeps the copies current.
    let full = BattleCreaturePatch {
        attack: Some(9),
        defense: Some(8),
        damage_min: Some(2),
        damage_max: Some(7),
        hit_points: Some(30),
        shots: Some(24),
        casts: Some(3),
        abilities: Some(vec![AbilityDef::Hate {
            target: key("ext:c"),
            percent: 50,
        }]),
    };
    let mut x = overriding("base:a", full);
    x.core.creature_overrides[0].patch = CreaturePatch {
        speed: Some(11),
        flags: Some(vec!["undead".into()]),
        ..CreaturePatch::default()
    };
    let r = compile(&[base(), x], "base:faithful").unwrap();
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
    assert_eq!((stats.shots, stats.casts), (24, 3));
    assert!(stats.flags.contains(CreatureFlag::Undead));
    assert_eq!(
        r.abilities(a),
        [CompiledAbility::Hate {
            target: r.creature_by_key("ext:c").unwrap(),
            percent: 50
        }]
    );
}

#[test]
fn override_errors() {
    assert_eq!(
        content(expect(
            &[
                base(),
                overriding("base:zzz", BattleCreaturePatch::default())
            ],
            "base:faithful"
        )),
        CompileError::UnknownReference {
            kind: "creature",
            key: "base:zzz".into()
        }
    );
    // `abc` is enabled and loads before `ext` (id order), but is not a dependency of `ext`. The
    // battle half alone names it (the core half is checked by the core).
    let other = Pack::new(manifest("abc", &[])).creatures([creature("abc:o")]);
    let mut x = overriding("abc:o", BattleCreaturePatch::default());
    x.core.creature_overrides.clear();
    assert_eq!(
        content(expect(&[base(), other, x], "base:faithful")),
        CompileError::ReferenceOutsideDependencies {
            key: "abc:o".into(),
            pack: "ext".into()
        }
    );
    // The patched creature is validated like a definition.
    let broken = BattleCreaturePatch {
        hit_points: Some(0),
        ..BattleCreaturePatch::default()
    };
    assert_eq!(
        content(expect(
            &[base(), overriding("base:a", broken)],
            "base:faithful"
        )),
        CompileError::InvalidCreature {
            key: "base:a".into(),
            reason: "hit_points must be at least 1"
        }
    );
    // Overrides resolve their abilities in the overriding pack's scope and are validated.
    let bad = BattleCreaturePatch {
        abilities: Some(vec![AbilityDef::Hate {
            target: key("ext:nobody"),
            percent: 50,
        }]),
        ..BattleCreaturePatch::default()
    };
    assert!(matches!(
        content(expect(
            &[base(), overriding("base:a", bad)],
            "base:faithful"
        )),
        CompileError::UnknownReference {
            kind: "creature",
            ..
        }
    ));
    let twice = BattleCreaturePatch {
        abilities: Some(vec![AbilityDef::Teleports, AbilityDef::Teleports]),
        ..BattleCreaturePatch::default()
    };
    assert_eq!(
        content(expect(
            &[base(), overriding("base:a", twice)],
            "base:faithful"
        )),
        CompileError::InvalidCreature {
            key: "base:a".into(),
            reason: "an ability is listed twice"
        }
    );
}

fn with_abilities(abilities: Vec<AbilityDef>) -> Pack {
    let mut p = base();
    p.battle.creatures[0].abilities = abilities;
    p
}

#[test]
fn abilities_compile_with_forward_references() {
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
    p.core.creatures[1].info.flags = vec!["area_attack".into(), "multi_headed".into()];
    p.battle.creatures[1].abilities = vec![
        AbilityDef::AreaShot {
            effect: AreaShot::Fireball,
        },
        AbilityDef::ThreeHeadedAttack,
    ];
    let r = compile(&[p], "base:faithful").unwrap();
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
    let plain = compile(&[with_abilities(vec![])], "base:faithful").unwrap();
    let hated = compile(
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
    let fails = |p: Pack| content(expect(&[p], "base:faithful"));
    assert_eq!(
        fails(with_abilities(vec![AbilityDef::Hate {
            target: key("base:zzz"),
            percent: 50
        }])),
        CompileError::UnknownReference {
            kind: "creature",
            key: "base:zzz".into()
        }
    );
    // A base creature cannot refer to a creature of a pack it does not depend on.
    assert_eq!(
        content(expect(
            &[
                with_abilities(vec![AbilityDef::Hate {
                    target: key("ext:c"),
                    percent: 50
                }]),
                ext()
            ],
            "base:faithful"
        )),
        CompileError::ReferenceOutsideDependencies {
            key: "ext:c".into(),
            pack: "base".into()
        }
    );
    assert_eq!(
        fails(with_abilities(vec![AbilityDef::Jousting {
            percent_per_hex: 0
        }])),
        invalid("jousting percent_per_hex must be at least 1")
    );
    assert_eq!(
        fails(with_abilities(vec![AbilityDef::AttacksAllAdjacent])),
        invalid("flag multi_headed requires attacks_all_adjacent or three_headed_attack")
    );
    let mut flagged = base();
    flagged.core.creatures[0].info.flags = vec!["area_attack".into()];
    assert_eq!(
        fails(flagged),
        invalid("flag area_attack requires an area_shot ability (and vice versa)")
    );
    // E04c T6 (BAT-SIE-013): a wall shooter carries the flag catapult, and the flag needs the
    // ability unless the creature is a war machine (the siege's Catapult).
    let walls = "flag catapult requires wall_shots (and vice versa), war machines neither";
    assert_eq!(
        fails(with_abilities(vec![AbilityDef::WallShots { level: 1 }])),
        invalid(walls)
    );
    let mut flagged = base();
    flagged.core.creatures[0].info.flags = vec!["catapult".into()];
    assert_eq!(fails(flagged), invalid(walls));
    // E04c T7: a turret is a shooting war machine and no catapult.
    let turret = "turret requires flags war_machine and shooter, not catapult";
    let towered = |flags: &[&str]| {
        let mut pack = with_abilities(vec![AbilityDef::Turret]);
        pack.core.creatures[0].info.flags = flags.iter().map(|&f| f.into()).collect();
        if flags.contains(&"war_machine") {
            pack.core.creatures[0].speed = 0;
        }
        pack
    };
    assert_eq!(fails(towered(&["shooter"])), invalid(turret));
    assert_eq!(fails(towered(&["war_machine"])), invalid(turret));
    assert_eq!(
        fails(towered(&["war_machine", "shooter", "catapult"])),
        invalid(turret)
    );
}

#[test]
fn roll_tables_index_levels_from_one_and_cap_at_the_top() {
    let r = compile(&[base()], "base:faithful").unwrap();
    let b = r.battle();
    assert_eq!((b.morale_dice, b.luck_dice), (24, 24));
    assert_eq!(b.good_morale.levels(), 3);
    assert_eq!(
        (0..=4).map(|l| b.bad_morale.chance(l)).collect::<Vec<_>>(),
        vec![0, 2, 4, 6, 6]
    );
    assert_eq!(b.good_luck.chance(1), 1);
    assert_eq!(RollTable::default().chance(1), 0);
    assert_eq!(MAX_ROLL_LEVELS, 8);
}

#[test]
fn aura_values_are_keyed_and_limited() {
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
        p.core.creatures.clear();
        p.battle.creatures.clear();
        for i in 0..n {
            let mut c = creature(&std::format!("base:c{i}"));
            c.battle.abilities = vec![morale(i + 1, AuraTarget::Allies)];
            p.add_creature(c);
        }
        p
    };
    compile(&[with_auras(8)], "base:faithful").unwrap();
    assert_eq!(
        content(expect(&[with_auras(9)], "base:faithful")),
        CompileError::TooManyEntries("distinct aura")
    );
}

#[test]
fn obstacles() {
    let r = compile(&[base()], "base:faithful").unwrap();
    let grass = r.terrain_by_key("base:grass").unwrap();
    let lava = r.terrain_by_key("base:lava").unwrap();
    let o = &r.obstacles()[0];
    assert_eq!(
        (o.terrains.as_slice(), o.width, o.absolute),
        (&[grass, lava][..], 2, false)
    );
    assert_eq!(r.obstacles()[1].blocked, vec![(5, 4), (6, 4)]);
    // By key and by handle.
    let rock = r.obstacle_by_key("base:rock").unwrap();
    assert_eq!(r.obstacle(rock), Some(o));
    assert_eq!(r.obstacle_by_key("base:nothing"), None);
    // Errors: unknown terrain, duplicate key, a key outside the pack, too many, invalid shapes.
    let mut p = base();
    p.battle.obstacles.push(p.battle.obstacles[0].clone());
    assert_eq!(
        content(expect(&[p], "base:faithful")),
        CompileError::DuplicateKey("base:rock".into())
    );
    let mut p = base();
    p.battle.obstacles[0].terrains.push(key("base:snow"));
    assert!(matches!(
        content(expect(&[p], "base:faithful")),
        CompileError::UnknownReference {
            kind: "terrain",
            ..
        }
    ));
    let mut p = base();
    p.battle.obstacles[0].key = key("ext:rock");
    assert!(matches!(
        content(expect(&[p], "base:faithful")),
        CompileError::KeyOutsidePack { .. }
    ));
    let mut p = base();
    p.battle.obstacles = (0..=u32::from(u16::MAX) + 1)
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
        content(expect(&[p], "base:faithful")),
        CompileError::TooManyEntries("obstacle")
    );
    // A usual obstacle is a removable piece of the battle, which holds 8 hexes (E04b T11b
    // review, F-robust-5); an absolute one has no such bound.
    let row = |hexes: i8, y: i8| -> Vec<(i8, i8)> { (0..hexes).map(|dx| (dx + 1, y)).collect() };
    assert_eq!(MAX_USUAL_OBSTACLE_HEXES, 8);
    for (blocked, absolute) in [(row(8, 0), false), (row(9, 4), true)] {
        let mut p = base();
        p.battle
            .obstacles
            .push(obstacle("base:o", &["base:grass"], &blocked, absolute));
        assert!(compile(&[p], "base:faithful").is_ok());
    }
    let shapes: [(ObstacleDef, &str); 6] = [
        (
            obstacle("base:o", &["base:grass"], &row(9, 0), false),
            "usual obstacles block at most 8 hexes",
        ),
        (
            obstacle("base:o", &[], &[(0, 0)], false),
            "an obstacle needs a terrain or a battlefield and one blocked hex",
        ),
        (
            obstacle("base:o", &["base:grass"], &[], false),
            "an obstacle needs a terrain or a battlefield and one blocked hex",
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
        p.battle.obstacles.push(o);
        assert_eq!(
            expect(&[p], "base:faithful"),
            BattleCompileError::InvalidObstacle {
                key: "base:o".into(),
                reason
            }
        );
    }
    for (x, y) in [(16, 4), (5, -1), (5, 11)] {
        let mut p = base();
        p.battle
            .obstacles
            .push(obstacle("base:o", &["base:grass"], &[(x, y)], true));
        assert!(matches!(
            expect(&[p], "base:faithful"),
            BattleCompileError::InvalidObstacle { .. }
        ));
    }
    let mut p = base();
    p.battle.obstacles.push(ObstacleDef {
        width: 0,
        ..obstacle("base:o", &["base:grass"], &[(0, 0)], false)
    });
    assert!(matches!(
        expect(&[p], "base:faithful"),
        BattleCompileError::InvalidObstacle { .. }
    ));
}
